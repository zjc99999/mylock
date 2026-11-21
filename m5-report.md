# M5 零额外开支研发收口 · 阶段报告

> 版本：v1.3 · 日期：2026-11-15 · 上游：[产品分析与概要设计.md](产品分析与概要设计.md) v1.9 / [m4-step4-report.md](m4-step4-report.md) §0 / §2.4.2 / [BETA-NOTICE.md](BETA-NOTICE.md)
> v1.1 变更：记录用户 m04223 报告的便携包「找不到 WebView2Loader.dll」真缺陷与热修复（见 §1.5）。
> v1.2 变更：记录用户 m04366 要求的[详细功能与操作说明 USER-GUIDE.md](USER-GUIDE.md)（以 `E:\testLock` 举例），已打进便携包（见 §1.6）。
> v1.3 变更：记录用户 m04697 授权的两项 GUI 完善（见 §1.7）——GUI「新建保险柜」按钮 + 使用前必要提示 + 密文容器文件夹提示性保护 / 保险柜图标；[USER-GUIDE.md](USER-GUIDE.md) 升 v1.1。当前最终包为 **8 文件 / 10208486 B**（见 §1.7 / §5）。
> 授权：用户 m03799「关于合规性调研，需要一笔不少的开支和时间成本，依本项目的现有条件，是无法做到的。能否在不花费额外的开支下，完成软件研发？之后，小范围用户测试、试用，有市场正向反馈再补齐相关手续。」
> **性质**：本报告是**工程与产品判断**，不是法律意见、不是合规结论、不是第三方审计报告、不是已通过认证。合规事实沿用 2026-11-15 抓取时点值。

---

## 0. 结论先行（两道闸门必须分清）

- **战略转向**：不再为合规调研投入额外现金/时间，先**零成本完成软件研发 + 封闭小范围试用**，拿到正向市场反馈后再补齐手续。这是产品与工程判断，**推迟 ≠ 豁免**：EAR 出口分类、PIPL/GDPR controller 定性、MPL-2.0 §3.2 源码可得义务不因规模小或免费而消失。
- **原冻结 NO-GO 措辞一字不动、只约束「公开发布 / 商店上架」**：A 第三方审计 / B 律师签发 / C 代码签名+可分发包 / D 商店申报四道外部闸门均仍 🔴（见 [prelaunch-checklist.md](prelaunch-checklist.md) §0 / §4）。这些**没有、也不该**在本轮被「翻绿」。
- **本轮另立「封闭内测闸门」**（见 [BETA-NOTICE.md](BETA-NOTICE.md)）：邀请制 ≤50 人、不公开下载链接、不收费、不做公开宣传、**未经第三方安全审计、未代码签名、请勿存放不可替代的唯一副本**；反馈只走主动渠道（访谈 / 问卷 / 用户主动发消息），不埋点、不上报、无崩溃上报、无账号、无云同步、无遥测。
- **授权范围（ask_user_question 结果）**：① 桌面端到端收口 ② 零成本合规最小动作 ③ Android 最小可用。**G4 跨平台绿灯未被选中、不在本轮授权内**（不擅自做）。iOS 因 Apple Developer $99/年明确暂缓。
- **三项授权全部完成**，但完成的定义是「可交付的桌面内测版 + 零成本合规准备 + Android 可行性设计」，**不是「可上架成品」**。当前对外发布状态仍为 **NO-GO**。

### 成本拆分（为什么可以零现金）

| 项目 | 是否花钱 | 本轮处理 |
|---|---|---|
| 第三方安全审计 | 数万～数十万，真花钱 | 未做，标为外部闸门 A |
| 律师签发 / 书面出口分类 / 逐依赖许可决定 | 数万+，真花钱 | 未做，标为外部闸门 B（只留模板与清单） |
| OV/EV 代码签名证书 | ~¥1–3k/年，真花钱 | 不买，免签名 + SmartScreen 警告可接受 |
| Apple Developer / Google Play / Microsoft 商店注册 | $99/年 / $25 / $99/年 | 不注册，不上架 |
| BIS 自我分类 / CCATS / §742.15(b) 邮件通知 | 政府不收费，只花时间 | 产出**未发出的模板**，先通知再开源的顺序坑已写明 |
| `cargo fetch` 补齐依赖 + THIRD_PARTY_NOTICES | 零现金 | 本轮已做完 |
| 开源加密核心 + 正确许可文件（MPL §3.2） | 零现金 | 未开源（对外发布动作，须先走 BIS 通知顺序），只完成本地告知材料 |
| 如实的内测版隐私政策 / 内测须知 | 零现金 | [privacy-policy.md](privacy-policy.md) v0.1（待签发）+ [BETA-NOTICE.md](BETA-NOTICE.md) 已就位 |

---

## 1. ① 桌面端到端收口（本轮核心工程交付）

### 1.1 实测证伪了 M3 的两个「从未真跑」的假设

[tools/e2e-probe.ps1](tools/e2e-probe.ps1)（端到端探针，造临时 vault 与 Z: 映射）第一次把整条链路**真实跑通**（日志 `tools/e2e-probe.log`，`PROBE_EXIT=0`）：

