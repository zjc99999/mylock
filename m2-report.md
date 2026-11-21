# M2 内核实现 · 阶段报告

> 版本：v1.0（M2 内核实现完成 · 逐字节验收达成）
> 日期：2026-11-14
> 授权：用户 m00983「进入 M2 内核实现」——本次是本项目**首次正式授权进入实际开发**（此前 m00241 选「先做 M1 密码学评审」、m00606 授权 PoC 时均明确「不进入开发」）。
> 验收契约：[poc/test-vectors.json](poc/test-vectors.json)（M1 PoC 产出的唯一逐字节契约）+ [vault-format-spec.md](vault-format-spec.md) v1.1.1（§2.1 字节级拼接冻结）。
> 文档状态：**内核实现完成并通过验收；不等于产品发布、不等于第三方审计通过、不等于五端一致性验收通过**。

---

## 1. 本阶段做了什么 / 不做什么

**做了**：把 v1.1.1 规范用 Rust 实现为一个**加密核心 crate** + 一个**独立恢复 CLI**，并用测试证明它能逐字节复现 M1 PoC 的向量契约。

**没做（诚实边界，必须保留）**：
- 不做 UI / 不做 Tauri 桌面壳 / 不做移动端（那是 M3+ 的桌面层、移动层）。
- 不做透明文件夹挂载（`mount` 命令在内核 CLI 里**明确拒绝实现**——它依赖 WinFsp/FUSE/OS，属桌面层，§6.8 两层拆分）。
- 不做网络/账号/云同步/遥测（产品定位 `No Cloud · No AI · No Telemetry · No Account` 不变）。
- **不是第三方密码学审计**（crypto-review-m1.md §8 第 10 条、§9 硬门槛 5 仍挂起）。
- **G4「五端 + 恢复 CLI 逐字节一致」只完成「Rust 单端复现 PoC 单端」，跨语言/跨平台一致性仍未验收**——这是 §12 硬门槛 1 的剩余部分。

**为什么这样切**：M1 PoC（Python，一次性验证脚手架）已经把「字节级拼接约定」钉死在 `test-vectors.json`；M2 的价值就是把它变成一个**可编译、可测试、可交付给后续桌面/移动层复用的真正内核**，而不是再写一遍原型。

---

## 2. 运行时与工具链（A 级实测）

| 项 | 结果 |
|---|---|
| Rust | rustc / cargo **1.100.0**（`stable-x86_64-pc-windows-gnu`，2026-11-09 发布） |
| Host 选择 | GNU host，复用本机 `C:\mingw64\bin\gcc.exe`，避开 MSVC Build Tools（本机无 VS/cl.exe） |
| 安装 | `tools/install-rust.ps1` v2（`install-rust.ps1` v1 因 `$ErrorActionPreference='Stop'` 把 rustup 的 stderr info 误杀，已修；日志 `tools/rust-install.log` 末尾 `INSTALL_OK`） |
| crates.io | 可达（`index.crates.io/config.json` HEAD 200），下载 47 包成功 |
| 钉死版本 | argon2 0.6.0 / chacha20poly1305 0.11.0 / aes-kw 0.3.1 / aes 0.9.4 / aead 0.6.1 / hkdf 0.13.0 / hmac 0.13.0 / sha2 0.11.1 / subtle 2.6.1 / zeroize 1.9.1 / base64 0.23.1 / data-encoding 2.11.1 / serde 1.0.229 / serde_json 1.0.154 / hex 0.4.3 / getrandom 0.4.3 / uuid 1.29.2 / clap 4.6.7（2026-11-14 crates.io max_stable 逐一核实） |

所有 crate API **先从 `.crate` 源码核对真实签名再写代码**（`tools/*-dump.txt`、`tools/dump_*.py` 为一次性探查工具，非产品代码），避免凭记忆调错库——最关键是 `argon2::Params::new(m_cost, t_cost, p_cost, output_len)` 的**参数顺序 m,t,p,output**（编译器抓不到顺序错，会导致整条字节链全错）。

