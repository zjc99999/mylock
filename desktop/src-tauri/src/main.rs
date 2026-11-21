// MyLock 桌面壳 —— M5「零成本研发第 1 项：桌面端到端收口」+ GUI 完善。
//
// 职责边界（与 m3-architecture.md / m3-report.md / 产品分析与概要设计.md 一致）：
// - 这个壳**不做密码学实现**：真正的引擎是 mylock-mount（VaultFs / WebDAV），
//   内核是 mylock-core；本壳以 tokio task 在当前进程内启动 loopback 服务，
//   管理 OS 盘符映射，并调用内核的建库入口（与 CLI 同一份实现，不另写一套）。
// - 不是 WinFsp/FUSE 驱动级挂载；解锁态只缓解不消除采集风险；
//   WebDAV loopback 无 per-request 认证（任何本机进程都能访问挂载点）。
// - 拒绝在云同步根目录内建库/挂载（is_sync_root / E_SYNC_ROOT）。
//
// M5 相对 M4 第 2 点的实质改动（端到端收口，全部实测）：
// 1. 不再 spawn 外部 sidecar 子进程并丢弃其 stdout/stderr；改为进程内启动
//    `run_listener(handler, listener)`（build_handler 已做口令解锁 + 云同步根拒绝），
//    因此「口令错 / 容器损坏 / 云同步根 / 端口占用」都能在返回前拿到真实错误码。
// 2. 解锁后由桌面壳自动执行 `net use <盘符>: http://127.0.0.1:<port>/` 映射，
//    锁定时自动 `net use <盘符>: /delete`；不再把命令打印给用户手敲。
//    实测（tools/e2e-probe.ps1，2026-11-15）：WebClient 服务显示 Stopped 时
//    `net use Z: http://127.0.0.1:51919/` 仍成功，Z:\ 读写、写回落盘、
//    整盘复制到另一路径后零明文命中全部通过。
// 3. 结构校验 `verify` 仍走 mylock_core::verify_phase1（不解锁、不解密）。
//
// M5 GUI 完善（用户 m04697 授权的两点）：
// 4. 新增 `create_vault`：把原先 CLI-only 的 Step 1（mylock.exe create）收进界面，
//    并做安全前置校验（非空 / 非云同步根 / 非盘符根 / 已存在非空目录一律拒绝，
//    因为内核 write_vault 会 remove_dir_all）；返回 Recovery Key 由前端醒目展示。
// 5. 新增文件夹「提示性」保护：desktop.ini（UTF-16LE+BOM，保留系统文件夹删除
//    警告）+ 保险柜图标 + 把内部零件文件设为 隐藏+系统 + 一份可见中文警示。
//    A 级依据：Microsoft Learn《How to Customize Folders with Desktop.ini》
//    https://learn.microsoft.com/en-us/windows/win32/shell/how-to-customize-folders-with-desktop-ini
//    实测（tools/probe-attrib.ps1）：+s +h 不阻止 Rust std::fs 读写/改名；
//    但 manifest 每次写回被 fs::rename 覆盖会丢属性 ⇒ 每次解锁幂等重放装饰。
//    A 级依据（写回不清根）：mylock-core/src/lib.rs rewrite_vault_incremental
//    只在 root/d/ 与 root/manifest 上写，不触碰根级装饰文件；verify_phase1 也只扫 d/。
//    ⚠ 诚实边界：这只是 shell 层「提示」，不是 ACL/权限/防删机制，
//    不保证数据不丢失；请勿存放不可替代的唯一副本。
//
// 诚实边界：本壳 / 内核 ≠ 产品发布 ≠ 五端一致性验收 ≠ 第三方安全审计；
//           未产出代码签名安装包、证书须自备；封闭内测 ≠ 公开发布/商店上架。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use tauri::State;
use tokio::net::TcpListener as TokioTcpListener;

use mylock_core::{random_bytes, CoreError};
use mylock_mount::{build_handler, run_listener};