- `CREATE_EXIT=0` → `mylock-serve` PID 36808 / `PROPFIND_STATUS=207` → `net use Z: http://127.0.0.1:51919/` **`NETUSE_OUT=The command completed successfully.` / `NETUSE_EXIT=0`** → `Z_EXISTS=True` / Z 盘真实读写（`Z_READ_SEED=seed-from-disk-visible-only-when-unlocked`、`Z_READ_NEW=hello-from-Z-drive-2026`）→ 断开 → `mylock export` 证明写回落盘 → 整盘复制到另一路径 → `VERIFY_COPY_EXIT=0` 且 **`PLAINTEXT_HITS_IN_COPY=0`**。
- **实测反直觉**：`WEBCLIENT_BEFORE=Stopped/Manual` 且 `Start-Service WebClient` 失败，但 `net use` 仍成功 ⇒ 不能拿 WebClient 服务状态当判据（本机环境事实，非产品保证）。
- **结论**：「解锁 → 映射盘符 → 编辑 → 锁定 → 拔盘拷移动硬盘仍是保险柜（密文）」整条链路成立，且拷盘后明文命中为 0。M3 报告宣称的 `net use Z:` 此前只 curl 打过 WebDAV，本轮才实测映射盘符。

### 1.2 探针暴露并修掉的两个真缺陷

1. 旧桌面壳 `open_vault` spawn 外部 `mylock-serve` sidecar 且 `Stdio::null()` 丢弃输出 ⇒ 口令错 / 端口占用 / 云同步根拒绝时用户只看到「已启动」。
2. 旧 UI 只把 `net use Z: http://127.0.0.1:4918/` 当**文字打印给用户自己敲**，不是真映射。

### 1.3 桌面壳重写（`desktop/src-tauri/src/main.rs` + `desktop/ui/index.html` + `tauri.conf.json`）

- **进程内引擎**：不再 spawn sidecar，改为 `mylock_mount::build_handler(...)` + `TokioTcpListener::bind(SocketAddr::from(([127,0,0,1],port)))` + `rt.spawn(run_listener(handler, listener))`；`build_handler` 在映射盘符之前完成，使 `E_AUTH_FAILED` / `E_FORMAT` / `E_SYNC_ROOT` / 端口占用在返回前端前即可透传。专用 `static RUNTIME: OnceLock<tokio::runtime::Runtime>`（不依赖 Tauri 全局 runtime / thread-local Handle）。
- **自动盘符映射 / 断开**：`#[cfg(windows)] fn net_use`（`Command::new("net")`，CREATE_NO_WINDOW、回收 stdout+stderr、`status.success()` 判成败）；`normalize_drive()` 只接受单个 ASCII 字母防命令注入；`open_vault` 自动 `net use <drive> .../ /persistent:no`，失败则 `server.abort()` 回滚并返回可手敲命令；`close_vault` 先 `net use <drive> /delete /y` 再 `abort`。
- **凭据二选一**：口令或 `recovery_display_to_raw` 恢复槽；新增 `probe_port`；`verify` 仍调 `mylock_core::verify_phase1`。
- **UI 同步**：删 `serveBin` 输入与「引擎路径」按钮（不再有外部 sidecar），新增 recovery / drive / port 输入与「端口探测」按钮，`.warn` 写明「未经第三方审计·未签名·请勿存放不可替代唯一副本·封闭内测≠公开发布」。
- **配置**：`tauri.conf.json` 删 `bundle.externalBin`（进程内方案下冗余）、窗口标题改「M5 · 进程内引擎 · 封闭内测」、longDescription 写明全部诚实边界；保留 `bundle.active=true` / `targets=["nsis","msi"]` 仅为将来上架配置占位。
- **编译**：首次 `cargo check` EXIT=101（`([{integer};4],u16)` 不实现 `ToSocketAddrs`），改用 `SocketAddr::from(...)` + 专用 `OnceLock<Runtime>` 后 **`CHECK_EXIT=0`**（`Finished dev profile in 1.01s`）。

### 1.4 打包工具链现实约束 → 免签名 standalone exe

实测本机 `cargo-tauri` / `makensis` / WiX `candle` / `signtool` 全缺，NSIS/WiX 目录全 False；但 **WebView2 运行时已装**（`C:\Program Files (x86)\Microsoft\EdgeWebView\Application`）⇒ 决策：**零成本内测包 = `cargo build --release` 免签名 standalone exe + portable ZIP**，不产 NSIS/MSI、不买证书、SmartScreen 警告可接受。

| 产物 | 路径 | 大小 | 验证 |
|---|---|---|---|
| 桌面 GUI | `desktop/src-tauri/target/release/mylock-desktop.exe` | 25216083 B | release 构建 EXIT=0；**强启动判据见 §1.5**（曾误用 `ALIVE_AFTER_3S=True` 弱判据，缺 DLL 时错误框仍让进程存活） |
| 独立恢复 CLI | `core/target/release/mylock.exe` | 2307675 B | release 构建 EXIT=0 |
| 独立 serve（备用） | `core/target/release/mylock-serve.exe` | 6389054 B | release 构建 EXIT=0 |

内核/VFS/WebDAV 层已有 7 个 Rust 单测（mount.rs），桌面壳「自动 net use + 错误码透传」无 Rust 单测、靠 e2e-probe.ps1 实测（诚实标注）。

### 1.5 热修复：便携包漏带 `WebView2Loader.dll`（用户报告 m04223）

