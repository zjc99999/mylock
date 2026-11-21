# MyLock 安全白皮书（草案 · 第三方审计准备材料）

> **性质声明（务必先读）**：本文是**内部技术白皮书草案**，用途是「上架前第三方安全审计的准备材料」与「面向技术决策者的设计说明」。它如实复述本仓库**已经落地并通过本机测试**的设计，不是安全认证、不是第三方审计结论、不是渗透测试报告、不是法律意见。
> 版本：v0.1（草案） · 日期：2026-11-15 · 授权：用户 m03444「继续」（M4 第 4 点：上架前准备）
> 上游：[产品分析与概要设计.md](产品分析与概要设计.md) · [crypto-review-m1.md](crypto-review-m1.md) · [vault-format-spec.md](vault-format-spec.md) · [threat-model.md](threat-model.md) · [m2-report.md](m2-report.md) · [m3-architecture.md](m3-architecture.md) · [m4-step2-report.md](m4-step2-report.md)
> **唯一冻结诚实边界**：内核/桌面壳 ≠ 产品发布 ≠ 五端一致性验收 ≠ 第三方审计；格式层是成熟范式组合、非创新；内部密码学评审 ≠ 第三方审计。

---

## 0. 一页纸摘要

**MyLock** 是「本地优先、零知识、多端同钥的私人数据保险柜」。它在用户**本机**把一组文件加密成一个**静态普通文件目录（vault）**，拔掉介质后是「谁也打不开的保险柜」；插回装了 App 的设备、输一次口令，它又变回用户原来那个文件夹。

反定位（这是产品概念的硬约束，不是营销话术）：**No Cloud · No AI · No Telemetry · No Account**。开发者/公司**从不接收、看不到、也无法访问**任何用户文件、口令、密钥或使用数据；没有服务端、没有账号体系、没有遥测、没有埋点、没有 AI。

本文回答三件事：①我们**怎么加密**（§2–§4）；②我们**防得住什么、防不住什么**（§5，十条诚实声明）；③**验证到什么程度、还差什么**（§6–§9）。

---

## 1. 威胁模型定位（为什么值得做）

SAAS 与 AI 背景下，隐私焦虑的真正变化不是「传输被窃听」——那早已被 TLS 解决——而是「**合法采集者**」的威胁模型迁移：云同步客户端、OS 索引/Recall、AI 训练数据抽取、桌面搜索、备份与遥测，会在用户**已授权**的前提下把明文复制走。MyLock 的定位不是「更强的 TLS」，而是「**让明文根本不出现在这些采集器能读到的地方**」。

必须诚实说清的边界（详见 [threat-model.md](threat-model.md) §3.1）：

- **仅未解锁态**防得住「合法采集者」：盘上永远是密文，采集器只能拿走乱码。
- **解锁态仅缓解**：一旦用户主动解锁并把 vault 暴露为可读写文件夹，运行中的明文就在内存与挂载点里；任何本机进程、任何能读该挂载点的采集器都可能拿到。这是「透明文件夹」这一形态的固有代价，不是缺陷可修补项。
- **库外持久副本锁定后不保证消失**：解锁期间用户/其他程序复制到别处的明文副本，锁定 vault 不会自动清除。

---

## 2. 密码学设计（已落地，逐字对齐 [vault-format-spec.md](vault-format-spec.md) v1.1.1）

### 2.1 算法与参数（单一算法，无运行时协商）

| 环节 | 算法/参数 | 说明 |
|---|---|---|
| KDF | Argon2id，m=65536 KiB、t=3、p=4、tag=32B、salt=16B | **必须显式 parallelism=4**；libsodium `crypto_pwhash` 硬编码 p=1，误用会跨端解锁得错误 Master → `E_AUTH_FAILED`（M1 PoC A 级实测） |
| 主密钥包裹 | RFC 3394 **AES-256-KW** | 用 KEK 把 `Master(32B) ∥ MAC_Key(32B)`=64B 包成 72B，存 `vault.meta` |
| 密钥派生 | HKDF-Expand(SHA-256) 域分离 | `wrap/password`、`wrap/recovery`、`meta-version-mac`、`manifest`+id、`object`+id |
| 内容/清单加密 | **XChaCha20-Poly1305**，24B 随机 nonce，16B tag | 单一 cipher；AES-GCM 运行时可选已删除；FIPS 场景另起 `format=2` |
| 分块 | 1 MiB / chunk，每块独立 nonce 与 tag | 空文件仍 1 个空 chunk（N4） |
| 防降级 | `versionMac = HMAC-SHA256(hkdf(mac_key,"meta-version-mac"), canonical(meta_core))` | **覆盖整个 meta 头**；解锁后比对 |
| 名称/目录 | 密文名 = 128-bit CSPRNG → **base32**（大写、无填充、26 字符），前 2 字符分片 | 大小写不敏感文件系统安全；不映射用户原始目录树 |