/// 编译期嵌入的保险柜图标（多尺寸 .ico，由 tools/make_vault_icon.py 生成）。
/// 用 include_bytes! 而不是打包时多拷一个文件，避免用户把图标当普通文件删掉。
static VAULT_ICON: &[u8] = include_bytes!("../icons/MyLockVault.ico");

/// 专用 multi-thread tokio runtime：驱动进程内 loopback WebDAV 服务。
/// 与 Tauri 自身的 async runtime 相互独立；同步命令里通过 `runtime().spawn`
/// 与 `runtime().block_on` 使用，不依赖 thread-local 的 `Handle::current()`，
/// 也不依赖「主线程一直持有 enter guard」这种脆弱假设。
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
    })
}

/// 当前挂载会话。一个桌面壳同时只挂一个 vault（M3 简化，M5 沿用）。
#[derive(Default)]
struct MountState(Mutex<Option<MountSession>>);

struct MountSession {
    port: u16,
    drive: String,
    /// OS 盘符映射是否确实建立成功（失败则锁定时不必尝试断开）。
    mapped: bool,
    /// 进程内 WebDAV 服务 task 的放弃句柄（Drop 即 abort）。
    server: tokio::task::JoinHandle<()>,
}

impl Drop for MountSession {
    fn drop(&mut self) {
        self.server.abort();
    }
}

fn core_err(e: CoreError) -> String {
    format!("{}: {}", e.code, e.detail)
}

/// 把盘符输入规范成 `X:`（接受 "Z" / "Z:" / "z" / " Z: "），并拒绝非法值。
/// 只允许单个字母，避免任何命令注入面。
fn normalize_drive(s: Option<String>) -> Result<String, String> {
    let raw = s.unwrap_or_else(|| "Z".to_string());
    let t = raw.trim().trim_end_matches(':').trim().to_ascii_uppercase();
    if t.len() != 1 || !t.as_bytes()[0].is_ascii_alphabetic() {
        return Err(format!("非法盘符 {:?}（请填单个字母，如 Z 或 Z:）", raw));
    }
    Ok(format!("{}:", t))
}

/// 调用 Windows `net use` 建立/断开 WebDAV 盘符映射。
/// `CREATE_NO_WINDOW` 避免每次解锁闪一个控制台窗口；stdout/stderr 全部回收用于诊断。
#[cfg(windows)]
fn net_use(args: &[String]) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = Command::new("net")
        .args(args.iter().map(|s| s.as_str()))
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行 net use 失败：{}", e))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if out.status.success() {
        Ok(text.trim().to_string())
    } else {
        Err(format!(
            "net use 失败（exit {:?}）：{}",
            out.status.code(),
            text.trim()
        ))
    }
}

#[cfg(not(windows))]
fn net_use(_args: &[String]) -> Result<String, String> {
    Err("自动盘符映射目前仅支持 Windows（macOS 请用 mount_webdav，见 m3-architecture.md）。".into())
}

/// 逐路径设置文件/目录属性。
///
/// 硬约束（tools/probe-attrib.ps1 实测）：`attrib` 一次传多个引号路径会报
/// `Parameter format not correct -`，必须一个路径一次调用；目录要带 `/d`。
/// 用 Command 直接起 attrib（不经 cmd /c），避免引号被二次解析。
#[cfg(windows)]
fn set_attrib(path: &Path, flags: &[&str], is_dir: bool) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut cmd = Command::new("attrib");
    for f in flags {
        cmd.arg(f);
    }
    if is_dir {
        cmd.arg("/d");
    }
    cmd.arg(path.as_os_str());
    let out = cmd
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("attrib 执行失败：{}", e))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "attrib 失败：{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
        .trim()
        .to_string())
    }
}

