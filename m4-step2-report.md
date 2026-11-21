# M4 第 2 点阶段报告：增量写回优化 + Tauri sidecar 化 + 安装包/签名配置

> 授权：用户 m02670「继续第 2 点」，即 M3 汇报「下一步」第 2 条
> 「增量写回优化 / Tauri sidecar 化 + 安装包签名」。
> 日期：2026-11-14。本文为阶段事实记录，不是产品发布、不是第三方审计。

## §1 做了什么 / 没做什么

做：

1. **core 增量写回**：新增 `IncNode` + `rewrite_vault_incremental`；`rewrite_vault_from_plaintext`（M3 全量版）退化为「全部节点 dirty」的包装，保持 M3 调用方与向量契约零改动。
2. **VaultFs 增量改造**：`Node` 增 `oid` / `dirty`，`from_open` 保留原 object_id，写操作只标脏节点，`persist()` 改调增量版并把新 oid 回写内存。
3. **增量写回测试**：把 M3「旧 id 全换新」断言改为真复用语义，新增专项测试证明「未变对象密文文件逐字节不动 / 连续写只换该文件 / rename 不重加密 / 删除不误删复用对象」。
4. **Tauri sidecar 化**：`bundle.externalBin` 把 `mylock-serve` 打进安装包；`resolve_serve_bin()` 四级解析；`tools/prepare-sidecar.ps1` stage 二进制；桌面壳 `cargo check` 通过。
5. **安装包/签名配置与诚实边界**：`bundle.active=true` / `targets=["nsis","msi"]` / 4 图标；`desktop/SIGNING.md` 写明签名所需证书与字段，明确**未签名**。

没做（诚实边界，逐字保留）：

- **没有产出可分发安装包**：未跑 `tauri build`、未生成 `.msi`/NSIS `.exe`、未对任何二进制签名；本仓库**不提供证书、不产出已签名安装包**。
- **不是 G4 五端验收通过**：本轮与五端一致性无关；`cargo test --workspace --locked` 仍以 Windows 单端实测，M4 第 1 点的三桌面 CI 仍待推送后实跑、Android/iOS 仍未实现。
- **增量写回不是增量日志**：每次写回仍重建整份 `manifest`（全新 nonce + 原子替换），只是**对象文件**复用；大 manifest / 高频写仍有开销，不伪装成全增量。
- 仍是 **loopback-only WebDAV 非 WinFsp/FUSE 驱动级挂载**；WebDAV loopback **无 per-request 认证**；拒绝云同步根内挂载；解锁态只缓解不消除采集风险；库外持久副本锁定后不保证消失。
- 内核 / 桌面壳 ≠ 产品发布 ≠ 五端一致性验收（G4 仍只完成 Windows 单端）≠ 第三方安全审计。
- `bundle.icon` 是自生成占位图标，非产品美术资源。

## §2 增量写回：设计与密码学依据

### 2.1 为什么能复用对象文件

对象文件的 AAD = `jti | format | oid | chunk_index | chunk_count | header_hash`，
**`name` 与 `parent_id` 只在 `manifest` 的 record 里，不进入对象 AAD**（m3-architecture / vault-format-spec §2.1）。
因此：