**已明确拒绝的过度设计**（见 [crypto-review-m1.md](crypto-review-m1.md) §9）：SIV 名称加密（RFC 5297 证明 SIV 是 AES 模式、无 XChaCha-SIV）、BLAKE3 额外校验和、全确定式 nonce、deterministic CBOR、per-record AEAD 日志、全量版本向量、自造 XChaCha wrap。

### 2.2 解锁顺序（§3.4，防御性降级）

1. **解锁前**：`verify_phase1` 只做结构检查 + **KDF 参数硬校验**（19456≤m≤1048576、2≤t≤10、1≤p≤16）。越界在 Argon2id 之前直接 `E_FORMAT`，**绝不分配 4 TiB 内存**（负向向量 `kdf_oob_m4tib` 实测）。
2. Argon2id → IKM → `wrap/<slot>` 域分离 KEK → AES-KW 解包 64B → `Master`/`MAC_Key`。
3. **解包后**才比对全头 `versionMac`（失败 `E_DOWNGRADE`）——即「versionMac 移解锁后验」。
4. 认证错误统一 `E_AUTH_FAILED`，不泄漏「是口令错还是槽错」。

### 2.3 两阶段 verify

- `phase1`（不解密）：结构 + KDF 硬校验 + orphan 检测（**`E_ORPHAN` 是警告非致命**）。
- `phase2`（解锁后）：逐对象、逐 chunk 用 AAD 重算认证；缺对象 `E_MISSING`，篡改 `E_CORRUPT`。

错误码集合：`E_FORMAT / E_AUTH_FAILED / E_DOWNGRADE / E_MANIFEST / E_CORRUPT / E_MISSING / E_ORPHAN`。

### 2.4 对象与清单的 AAD（字节级冻结，见 spec §2.1）

- 对象 header 固定 32B：`magic(4)="MYOB" | obj_version(1)=1 | object_id(16) | chunk_size(4) | chunk_count(4) | cipher_id(1)=1 | reserved(2)=0`，`header_hash=SHA-256(header)`。
- chunk AAD = `b"MYVAULT" + u32be(format) + jti.ascii + object_id_raw(16) + u32be(chunk_index) + u32be(chunk_count) + header_hash(32)`。
- manifest AAD = `b"MYVAULT" + u32be(format) + jti.ascii + manifest_id_raw(16) + u32be(manifest_seq)`。
- **canonical JSON**：键排序、控制字符一律 `\u00XX` 小写（`\n` 必须 `\u000a`）、非 ASCII 原样、禁浮点。
- `name`/`parent_id` **只存在于加密后的 manifest record，不进入对象 AAD** —— 这是 M4 第 2 点「增量写回」得以在重命名/移动时复用原密文对象的密码学依据。

---

## 3. vault 静态格式与可移植性（回答「拷到移动硬盘还是不是保险柜」）

```
<vault-root>/
├── vault.meta        # 公开头：magic/format/jti/key_epoch/cipher/kdf/wrapped[]/manifest_ref/versionMac
├── vault.meta.bak    # 同内容备份
├── manifest          # 加密的记录数组（nonce24 + XChaCha20-Poly1305 密文+tag）
├── README.txt        # 面向拾得者的恢复指引（无明文）
└── d/<2字符分片>/<26字符base32密文名>   # 每个对象一个密文文件
```

