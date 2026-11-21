# M3 桌面层阶段报告（m3-report.md）

> 版本：v1.0（2026-11-14）
> 授权：用户 m01639「M3 桌面层」
> 上游：[产品分析与概要设计.md](产品分析与概要设计.md) v1.9（M3 当时为 v1.4；M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step3-report.md](m4-step3-report.md) / [m4-step4-report.md](m4-step4-report.md)）/ [m2-report.md](m2-report.md) / [vault-format-spec.md](vault-format-spec.md) v1.1.1 / [threat-model.md](threat-model.md) v1.1
> 架构决策：[m3-architecture.md](m3-architecture.md)
> 状态：**M3 实现完成并全绿**；非产品发布、非五端一致性验收、非第三方安全审计。

---

## 1. 这一阶段做了什么 / 不做什么

**做了**：把 M2 的静态密文容器（`mylock-core`）包成「透明加密文件夹」——在装了 App 的机器上，用一次口令解锁，通过 **loopback-only WebDAV** 暴露成操作系统可映射的盘符，双击/拖拽/另存为都能用；写操作实时重新加密落盘。

**没做（明确边界）**：
- 不做 WinFsp/FUSE **驱动级挂载**（WinFsp 未安装且 `winfsp` crate 是 GPL-3.0）；
- 不做移动端、不做云同步、不做账号体系、不埋点、不接 AI；
- 不做产品发布：没有安装包、没有签名、没有自动更新、没有上架；
- 不是第三方安全审计；G4「五端 + 恢复 CLI 逐字节一致」仍只完成 Rust 单端。

**产品口径兑现**（用户 m00214 的问题）：静态密文容器仍是普通文件目录，不依赖 OS、拷到移动硬盘仍是保险柜；M3 只是在本机把那个保险柜「变回文件夹」。插盘输口令 → 文件夹；拔盘 → 保险柜。

---

## 2. 交付物

| 路径 | 内容 |
|---|---|
| [core/crates/mylock-mount/Cargo.toml](core/crates/mylock-mount/Cargo.toml) | 新 crate `mylock-mount`，依赖钉死 |
| [core/crates/mylock-mount/src/lib.rs](core/crates/mylock-mount/src/lib.rs) | `VaultFs`（`DavFileSystem` 7 方法 + `DavFile` 6 方法）、`is_sync_root`、`build_handler`、`run_listener`、`serve`、完整安全 banner |
| [core/crates/mylock-mount/src/bin/mylock-serve.rs](core/crates/mylock-mount/src/bin/mylock-serve.rs) | 独立服务入口 `--root/--password\|--recovery/--port` |
| [core/crates/mylock-mount/tests/mount.rs](core/crates/mylock-mount/tests/mount.rs) | M3 测试套件 6 个 |
| [core/crates/mylock-core/src/lib.rs](core/crates/mylock-core/src/lib.rs) | M3 唯一新增内核 API：`PlainNode` + `rewrite_vault_from_plaintext`（写回 helper） |
| [core/crates/mylock-cli/src/main.rs](core/crates/mylock-cli/src/main.rs) | `mount` 措辞改为指向 `mylock-serve`（恢复 CLI 保持零网络/零 tokio） |
| [desktop/src-tauri/](desktop/src-tauri/) | Tauri 2 桌面壳脚手架：4 个命令 + 完整安全警示 UI |
| [desktop/ui/index.html](desktop/ui/index.html) | 深色 UI，含黄色安全边界块 |
| [m3-architecture.md](m3-architecture.md) | 架构决策（为什么 loopback WebDAV / 写回机制 / 安全边界） |
| [tools/m3-smoke.ps1](tools/m3-smoke.ps1) | 真实网络链路冒烟脚本 |

复现：
```
cd D:\Projects\dsh\mylock\core
cargo build --workspace
cargo test --workspace
cd ..\desktop\src-tauri
cargo check
```

---

## 3. 架构决策摘要（详见 m3-architecture.md）

1. **loopback WebDAV 而非 WinFsp**：零驱动安装（Windows 自带 `net use`）、许可干净（避 GPL-3.0 传染）、跨平台同一套 Rust 代码、符合「无服务端」反定位。代价=非驱动级语义 + 无 per-request 认证（已如实声明）。
2. **桌面壳只做子进程管理**：Tauri 壳不碰加密、不碰挂载，只 spawn/stop `mylock-serve` 并展示 `net use Z:` 映射说明；唯一内核集成点 `verify` 调 `verify_phase1`（结构检查，不解密不解锁）。桌面壳用独立 `[workspace]` 隔离重依赖。
3. **两次 Argon2id 换 core 零改动**：`build_handler` 里 `unwrap_master`（拿会话密钥供写回）+ `full_open`（拿记录/明文建树），不改 M2 冻结的公开 API。
4. **写回 = 严格 copy-on-write + 原子 manifest 替换**：每次写都给全部节点生成全新 object_id + 全新 nonce，先写新对象 → `manifest.tmp` + `fsync` + 原子 rename → 再清旧孤儿；`vault.meta` 头字节会话内永不重写。崩溃窗口只留无害孤儿。
5. **依赖钉死**：dav-server 0.11.0（`default-features=false`）/ hyper 1.12.0 / hyper-util 0.1.21 / tokio 1.54.1 / bytes 1.12.1 / futures-util 0.3.34 / tauri 2.12.5 / tauri-build 2.7.4（2026-11-14 crates.io 实测编译通过）。

