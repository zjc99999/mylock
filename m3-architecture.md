# M3 桌面层架构决策（m3-architecture.md）

> 版本：v1.0（M3 桌面层实现配套架构决策，2026-11-14）
> 授权：用户 m01639「M3 桌面层」
> 上游：[产品分析与概要设计.md](产品分析与概要设计.md) v1.9（M3 当时为 v1.4；M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step3-report.md](m4-step3-report.md) / [m4-step4-report.md](m4-step4-report.md)）/ [vault-format-spec.md](vault-format-spec.md) v1.1.1 / [threat-model.md](threat-model.md) v1.1 / [m2-report.md](m2-report.md)
> 实现：[core/crates/mylock-mount/](core/crates/mylock-mount/) / [desktop/](desktop/) / 报告 [m3-report.md](m3-report.md)
> 状态：架构已落地并编译/测试通过；**非产品发布、非第三方审计、非五端一致性验收**。

---

## 0. 这一层解决什么

用户 m00214 的原始问题：「透明的加密文件夹，它是否依赖于操作系统，可否实现拷贝到移动硬盘时，它依然是如一个保险柜一样呢？」

M1/M2 已经把**静态密文容器**（`vault.meta` + `manifest` + `d/<shard>/<b32>`）做成了不依赖 OS 的普通文件目录——拷到移动硬盘仍是保险柜。M3 补的是另一半：**在装了 App 的机器上，把这个密文容器「变回」一个能双击、能拖拽、能另存为的普通文件夹**。

这两半必须分开，否则就会犯「透明性随盘走」的概念错误：

| | 静态密文容器（M1/M2 已有） | 透明加密文件夹（M3 新增） |
|---|---|---|
| 本质 | 普通文件目录（乱码名 + 乱码内容） | 本机运行时形态（一个网络盘符 / 挂载点） |
| 依赖 OS 吗 | **不依赖**（跨文件系统安全，base32 大小写不敏感） | **依赖**：需要本机进程把密文解码成文件视图 |
| 拷到移动硬盘后 | **仍是保险柜**，任何 OS 只看到乱码 | **不会**自动变明文目录；拔盘即回到「保险柜」态 |
| 读明文的前提 | 装了 App 的设备 + 主口令 / Recovery Key | 同上，且必须启动 M3 的 loopback 服务 |

一句话（写进产品口径）：**插上装了 App 的电脑、输一次口令，它是你原来那个文件夹；拔掉硬盘，它是谁也打不开的保险柜。**

---

## 1. 核心决策：为什么是 loopback WebDAV，而不是 WinFsp/FUSE

### 1.1 工具链实测（A 级）

| 项 | 实测 | 结论 |
|---|---|---|
| Node / pnpm | node v24.21.0 + pnpm 齐 | 桌面壳前端可构建 |
| WebView2 | 已装 `C:\Program Files (x86)\Microsoft\EdgeWebView\Application\157.0.4341.46` | Tauri 2 运行时可用 |
| **WinFsp** | **未安装**；`winfsp` crate 0.13.2 是 **GPL-3.0** 且需装内核驱动 | 不作为 M3 首选 |
| crates.io | 可达；dav-server 0.11.0 等 97 个依赖全部下载编译成功 | 纯 Rust 用户态路径可行 |

### 1.2 决策

> **首选：loopback-only WebDAV（OS 原生 `net use` / `mount_webdav`），零驱动、零 GPL 传染。WinFsp 仅列为「增强路径」，M3 不实现、不宣称。**

理由：
1. **零安装驱动**：Windows 自带 WebDAV 重定向器，`net use Z: http://127.0.0.1:4918/` 即可；不需要管理员权限装 `.sys` 驱动。
2. **许可干净**：`dav-server 0.11.0` 是 MIT/Apache-2.0 系；`winfsp` 是 GPL-3.0，链接进闭源/双许可产品会引发传染性质疑，产品早期不值得背这个法务风险。
3. **跨平台同一套代码**：macOS `mount_webdav`、Linux `davfs2`/GVfs 都能连同一个 loopback handler，未来 M4 移动端/桌面端复用成本最低。
4. **威胁模型一致**：loopback 只监听 `127.0.0.1`，符合「本地优先、零知识、无服务端」的产品反定位（No Cloud · No AI · No Telemetry · No Account）。

代价（必须诚实）：
- **不是驱动级挂载**，某些要求真实本地文件语义的程序（如部分数据库、需要文件锁/内存映射的工业软件）不兼容；
- **无 per-request 认证**：任何本机进程都能连 `127.0.0.1:<port>` 访问挂载点。这是 WebDAV loopback 的固有边界，已写进服务启动 banner、桌面 UI 警示块、测试与报告。

### 1.3 依赖钉死（2026-11-14 crates.io max_stable，实测编译通过）