/// 给密文容器加「提示性」保护：自定义图标 / 隐藏内部零件 / 删除警告 / 可见中文警示。
///
/// 幂等：每次解锁都会重放（因为 manifest 每次写回被 rename 覆盖会丢 +s +h 属性）。
/// best-effort：任何一步失败只记进返回说明，绝不阻断建库或解锁。
/// ⚠ 这不是防删 / ACL 机制，不保证数据不丢失（冻结诚实边界）。
#[cfg(windows)]
fn decorate_vault(root: &Path) -> String {
    let mut ok: Vec<String> = Vec::new();
    let mut warn: Vec<String> = Vec::new();

    // 1) desktop.ini：UTF-16LE + BOM（Microsoft 要求 Unicode 格式）。
    //    刻意【不写】ConfirmFileOp=0 —— 保留 "You Are Deleting a System Folder"
    //    删除/移动警告；刻意【不写】NoSharing —— Vista+ 不支持。
    //    IconFile 用相对路径 + IconIndex=0，跨网络查看也生效。
    let ini_text = concat!(
        "[.ShellClassInfo]\r\n",
        "IconFile=MyLockVault.ico\r\n",
        "IconIndex=0\r\n",
        "InfoTip=MyLock 密文保险柜：请勿删除或改动其中任何文件\r\n",
    );
    let mut ini_bytes: Vec<u8> = vec![0xFF, 0xFE]; // UTF-16LE BOM
    for unit in ini_text.encode_utf16() {
        ini_bytes.extend_from_slice(&unit.to_le_bytes());
    }
    match std::fs::write(root.join("desktop.ini"), &ini_bytes) {
        Ok(_) => ok.push("desktop.ini（UTF-16LE+BOM，保留删除系统文件夹警告）".into()),
        Err(e) => warn.push(format!("desktop.ini 写入失败：{}", e)),
    }

    // 2) 保险柜图标（编译期资源写进容器根目录）
    match std::fs::write(root.join("MyLockVault.ico"), VAULT_ICON) {
        Ok(_) => ok.push("保险柜图标 MyLockVault.ico".into()),
        Err(e) => warn.push(format!("图标写入失败：{}", e)),
    }

    // 3) 一份【可见】的中文警示（不隐藏，作为「删除/改动提醒」的天天可见版本）
    let warn_txt = concat!(
        "MyLock 密文保险柜 —— 请勿删除 / 改名 / 移动本文件夹内的任何内容\r\n",
        "============================================================\r\n",
        "\r\n",
        "这个文件夹里存放的是「密文容器」：vault.meta / vault.meta.bak / manifest / d/\r\n",
        "等是保险柜的零件，不是普通文件。删除或改动任何一个，都可能导致里面的数据\r\n",
        "永久无法解密（本版本没有云端备份、没有账号、没有回收站）。\r\n",
        "\r\n",
        "本文件夹已做三层「提示性」保护 —— 注意：只是 shell 层提示，不是防删 / 权限\r\n",
        "机制，不能保证数据不丢失：\r\n",
        " 1. 内部零件文件已设为「隐藏 + 系统」属性，正常浏览时不显示；\r\n",
        " 2. 文件夹已设为「系统文件夹」并配 desktop.ini：删除 / 移动时 Windows 会弹\r\n",
        "    「You Are Deleting a System Folder」警告（刻意没有关闭它）；\r\n",
        " 3. 文件夹图标已换成保险柜，鼠标悬停有提示文字。\r\n",
        "\r\n",
        "真正的安全靠：\r\n",
        " · 请勿存放「不可替代的唯一副本」（本版本未经第三方安全审计、未代码签名）；\r\n",
        " · 把整个文件夹原样拷到移动硬盘仍是保险柜，但不会自动变明文；\r\n",
        " · 解锁请用 MyLock 桌面程序（或 mylock.exe），不要直接改这些文件。\r\n",
        "\r\n",
        "详见 USER-GUIDE.md。\r\n",
    );
    match std::fs::write(root.join("⚠ 请勿删除.txt"), warn_txt.as_bytes()) {
        Ok(_) => ok.push("可见中文警示 ⚠ 请勿删除.txt".into()),
        Err(e) => warn.push(format!("中文警示写入失败：{}", e)),
    }

    // 4) 把容器根目录设为「系统文件夹」以启用 Desktop.ini 特殊行为（不隐藏根目录本身，
    //    否则用户看不到保险柜图标了）。A 级：attrib +s FolderName。
    match set_attrib(root, &["+s"], true) {
        Ok(_) => ok.push("根目录 attrib +s（启用 desktop.ini，保留删除警告）".into()),
        Err(e) => warn.push(format!("根目录 +s 失败：{}", e)),
    }

    // 5) 隐藏内部零件（只隐藏，不删；程序化读写不受影响 —— probe-attrib 实测）。
    //    d/ 是目录，要带 /d；其余是文件。
    let hidden: &[(&str, bool)] = &[
        ("desktop.ini", false),
        ("MyLockVault.ico", false),
        ("vault.meta", false),
        ("vault.meta.bak", false),
        ("manifest", false),
        ("README.txt", false),
        ("d", true),
    ];
    for (name, is_dir) in hidden {
        let p = root.join(name);
        if !p.exists() {
            continue;
        }
        match set_attrib(&p, &["+s", "+h"], *is_dir) {
            Ok(_) => ok.push(format!("隐藏 {}", name)),
            Err(e) => warn.push(format!("隐藏 {} 失败：{}", name, e)),
        }
    }

    if warn.is_empty() {
        format!("已应用文件夹提示性保护（{} 项）。", ok.len())
    } else {
        format!(
            "已应用 {} 项提示性保护；{} 项未生效（不影响加解密）：{}",
            ok.len(),
            warn.len(),
            warn.join("；")
        )
    }
}