---

## 4. 验证结果（全部实测）

### 4.1 编译

`cargo build --workspace` → **EXIT=0**（`core/m3-build.log`）。首次编译有 3 类 5 个错误，已修：
- `unused import: Component` → 删；
- `TcpListener::bind(([127,0,0,1], port))` 不满足 `ToSocketAddrs` → 改 `SocketAddr::from(([127,0,0,1], port))`；
- `metadata()` 闭包 `Box<Meta>` 与 `Box<dyn DavMetaData>` 类型推断冲突 → 给闭包显式返回类型 `-> FsResult<Box<dyn DavMetaData>>`。

> 日志里的 `NativeCommandError` 是 pwsh 把 cargo 的 stderr 进度行渲染成错误记录的假象，`EXIT` 才是真值。

### 4.2 测试（`cargo test --workspace` → **EXIT=0**，21 passed / 0 failed）

| 组 | 数量 | 结果 |
|---|---|---|
| mylock-core 7 个权威 KAT 单测 | 7 | ok 6.58s（RFC 3394 §4.6 / draft-xchacha §A.3.1 / RFC 5869 A.1 / RFC 4231 TC1 / Argon2id p=4 绑定 / canonical profile / 2.5MiB 分块） |
| mylock-core 8 个集成 | 8 | ok 44.49s（13/13 负向、零/真吊销、200 轮 fuzz「永不返回未认证明文」、逐字节复现 30 derived_outputs） |
| **mylock-mount/tests/mount.rs（M3 新增）** | **6** | **ok 7.53s** |
| mylock-cli / mylock-mount lib / mylock-serve bin / doc-tests | 0 | — |

**M2 的 15 个测试零回退。**

M3 六个测试：
1. `sync_root_detection_is_case_insensitive` — OneDrive/onedrive/Dropbox/Google Drive/坚果云 命中，普通路径不命中；
2. `build_handler_rejects_cloud_sync_root` — 同步根内 `build_handler` 返回 `E_SYNC_ROOT`；
3. `write_back_persists_new_file_and_keeps_header_unchanged` — 写 `/new.txt` 后 `vault.meta` **逐字节不变**、旧 object_id **全部换新**（证明真 copy-on-write）、重新解锁 new.txt/hello.txt/empty.bin 逐字节一致；
4. `vfs_create_dir_write_rename_remove_roundtrip` — 建目录/写/读目录/改名/删文件/删目录全链路，重新解锁只剩 2 个老文件；
5. `unicode_filename_roundtrip` — `%E4%BF%9D...` 解码成 `保险柜🔐.txt` 并能重新解锁；
6. `http_put_get_roundtrip` — 直接 `DavHandler.handle(Request)` 发 GET/PUT，2xx 且回读一致，重新 `full_open` 证明真落盘。

测试迭代中修掉一个真问题：`DavHandler` 不实现 `Debug` → `Result::unwrap_err()` 编译失败（`error[E0277]`），改 `match` 取 `err.code`。

### 4.3 真实网络链路冒烟（`tools/m3-smoke.ps1`，全部通过）

不是只测 handler，而是**真起进程 + 真发 HTTP**：

```
mylock.exe create <root> --password <pw> --file <seed.txt>   → CREATE_EXIT=0，objects=1
Start-Process mylock-serve.exe --root <root> --password <pw> --port 51918
curl.exe -X PROPFIND http://127.0.0.1:51918/                 → 207
curl.exe -X PUT    .../loop.txt  "written over loopback"      → 201
curl.exe GET       .../loop.txt                                → written over loopback
curl.exe GET       .../seed.txt                                → seed from disk
mylock.exe export <root> --out <dir> --password <pw>
  → loop.txt(21B) + seed.txt(14B)，EXPORT_LOOP=written over loopback
```

**证明**：`TcpListener` + hyper 真实链路通、OS 原生 WebDAV 客户端语义通、写回真的重新加密落盘、独立恢复 CLI 能读出 WebDAV 写进去的内容。

---

## 5. 工具链与运行时

