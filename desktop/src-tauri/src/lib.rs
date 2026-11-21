// MyLock 移动端（Android）Tauri 库入口。
//
// 这里只放**跨平台、无桌面挂载语义**的命令：建库、校验、解锁列目录、导出明文、
// 锁定、以及 G4 一致性自检。真正的密码学全部委托给 mylock-core（同一份实现，
// 不另写一套）；桌面 main.rs 的 net use / 盘符映射 / WebDAV loopback 在这里刻意不复用。
//
// 安全边界（与 android-feasibility.md §3 一致，务必保持）：
// - 跨端主密钥 = 可移植软件密钥（口令经 argon2id(m=65536,t=3,p=4) 派生）。
// - 生物识别 / Android Keystore 只在本机 wrap 一份「口令字节副本」，口令兜底必须可用。
// - 本模块**不**在 Kotlin/Java 层重实现任何密码学；生物识别解出的口令经 command 传入。
// - 解锁态只在进程内保存口令字节（用于后续读导出），锁定即清零；不解密到磁盘。
// - G4 全中之前 Android 仍为 🔴 未验收，封闭内测，不是发布/审计。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use mylock_core::g4::{g4_verify, G4Report};

/// Kotlin 生物识别插件（app.tauri.biometric）通过 run_mobile_plugin 返回的载荷：
/// base64(口令原始字节)。解密出的口令只在 Rust 侧补 oneUnlock 用，绝不回传前端。
#[derive(Deserialize)]
struct BioSecret {
    password: String,
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    fn d(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = s.bytes().filter(|b| *b != b'=').collect();
    if bytes.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let mut buf = [0u8; 3];
        let n = chunk.len();
        let v: Option<Vec<u8>> = chunk.iter().map(|&c| d(c)).collect();
        let v = v?;
        let acc = (v[0] as u32) << 18
            | (if n > 1 { (v[1] as u32) << 12 } else { 0 })
            | (if n > 2 { (v[2] as u32) << 6 } else { 0 })
            | (if n > 3 { v[3] as u32 } else { 0 });
        buf[0] = (acc >> 16) as u8;
        if n > 2 { buf[1] = (acc >> 8) as u8; }
        if n > 3 { buf[2] = acc as u8; }
        out.extend_from_slice(&buf[..n - 1]);
    }
    Some(out)
}