---

## 3. 交付物清单

| 文件 | 作用 |
|---|---|
| [core/Cargo.toml](core/Cargo.toml) | workspace root：resolver=2，members=[`crates/mylock-core`, `crates/mylock-cli`] |
| [core/crates/mylock-core/Cargo.toml](core/crates/mylock-core/Cargo.toml) | 加密内核 crate，密码学依赖全部钉死 |
| [core/crates/mylock-core/src/lib.rs](core/crates/mylock-core/src/lib.rs) | **内核主体**（~1570 行）：canonical JSON / Argon2id / AES-KW / HKDF-Expand / XChaCha20-Poly1305 / HMAC-SHA256 / 字节级拼接 / 建库 / 两阶段 verify / 解锁 / 凭据轮换 / 独立恢复 / 权威 KAT 单测 / 多分块往返 |
| [core/crates/mylock-core/tests/vectors.rs](core/crates/mylock-core/tests/vectors.rs) | **验收测试**（8 个集成测试）：逐字节复现 30 个 derived_outputs + 主链路 + 13/13 负向 + 恢复槽端到端 + 零/真吊销 + 确定性 fuzz 骨架 |
| [core/crates/mylock-cli/Cargo.toml](core/crates/mylock-cli/Cargo.toml) | 独立恢复 CLI crate（`[[bin]] name="mylock"`） |
| [core/crates/mylock-cli/src/main.rs](core/crates/mylock-cli/src/main.rs) | **独立恢复 CLI**（9 命令，不联网、不依赖主 App）：`info` / `verify --json` / `create` / `export` / `export-object` / `change-password` / `rotate-recovery` / `rotate-keys` / `mount`(明确拒绝) |
| [core/test-all.log](core/test-all.log) | 全量测试日志（全绿） |

复现命令（Windows，需先把 `C:\Users\zjc_9\.cargo\bin` 与 `C:\mingw64\bin` 加入 PATH）：
```
cd D:\Projects\dsh\mylock\core
cargo build --workspace
cargo test --workspace
```

---

## 4. 第一验收门槛：Rust 端逐字节复现 PoC 向量 —— **达成**

`cargo test -p mylock-core --test vectors` 的 `derived_outputs_match_byte_for_byte` 把 `test-vectors.json` 里 **30 个派生输出逐字段比对**（IKM_password / IKM_recovery / 两个 KEK / 两个 wrapped.blob / versionMac / versionMac.input_hex / versionMac.key_hex / manifest_key / object_key_A/B / object_A·B 文件 hex / 4 个 combined_hex / manifest 文件+combined+plaintext_canonical hex / chunk AAD / manifest AAD / meta_canonical / RK display / b32 ids），**全部逐字节一致，零差异**。

这证明：Rust 内核对 §2.1 冻结的字节级约定（u32 BE、jti ASCII、object_id 用 16B 原始值参与 HKDF info 与 AAD、HKDF info 拼接、对象 header 固定 32B、chunk=`nonce24+u32(ct_len)+ct+tag16`、AAD 拼接、canonical JSON 控制字符一律 `\u00xx`、两 secret 各自 Argon2id→slot 域分离）理解与 PoC **完全一致**。

**M2 与 PoC 的一处有意修正**：PoC 的 `full_open` 用模块常量 `MANIFEST_ID`（简化），M2 按规范**从 `meta.manifest_ref.id` 读取** manifest id；固定向量下两者值相同，不影响逐字节复现，但 M2 更符合规范语义。

---

## 5. 全量测试结果（全绿）

| 测试组 | 数量 | 结果 | 覆盖 |
|---|---|---|---|
| lib 单元测试 `kat_tests` | 7 | **7 passed** | 权威 KAT 直接锚定 + 多分块往返 |
| 集成测试 `vectors.rs` | 8 | **8 passed** | 逐字节契约 + 主链路 + 负向 + 吊销 + fuzz |
| mylock-cli / doc-tests | 0 | ok | — |
| **合计** | **15** | **15 passed / 0 failed** | `test-all.log` |