- **全目录用户数据零明文落盘**；`vault.meta` 是**公开头**（含 KDF 参数、salt、jti、key_epoch、密文包裹槽、versionMac），**不含用户数据明文**。
- **完全不依赖 OS/文件系统特性**：不依赖 xattr / NTFS ADS / 硬链接 / 符号链接 / 稀疏文件 / 权限位 / 大小写敏感。base32 大写定长名在大小写不敏感文件系统安全；1 MiB 分块规避 FAT32 4 GiB 单文件上限；目录扁平化不映射用户原始深层路径。
- 因此**拷到移动硬盘 / U 盘 100% 仍是同一个保险柜**（任何 OS 只看到乱码名 + 乱码内容），「离线移动介质」是 BYO 最纯粹的载体（物理无网络、天然无上传通道）。
- **三个诚实边界**：①透明性不随盘走（要读须在装了 App 的设备上用主口令/Recovery Key 解锁）；②不依赖 OS ≠ 不依赖解锁器 → 已把「开放格式规范 + 独立恢复 CLI」前移为 M1 必备；③拷贝完整性由 `manifest` + `vault verify` 保证（`verify` 绿 ≠ 未被恶意替换/不是最新版）。

金句：**「插上装了 App 的电脑、输一次口令，它是你原来那个文件夹；拔掉硬盘，它是谁也打不开的保险柜。」**

---

## 4. 桌面层形态（M3）与增量写回（M4 第 2 点）

### 4.1 loopback-only WebDAV（而非 WinFsp 驱动级挂载）

- 架构定案（[m3-architecture.md](m3-architecture.md) §1、[m4-step3-report.md](m4-step3-report.md) §0）：`mylock-serve` 在 `127.0.0.1:<port>`（默认 4918）起 WebDAV，OS 原生 `net use Z:` / `mount_webdav` 映射盘符。**零驱动、避 GPL-3.0 传染**。
- **无 per-request 认证**：loopback 上的 WebDAV 不校验调用者，**任何本机进程都可访问挂载点**。这是公开的已知限制，不是「已修复」项。
- `is_sync_root` **拒绝在云同步根目录内挂载**（OneDrive / Dropbox / Google Drive / iCloud Drive / 坚果云 / SharePoint / 百度网盘 / 阿里云盘 等），避免明文被云客户端采集。
- **不是 WinFsp/FUSE 驱动级挂载**：M4 第 3 点已评估并决策不采用 WinFsp（FLOSS 例外禁止与专有软件混用 + 6000 USD/3yr 商业许可 + GPL-3.0 Rust 绑定第二层 copyleft）；Dokany 仅列备选、需新授权。

### 4.2 增量写回（严格 copy-on-write，仍不是增量日志）

mounted session 内 **`vault.meta` 头字节保持不变**（不重算 Argon2id / AES-KW / versionMac）。每次持久化：

1. 内容**未变**的节点：复用原 128-bit object_id 与盘上原密文对象文件（**不重加密、不重写 d/ 文件**）；`name`/`parent_id` 只改 manifest。
2. 内容**已变/新建**的节点：分配**全新** object_id + **全新** 24B nonce，写全新密文文件（严格 copy-on-write，绝不原地改密文）。
3. manifest：每次用**全新 nonce** 全量重写，`manifest.tmp` + `fsync` + 原子 `rename`（Windows 覆盖失败则删旧再 rename）。
4. 崩溃安全顺序：**先写全部新对象 → 原子替换 manifest → 再清理旧孤儿**（崩溃只留无害孤儿）。
5. `keep` 集合必须含**复用 + 新分配的全部 id**，否则会误删复用对象（已由专项测试覆盖）。

**诚实边界**：这仍是「每次写重建整份 manifest」，**不是增量日志 / 不是增量 manifest**；大库高频写的性能收益有限。

---

## 5. 十条对外诚实声明（[crypto-review-m1.md](crypto-review-m1.md) §8 为唯一冻结措辞，逐条不得软化）