#[cfg(not(windows))]
fn decorate_vault(_root: &Path) -> String {
    "（非 Windows 平台：跳过 desktop.ini / 图标 / attrib 文件夹提示性保护）".into()
}

/// 探测端口是否可用（在专用 runtime 上用 tokio 试绑后立即释放）。
fn port_available(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    runtime().block_on(TokioTcpListener::bind(addr)).is_ok()
}

/// 把底层错误码翻译成给用户看的人话（保留底层串便于诊断）。
/// 解锁/结构校验前统一的人话预检：该路径还没有保险柜（找不到 vault.meta）。
/// 用户 m04638 实测报过 `E_FORMAT: io: 系统找不到指定的路径。 (os error 3)`，
/// 根因是 load_meta 直接 fs::read(vault.meta)；这里把底层串换成人话指引。
fn human_missing_vault(root_display: &str, root_path: &Path) -> Option<String> {
    if root_path.join("vault.meta").exists() {
        return None;
    }
    Some(format!(
        "路径 {} 下还没有保险柜（找不到 vault.meta）。\n\
         请先点「新建保险柜」完成建库（Step 1 已收进界面），或确认 Vault 根目录是否填错。\n\
         注意：这不是「原地加密」——MyLock 不会改你原来的明文目录（如 E:\\testLock），\n\
         保险柜是另一个新目录；把文件拖进解锁后的 Z: 盘才算写进保险柜。",
        root_display
    ))
}

fn humanize_open_error(e: CoreError) -> String {
    let raw = core_err(e);
    if raw.starts_with("E_AUTH_FAILED") {
        return format!(
            "主口令或 Recovery Key 不正确（AES-KW 完整性校验失败）。\n\
             请重新输入；若忘记口令，请用建库时抄存的 Recovery Key 解锁。\n\
             （底层：{}）",
            raw
        );
    }
    if raw.contains("E_SYNC_ROOT") {
        return format!(
            "拒绝在云同步目录内挂载（OneDrive / Dropbox / Google Drive / iCloud / 坚果云等）。\n\
             请换到本地非同步路径，或把保险柜拷出同步目录后再解锁。\n\
             （底层：{}）",
            raw
        );
    }
    raw
}

/// 新建保险柜的返回（结构化，便于前端把 Recovery Key 醒目展示而不淹没在日志里）。
/// Debug 便于单测里 `.expect_err()` / `.expect()` 打印。
#[derive(Debug, serde::Serialize)]
struct CreateResult {
    root: String,
    jti: String,
    key_epoch: u32,
    recovery_key: String,
    decoration: String,
    message: String,
}