| 项 | 值 |
|---|---|
| Rust | rustc/cargo 1.100.0 `stable-x86_64-pc-windows-gnu`（复用 `C:\mingw64\bin\gcc.exe`，避 MSVC） |
| Node / pnpm | v24.21.0 / 齐 |
| WebView2 | 已装 `C:\Program Files (x86)\Microsoft\EdgeWebView\Application\157.0.4341.46` |
| WinFsp | **未安装**（故不实现驱动级挂载） |
| 新增依赖 | dav-server 0.11.0 + 97 个传递依赖，全部下载编译成功；`default-features=false` 正常 |
| Tauri 壳 | 独立 workspace，`cargo check` 结果见 §7 |

---

## 6. 安全边界（唯一冻结措辞，与 threat-model §3.1 / crypto-review §8 一致）

- **仅未解锁态防得住合法采集者**；解锁态只缓解，不消除。
- **WebDAV loopback 无 per-request 认证**：任何本机进程都能访问挂载点 —— 已写进服务 banner、桌面 UI 黄色警示块、本报告、架构文档。
- **拒绝在云同步根目录内挂载**（OneDrive/Dropbox/Google Drive/iCloud Drive/坚果云/SharePoint/pCloud/MEGA/Seafile/百度网盘/阿里云盘/同步空间），命中即 `E_SYNC_ROOT`：云同步客户端会增量上传密文容器，等于自破「无云端」反定位。
- **库外已存在的明文/临时副本，锁定后不保证消失**，产品从不做「自动擦除」承诺。
- **不是 WinFsp/FUSE 驱动级挂载**；CLI `mount` 仍明确拒绝并指向 `mylock-serve`。
- **内核实现 / 桌面壳 ≠ 产品发布 ≠ 五端一致性验收 ≠ 第三方安全审计**；格式层是成熟范式组合、非创新；`verify` 绿 ≠ 未被恶意替换；口令熵是唯一上限。
- **已知性能限制（M3 当时）**：每次写都全量 copy-on-write 重加密整库，大库/高频写性能有限（正确性优先，不伪装成增量写）。**M4 第 2 点已升级为增量写回**（未变对象复用原密文文件、manifest 每次重建），见 [m4-step2-report.md](m4-step2-report.md)；本节保留为 M3 历史记录。

---

## 7. Tauri 桌面壳状态

`desktop/src-tauri/` 为可编译脚手架（独立 `[workspace]`）：
- `Cargo.toml` / `build.rs` / `tauri.conf.json`（`frontendDist="../ui"`）/ `src/main.rs` / `../ui/index.html` / `icons/icon.ico`；
- 4 个 `#[tauri::command]`：`open_vault` / `close_vault` / `status` / `verify`；
- `cargo check` 首次因缺 `icons/icon.ico` 失败（`package.metadata does not exist` + Windows Resource 需要 ico），已用 Pillow 生成合法多尺寸 ICO（16/32/48/64/128/256）后复验 → `Finished dev profile ... in 5.37s`，`EXIT=0`（`desktop/src-tauri/check2.log`；日志里 `NativeCommandError` 仍是 pwsh 渲染 cargo stderr 的假象）。**桌面壳脚手架编译通过。**

> 桌面壳 **未**做安装包/签名/sidecar 打包；`mylock-serve` 目前靠「同目录或 PATH」查找，生产化应改 Tauri `externalBin`。

---

## 8. 硬门槛对照（沿用 m2-report.md §8 编号）

| 门槛 | M2 状态 | M3 状态 |
|---|---|---|
| ① 五端 + 恢复 CLI 逐字节一致 | 部分（Rust 单端） | **不变，仍部分** —— 桌面层未新增任何新端实现，G4 仍待 M4 |
| ② 向量 + 主链路 | ✅ | ✅（新增 6 个 M3 测试 + 真实网络冒烟） |
| ③ 负向/吊销/fuzz | ✅ | ✅ 零回退 |
| ④ 独立恢复 CLI | ✅ | ✅ 保持纯净（零网络/零 tokio） |
| ⑤ 第三方安全审计 | ⏳ | ⏳ 仍未做 |

---

## 9. 下一步（均需用户新授权，非本阶段承诺）

1. **M4 移动端 / 跨平台 KAT 门禁**：`poc/test-vectors.json` 归档 `tests/vectors/`，Linux/macOS/Windows/Android/iOS 目标逐字节复现，补齐 G4。
2. **WinFsp / macFUSE 增强路径**：若需真实本地文件语义，另评 GPL-3.0 法务 + 驱动签名。
3. **增量写回优化**：脏节点追踪替代全量 copy-on-write。
4. **Tauri sidecar 化 + 安装包/签名**。
5. **上架前**：第三方安全审计、隐私政策/安全白皮书、移动端密钥失效恢复规范、上架法务核验。

用户明确选择前，停留在当前状态。