1. **用户数据零明文**落盘 —— 但**不写「零元数据」**：公开元数据（对象数量、大致大小、密文目录结构）可见。
2. **形状泄漏未 100% 消除** —— 对象数量/大致大小仍可能泄漏。
3. **XChaCha20-Poly1305 非 IETF 标准**（draft-irtf-cfrg-xchacha-03），AES-GCM 未采用；FIPS 场景需另起 `format=2` 并重新评审。
4. **`verify` 绿 ≠ 最新版**：通过校验不代表未被恶意替换或不是旧版副本。
5. **换口令是零吊销**：`change-password` 只重包裹头，旧 `vault.meta` 副本 + 旧口令持有者仍可解密；真吊销必须 `rotate-keys`（新 Master/MAC/salt/jti、key_epoch++、全量重加密 + 物理销毁旧介质）。
6. **仅未解锁态防得住合法采集者**；解锁态只缓解。
7. **口令熵是唯一安全上限**：Argon2id 只抬高单次爆破成本，不能替代用户选一个高熵口令；离线无限爆破在已拿到 vault 副本时仍可能成功。
8. **承认「可见」**：App 不隐藏 vault 的存在，只隐藏内容。
9. **不防已攻陷的设备**（keylogger / 内存取证 / 已解锁会话内恶意进程）。
10. **格式层是成熟范式组合、非创新；内部密码学评审 ≠ 第三方审计**（Cryptomator 已做 BYO 密文容器十年；真正空白是「威胁模型迁移 + 双形态同钥」）。

---

## 6. 验证状态（本机实测，非第三方审计）

| 层次 | 结果 | 证据 |
|---|---|---|
| 权威 KAT | 26/26（PoC）+ 7 单测（M2 Rust） | RFC 3394 §4.6 / draft-xchacha §A.3.1 / RFC 9106 §5.3 / RFC 5869 A.1 / RFC 4231 TC1 / canonical profile |
| 逐字节契约 | Rust 端复现 `test-vectors.json` 30 `derived_outputs` | [tests/vectors/](tests/vectors/) |
| 全量测试 | **26 passed / 0 failed**（`cargo test --workspace --locked`） | 7 KAT + 4 conformance + 8 集成（13/13 负向、零/真吊销、200 轮 fuzz「永不返回未认证明文」、2.5 MiB 分块）+ 7 mount（含增量写回专项） |
| 向量门禁 | `check_vectors.py` OK / 指纹门禁命中 / `gate_selftest.py` 5/5 | 锚 `07af0e8b…9939`（13499 B、纯 CRLF×190）、G4 指纹 `d45b8659…02fa` |
| 真实网络链路 | PROPFIND 207 / PUT 201 / GET/export 逐字节一致 | [tools/m3-smoke.ps1](tools/m3-smoke.ps1) |
| 桌面壳 | `cargo check` EXIT=0（Tauri 2 + sidecar） | [desktop/SIGNING.md](desktop/SIGNING.md) |

**G4「五端 + 恢复 CLI 逐字节一致」硬门槛：仅 Rust/Windows 单端完成**。macOS / Linux 为 CI 配置但**未实跑**；Android / iOS **未实现**。详见 [tests/vectors/CONTRACT.md](tests/vectors/CONTRACT.md) §3。

---

## 7. 依赖与许可（本地只读审计，[tools/audit_licenses.py](tools/audit_licenses.py)）

- core `Cargo.lock` 147 包（134 可核验）；desktop `Cargo.lock` 466 包（300 可核验）。
- **可核验范围内 GPL 家族 0**；已存在 **MPL-2.0 弱 copyleft**（core 1 个 `htmlescape`、desktop 6 个 `cssparser`/`selectors` 系）。第一方 crate 双许可 `MIT OR Apache-2.0`。
- **179 个包本地不可核验**（多为非 Windows 目标分支 / 未展开），**不作任何许可声明**；「全图无 GPL / 纯宽松许可」是不准确的说法。
- 引入 WinFsp 会引入**首个强 copyleft**（已决策不采用，[m4-step3-report.md](m4-step3-report.md)）。
- 上架前须 `cargo fetch` 补齐 179 包、生成完整 `THIRD_PARTY_NOTICES.md`（含 MPL-2.0 §3.1–3.4 源码可得义务与 Apache-2.0 §4(d) NOTICE 传递），并由律师签发。详见 [prelaunch-checklist.md](prelaunch-checklist.md)。