/// 【M5 GUI 完善 1】把 CLI-only 的 Step 1 收进界面：新建一个空的密文容器。
///
/// 与 CLI 的 `mylock.exe create` 走完全相同的内核函数，不另写一套密码学：
///   random_bytes(32) → random_vault_params → build_vault → write_vault → rk_display。
///
/// 安全前置校验（因为 write_vault 内部有 remove_dir_all(root)）：
///  - 路径/口令非空；拒绝云同步根；拒绝盘符根；已存在且非空的目录一律拒绝。
/// 建库后再加「提示性」文件夹保护（decorate_vault，best-effort）。
#[tauri::command]
fn create_vault(root: String, password: String) -> Result<CreateResult, String> {
    let root = root.trim().to_string();
    if root.is_empty() {
        return Err("请填写「Vault 根目录」：新建保险柜要放在一个全新空路径。".into());
    }
    if password.is_empty() {
        return Err("请填写「主口令」（新建保险柜用）。".into());
    }
    let root_path = PathBuf::from(&root);

    if let Some(marker) = mylock_mount::is_sync_root(&root_path) {
        return Err(format!(
            "拒绝在云同步目录内新建保险柜（命中 {}）。请换到本地非同步路径。",
            marker
        ));
    }
    if root_path.parent().is_none() {
        return Err(
            "不能把盘符根目录（如 E:\\）当作保险柜：新建会清空整个目标目录，请改填一个子目录。".into(),
        );
    }
    if root_path.exists() {
        let non_empty = std::fs::read_dir(&root_path)
            .map(|mut it| it.next().is_some())
            .unwrap_or(true);
        if non_empty {
            return Err(format!(
                "目标路径 {} 已存在且非空。\n\
                 新建保险柜会清空该目录（内核 write_vault 内部 remove_dir_all），为避免误删，\n\
                 请改填一个「全新空路径」，或先手动把该目录里的内容移走。\n\
                 注意：MyLock 不会碰你原来的明文目录（如 E:\\testLock）。",
                root
            ));
        }
    }

    let rk = random_bytes(32).map_err(core_err)?;
    let params = mylock_core::random_vault_params(password.as_bytes(), &rk, 0).map_err(core_err)?;
    let jti = params.jti.clone();
    let c = mylock_core::build_vault(&params, &[]).map_err(core_err)?;
    mylock_core::write_vault(&root_path, &c).map_err(core_err)?;
    let recovery_key = mylock_core::rk_display(&rk, 0);
    let decoration = decorate_vault(&root_path);

    let message = format!(
        "已新建保险柜：{}\n\
         结构：vault.meta / vault.meta.bak / manifest / d/ / README.txt（均为密文或元数据，不是你的文件）\n\
         jti：{}\n\
         key_epoch：0\n\n\
         ★ 请【立刻离线抄存】Recovery Key：{}\n\
           （忘记口令时唯一能解锁的兜底；换口令/换 RK 是零吊销，真吊销用 rotate-keys。）\n\n\
         文件夹提示性保护：{}\n\
         ⚠ 这只是 shell 层提示（隐藏 + 系统文件夹删除警告 + 保险柜图标），\n\
           不是防删/权限机制，不保证数据不丢失。\n\n\
         下一步：回到上方点「解锁并映射」→ 出现 Z: 盘 → 把文件拖进 Z: 即写入保险柜\n\
         → 用完点「锁定并断开」。详见 USER-GUIDE.md。\n\n\
         ⚠ 本版本未经第三方安全审计、安装包未代码签名；请勿存放不可替代的唯一副本。",
        root, jti, recovery_key, decoration
    );

    Ok(CreateResult {
        root,
        jti,
        key_epoch: 0,
        recovery_key,
        decoration,
        message,
    })
}