`dav-server 0.11.0`（`default-features=false`）、`hyper 1.12.0`、`hyper-util 0.1.21`、`tokio 1.54.1`、`bytes 1.12.1`、`futures-util 0.3.34`、`http 1.5.0`(dev)、`tauri 2.12.5` / `tauri-build 2.7.4`（桌面壳，独立 workspace）、`serde 1.0.229` / `serde_json 1.0.154`。

> 选 `dav-server 0.11.0` 而非 0.12.0：后者当时发布仅 5 天且默认开 `proppatch`，早期产品不追新。

---

## 2. 架构分层

```
┌──────────────────────────────────────────────────────────────┐
│ 桌面壳  desktop/src-tauri/  （Tauri 2, 独立 workspace）        │
│  · 只做：拉起/停止 mylock-serve 子进程 + 如实告知映射方法      │
│  · verify 命令 = mylock_core::verify_phase1（不解密不解锁）    │
│  · 不做加密、不做挂载、不碰明文                                │
└───────────────┬──────────────────────────────────────────────┘
                │ spawn 子进程
┌───────────────▼──────────────────────────────────────────────┐
│ mylock-serve（core/crates/mylock-mount/src/bin/）              │
│  · 解析 --root/--password|--recovery/--port                    │
│  · ensure_not_sync_root（拒绝云同步目录内挂载）                │
│  · TcpListener::bind(127.0.0.1:port) + peer.is_loopback()     │
│  · hyper http1 + service_fn → DavHandler.handle(req)           │
└───────────────┬──────────────────────────────────────────────┘
                │ DavFileSystem trait
┌───────────────▼──────────────────────────────────────────────┐
│ VaultFs（core/crates/mylock-mount/src/lib.rs）                 │
│  · 内存明文树：稳定逻辑 u64 id，与盘上 object_id 解耦          │
│  · 两遍建树：records_raw 的 parent_id + exported_by_id          │
│  · 7 个方法：open/read_dir/metadata/create_dir/remove_dir/      │
│    remove_file/rename，每个写操作 → persist()                  │
│  · DavFile 6 方法：read_bytes/seek/write_buf/write_bytes/       │
│    metadata/flush                                              │
└───────────────┬──────────────────────────────────────────────┘
                │ rewrite_vault_from_plaintext(...)
┌───────────────▼──────────────────────────────────────────────┐
│ mylock-core（M2 内核，零改动）                                 │
│  · unwrap_master：Argon2id → AES-KW → 全头 versionMac 认证     │
│  · full_open：manifest + 逐对象 XChaCha20-Poly1305 解密        │
│  · rewrite_vault_from_plaintext：M3 新增的写回 helper          │
└──────────────────────────────────────────────────────────────┘
                │
        静态密文容器（可整个拷到移动硬盘）
```

**关键取舍**：`build_handler` 里同时调 `unwrap_master`（拿会话密钥供写回）和 `full_open`（拿记录/明文供建树），**刻意接受两次 Argon2id 的一次性启动开销，换 `mylock-core` 零改动**。挂载是一次性动作，不值得为省一次 KDF 去改内核公开 API、破坏 M2 已冻结的 15 个测试契约。

---

## 3. 写回机制（本层最需要设计正确的地方）

### 3.1 约束

- 挂载会话内 **`vault.meta` 头字节保持不变**：不重算 Argon2id / AES-KW / versionMac。否则每次写文件都要用户重新输口令，且会破坏「换口令零吊销」等 M1 定案语义。
- 盘上永远是密文：**任何一次写操作都必须重新加密落盘**，不能把明文缓存在 vault 目录里。
- 崩溃安全：写一半断电不能让整个保险柜打不开。

### 3.2 定案：严格 copy-on-write + 原子 manifest 替换

每次 `persist()` 调 `mylock_core::rewrite_vault_from_plaintext(root, &Unwrapped, device_id, &[PlainNode])`：

1. 内存树转 `Vec<PlainNode>`（`parent_index` 按逻辑 id 下标映射，父节点顺序无关——helper 先为全部节点预生成 object_id 数组）。
2. **每个节点生成全新 128-bit `object_id` + 全新随机 24B nonce**，逐对象用 `hkdf_expand(master, b"object"+oid)` 加密，写 `d/<b32[:2]>/<b32>`。→ 旧密文对象一个字节都不复用，彻底避免 nonce 重用。
3. 用**同一个 `manifest_key`、同一个 `manifest AAD`**（`mid`/`seq` 不变）但**全新 nonce** 重加密 manifest → 先写 `manifest.tmp` + `File::sync_all()` → 原子 `rename` 覆盖 `manifest`（Windows 覆盖失败则删旧再 rename）。
4. manifest 已只指向新对象后，才遍历 `d/` 各 shard **清理旧孤儿**。
5. `vault.meta` / `vault.meta.bak` **完全不重写**。