- 内容未变的节点：直接复用原 `object_id` 与盘上密文对象文件，**不重加密、不重写 d/**。
- 重命名 / 移动：只改 `manifest` record 的 `name`/`parent_id`，对象文件可安全复用。
- 内容已变 / 新建：分配**全新 128-bit object_id + 全新随机 nonce** 重加密（严格 copy-on-write，绝不原地改密文）。

`vault.meta` 头（含 `versionMac`）在 mounted session 内**永不重写**，不重算 Argon2id / AES-KW。

### 2.2 core API

- `pub struct IncNode { name, kind, parent_index: Option<usize>, plaintext: Vec<u8>, reuse_oid: Option<[u8;16]> }`
  - `reuse_oid=Some(oid)`：尝试复用；helper 会读 `d/<b32[:2]>/<b32>` 并 `parse_object_file`，
    校验 header 的 `oid` 与 `chunk_count` 与当前明文一致才复用，否则**降级为全新 oid + 重加密**（防御文件缺失/损坏）。
  - `None`：新建或内容已变，全新 oid + nonce。
- `pub fn rewrite_vault_incremental(root, &Unwrapped, device_id, &[IncNode]) -> Result<Vec<String>, CoreError>`
  - 先决定全部 oid 与 `must_write`，再写盘、建 record → **切片顺序无关**。
  - manifest 仍 `manifest_key = hkdf_expand(master, b"manifest"+mid)` + 全新 24B nonce + `aad_manifest(jti,format,mid,seq)`。
  - **崩溃安全顺序不变**：先写全部新对象 → `manifest.tmp` + `sync_all` + 原子 rename（Windows 覆盖失败删旧再 rename）→ 清 `d/` 中不在 keep 的孤儿。
  - keep 集合**包含复用 + 新分配的全部 id**，否则会误删复用对象。
  - 返回与 nodes 同序的 `Vec<String>`（b32 oid 列表），供调用方回写状态。
- `rewrite_vault_from_plaintext` = 把 `&[PlainNode]` 全部映射为 `reuse_oid:None` 后调增量版 → M3 全量语义与向量逐字节契约不变。

### 2.3 VaultFs 改造

- `Node` 增 `oid: Option<Vec<u8>>`（盘上 16 raw bytes）与 `dirty: bool`。
- `from_open` 用 `b32d(rec["id"])` 保留原 oid，`dirty=false`；解析失败降级为「会话新建」（安全但不省 IO）。
- 标 `dirty=true` 的点：`write_at`（写字节）、`open` 的 `truncate` 清空、新建文件/目录。
  **重命名/移动不标 dirty**（只改 name/parent，不进对象 AAD）。
- `persist()` 持内存锁跨「快照 → 增量写回 → oid 回写」全过程（同步函数无 await，helper 不回调本结构）；
  比 M3「出锁再写」更抗并发丢改动。写回成功把 `b32d(all_ids[pos])` 写回 `n.oid` 并清 dirty。

## §3 验证结果（全部本机 Windows 实测，EXIT 为真值）

| 项 | 命令 | 结果 |
|---|---|---|
| core 增量 helper 编译 | `cargo check -p mylock-core` | EXIT=0 |
| 全 workspace 编译 | `cargo check --workspace` | EXIT=0 |
| mount 测试套件 | `cargo test -p mylock-mount --test mount` | **7 passed / 0 failed** |
| 全量回归 | `cargo test --workspace --locked` | **26 passed / 0 failed**（7 KAT + 4 conformance + 8 vectors + 7 mount） |
| 全量构建 | `cargo build --workspace --locked` | EXIT=0 |
| 向量门禁（回归） | `check_vectors.py` / `compute_fingerprint.py` / `gate_selftest.py` | OK / 指纹命中 / **5/5 PASS** |
| sidecar 二进制构建 | `cargo build -p mylock-mount --bin mylock-serve` | EXIT=0 |
| sidecar stage | `tools/prepare-sidecar.ps1` | `mylock-serve-x86_64-pc-windows-gnu.exe`（66,303,385 B），triple 自动取 `rustc -vV` |
| 桌面壳编译 | `cargo check`（desktop/src-tauri） | EXIT=0，`Finished dev profile in 1.28s`；build script 已把 sidecar 复制为 `target/debug/mylock-serve.exe` |

**26 测试构成**：M3 的 21 个零回退（7 KAT + 8 vectors.rs + 6 mount.rs→现 7）
+ M4 第 1 点 4 个 conformance + M4 第 2 点新增 1 个增量专项 = 7+4+8+7 = 26。

### 3.1 增量写回的关键测试证据

- `write_back_persists_new_file_and_keeps_header_unchanged`：`object_bytes()` 返回「b32 文件名 → 密文文件逐字节内容」；
  写新文件后断言 `before` 里每个未变对象的 `after.get(id) == Some(bytes)`。
  **因密文含全新随机 nonce，重加密必然不同，字节相同即证明没重加密**。
- `incremental_writeback_only_rewrites_changed_objects`：改写 hello.txt 后仅 1 个未变对象复用且字节不变；
  rename 后对象文件集合完全不变（证明重命名只改 manifest）；删除后复用对象不被误删；每次重开 `full_open` 内容逐字节一致。

## §4 交付物清单

```
core/crates/mylock-core/src/lib.rs                IncNode + rewrite_vault_incremental + 全量退化包装
core/crates/mylock-mount/src/lib.rs               VaultFs: Node.oid/dirty、persist 增量、写操作标脏、docstring
core/crates/mylock-mount/tests/mount.rs           object_bytes + 复用断言 + incremental_writeback_only_rewrites_changed_objects
desktop/src-tauri/tauri.conf.json                 bundle.active/targets/icon/externalBin/short+longDescription
desktop/src-tauri/src/main.rs                    Manager + resolve_serve_bin 四级 + serve_bin_path 命令 + 诚实文案
desktop/ui/index.html                            sidecar 文案 + 引擎路径按钮 + 未签名告警
desktop/src-tauri/icons/{32x32,128x128,128x128@2x}.png   make_icons.py 派生（占位图标）
desktop/SIGNING.md                                签名/证书诚实边界与发布前清单
tools/prepare-sidecar.ps1                         构建产物 → Tauri externalBin 约定命名
tools/make_icons.py                               从 M3 ICO 派生 bundle PNG 图标
.gitignore                                        新增 desktop/src-tauri/binaries/
m4-step2-report.md                                本报告
产品分析与概要设计.md                            升 v1.6（M4 第 2 点当时；M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step4-report.md](m4-step4-report.md)）
```

## §5 硬门槛对照（沿用 vault-format-spec §12 / m4-report §6）

| 门槛 | 状态 |
|---|---|
| ① 五端 + 恢复 CLI 逐字节一致（G4） | 🟡 部分：Windows 单端实测；三桌面 CI 已配置未实跑；Android/iOS 未实现 |
| ② 字段冻结 / 权威 KAT / 负向向量 | ✅ 未回退（26/0，向量门禁 5/5） |
| ③ 开放格式 + 独立恢复 CLI | ✅ 未回退 |
| ④ 崩溃安全 + 可移植密文容器 | ✅ 增量写回保持原子 manifest + 清孤儿 + vault.meta 不重写 |
| ⑤ 第三方安全审计 | ⏳ 未做 |

## §6 唯一冻结诚实边界

见 §1「没做」逐条 + [m3-report.md](m3-report.md) §6 / [m3-architecture.md](m3-architecture.md) §4 /
[desktop/SIGNING.md](desktop/SIGNING.md) §3 / [tests/vectors/CONTRACT.md](tests/vectors/CONTRACT.md) §6。
本阶段新增并冻结：**增量写回不是增量日志（manifest 每次全量重建）；安装包未签名、未产出成品、证书自备；
图标为占位资源**。

## §7 下一步（需新授权）

1. M3 汇报「下一步」第 3 条：WinFsp 增强路径（需另评 GPL-3.0 法务）。
2. M3 汇报「下一步」第 4 条：上架前第三方审计、隐私政策/白皮书、上架法务核验。
3. 把仓库推到远端启用 Actions，拿 macOS/Linux 真实绿灯（M4 第 1 点的 CI 配置落地）。
4. 真增量 manifest（增量日志）与 per-request 认证（当前 loopback WebDAV 的安全边界），若产品需要须单独立项评审。

## §8 文档接口

- 主文档：[产品分析与概要设计.md](产品分析与概要设计.md) v1.6（M4 第 2 点当时；M4 第 3 点后升 v1.7，见 [m4-step3-report.md](m4-step3-report.md)；M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step4-report.md](m4-step4-report.md)）
- 上游：[m2-report.md](m2-report.md) / [m3-architecture.md](m3-architecture.md) / [m3-report.md](m3-report.md) / [m4-report.md](m4-report.md)
- 契约：[vault-format-spec.md](vault-format-spec.md) v1.1.1 / [tests/vectors/CONTRACT.md](tests/vectors/CONTRACT.md) / [crypto-review-m1.md](crypto-review-m1.md)
- 签名：[desktop/SIGNING.md](desktop/SIGNING.md)