/// 解锁并自动映射为盘符。
///
/// 返回给前端的是可直接展示的说明文本；任何失败都以 `Err(String)` 返回真实原因，
/// 不会再出现「口令错了却告诉用户已启动」的情况。
/// 【M5 GUI 完善】解锁前先预检 vault.meta 给出人话提示；成功后幂等重放文件夹保护。
#[tauri::command]
fn open_vault(
    root: String,
    password: String,
    port: Option<u16>,
    drive: Option<String>,
    recovery: Option<String>,
    state: State<'_, MountState>,
) -> Result<String, String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Err("已有挂载会话在运行，请先「锁定并断开」。".into());
    }
    if root.trim().is_empty() {
        return Err("请填写 Vault 根目录。".into());
    }
    let port = port.unwrap_or(4918);
    let drive = normalize_drive(drive)?;
    if !port_available(port) {
        return Err(format!(
            "端口 {} 已被占用：请换一个端口，或先停止占用该端口的程序。",
            port
        ));
    }

    // 凭据：口令（UTF-8 字节）或 Recovery Key 展示码，二者只能给一个。
    let (secret, slot) = match (password.is_empty(), recovery.as_deref()) {
        (false, None) => (password.into_bytes(), "password".to_string()),
        (false, Some(_)) => {
            return Err("--password 与 Recovery Key 只能提供其中一个".into())
        }
        (true, Some(r)) if !r.trim().is_empty() => {
            let (_, raw) = mylock_core::recovery_display_to_raw(r.trim())
                .map_err(core_err)?;
            (raw, "recovery".to_string())
        }
        _ => return Err("请填写主口令（或 Recovery Key 展示码）".into()),
    };

    let root_path = std::path::PathBuf::from(&root);

    // 【人话预检】用户最常见的误操作：把全新路径直接拿去解锁。
    // 原先会漏出 `E_FORMAT: io: 系统找不到指定的路径。 (os error 3)` 这种底层串。
    if let Some(msg) = human_missing_vault(&root, &root_path) {
        return Err(msg);
    }

    let device_id = random_bytes(16).map_err(core_err)?;

    // 建 handler 会做：云同步根拒绝（E_SYNC_ROOT）→ unwrap_master（口令错 E_AUTH_FAILED）
    // → full_open（容器损坏 E_FORMAT/E_MANIFEST/E_CORRUPT）。全部在映射盘符之前完成。
    let handler = build_handler(&root_path, &secret, &slot, device_id).map_err(humanize_open_error)?;

    // 直接在专用 runtime 上用 tokio 异步 TcpListener 绑定 loopback，
    // 不经过 std listener + from_std（那条路要求 reactor context，易踩坑）。
    let rt = runtime();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = rt
        .block_on(TokioTcpListener::bind(addr))
        .map_err(|e| format!("loopback 绑定 127.0.0.1:{} 失败：{}", port, e))?;

    let server = rt.spawn(run_listener(handler, listener));

    // 自动映射盘符；失败则回滚（abort server），不留下半开的会话。
    let uri = format!("http://127.0.0.1:{}/", port);
    let map_args = vec![
        "use".to_string(),
        drive.clone(),
        uri.clone(),
        "/persistent:no".to_string(),
    ];
    let mapped = match net_use(&map_args) {
        Ok(_) => true,
        Err(e) => {
            server.abort();
            return Err(format!(
                "已解锁并启动 loopback 服务，但自动映射盘符 {} 失败：{}\n\
                 可手动执行：net use {} {}",
                drive, e, drive, uri
            ));
        }
    };

    *guard = Some(MountSession {
        port,
        drive: drive.clone(),
        mapped,
        server,
    });

    // 【M5 GUI 完善 2】每次解锁幂等重放文件夹提示性保护：
    // 自愈 CLI 建的库（没有 desktop.ini）以及写回后被 rename 冲掉的属性。
    // best-effort，失败只写进说明文本，不影响已建立的挂载。
    let decoration = decorate_vault(&root_path);

    Ok(format!(
        "已解锁并映射 {} → {}\n\
         引擎：进程内 loopback WebDAV（127.0.0.1:{}）\n\
         现在可以像普通文件夹一样双击、拖拽、另存为。\n\
         用完后请点「锁定并断开」，或手动执行 net use {} /delete\n\n\
         文件夹提示性保护（每次解锁自动重放）：{}\n\
         ⚠ 只是 shell 层提示，不是防删/权限机制，不保证数据不丢失。\n\n\
         ⚠ 安全边界（M3 诚实声明）：\n\
         · 这是 loopback WebDAV，非 WinFsp/FUSE 驱动级挂载，非产品发布。\n\
         · WebDAV loopback 无 per-request 认证：任何本机进程都能访问挂载点。\n\
         · 仅在「未解锁态」防得住合法采集者；解锁态只缓解，不消除风险。\n\
         · 库外已存在的明文/临时副本，锁定后不保证消失。\n\
         · 已拒绝在 OneDrive/Dropbox/Google Drive/iCloud/坚果云等同步目录内挂载。\n\
         · 本版本未经第三方安全审计、安装包未代码签名；请勿存放不可替代的唯一副本。\n\
         · 内核/桌面壳 ≠ 产品发布 ≠ 五端一致性验收 ≠ 第三方安全审计。",
        drive, uri, port, drive, decoration
    ))
}