### 5.1 集成测试 8 项（`core/crates/mylock-core/tests/vectors.rs`）
1. `derived_outputs_match_byte_for_byte` —— 30 字段逐字节一致（§4）。
2. `canonical_profile_matches` —— versionMac.input_hex / meta_canonical_hex / manifest_plaintext_canonical_hex 三条 canonical JSON 契约。
3. `full_pipeline_and_positive_vectors` —— 建库 → **整盘复制**（§6.8 载体无关）→ verify_phase1 → `full_open(password)` → export 比对 → `full_open(recovery)` 恢复槽端到端 → RK 喂 password 槽得 `E_AUTH_FAILED`（F13 域分离）→ `export_object_by_id` 独立恢复。
4. `negative_vectors_13_of_13` —— 13 个 mutate 的 expect 全部取自契约 JSON，命中率 13/13：`E_DOWNGRADE` / `E_FORMAT`(含 KDF m=4TiB 越界在 Argon2id 之前被拦) / `E_AUTH_FAILED` / `E_CORRUPT` / `E_MISSING` / `E_MANIFEST` / `NO_ERROR`(孤儿是警告非致命)。
5. `recovery_display_roundtrip` —— RK 展示串（含校验位）往返，校验位篡改被拒。
6. `change_credential_is_zero_revocation` —— 换口令后对象文件逐字节不变、新口令可解、旧口令 `E_AUTH_FAILED`、**旧 RK 仍可解**（诚实兑现「零吊销」）。
7. `rotate_keys_is_true_revocation` —— `key_epoch+1`、jti 轮换、旧口令打不开新库、新口令+新 RK 可解（诚实兑现「真吊销」N2）。
8. `fuzz_never_returns_unauthenticated_plaintext` —— 确定性 fuzz 骨架（§9 硬要求）。

### 5.2 权威 KAT 单元测试 7 项（`lib.rs` 内 `#[cfg(test)] mod kat_tests`）
不只靠 PoC 间接对齐，直接把每个 Rust 原语锚定到一手页已发布向量：

| 测试 | 一手来源 | 关键字面值 |
|---|---|---|
| `rfc3394_aes_kw_256_kek_kat` | RFC 3394 §4.6 | 密文 `28c9f404…b7a02dd21`；unwrap 还原；本产品实况 **64B→72B** |
| `xchacha_draft_a31_kat` | draft-irtf-cfrg-xchacha-03 §A.3.1 | CT + Tag=`c0875924c1c7987947deafd8780acf49` 逐字节通过；AAD 篡改失败；**空明文仍 16B tag**（坐实 N4） |
| `rfc5869_hkdf_sha256_kat` | RFC 5869 A.1 | PRK=`07770936…2b3e5`、OKM 前 32B=`3cb25f25…4c5bf`；仅 Expand（跳 Extract） |
| `rfc4231_hmac_sha256_kat` | RFC 4231 TC1 | `b0344c61…e32cff7` |
| `argon2id_params_and_parallelism_are_binding` | RFC 9106 §4 参数 | p=4 输出 `03aab965…6d0c2e`、p=1 不同且 `assert_ne`——**坐实 crypto-review §12.4：不可用 libsodium `crypto_pwhash`（硬编码 p=1）替代** |
| `canonical_json_profile_kat` | vault-format-spec §3.3 | `{"a":"line\u000aquote\"","b":true,"f":null,"n":"保险柜","z":1}`（\n 必须 `\u000a`、非 ASCII 原样、禁浮点） |
| `multi_chunk_roundtrip` | vault-format-spec §4.2 | 2.5 MiB → 3 chunk（1+1+0.5 MiB）→ 整盘复制 → `full_open` 逐字节一致 → `export_object_by_id` 一致 |