- **User said (m04223): "Windows主程序mylock-desktop.exe运行报错：由于找不到 WebView2Loader.dll，无法继续执行代码。重新安装程序可能会解决此问题。"** 这是真运行时缺陷：用户解压后运行 GUI 弹系统缺 DLL 错误框。此前 §1.4 表格里的 `ALIVE_AFTER_3S=True` 是**过弱的冒烟判据**——缺 DLL 时进程会挂在系统错误框上仍「存活」，不能证明窗口起来，本轮据此推翻该判据。
- **根因（A 级本地证据）**：本项目构建目标为 `x86_64-pc-windows-gnu`（非 MSVC）。`webview2-com-sys 0.39.1` 的 `src/lib.rs:13-27` 中 `link_webview2!` 宏对 `target_env = "msvc"` 用 `#[link(name="WebView2LoaderStatic", kind="static")]` 静态链接，对 `not(target_env = "msvc")` 用 `#[link(name="WebView2Loader.dll")]` **动态链接**；`src/bindings.rs` 的 `CreateCoreWebView2Environment` / `GetAvailableCoreWebView2BrowserVersionString` 等经 `link!("webview2loader" "system" ...)` 导入。`build.rs` 把 x64/x86/arm64 三份 DLL 复制到 OUT_DIR 并按 `CARGO_CFG_TARGET_ARCH` 设 link-search。`C:\mingw64\bin\objdump.exe -p mylock-desktop.exe` 的导入表确含 `DLL Name: WebView2Loader.dll`（其余均为系统 DLL）。⇒ **GNU 产物必须把 `WebView2Loader.dll` 与 exe 放在同一目录；MSVC 构建才不需要。**
- **缺失点**：[tools/assemble-beta.ps1](tools/assemble-beta.ps1) 首版只拷 3 个 exe + 2 个 md + 写 README，`dist\MyLock-Beta-0.1.0\` 6 文件与 ZIP 均无该 DLL。release 目录本身**已有** x64 版（160880 B，SHA256 `86545B66CDB0603BC26B626FB9AD610CB6E71F28D468F5EA66DF23B03DDA96D5`，与 registry `x64\WebView2Loader.dll` / `target\release\build\...\out\x64\` 三者哈希一致），只是没被拷进包。
- **修复（已落盘）**：脚本新增拷贝该 DLL + 源/目标 SHA256 一致性断言 + **导入表门禁**（`objdump` 列出 exe 所有非系统 `DLL Name:`，逐个断言 stage 内存在同名文件，缺一即 `exit 4`）+ README 区分「随包的 WebView2Loader.dll 加载器」与「系统需装的 Evergreen Runtime」并写明「必须整目录拷贝、勿只发单个 exe」。
- **重跑结果**：`ASSEMBLE_BETA_OK` / `LOADER_SHA256=86545B66…` / `IMPORT_DLL=WebView2Loader.dll STAGED=True` / `IMPORT_GATE=PASS`；stage 由 6 文件变 7 文件（新增 `WebView2Loader.dll 160880 B`），ZIP 由 10087482 B（6 文件、漏 DLL）→ 首次修复 10168745 B → **同步 [BETA-NOTICE.md](BETA-NOTICE.md) §3 整目录拷贝说明后热修复当时包 10169169 B / 7 entries**（此为该步时点值，后在 §1.6 随 USER-GUIDE.md 入包升级为 8 文件 / 10181612 B）；`EXE_IDENTICAL=True` / `LOADER_IDENTICAL=True`（DLL 与 exe 逐字节未变，仅文档字节变化）。
- **真正的验证（不再是进程存活）**：从修复后的 `dist\MyLock-Beta-0.1.0\mylock-desktop.exe` 启动 5 秒后 `ALIVE=True` 且 `MAIN_WINDOW_COUNT=1`，窗口标题 `MyLock · 透明加密文件夹（M5 · 进程内引擎 · 封闭内测）`（`EnumWindows` 该 PID 可见窗口 2 个）；**负向对照**：只拷 exe 不带 DLL 到干净目录 → `NEG_VISIBLE_WINDOWS=0`（窗口起不来，复现故障），负向目录已清理。这才证明修复有效。
- **对「发出去的那个 ZIP」本身做最终强验证（[tools/verify-beta-final.ps1](tools/verify-beta-final.ps1)，可重放）**：把**热修复当时的 ZIP**（10169169 B / 7 entries）解压到一个全新干净目录（而非只看 stage 目录）→ 校验 7 entries 含 `WebView2Loader.dll` 与 `mylock-desktop.exe`、解压出的 DLL SHA256 = `86545B66…`（与源一致）→ 从该解压目录启动 → `VISIBLE_WINDOWS=1` / `WINDOW=… TITLE=MyLock · 透明加密文件夹（M5 · 进程内引擎 · 封闭内测）` / `MAIN_WINDOW_TITLE_OK=True` / `STOPPED_OK=True` / `CLEANED=True` / `VERIFY_BETA_FINAL=PASS` / `FINAL_VERIFY_EXIT=0`。**这直接证明了用户实际会拿到的压缩包解压后能弹出主窗口，而不是只在开发目录里能跑。**（此后该脚本又随 §1.6 增加 `HAS_GUIDE` 断言，对当前 8 文件最终包重跑仍 PASS，见 §1.6。）
- **诚实标注**：这是分发打包缺陷，**不是加密/内核缺陷**；免签名、未审计、未产 NSIS/MSI、封闭内测≠公开发布等边界不变。

### 1.6 详细功能与操作说明（用户 m04366，v1.2 新增）

- **User said (m04366): "我看了dist文件夹下的readme-beta.md，但对于现有打包的程序能干什么、怎么测试还是不太清楚，你能写一个详细点的软件功能和操作说明文档吗？比如我在E盘有个文件夹E:\testLock，里面有些文档，我可否将此文件夹加密、解密，你可以在说明文档中举例解释有关功能。"**
- **交付**：[USER-GUIDE.md](USER-GUIDE.md) v1.0（12 节 + §0 直接回答，约 448 行），并打进便携包。核心诚实结论 = **不是原地加密、没有「加密 / 解密」按钮**：真实模型是「新建一个空 vault 目录 → 解锁映射成 `Z:` 盘 → 把内容写进 / 拖进 `Z:` 即自动加密落盘 → 锁定即回密文态」；原 `E:\testLock` 始终是明文、MyLock 不删它。
- **全部行为由 3 个真实探针实测（%TEMP% 模拟，绝不碰用户真实 `E:\testLock`）**：[tools/probe-userguide.ps1](tools/probe-userguide.ps1)（9 命令 / 错口令 `E_AUTH_FAILED` / `change-password` 零吊销 / `rotate-keys` 真吊销 / **`create` 到已存在非空目录会先清空** / `--file` 传目录报 `E_FORMAT`）、[tools/probe-folder-workflow.ps1](tools/probe-folder-workflow.ps1)（子目录写回 + `PLAINTEXT_HITS=0`）、[tools/probe-nested-folder.ps1](tools/probe-nested-folder.ps1)（整文件夹拖进 `Z:` 得 5 记录、`export` 摊平）。数字校准用 [tools/probe-guide-numbers.ps1](tools/probe-guide-numbers.ps1)：`jti : a76394c5-…`、`{"phase":"pre-unlock","result":"OK","objects":3,"bytes":278}`（此前文档 `bytes:201` 是另一次 2 文件探针的错值，已改）。
- **关键实测限制（写进指南，不粉饰）**：`create` 目标必须全新空路径（`write_vault` 里 `if root.exists() { fs::remove_dir_all(root) }`）；`export` 按对象 `name` 平铺写入、不还原目录树（`full_open` 把 `name → Vec<u8>` 放 `BTreeMap`、CLI 用 `out.join(name)`），目录被导成 0 字节文件、同名覆盖 → 要目录树就解锁后从 `Z:` 拷；`export-object` 不读 manifest 可单对象抢救；`E_SYNC_ROOT` 拒绝云同步根；`change-password` / `rotate-recovery` 零吊销、`rotate-keys` 才真吊销（须物理销毁旧介质）。
- **组装脚本再修**：[tools/assemble-beta.ps1](tools/assemble-beta.ps1) 把 `USER-GUIDE.md` 纳入随包文档、README-BETA.md 首行指向指南；同时把「先删整个 stage 目录」改为「只清 stage 内容、不删被用户资源管理器锁住的目录根」——重建时曾因用户正打开 `dist` 文件夹导致 `Remove-Item` 报「being used by another process」，改后重跑 `ASSEMBLE_EXIT=0`。
- **最终包与强验证**：ZIP 由 `10169169 B / 7 entries` → **`10181612 B / 8 entries`**（新增 `USER-GUIDE.md 27754 B`、README-BETA.md 4398 B；指南头部「8 文件包」口径与示例数字校准后 ZIP 字节数逐次微调，最终 10181612 B）；`IMPORT_GATE=PASS` / `LOADER_SHA256=86545B66…`。[tools/verify-beta-final.ps1](tools/verify-beta-final.ps1) 增加 `HAS_GUIDE` 断言后重跑 → `ZIP_ENTRIES=8` / `HAS_LOADER=True HAS_GUI=True HAS_GUIDE=True` / 解压 DLL SHA256 一致 / `VISIBLE_WINDOWS=1` / `MAIN_WINDOW_TITLE_OK=True` / `VERIFY_BETA_FINAL=PASS` / `FINAL_VERIFY_EXIT=0`。**证明用户拿到的那个压缩包解压后既含指南、又能弹出主窗口。**
- **诚实边界不变**：指南是操作说明，不是合规结论 / 第三方审计 / 法律意见；免签名、未审计、请勿存唯一副本、封闭内测 ≠ 公开发布、推迟 ≠ 豁免、未产 NSIS/MSI 与 APK、合规事实 2026-11-15 时点值。

### 1.7 M5 GUI 完善（用户 m04697，v1.3 新增）

- **User said (m04697): "按照step1-4跑了一边，跑通了，需要做一些完善：1. 按照你说的在 GUI 加一个「新建保险柜」按钮，把 Step 1 收进界面，且在用户使用前做出必要的提示。2. 创建的的密码柜（文件夹）下，会默认多出很多文件和文件夹，为了防止用户删除而导致失效，可否采用一些措施吗？比如隐藏、删除更改提醒等，另外可否给这个文件夹（保密柜）改变显示图标，变成一个保险柜的icon，这样用户更好理解和使用。"**（此前的 m04638「`E_FORMAT: io: 系统找不到指定的路径。 (os error 3)`」已由 §1.6 的指南 + 本轮人话预检直接消解）
- **① GUI 新建保险柜**：[desktop/src-tauri/src/main.rs](desktop/src-tauri/src/main.rs) 新增 `#[tauri::command] fn create_vault(root, password) -> Result<CreateResult, String>`，与 CLI 同一份内核实现复刻（`random_bytes(32)` → `random_vault_params(pw.as_bytes(), &rk, 0)` → `build_vault(&params, &[])` → `write_vault(&root_path, &c)` → `rk_display(&rk, 0)`），返回 `jti` / `key_epoch:0` / `recovery_key` / `decoration` / `message`；**前置校验** = 非空 / 口令非空 / `mylock_mount::is_sync_root` 拒绝云同步根 / `parent().is_none()` 拒绝盘符根 / **已存在且 `read_dir` 非空则拒绝**（避免 `write_vault` 的 `remove_dir_all` 误删用户数据）。[desktop/ui/index.html](desktop/ui/index.html) 新增 `.mustread` 红框「⚠ 使用前必读」6 条 + `h2 ① 第一次使用：新建保险柜` + 橙色 `#createBtn`（`window.confirm` 弹出「使用前必要提示」5 条，含显示目标路径、会清空已存在目录、拒绝云同步根、抄存 RK、勿存唯一副本）+ `.rk` 绿色 Recovery Key 醒目面板（带复制按钮）。**人话错误预检** `human_missing_vault`：拿没建库的路径去点「解锁并映射」/「结构校验」直接给「路径 … 下还没有保险柜（找不到 vault.meta），请先点『新建保险柜』…不是原地加密」指引，取代底层 `E_FORMAT: io: … (os error 3)`；`humanize_open_error` 把 `E_AUTH_FAILED` / `E_SYNC_ROOT` 翻成人话并附底层串。
- **② 文件夹「提示性」保护 + 保险柜图标**：A 级依据 [Microsoft Learn：desktop.ini 文件夹自定义](https://learn.microsoft.com/en-us/windows/win32/shell/how-to-customize-folders-with-desktop-ini)（三步 = `attrib +s FolderName` 启用特殊行为 / Desktop.ini 标 hidden+system / **须 Unicode 格式**；`IconFile` 相对路径 + `IconIndex=0`；`ConfirmFileOp=0` 是「关闭」删除警告 ⇒ **刻意不写以保留「删除系统文件夹」警告**）。已生成 [tools/make_vault_icon.py](tools/make_vault_icon.py) → [desktop/src-tauri/icons/MyLockVault.ico](desktop/src-tauri/icons/MyLockVault.ico)（7955 B / 16·32·48·64·128·256 六尺寸，`vision_describe` 确认「非常清晰地描绘了一个保险柜」），用 `static VAULT_ICON: &[u8] = include_bytes!("../icons/MyLockVault.ico")` **编译期嵌入 exe、不在包里多一个文件**。`fn decorate_vault(root) -> String`（best-effort 幂等）：写 UTF-16LE+BOM 的 `desktop.ini`（`[.ShellClassInfo]` + `IconFile=MyLockVault.ico` + `IconIndex=0` + 中文 `InfoTip`）、写图标、写**可见** `⚠ 请勿删除.txt` 中文警示、`attrib +s /d` 根目录、`attrib +s +h` 逐项隐藏 `desktop.ini / MyLockVault.ico / vault.meta / vault.meta.bak / manifest / README.txt / d(/d)`。
- **三个决定性实测/读码依据**：① `rewrite_vault_incremental`（`core/crates/mylock-core/src/lib.rs:1527-1691`）只写 `d/<shard>/<oid>`、原子 rename 覆盖 `manifest`、清理孤儿只在 `d/`；`verify_phase1` 的 `list_object_files` 只扫 `d/` ⇒ **根级装饰文件永不被写回/校验误伤**；② [tools/probe-attrib.ps1](tools/probe-attrib.ps1) 实测 `+s +h` 不阻止 `fs::read`/`write`/`rename`，但 **`manifest` 每次被 rename 覆盖会丢 `+h +s` 属性** ⇒ 故 `open_vault` 成功解锁后**每次幂等重放** `decorate_vault`（自愈，失败只告警、绝不阻断解锁）；③ `attrib` 必须**逐路径单独调用**（一次多路径报 `Parameter format not correct -`）。
- **★ 诚实边界（本轮最重要）**：这些是 **shell 层「提示性」保护，不是防删 / 权限（ACL）机制，不保证数据不丢失**；真正删掉整目录或 `vault.meta` / `manifest` / `d\` 里任何一个，保险柜照样失效。防误删的最终手段仍是**备份 + 请勿存唯一副本**。GUI、README-BETA.md、可见警示、`⚠ 使用前必读`、USER-GUIDE.md §10 均逐字写明这一点。
- **验证**：`cargo check` EXIT=0（无 warning）；`cargo test --bin mylock-desktop` **4 项真实代码路径单测全绿**（`create_vault_builds_decorated_vault_that_still_verifies` 断言 `desktop.ini` 前 2 字节 `FF FE`、图标字节 == `VAULT_ICON`、ini 含 `IconFile`/`IconIndex` 且不含 `ConfirmFileOp`、可见警示存在、重放后 `verify` 仍绿；`create_vault_rejects_nonempty_existing_dir`；`missing_vault_gets_human_hint_not_raw_io_error`；`humanize_open_error_translates_auth_failure`）；`cargo build --release` `RELEASE_EXIT=0`（1m 00s，新 `mylock-desktop.exe` **25315893 B**）。[USER-GUIDE.md](USER-GUIDE.md) 升 v1.1（§4 Step 1 改为 GUI 主路径 + CLI 备用、§3 新建面板、§9 排错、§10 提示性保护非防删机制、§11 清单、§12 索引）；[tools/assemble-beta.ps1](tools/assemble-beta.ps1) 的 README-BETA.md 同步改为「GUI 新建保险柜」为主 + 新增 step5 文件夹保护说明。
- **重建便携包 + 强验证（当前 v1.3 最终包）**：`ASSEMBLE_EXIT=0` / `IMPORT_GATE=PASS` / `LOADER_SHA256=86545B66…` → **8 文件 / ZIP `10208486 B`**（`mylock-desktop.exe 25315893 B` / `BETA-NOTICE.md 8815 B` / `README-BETA.md 5742 B` / `USER-GUIDE.md 33041 B` / 其余同前）；[tools/verify-beta-final.ps1](tools/verify-beta-final.ps1) 对**最终 ZIP 解压后**启动 → `ZIP_ENTRIES=8` / `HAS_LOADER=HAS_GUI=HAS_GUIDE=True` / 解压 DLL SHA256 一致 / `VISIBLE_WINDOWS=1` / `MAIN_WINDOW_TITLE_OK=True` / `VERIFY_BETA_FINAL=PASS` / `FINAL_VERIFY_EXIT=0`。**全量回归** `tools/verify-m4s4.ps1` 重跑 `VERIFY_EXIT=0`（26/0 = 7 KAT + 4 conformance + 8 vectors + 7 mount、7 个 `STEP_EXIT=0`、门禁 5/5、指纹 `d45b8659…`）。
- **诚实边界不变**：本轮是封闭内测版 GUI 易用性改进，**未改变任何冻结的 NO-GO 措辞 / 四道外部闸门 / 加密内核 / 向量契约**；仍免签名、未审计、请勿存唯一副本、封闭内测 ≠ 公开发布、推迟 ≠ 豁免、未产 NSIS/MSI 与 APK、合规事实 2026-11-15 时点值。

---

## 2. ② 零成本合规最小动作（子代理 `6521fa0e` 交付，父代理追加口径补遗）

### 2.1 本地事实（A，`cargo fetch --locked` 两 workspace 均 EXIT=0）

- 两份 `Cargo.lock` 未被改动（mtime 2026-11-14）。
- fetch 前：core 147（134 可核验 / 13 不可核验）+ desktop 466（300 / 166）= 可核验 434 / 不可核验 179。
- **fetch 后：core 147/147 + desktop 466/466 = 613 包可核验全覆盖，不可核验 179→0**。记为「可核验全覆盖已达成」；冻结边界「179 个不可核验包不作声明」是**历史时点值、保留不改写**。

### 2.2 五件交付（均未给合规结论 / 法律意见）

| 文件 | 内容 |
|---|---|
| [tools/gen_third_party_notices.py](tools/gen_third_party_notices.py) | 只读幂等生成器，多次 EXIT=0 字节恒定 |
| [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) | 148184 B / 1179 行：613 包逐行清单 + MPL（§3.1–3.4/Exhibit A/FAQ Q8·Q11 + 源码可得要约）/ Apache-2.0 §4(a)–(d)（§4(d) 仅上游含 NOTICE 触发）/ MIT 归属 / 双许可非 SPDX 逐依赖判定 / NOT VERIFIABLE 不猜 |
| [BIS-742.15b-notice-template.md](BIS-742.15b-notice-template.md) | §742.15(b) 通知**模板、未发出、未填写**：告知源码公开位置、ECCN 5D002、XChaCha20-Poly1305 自实现；写明**「先通知、再公开」顺序坑**（推 GitHub 美国服务器本身即算出口 / 开源豁免只针对源码 / §734.7(b) 须对应源码也满足）+ §740.17(e)(3) 次年 2/1 报告 + 算法变更新分类请求 |
| [BETA-NOTICE.md](BETA-NOTICE.md) | 中文内测须知（邀请制 ≤50 / 未审计未签名 / 请勿存唯一副本 / 反馈只走主动渠道 / 正向反馈门槛 N≥30 + 连续 4 周 + 付费意愿≥阈值〔阈值未定不臆造〕 + 零高危数据丢失） |
| [zerocost-compliance-report.md](zerocost-compliance-report.md) | fetch 前后数字、第 6 个 MPL 结论、产出清单、失败/未完成、冻结诚实边界 |

### 2.3 fetch 后口径补遗（**追加、不改写已冻结诚实边界**）

- 落地位置：[m4-step4-report.md](m4-step4-report.md) 新增 §2.4.2、[prelaunch-checklist.md](prelaunch-checklist.md) §3.4 追加块、[security-whitepaper.md](security-whitepaper.md) §7 追加块、主文档 v1.9 变更块 / §9.3。
- **§2.4「可核验范围内 GPL 家族 0」是 fetch 前（434 包子集）口径**。
- **第 6 个 MPL = `htmlescape 0.3.1`**（`Apache-2.0 / MIT / MPL-2.0`，"/" 分隔非 SPDX "OR" 三许可、MPL 仅三选一之一），与 core 那 1 个是同包；**不存在第 6 个纯 MPL-2.0 包、不臆造**。fetch 后 desktop 纯 MPL-2.0 5→7（新增 `cssparser 0.38.0`、`selectors 0.41.0`），加 htmlescape 共 8 个 MPL 相关标志。registry src 带 `NOTICE*` 的 crate 数 = 0。
- **fetch 后新观察（事实、非法律结论）**：`r-efi 5.3.0` / `6.0.0` = `MIT OR Apache-2.0 OR LGPL-2.1-or-later`（SPDX OR 表达式里的 LGPL 选项）→ 脚本 GPL-expression hits core 0→1 / desktop 0→2；`r-efi` 是 `getrandom` 在 UEFI + `efi_rng` 下 **target-specific 传递依赖、对 `x86_64-pc-windows-gnu` 不参与编译**，不作「已编译进产物」断言。脚本只 print SUMMARY、无失败退出码，新增命中不影响 `EXIT=0`。
- **统一对外措辞（冻结）**：「**无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft**」+ 补充该观察；**绝不宣称「依赖树绝对没有 GPL/LGPL 字样」**。`htmlescape` 三许可择一与 `r-efi` 逐依赖许可决定、律师签发、SBOM 仍属外部闸门（go 判据未达成）。

---

## 3. ③ Android 最小可用（可行性设计，[android-feasibility.md](android-feasibility.md) 560 行）

- **本轮不产 APK**：本机缺 Android SDK / NDK / gradle / adb、无四个 Rust Android target、5 个环境变量（`ANDROID_HOME` 等）全 unset。交付的是可行性设计 + 绑定路径 + 零成本前置清单 + 验收判据，不是可侧载包。
- **三条核心结论**：① Android 构建链软件前置均可零现金取得（SDK cmdline-tools、NDK r30 `30.0.16248370`、四 Rust target、JDK 26.0.1、gradle wrapper、adb）；Apple Developer $99/年只影响 iOS/TestFlight ⇒ **iOS 本轮暂缓**。② 绑定推荐 Tauri 2 `#[tauri::command]` / mobile plugin 为主、UniFFI 可选、JNI 手写不作主路径 —— `mylock-core` 纯 Rust 无 tokio 同步内核，密码学只能一份实现以守 G4 向量契约。③ **冻结 A 级约束**：跨端主密钥必须是可移植软件密钥（Apple "Can't encode preexisting keys…" + AOSP "The key material of the generated symmetric and private keys is not accessible"）⇒ 硬件密钥仅本机 wrap、口令兜底必做；BiometricPrompt 只解锁 Keystore 密钥 `K_dev` 再 wrap 软件主密钥 MK，严禁把生物特征当密钥；同钥须同一 `format=1` 容器 + `argon2id(m=65536,t=3,p=4)` 显式 `parallelism=4`（不可回退 libsodium p=1）；Android 加入后须跑 30 derived + 13 negative + 指纹 `d45b8659…`。
- NOT RETRIEVED（未臆造）：`v2.tauri.app/distribute/mobile/android/` HTTP 404、github/raw `TypeError: fetch failed`、Tauri 官方页未给精确 SDK API level / NDK 绑定 / gradle 版本、Play Console 注册费金额、Android SAF 官方 API 页。

---

## 4. 封闭内测闸门与分发纪律（[BETA-NOTICE.md](BETA-NOTICE.md)）

- 邀请制 ≤50 人、不公开下载链接、不收费、不公开宣传；**未审计 / 未签名 / 请勿存放不可替代的唯一副本**。
- 反馈只走主动渠道（访谈 / 问卷 / 用户主动发消息），**不埋点、不上报、无崩溃上报、无账号、无云同步、无遥测**（No Telemetry 是最贵的品牌资产）。
- 正向反馈数字化门槛：N≥30 名内测用户 + 连续 4 周 + 明确付费意愿比例 ≥ 阈值（阈值未定不臆造）+ 零高危数据丢失事件；达标才启动审计 / 律师预算。
- **真要上架或收费那天，A/B/C/D 四道闸门一个不能跳。**

---

## 5. 验证结果（可重放）

| 项 | 命令 / 脚本 | 结果 |
|---|---|---|
| 全量回归 | `cargo test --workspace --locked` | **26 passed / 0 failed**（7 KAT + 4 conformance + 8 vectors + 7 mount） |
| 构建 | `cargo build --workspace --locked` / 桌面 `cargo check` | EXIT=0 |
| 向量归档 | `tools/check_vectors.py` | OK（锚 `07af0e8b…9939`、13499 B、CRLF×190、30 derived / 13 negative） |
| 指纹门禁 | `tools/compute_fingerprint.py` | `OK: G4 fingerprint matches frozen contract`（`d45b8659…02fa`） |
| 门禁负向 | `tools/gate_selftest.py` | 5/5 PASS（单字节 / CRLF→LF / 截断 / 陈旧锚 4 篡改被拒 + 好路径） |
| 许可审计 | `tools/audit_licenses.py` | EXIT=0（fetch 后 613 全覆盖；新增 LGPL-OR 命中不影响退出码） |
| 端到端 | `tools/e2e-probe.ps1` | `PROBE_EXIT=0`、`PLAINTEXT_HITS_IN_COPY=0` |
| Release | 桌面 + core `cargo build --release` | 3 个 exe EXIT=0 |
| 桌面 GUI 单测（v1.3 新增） | `cargo test --bin mylock-desktop` | **4 passed / 0 failed**（建库+装饰+verify / 拒绝非空目录 / 人话缺失库提示 / 错误码人话映射） |
| **便携包组装（当前 v1.3 最终包）** | `tools/assemble-beta.ps1` | `ASSEMBLE_BETA_OK` / `IMPORT_GATE=PASS` / **8 文件 ZIP 10208486 B**（含 `USER-GUIDE.md`、新 GUI exe） / `LOADER_SHA256=86545B66…`；v1.2 当时为 10181612 B、热修复时为 7 文件 10169169 B |
| **GUI 真启动（强判据）** | 从修复后 stage 启动 + `EnumWindows` | `MAIN_WINDOW_COUNT=1`、标题 `MyLock · 透明加密文件夹（M5 · 进程内引擎 · 封闭内测）` |
| **GUI 负向对照** | 干净目录只拷 exe 不带 DLL | `NEG_VISIBLE_WINDOWS=0`（复现 m04223 故障） |
| **最终 ZIP 解压后强验证（当前包）** | `tools/verify-beta-final.ps1` | **8 entries** / `HAS_GUIDE=True` / DLL SHA256 一致 / 从解压目录启动 `VISIBLE_WINDOWS=1` / `MAIN_WINDOW_TITLE_OK=True` / `VERIFY_BETA_FINAL=PASS` / `FINAL_VERIFY_EXIT=0` |

> 说明：§1.4 表格中旧版「`ALIVE_AFTER_3S=True`」是**弱冒烟判据**，缺 DLL 时错误框会让进程仍「存活」，不能证明窗口起来；已在 §1.5 用「窗口标题 + 负向对照」取代。复验脚本 [tools/verify-m4s4.ps1](tools/verify-m4s4.ps1) 仍可重放（7 步全部 `STEP_EXIT=0`）。

---

## 6. NOT RETRIEVED / 未完成 / 明确不做

- NOT RETRIEVED（合规事实，沿用 m4-step4-report.md §6）：EUR-Lex 官方 GDPR 全文（HTTP 202 WAF）、15 CFR part 774 Supp.1 的 5D002/5E002 条目正文、§772.1 "non-standard cryptography" 逐字定义、Apple 出口合规豁免类别清单 / CCATS 与 ERC 表格、Google Play 出口问卷逐字措辞、Microsoft Store 加密出口声明一手陈述、中国《出口管制法》正文与两份管制清单 /「大众消费类产品」法定定义、欧盟附件 I "in the public domain" 逐字定义、NTIA SBOM 最小元素 PDF、Play Console / Android SAF 官方页。
- **未完成**：BIS §742.15(b) 通知**模板占位全空、未发出**；`BETA-NOTICE.md` 付费意愿阈值未定；`htmlescape` 三许可择一与 `r-efi` LGPL-OR 逐依赖许可决定留待律师；**未产出可分发包 / 未签名 / 未审计 / 未上架 / 未开源加密核心**。
- **明确不做（本轮授权外）**：G4 跨平台绿灯（未被选中）、iOS（$99/年）、NSIS/MSI 安装包、代码签名证书、第三方审计、律师签发、云同步 / 账号 / 崩溃上报 / 自动遥测。

---

## 7. 诚实边界（唯一冻结措辞，以 [m5-report.md](m5-report.md) §0/§7 与 [BETA-NOTICE.md](BETA-NOTICE.md) 为准）

- **原 NO-GO 措辞一字不动、只约束「公开发布 / 商店上架」**；本轮完成的「软件研发 + 封闭内测」零现金可完成，但**封闭内测 ≠ 公开发布**。
- **推迟 ≠ 豁免**：EAR 出口分类、PIPL/GDPR controller 定性、MPL-2.0 §3.2 源码可得义务不因规模小或免费消失。
- 本版本**未经第三方安全审计、未代码签名，请勿存放不可替代的唯一副本**；不保证数据不丢失；SmartScreen 警告预期内、不买证书。
- 草案 / 模板**未经律师签发不得对外发布**；不是合规结论、不是第三方审计报告、不是法律意见、不是已通过认证。
- **未产出可分发安装包 / 未签名 / 证书自备**；仅免签名 standalone exe + portable ZIP；**未产 NSIS/MSI 与 APK**；G4 仍只完成 Rust/Windows 单端、不在本轮授权内。
- 非 WinFsp 驱动级挂载；loopback WebDAV 无 per-request 认证（任何本机进程可访问解锁态挂载点）；拒绝云同步根内挂载；解锁态只缓解、库外持久副本锁定后不保证消失；换口令零吊销（真吊销须 `rotate-keys` 全量重加密 + 销毁旧介质）。
- **M5 GUI 完善（§1.7）新增并冻结**：保险柜图标 / `desktop.ini` / 隐藏+系统属性 / 删除「系统文件夹」警告 / `⚠ 请勿删除.txt` **只是 shell 层「提示性」保护，不是防删 / 权限（ACL）机制，不保证数据不丢失**；真正删掉整目录或 `vault.meta` / `manifest` / `d\` 里任何一个，保险柜照样失效；防误删最终手段仍是备份 + 请勿存唯一副本。GUI「新建保险柜」与 CLI `create` 共用同一份内核、不改变任何加密内核 / 向量契约 / 冻结 NO-GO 措辞与四道外部闸门。
- 依赖审计：**无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft**；fetch 后 613 包全覆盖、`r-efi` 的 LGPL-OR 选项为事实观察、非法律结论、对 Windows GNU 不参与编译；**绝不宣称「依赖树绝对没有 GPL/LGPL 字样」**；「179 个不可核验包不作声明」是 fetch 前历史时点值。
- 合规事实为 **2026-11-15 抓取时点值**，法规/商店政策会变化，签发前须复核最新文本。
- 文档仍有 C 级设计产物（[vault-format-spec.md](vault-format-spec.md) v1.1.1），不是安全认证、不是第三方审计通过。

---

## 8. 文档接口

[产品分析与概要设计.md](产品分析与概要设计.md) v1.9 · [BETA-NOTICE.md](BETA-NOTICE.md) · [USER-GUIDE.md](USER-GUIDE.md) v1.1 · [tools/make_vault_icon.py](tools/make_vault_icon.py) · [tools/probe-attrib.ps1](tools/probe-attrib.ps1) · [android-feasibility.md](android-feasibility.md) · [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) · [BIS-742.15b-notice-template.md](BIS-742.15b-notice-template.md) · [zerocost-compliance-report.md](zerocost-compliance-report.md) · [tools/e2e-probe.ps1](tools/e2e-probe.ps1) · [tools/gen_third_party_notices.py](tools/gen_third_party_notices.py) · [tools/verify-m4s4.ps1](tools/verify-m4s4.ps1) · [prelaunch-checklist.md](prelaunch-checklist.md) · [m4-step4-report.md](m4-step4-report.md) §2.4.2 · [security-whitepaper.md](security-whitepaper.md) §7 · [privacy-policy.md](privacy-policy.md) · [desktop/SIGNING.md](desktop/SIGNING.md)