/// 锁定：先断开 OS 盘符映射，再停掉进程内 WebDAV 服务。
#[tauri::command]
fn close_vault(state: State<'_, MountState>) -> Result<String, String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    match guard.take() {
        Some(session) => {
            let unmap = if session.mapped {
                match net_use(&[
                    "use".to_string(),
                    session.drive.clone(),
                    "/delete".to_string(),
                    "/y".to_string(),
                ]) {
                    Ok(_) => format!("已断开 {} 盘符映射。", session.drive),
                    Err(e) => format!(
                        "断开 {} 失败：{}\n（WebDAV 服务已停止，可手动执行 net use {} /delete）",
                        session.drive, e, session.drive
                    ),
                }
            } else {
                "（未建立盘符映射）".to_string()
            };
            session.server.abort();
            Ok(format!(
                "已锁定（127.0.0.1:{} 服务已停止）。{}\n\
                 密文容器仍在原路径；拷贝走仍是保险柜，但不会自动变明文。",
                session.port, unmap
            ))
        }
        None => Ok("当前没有运行中的挂载会话。".into()),
    }
}

/// 查询是否有运行中的挂载会话。
#[tauri::command]
fn status(state: State<'_, MountState>) -> Result<bool, String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;
    Ok(guard.is_some())
}

/// 桌面壳 → 内核的真实集成点：对密文容器做结构检查（不解密、不解锁）。
#[tauri::command]
fn verify(root: String) -> Result<String, String> {
    if let Some(msg) = human_missing_vault(&root, std::path::Path::new(&root)) {
        return Err(msg);
    }
    let r = mylock_core::verify_phase1(std::path::Path::new(&root))
        .map_err(|e| format!("{}: {}", e.code, e.detail))?;
    Ok(serde_json::to_string(&serde_json::json!({
        "result": r.result,
        "objects": r.objects,
        "bytes": r.bytes,
    }))
    .unwrap())
}

/// 端口占用探测（供前端在解锁前给出可诊断提示）。
#[tauri::command]
fn probe_port(port: Option<u16>) -> bool {
    port_available(port.unwrap_or(4918))
}