> 诚实说明：RFC 9106 §5.3 的**精确** KAT 含可选 Secret[8]+AD[12]，而 `argon2` crate 高层 `hash_password_into` 不暴露 AD 输入，故 Rust 侧**未**写该精确 KAT 的伪通过；改以「同参确定性 + p=4≠p=1」锚定，PoC 侧已用 `argon2.low_level.core` 复现过完整 §5.3（见 [poc-report-m1.md](poc-report-m1.md) §2）。

### 5.3 模糊测试骨架（确定性，无外部 crate）
`fuzz_never_returns_unauthenticated_plaintext`：固定种子 xorshift，对 `vault.meta` / `manifest` / 对象文件做 **200 轮**随机单字节翻转 / 随机截断，断言 ①永不 panic（catch_unwind）②要么报**已知错误码**、要么返回与原始明文逐字节一致的内容——**绝不返回未认证明文**（兑现 §9 硬要求）。
为控制耗时，用 `m=19456,t=2,p=1`（恰是 §3.5 硬校验下界）建库，使 `full_open` 仍走真实解锁路径但每轮 Argon2id 成本最低；200 轮约 44s（若用全量 m=65536,t=3,p=4 约 210s）。

---

## 6. 独立恢复 CLI 冒烟（8 步，全部符合预期）

`core/target/debug/mylock.exe`，不联网、不读账号：

| 步 | 命令 | 结果 |
|---|---|---|
| 1 | `create` | 新 jti `113d6b44-…`、RK 展示串 `E000-TU6J…00DB` |
| 2 | `info` | format=1 / cipher=XCHACHA20-POLY1305 / kdf m=65536,t=3,p=4,salt16B（不解密、不需凭据） |
| 3 | `verify --json` | `{"phase":"pre-unlock","result":"OK","objects":1,"bytes":91}`（布尔 flag 解析修复生效） |
| 4 | `export --password` | `note.txt` 15B = `top secret note` 逐字节还原 |
| 5 | `rotate-keys` | 新 jti `91a69ce8-…`、新 RK `E001-TROK…20D3`、**key_epoch 0→1** |
| 6 | 旧口令开新库 | `E_AUTH_FAILED: aes-kw unwrap: integrity check failed`（真吊销成立） |
| 7 | 新口令开新库 | 成功 |
| 8 | `mount` | 明确拒绝（透明挂载属桌面层，§6.8） |

CLI 在 `change-password` / `rotate-recovery` 成功后**主动打印「零吊销」诚实声明**，`rotate-keys` 成功后打印「须物理销毁旧介质」——把 crypto-review §8 的措辞冻结落到用户可见处。

---

## 7. 已修的工程问题（教训）

| 问题 | 根因 | 修法 |
|---|---|---|
| rustup 安装脚本 v1 失败 | `$ErrorActionPreference='Stop'` + `2>&1 |` 把 rustup 的 stderr info 当 NativeCommandError | v2 改 `$ErrorActionPreference='Continue'` + `cmd /c "... > out 2> err"` 重定向到文件 |
| `Hmac::new_from_slice` E0599 | hmac 0.13 / digest 0.11 的 `new_from_slice` 来自 `KeyInit` trait | `use hmac::KeyInit as _` |
| `XNonce::from_slice` 弃用警告 | aead 0.6 / hybrid-array 新 API | 改 `core::convert::TryFrom::try_from(nonce)` |
| `--json`/`--help` 被当取值选项 | `Args::parse()` 先判 `strip_prefix("--")` 消费下一 arg | 布尔 flag 判断提到 `strip_prefix` 之前 |
| `rotate_keys` 引用 `exported_by_id` 不存在 | `OpenResult` 原只有按 name 索引的 `exported` | 增 `exported_by_id: BTreeMap<String,Vec<u8>>`，`full_open` 同时按 id 索引 |
| 1 MiB 分块只有「读」没有「写」 | 初版 `build_vault` 写死 1 chunk | `ObjectSpec.nonce: Vec<u8>` → `nonces: Vec<Vec<u8>>`，`chunk_count_for` + 每 chunk 独立 CSPRNG nonce 循环写 |
| fuzz 测试 210s | 每轮跑全量 Argon2id | 用 §3.5 下界 KDF 建库，44s |
| `argon2` parallelism 语义 | libsodium `crypto_pwhash` 硬编码 p=1 | 坚持用 RustCrypto `argon2` crate 显式 `parallelism=4`，KAT 测试 `assert_ne(p4,p1)` 防回归 |