> **M5 追加补遗（2026-11-15 零成本最小动作已执行；追加记录、上面 5 条是 fetch 前时点口径、保留不改写）**：两 workspace `cargo fetch --locked` 均 EXIT=0、两份 `Cargo.lock` 未改动，**不可核验包 179→0（613 包可核验全覆盖）**；已生成 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)（613 包逐行 + 各许可专门段）与只读幂等生成器 [tools/gen_third_party_notices.py](tools/gen_third_party_notices.py)。**第 6 个 MPL 已查明 = `htmlescape 0.3.1`**（`Apache-2.0 / MIT / MPL-2.0` 三许可、非 SPDX "OR"、与 core 那 1 个同包）；不存在第 6 个纯 MPL-2.0 包、不臆造；fetch 后 desktop 纯 MPL-2.0 由 5 增至 7（新增 `cssparser 0.38.0` / `selectors 0.41.0`）。**fetch 后新观察（事实、非法律结论）**：`r-efi 5.3.0` / `6.0.0` = `MIT OR Apache-2.0 OR LGPL-2.1-or-later`（SPDX OR 表达式里的 LGPL 选项），使脚本 GPL-expression hits 由 core 0 / desktop 0 变为 core 1 / desktop 2；`r-efi` 是 `getrandom` 在 UEFI + `efi_rng` 下的 **target-specific 传递依赖、对 `x86_64-pc-windows-gnu` 不参与编译**，不作「已编译进产物」断言。**「可核验范围内 GPL 家族 0」是 fetch 前（434 包可核验子集）口径**；对外措辞仍为「**无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft**」+ 补充该观察，**绝不宣称「依赖树绝对没有 GPL/LGPL 字样」**。`htmlescape` 三许可择一与 `r-efi` 逐依赖许可决定、律师签发与 SBOM 仍属外部闸门（go 判据未达成）。详见 [zerocost-compliance-report.md](zerocost-compliance-report.md) / [m4-step4-report.md](m4-step4-report.md) §2.4.2。

---

## 8. 明确「未做 / 不是」清单（上架前外部闸门，本仓库无法代劳）

- ❌ **第三方安全审计**（密码学/代码/渗透测试）、安全认证（如 CSPN / FIPS / SOC2）—— 本文是审计准备材料，不是审计结果。
- ❌ **法律意见 / 律师签发**（加密出口管制分类、PIPL/GDPR 适用性、商店合规、第三方许可告知义务）—— 见 [m4-step4-report.md](m4-step4-report.md) 与 [prelaunch-checklist.md](prelaunch-checklist.md)。
- ❌ **代码签名 / 可分发安装包**：`bundle.active=true` 但**未跑 `tauri build`、未产出 .msi/NSIS、未签名**；证书须自备（[desktop/SIGNING.md](desktop/SIGNING.md)）。
- ❌ **五端一致性验收（G4）**、移动端实现、macOS/Linux runner 绿灯。
- ❌ **产品发布**：内核 + 桌面壳是可编译脚手架，不是成品。
- ❌ **WinFsp 驱动级挂载**、per-request 认证、真增量 manifest、增量日志。

---

## 9. 复现与文档索引

```powershell
# 全量测试 + 构建
cd D:\Projects\dsh\mylock\core
cargo test --workspace --locked
cargo build --workspace --locked
# 向量门禁 + 负向自检 + 指纹门禁
& 'C:\Python312\python.exe' ..\tools\check_vectors.py
& 'C:\Python312\python.exe' ..\tools\compute_fingerprint.py
& 'C:\Python312\python.exe' ..\tools\gate_selftest.py
# 依赖许可只读审计
& 'C:\Python312\python.exe' ..\tools\audit_licenses.py
```

一手设计文档：[产品分析与概要设计.md](产品分析与概要设计.md) · [vault-format-spec.md](vault-format-spec.md) · [crypto-review-m1.md](crypto-review-m1.md) · [threat-model.md](threat-model.md) · [m2-report.md](m2-report.md) · [m3-architecture.md](m3-architecture.md) · [m4-report.md](m4-report.md) · [m4-step2-report.md](m4-step2-report.md) · [m4-step3-report.md](m4-step3-report.md) · [m4-step4-report.md](m4-step4-report.md)
隐私与上架文件：[privacy-policy.md](privacy-policy.md) · [security-whitepaper.md](security-whitepaper.md)（本文） · [prelaunch-checklist.md](prelaunch-checklist.md) · [m4-step4-report.md](m4-step4-report.md) · [desktop/SIGNING.md](desktop/SIGNING.md)