fn main() {
    tauri::Builder::default()
        .manage(MountState::default())
        .invoke_handler(tauri::generate_handler![
            create_vault,
            open_vault,
            close_vault,
            status,
            verify,
            probe_port
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ---------------------------------------------------------------------------
// M5 GUI 完善的真实代码路径测试（不是「照着 CLI 复刻一遍」的旁证）：
// 直接调用 `create_vault` / `decorate_vault` / `human_missing_vault` 本身。
// 用 cargo test 跑；发布构建里这些测试被 cfg(test) 完全剔除。
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// 清掉 attrib 再删目录，避免 +s +h 让清理变麻烦（probe-attrib 实测）。
    #[cfg(not(windows))]
    fn undecorate_for_cleanup(_root: &Path) {}

    #[cfg(windows)]
    fn undecorate_for_cleanup(root: &Path) {
        for (name, is_dir) in [
            ("desktop.ini", false),
            ("MyLockVault.ico", false),
            ("vault.meta", false),
            ("vault.meta.bak", false),
            ("manifest", false),
            ("README.txt", false),
            ("d", true),
        ] {
            let p = root.join(name);
            if p.exists() {
                let _ = set_attrib(&p, &["-s", "-h"], is_dir);
            }
        }
        let _ = set_attrib(root, &["-s"], true);
    }

    fn temp_root(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("mylock_gui_test_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn create_vault_builds_decorated_vault_that_still_verifies() {
        let root = temp_root("create");
        let r = create_vault(root.to_string_lossy().into_owned(), "Test-Pw-2026".into())
            .expect("create_vault must succeed on a fresh path");

        // 内核侧：容器确实被建出来了，且 verify_phase1（只扫 d/）仍通过。
        assert!(root.join("vault.meta").exists(), "vault.meta missing");
        assert!(root.join("manifest").exists(), "manifest missing");
        assert!(root.join("d").is_dir(), "d/ missing");
        assert_eq!(r.key_epoch, 0);
        assert!(r.recovery_key.starts_with("E000-"), "rk={}", r.recovery_key);
        assert!(!r.jti.is_empty());
        assert!(r.message.contains("Recovery Key"));
        let v = verify(root.to_string_lossy().into_owned())
            .expect("verify must succeed after create");
        assert!(v.contains("\"result\":\"OK\"") || v.contains("\"result\": \"OK\""), "verify={}", v);

        // 装饰侧（Windows）：desktop.ini 必须是 UTF-16LE+BOM；图标字节与内嵌资源一致。
        #[cfg(windows)]
        {
            let ini = std::fs::read(root.join("desktop.ini")).expect("desktop.ini missing");
            assert_eq!(&ini[0..2], &[0xFF, 0xFE], "desktop.ini must be UTF-16LE+BOM");
            let icon = std::fs::read(root.join("MyLockVault.ico")).expect("ico missing");
            assert_eq!(icon.as_slice(), VAULT_ICON, "icon bytes differ from include_bytes!");
            let ini_txt = String::from_utf16_lossy(
                &ini[2..]
                    .chunks(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect::<Vec<u16>>(),
            );
            assert!(ini_txt.contains("IconFile=MyLockVault.ico"));
            assert!(ini_txt.contains("IconIndex=0"));
            assert!(!ini_txt.contains("ConfirmFileOp"), "must NOT disable delete warning");
            assert!(root.join("⚠ 请勿删除.txt").exists(), "visible warning missing");
            // decorate 是幂等的：再跑一次不应破坏容器，verify 仍绿。
            let again = decorate_vault(&root);
            assert!(again.contains("已应用"), "replay={}", again);
            let v2 = verify(root.to_string_lossy().into_owned()).expect("verify after replay");
            assert!(v2.contains("OK"), "verify after replay={}", v2);
        }

        undecorate_for_cleanup(&root);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn create_vault_rejects_nonempty_existing_dir() {
        let root = temp_root("nonempty");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("keep.txt"), b"do not delete").unwrap();
        let e = create_vault(root.to_string_lossy().into_owned(), "pw".into())
            .expect_err("must refuse to wipe a non-empty dir");
        assert!(e.contains("已存在且非空") || e.contains("清空"), "err={}", e);
        assert!(root.join("keep.txt").exists(), "existing file must survive");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn missing_vault_gets_human_hint_not_raw_io_error() {
        let root = temp_root("missing");
        let msg = human_missing_vault("E:\\noSuchVault", &root)
            .expect("a path without vault.meta should be reported");
        assert!(msg.contains("新建保险柜"), "msg={}", msg);
        assert!(msg.contains("找不到 vault.meta"));
        assert!(!msg.contains("os error 3"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn humanize_open_error_translates_auth_failure() {
        let e = CoreError { code: "E_AUTH_FAILED", detail: "aes-kw unwrap: integrity check failed".into() };
        let s = humanize_open_error(e);
        assert!(s.contains("不正确"));
        assert!(s.contains("Recovery Key"));
        assert!(s.contains("E_AUTH_FAILED"));
    }
}