---

## 8. 硬门槛对照（vault-format-spec.md §12）

| 门槛 | 状态 |
|---|---|
| ① §9 正负向向量实测通过 + **五端 + 恢复 CLI 逐字节一致** | **部分**：正负向向量 + Rust 单端 + 恢复 CLI 已通过；**五端跨平台一致性仍 ⏳**（本阶段只做了 Rust 单端） |
| ② threat-model §6 K1–K10 回填 | ✅ 已在 M1 评审完成 |
| ③ 主文档同步 | ✅ 本报告 + 主文档 v1.3 同步（M2 当时；M3 后升 v1.4、M4 第 1 点后升 v1.5、M4 第 2 点后升 v1.6、M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m3-report.md](m3-report.md) / [m4-step4-report.md](m4-step4-report.md) / [m4-report.md](m4-report.md) / [m4-step2-report.md](m4-step2-report.md) / [m4-step3-report.md](m4-step3-report.md)） |
| ④ §11 裁定全部有记录 | ✅ crypto-review-m1.md §11 + §12 |
| ⑤ 上线前第三方密码学审计 | ⏳ **仍未做**（内核实现 ≠ 审计通过） |

跨阶段边界：用户 m00983 已授权 M2 开发，但「**不等于产品发布 / 不等于五端验收 / 不等于第三方审计**」的诚实口径继续保留。

---

## 9. 下一步建议（用户明确选择前停留在当前状态）

1. **（倾向）M3 桌面层**：Tauri 2 + WinFsp/FUSE 透明文件夹挂载，复用 `mylock-core`；把「透明加密文件夹」这层 OS 依赖形态真正做出来，兑现 §6.8 两层拆分。**（后续实际执行注记：用户 m01639 授权 M3 后，因 WinFsp 未安装且 `winfsp` 是 GPL-3.0，改走 loopback-only WebDAV 零驱动路径，WinFsp 仅列增强路径；详见 [m3-architecture.md](m3-architecture.md) §1 与 [m3-report.md](m3-report.md)）**
2. **跨平台/跨语言一致性验收**：把 `poc/test-vectors.json` 归档为 `tests/vectors/`，加 CI 门禁；将来移动端（Android/iOS）用同一契约逐字节复现，清掉 G4 剩余部分。
3. **上架前**：第三方密码学审计、隐私政策与安全白皮书、移动端备份/密钥失效恢复规范、加密申报/出口管制法务核验。

---

## 10. 文档接口

本文与 [产品分析与概要设计.md](产品分析与概要设计.md) **v1.3（M2 当时版本；M3 后升 v1.4、M4 第 1 点后升 v1.5、M4 第 2 点后升 v1.6、M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m3-report.md](m3-report.md) / [m4-step4-report.md](m4-step4-report.md) / [m4-report.md](m4-report.md) / [m4-step2-report.md](m4-step2-report.md) / [m4-step3-report.md](m4-step3-report.md)）**（§7.1 技术栈 / §8.2 里程碑 M2 / §9.3 维护约定同步）、[vault-format-spec.md](vault-format-spec.md) v1.1.1、[crypto-review-m1.md](crypto-review-m1.md) v1.1+§12、[threat-model.md](threat-model.md) v1.1、[m3-report.md](m3-report.md)、[poc-report-m1.md](poc-report-m1.md) 相互引用。**参数总表与十条对外诚实声明以 crypto-review-m1.md §8 为唯一冻结措辞**，本文不新增、不矛盾。

> 🧭 一句话：**M2 把「纸上冻结的字节级契约」变成了「编译得过、测得过、能被独立恢复 CLI 取回数据的 Rust 内核」——逐字节验收达成，但产品发布、五端一致性、第三方审计仍在前方。**