崩溃窗口分析：
- 步骤 2 中途崩 → 多出无害孤儿对象，manifest 仍指向旧对象，库完好（`verify` 会报 orphan 警告，非致命）。
- 步骤 3 `rename` 前后崩 → 要么旧 manifest 有效、要么新 manifest 有效，两者都自洽。
- 步骤 4 中途崩 → 只残留孤儿，下次解锁正常。

### 3.3 已知限制（诚实标注）

**全量 copy-on-write（M3 当时）**：每写一次文件都重写整个库的全部对象。对「少量私人文档」这个滩头场景完全够用，但**大库 / 高频写性能有限**。这是 M3 的正确性优先取舍，不伪装成增量写。**M4 第 2 点已落地增量写回**（`IncNode` + `rewrite_vault_incremental`：未变对象复用原 oid 与原密文文件，只重写 manifest；重命名只改 record，不进对象 AAD），见 [m4-step2-report.md](m4-step2-report.md)。**仍不是增量日志**：manifest 每次全量重建，见 m4-step2-report.md §1/§6。本节保留为 M3 架构决策的历史记录。

---

## 4. 安全边界（threat-model.md §3.1 口径，逐条落到实现）

| 边界 | 实现落点 |
|---|---|
| 仅**未解锁态**防得住合法采集者 | 盘上永远只有密文；明文只在 `VaultFs` 内存 + 解锁会话内 |
| 解锁态只**缓解**，不消除 | loopback 只绑 `127.0.0.1`，但**无 per-request 认证** → banner + UI + 报告明确写出「任何本机进程可访问挂载点」 |
| 库外已存在的持久副本，锁定后**不保证消失** | 文档与 UI 明示；产品从不做「自动擦除」承诺 |
| **拒绝同步根内挂载** | `is_sync_root()` 逐路径组件大小写不敏感匹配 OneDrive/Dropbox/Google Drive/iCloud Drive/坚果云/SharePoint/pCloud/MEGA/Seafile/百度网盘/阿里云盘/同步空间；`build_handler` 命中即 `E_SYNC_ROOT`。理由：云同步客户端会把密文容器增量上传，等于自破「无云端」反定位 |
| **不宣称已实现 WinFsp 驱动级挂载** | CLI `mount` 仍明确拒绝并指向 `mylock-serve`；桌面壳 docstring/UI 都写「loopback WebDAV，非 WinFsp/FUSE」 |
| 内核/桌面壳 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计 | 所有 docstring、banner、UI、报告统一措辞 |

---

## 5. 与上游文档的一致性

- 密文格式、算法、KDF、AAD、错误码 **一律沿用 [vault-format-spec.md](vault-format-spec.md) v1.1.1**，M3 未新增任何密码学原语、未改任何字节级约定。
- `rewrite_vault_from_plaintext` 是 M3 唯一新增的内核 API，且只做「按既有格式重新加密落盘」，不引入新格式。
- 独立恢复 CLI（`mylock`）**保持纯净**：零网络、零 tokio、零 dav-server；`mount` 继续拒绝，只把措辞改为指向 `mylock-serve`。→ 恢复通道与桌面层故障隔离。
- 桌面壳用**独立 `[workspace]`**，避免把 Tauri 重依赖与 core workspace 的 97 个依赖混编，保证 M2/M3 的 21 个测试不被桌面壳构建拖累。

---

## 6. 后续路径（均需用户新授权，非本阶段承诺）

1. **M4 移动端 / 跨平台 KAT 门禁**：把 `poc/test-vectors.json` 归档 `tests/vectors/`，在 Linux/macOS/Windows/Android/iOS 目标上跑逐字节复现，补齐 G4「五端 + 恢复 CLI 一致性」。
2. **WinFsp / macFUSE 增强路径**：**M4 第 3 点已评估并决策不采用**（用户 m03177「继续第 3 点」）——WinFsp 的 FLOSS 例外逐字禁止与专有软件混用、闭源须商业许可（6000 USD/3yr 组织 / 2400 USD/3yr 个人）、GPL-3.0 Rust 绑定第二层 copyleft、delay-load 规避论无第一方安全港、需 EV 证书 + 已签名内核驱动；继续保留 loopback-only WebDAV。**Dokany（LGPLv3 驱动 + MIT Rust 绑定）是唯一同时满足「不引入 GPL-3.0 内核驱动」+「真 RW 本地语义」的备选，须先过法务复核且需用户新授权**；ProjFS / CfAPI / WSL / SMB loopback / rclone 均不满足。详见 [m4-step3-report.md](m4-step3-report.md) §0/§3/§5/§6。
3. **增量写回优化**：脏节点追踪，替代全量 copy-on-write。
4. **Tauri sidecar 化**：`mylock-serve` 做成 `externalBin` 随桌面壳分发，替代当前「同目录/PATH 查找」。
5. **上架前**：第三方安全审计、隐私政策/安全白皮书、移动端密钥失效恢复规范、上架法务核验。