/// 把内核错误摊成人话（code: detail），不吞错误码，便于 G4/诊断定位。
fn ce(e: mylock_core::CoreError) -> String {
    format!("{}: {}", e.code, e.detail)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResult {
    pub root: String,
    pub jti: String,
    pub key_epoch: u32,
    pub recovery_key: String,
    pub message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryInfo {
    pub id: String,
    pub name: String,
    pub size: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockResult {
    pub root: String,
    pub records: usize,
    pub orphans: Vec<String>,
    pub entries: Vec<EntryInfo>,
    pub message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatus {
    pub unlocked: bool,
    pub root: Option<String>,
    pub records: usize,
}

/// 进程内解锁会话：只保存尚未派生主密钥的「口令原始字节」。
/// 不把 master/mac_key 长期驻留——每次读都走 full_open / export_object_by_id
/// 由内核重新派生（Argon2id 在手机上约 1-2 秒，MVP 可接受，换来的是锁定即清零）。
/// 口令在 Rust 侧也只是 Vec<u8>，lock 时显式覆盖清零。
struct OpenSession {
    root: PathBuf,
    secret: Vec<u8>,
    slot: String,
    records: usize,
}

#[derive(Default)]
struct SessionState(Mutex<Option<OpenSession>>);

impl SessionState {
    fn clear(&self) {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(mut s) = guard.take() {
            for b in s.secret.iter_mut() {
                *b = 0;
            }
            s.root = PathBuf::new();
            s.slot.clear();
        }
    }
}

fn session_error() -> String {
    "保险柜尚未解锁，请先用口令或生物识别解锁。".to_string()
}

/// 建库：与桌面/CLI 完全同一条内核路径（random_bytes → random_vault_params →
/// build_vault → write_vault → rk_display）。Android 侧 root 由 Kotlin 传入一个
/// 应用私有目录（如 filesDir/vaults/<name>），避免 SAF URI 在 Rust 侧无法直接 open。
#[tauri::command]
fn create_vault(root: String, password: String) -> Result<CreateResult, String> {
    if root.trim().is_empty() {
        return Err("请填写保险柜目录名。".into());
    }
    if password.is_empty() {
        return Err("请填写主口令。".into());
    }
    let root_path = PathBuf::from(root.trim());
    if root_path.exists() {
        let non_empty = std::fs::read_dir(&root_path)
            .map(|mut it| it.next().is_some())
            .unwrap_or(true);
        if non_empty {
            return Err("目标目录已存在且非空；新建会清空它，请换一个空目录名。".into());
        }
    }
    let rk = mylock_core::random_bytes(32).map_err(ce)?;
    let params = mylock_core::random_vault_params(password.as_bytes(), &rk, 0).map_err(ce)?;
    let jti = params.jti.clone();
    let built = mylock_core::build_vault(&params, &[]).map_err(ce)?;
    mylock_core::write_vault(&root_path, &built).map_err(ce)?;
    let recovery_key = mylock_core::rk_display(&rk, 0);

    Ok(CreateResult {
        root: root_path.to_string_lossy().into_owned(),
        jti,
        key_epoch: 0,
        recovery_key: recovery_key.clone(),
        message: format!(
            "已新建保险柜：{}\n\
             ★ 请立刻离线抄存 Recovery Key：{}\n\
                （忘记口令时唯一的兜底）\n\
             ⚠ 未经第三方审计、未签名；封闭内测，请勿存放唯一副本。",
            root_path.display(),
            recovery_key
        ),
    })
}

/// 返回应用数据目录（Android 上为 filesDir），前端据此拼出应用私有的保险柜路径。
/// 保险柜必须放在应用私有目录，避免 SAF URI 在 Rust 侧无法直接 open，
/// 也避免被系统媒体扫描/云同步意外上传。
#[tauri::command]
fn app_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    Ok(dir.to_string_lossy().into_owned())
}

/// 结构校验（不解密）：给「导入一个已有保险柜目录」时先用。
#[tauri::command]
fn verify_vault(root: String) -> Result<String, String> {
    let p = mylock_core::verify_phase1(Path::new(&root.trim())).map_err(ce)?;
    Ok(format!(
        "结构校验通过：对象 {} 个，密文字节 {} B。",
        p.objects, p.bytes
    ))
}

/// 解锁内核：口令/恢复码两路以及生物识别解出的口令，都归并到这里，
/// 保证「同一份口令 → 同一条 full_open 路径 → 同一会话结构」。
fn unlock_with_secret(
    root_path: &Path,
    secret: Vec<u8>,
    slot: String,
    state: &SessionState,
) -> Result<UnlockResult, String> {
    if secret.is_empty() {
        return Err("口令不能为空。".into());
    }
    let opened = mylock_core::full_open(root_path, &secret, &slot).map_err(ce)?;
    let mut entries: Vec<EntryInfo> = opened
        .records_raw
        .iter()
        .filter_map(|r| {
            let id = r.get("id")?.as_str()?.to_string();
            let name = r.get("name")?.as_str()?.to_string();
            let size = opened.exported.get(&name).map(|v| v.len()).unwrap_or(0);
            Some(EntryInfo { id, name, size })
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(OpenSession {
        root: root_path.to_path_buf(),
        secret,
        slot,
        records: opened.records,
    });

    Ok(UnlockResult {
        root: root_path.to_string_lossy().into_owned(),
        records: opened.records,
        orphans: opened.orphans,
        entries,
        message: format!("已解锁 {} 个文件。", opened.records),
    })
}

/// 解锁：调 full_open 校验口令 + 完整解密重建明文（不写盘），成功后把口令存入会话。
/// slot 二选一："password" / "recovery"。
#[tauri::command]
fn unlock_vault(
    root: String,
    password: String,
    slot: Option<String>,
    state: State<'_, SessionState>,
) -> Result<UnlockResult, String> {
    let slot = slot.filter(|s| !s.is_empty()).unwrap_or_else(|| "password".into());
    unlock_with_secret(&PathBuf::from(root.trim()), password.into_bytes(), slot, &state)
}

// ── 生物识别解锁 ────────────────────────────────────────────────────────────
// Kotlin 侧的自定义 Tauri mobile plugin（包名 app.tauri.biometric，类名
// "BiometricPlugin"）用 Android Keystore 的 AES-GCM 密钥 wrap 一份「口令字节」，
// 经 BiometricPrompt(CryptoObject) 解出后回传 base64(口令字节)。Rust 侧只做
// base64 解码 → 复用 unlock_with_secret → 同一份 full_open，绝不在 Rust/Kotlin
// 重现任何密码学原语，也不把口令/主密钥回传前端。

/// 首次启用生物识别：Kotlin 用 K_dev 加密口令字节并持久化；Rust 只是转发。
#[cfg(mobile)]
#[tauri::command]
fn biometric_enroll(
    root: String,
    password_b64: String,
    bio: State<'_, BioPluginHandle>,
) -> Result<String, String> {
    if root.trim().is_empty() || password_b64.trim().is_empty() {
        return Err("保险柜路径与口令均不能为空。".into());
    }
    let handle = bio.0.as_ref()
        .ok_or_else(|| "生物识别插件未初始化。".to_string())?;
    let _: serde_json::Value = handle.run_mobile_plugin(
        "enroll",
        serde_json::json!({ "root": root.trim(), "passwordB64": password_b64.trim() }),
    ).map_err(|e| format!("启用生物识别失败：{}", e))?;
    Ok("已启用生物识别。口令副本由 Android Keystore 密钥本机加密保存；增删指纹后需重新启用。".into())
}

/// 生物识别解锁：Kotlin 解出 wrap 的口令字节回传，Rust 解码后走与手输口令相同的完整路径。
#[cfg(mobile)]
#[tauri::command]
fn biometric_unlock(
    root: String,
    state: State<'_, SessionState>,
    bio: State<'_, BioPluginHandle>,
) -> Result<UnlockResult, String> {
    if root.trim().is_empty() {
        return Err("请填写保险柜路径。".into());
    }
    let handle = bio.0.as_ref()
        .ok_or_else(|| "生物识别插件未初始化。".to_string())?;
    let secret_b64: BioSecret = handle
        .run_mobile_plugin("unlock", serde_json::json!({ "root": root.trim() }))
        .map_err(|e| format!("生物识别失败：{}", e))?;
    let secret = base64_decode(&secret_b64.password)
        .filter(|b| !b.is_empty())
        .ok_or_else(|| "生物识别返回了无效的口令数据。".to_string())?;
    unlock_with_secret(&PathBuf::from(root.trim()), secret, "password".into(), &state)
}

/// 桌面/非移动构建：前端不应调用，给出明确错误而不是编译失败。
#[cfg(not(mobile))]
#[tauri::command]
fn biometric_enroll(_root: String, _password_b64: String) -> Result<String, String> {
    Err("生物识别仅支持 Android 移动端。".into())
}

#[cfg(not(mobile))]
#[tauri::command]
fn biometric_unlock(_root: String) -> Result<UnlockResult, String> {
    Err("生物识别仅支持 Android 移动端。".into())
}

/// 列目录：用当前会话重新 full_open，返回名字/大小/id（不导出明文内容）。
#[tauri::command]
fn list_entries(state: State<'_, SessionState>) -> Result<Vec<EntryInfo>, String> {
    let guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
    let s = guard.as_ref().ok_or_else(session_error)?;
    let opened = mylock_core::full_open(&s.root, &s.secret, &s.slot).map_err(ce)?;
    let mut entries: Vec<EntryInfo> = opened
        .records_raw
        .iter()
        .filter_map(|r| {
            let id = r.get("id")?.as_str()?.to_string();
            let name = r.get("name")?.as_str()?.to_string();
            let size = opened.exported.get(&name).map(|v| v.len()).unwrap_or(0);
            Some(EntryInfo { id, name, size })
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// 读取单个对象明文：走 export_object_by_id（不读 manifest 的独立恢复路径），
/// 返回 base64，由前端决定按文本还是二进制展示。从不把明文写回磁盘。
#[tauri::command]
fn read_object(id: String, state: State<'_, SessionState>) -> Result<String, String> {
    let guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
    let s = guard.as_ref().ok_or_else(session_error)?;
    let plain = mylock_core::export_object_by_id(&s.root, &s.secret, &s.slot, id.trim()).map_err(ce)?;
    Ok(base64_encode(&plain))
}

fn base64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | data[i + 2] as u32;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(T[((n >> 6) & 63) as usize] as char);
        out.push(T[(n & 63) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push_str("==");
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(T[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

/// 锁定：清零口令、清空会话。
#[tauri::command]
fn lock_vault(state: State<'_, SessionState>) -> SessionStatus {
    state.clear();
    SessionStatus { unlocked: false, root: None, records: 0 }
}

#[tauri::command]
fn session_status(state: State<'_, SessionState>) -> SessionStatus {
    let guard = state.0.lock().unwrap_or_else(|e| e.into_inner());
    match guard.as_ref() {
        Some(s) => SessionStatus {
            unlocked: true,
            root: Some(s.root.to_string_lossy().into_owned()),
            records: s.records,
        },
        None => SessionStatus { unlocked: false, root: None, records: 0 },
    }
}

/// G4 一致性自检：内置冻结的 test-vectors.json，返回结构化报告。
/// run_negative=true 时需要一个可写 work_dir（由 Kotlin 传 cacheDir）；
/// 纯 derived + 指纹路径不需要 work_dir，风险最低，优先跑。
#[tauri::command]
fn g4_self_check(run_negative: bool, work_dir: Option<String>) -> Result<G4Report, String> {
    const CONTRACT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/vectors/test-vectors.json"
    ));
    let wd = if run_negative {
        work_dir.filter(|s| !s.is_empty()).map(PathBuf::from)
    } else {
        None
    };
    g4_verify(CONTRACT, wd.as_deref()).map_err(ce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
    }

    /// 复核手机上的解锁路径：移动 create_vault（空对象）产出的保险柜，
    /// 必须能用同一口令经 unlock_with_secret → full_open 打开；这是
    /// 「生物识别解出的口令字节 → 与手输口令走同一条 full_open」成立的前提。
    #[test]
    fn mobile_create_then_unlock_with_secret_is_green() {
        let base = std::env::temp_dir().join(format!(
            "mylock-mobile-unlock-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let root = base.join("vaults/demo");
        let rk = mylock_core::random_bytes(32).unwrap();
        let params = mylock_core::random_vault_params(b"correct horse battery staple", &rk, 0).unwrap();
        let built = mylock_core::build_vault(&params, &[]).unwrap(); // 与 lib.rs create_vault 完全一致
        mylock_core::write_vault(&root, &built).unwrap();

        let state = SessionState::default();
        let r = unlock_with_secret(&root, b"correct horse battery staple".to_vec(), "password".into(), &state).unwrap();
        assert_eq!(r.records, 0, "新建空保险柜应解锁出 0 个文件");
        assert!(state.0.lock().unwrap().is_some(), "解锁后会话应保存 secret");

        // 错误口令必须失败（保证生物识别解出的字节确需正确）。
        let state2 = SessionState::default();
        assert!(unlock_with_secret(&root, b"wrong password".to_vec(), "password".into(), &state2).is_err());

        state.clear();
        assert!(state.0.lock().unwrap().is_none(), "clear 后会话应为空");
        let _ = std::fs::remove_dir_all(&base);
    }
}

/// 生物识别插件注册：Kotlin 侧类名 app.tauri.biometric.BiometricPlugin。
/// 只在 Android 注册；桌面构建给出空插件，命令本身已 #[cfg(not(mobile))] 报错。
/// 生物识别插件句柄（由 plugin setup 注入 state，命令通过 State 取用）。
/// 非 Android 构建用 None 占位，命令本身已报错，不会真正调用。
#[cfg(mobile)]
struct BioPluginHandle(Option<tauri::plugin::PluginHandle<tauri::Wry>>);

fn biometric_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("app.tauri.biometric")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            {
                let handle = api.register_android_plugin("app.tauri.biometric", "BiometricPlugin")?;
                app.manage(BioPluginHandle(Some(handle)));
            }
            #[cfg(not(target_os = "android"))]
            {
                let _ = (app, api);
            }
            Ok(())
        })
        .build()
}

/// 移动端入口（Tauri CLI 通过 `mobile` cfg 注入）。桌面二进制仍走 main.rs。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(biometric_plugin())
        .manage(SessionState::default())
        .invoke_handler(tauri::generate_handler![
            create_vault,
            app_data_dir,
            verify_vault,
            unlock_vault,
            list_entries,
            read_object,
            lock_vault,
            session_status,
            g4_self_check,
            biometric_enroll,
            biometric_unlock,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MyLock mobile application");
}
