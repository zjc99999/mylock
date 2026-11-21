# M1 PoC 报告（可运行主链路 + 契约级测试向量）

> 性质：**一次性验证脚手架（throwaway verification scaffold），不是产品代码、不是 M2 开发、不是安全认证、不是第三方审计通过**。
> 授权：用户 m00606「根据你说的倾向执行」→ 选定「做可运行 PoC，只验主链路与测试向量，清掉能先验的硬门槛」。跨阶段边界（m00001「后面视情况进入开发阶段」）不动。
> 日期：2026-11-14　依据：[vault-format-spec.md](vault-format-spec.md) **v1.1.1**（§2.1 字节级拼接冻结 + §9 测试向量）、[crypto-review-m1.md](crypto-review-m1.md) v1.1 + §12 补遗、[threat-model.md](threat-model.md) v1.1、[产品分析与概要设计.md](产品分析与概要设计.md) v1.2（PoC 当时版本；M2 后升 v1.3、M3 后升 v1.4、M4 第 1 点后升 v1.5、M4 第 2 点后升 v1.6、M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m2-report.md](m2-report.md) / [m4-step4-report.md](m4-step4-report.md) / [m3-report.md](m3-report.md) / [m4-report.md](m4-report.md) / [m4-step2-report.md](m4-step2-report.md) / [m4-step3-report.md](m4-step3-report.md)）。
> 一句话结论：**主链路 ALL PASS、13/13 负向向量命中、恢复槽端到端跑通、每个密码学原语先经权威 KAT 校验 26/26 PASS**；产出的 [poc/test-vectors.json](poc/test-vectors.json) 是未来 Rust 实现必须逐字节对齐的唯一契约。但**G4「五端逐字节一致」尚未满足**（见 §6 诚实边界）。

---

## 1. 为什么要做、以及「不做什么」

M1 密码学评审把字段/算法/参数都冻结了，但有一类风险是纸面评审验不出来的：**「库调错了，但生成的向量照样自洽」**，以及 **「规范里 `‖`/`info`/AAD 只写了语义、没冻结字节，两个正确实现也可能逐字节不一致」**。PoC 的目的就是用一个可运行、可复现的脚手架把这些先验掉：

- ✅ 做：用**被审计过的库**按冻结参数跑通完整主链路；产出固定输入/输出测试向量；跑全部负向向量；把字节级拼接约定钉死并回写规范。
- ❌ 不做：不写产品代码（不是 M2 内核实现）、不做 UI/驱动/挂载、不做五端移植、不做性能优化、不做第三方审计、不宣称「已实现 / 已认证」。

**硬约束**：测试向量必须由被审计过的库产出，不能用自搓实现自证。

---

## 2. 运行时与原语（全部先过权威 KAT）

环境：Windows，python 3.12.0rc3 (`C:\Python312\python.exe`)。所有原语委托成熟库，且每个库调用在写 PoC 之前都先对照**权威已发布测试向量**逐字节校验（[poc/verify_primitives.py](poc/verify_primitives.py) → [poc/verify-primitives.json](poc/verify-primitives.json)，**26 PASS / 0 FAIL / 2 NOTE**）：

| 原语 | 库 | 权威 KAT | 结果 |
|---|---|---|---|
| XChaCha20-Poly1305 | PyNaCl 1.6.2 (libsodium) | draft-irtf-cfrg-xchacha-03 §A.3.1（ct/tag/decrypt、AAD 篡改、ct 篡改、空明文仍 16B tag） | PASS |
| Argon2id | argon2-cffi 25.1.0 (libargon2) | **RFC 9106 §5.3 精确 KAT**（含可选 Secret[8]+AD[12]，经 `argon2.low_level.core(ctx, Type.ID)` + ffi 手工填 `argon2_context` 复现） | PASS |
| AES-256-KW | cryptography 50.0.2 | RFC 3394 §4.6（256/256）+ 我们 64B→72B 的 n=8 实况 + 1bit 篡改拒绝 + 错 slot KEK 拒绝 | PASS |
| HKDF-Expand | cryptography 50.0.2 | RFC 5869 A.1（PRK/OKM + 「仅 Expand」变体逐字节一致） | PASS |
| HMAC-SHA256 / SHA-256 | cryptography 50.0.2 + stdlib | RFC 4231 TC1 / FIPS 180-2 | PASS |
| canonical JSON（§3.3 八条） | 本项目序列化器 | golden string | PASS（暴露措辞问题，见下） |

**两个 NOTE（均已回写）**：
1. **技术栈 A 级实测**：libsodium `crypto_pwhash` **硬编码 parallelism=1**，与 libargon2 的 p=1 逐字节一致，但**产不出我们冻结的 p=4 IKM**（p=4 = `b6393f9d…` ≠ p=1 = `ef99bda9…`）→ Rust 内核**必须用 RustCrypto `argon2` crate 显式 parallelism=4**，不可用 libsodium crypto_pwhash 替代；误用 p=1 跨端解锁会确定性 `E_AUTH_FAILED`。已回写 [产品分析与概要设计.md](产品分析与概要设计.md) §7.1 与 [crypto-review-m1.md](crypto-review-m1.md) §12.4。
2. **规格勘误（v1.1.1）**：§3.3 规则⑤「最短转义」与「控制字符一律 `\u00XX`」在 `\n` 上冲突；PoC golden（`line␊quote"`）实测触发。裁定「枚举集合优先」，已收紧为转义集合封闭；[vault-format-spec.md](vault-format-spec.md) §3.3 与 [crypto-review-m1.md](crypto-review-m1.md) §12.2 已同步。

---

## 3. 主链路（[poc/mylock_poc.py](poc/mylock_poc.py)）

固定输入见 [poc/test-vectors.json](poc/test-vectors.json) `fixed_inputs`：passphrase 含非 ASCII + NULL 字节、固定 16B salt、固定 Master/MAC_Key/RK(hex)、固定 `jti`/`key_epoch=0`/`object_id`/`manifest.id`/`device_id`/`base_version=0`、**固定 3×24B nonce**、37B 明文 + **0B 空对象明文**、`manifest_seq=1`、KDF `m=65536,t=3,p=4,tag=32`。**固定 nonce 才逐字节一致**（正常运行用 CSPRNG 随机 nonce，重复加密不会得相同密文——这是安全属性）。

```
[1] create_vault  →  poc/vault_poc/   （2 对象，含 0 字节空对象；空对象仍发 1 个带 tag 空 chunk，F15/N4）
[2] shutil.copytree → poc/usb_copy/   （整盘复制到「移动硬盘」：§6.8 载体无关，零 OS/文件系统特性依赖）
[3] verify phase-1（不输口令） → {"phase":"pre-unlock","result":"OK","objects":2,"bytes":189}
[4] full_open（口令，按 §3.4 冻结顺序）：
    结构检查 + KDF 硬校验 → Argon2id(p=4) → KEK_slot 域分离 → AES-256-KW unwrap
    → vmac_key → HMAC 常量时间比对 versionMac → manifest 解密 → 逐 chunk AEAD 解密 → 明文重建
    → verify phase-2 OK (records=2, orphans=0) → export 到 poc/export_out/（hello.txt=37B, empty.bin=0B）
[4b] 恢复槽端到端：RK 原始 32B 解出同样明文（F13 slot 域分离被真正跑通）
[4c] RK 喂给 password 槽 → E_AUTH_FAILED（域分离生效）
[6] 13 个负向向量
[7] 写出 test-vectors.json
```

运行结果（两次全绿，最新一次含 4b/4c）：

```
RESULT: ALL PASS
[3] verify phase1: {"phase": "pre-unlock", "result": "OK", "objects": 2, "bytes": 189}
[4] unlock + verify phase2 OK  records=2 orphans=0 ; export rebuild OK: hello.txt=37B empty.bin=0B
[4b] recovery-slot end-to-end OK (F13 slot domain separation exercised)
[4c] RK offered to password slot -> E_AUTH_FAILED
[6] negative vectors: 13/13 expected-code matched
[7] wrote test-vectors.json (13499 bytes)
```

复现命令：`& 'C:\Python312\python.exe' 'D:\Projects\dsh\mylock\poc\mylock_poc.py'`（依赖见 §2；PoC 目录内自带 `vault_poc/`、`usb_copy/`、`export_out/` 产物）。

---

## 4. 负向向量：13/13 命中（对应 §6.1 错误码，永不返回未认证明文）

| 扰动 | 期望 | 实测 | 结论 |
|---|---|---|---|
| 改 1 bit meta（versionMac 覆盖字段） | `E_DOWNGRADE` | `E_DOWNGRADE` | OK |
| 降级 `format` | `E_FORMAT`/`E_DOWNGRADE` | `E_FORMAT` | OK |
| 改 `key_epoch` | `E_DOWNGRADE` | `E_DOWNGRADE` | OK |
| 改 1 bit `wrapped.blob` | `E_AUTH_FAILED` | `E_AUTH_FAILED` | OK |
| password↔recovery slot 互换 | `E_AUTH_FAILED` | `E_AUTH_FAILED` | OK（F13 域分离确定性检出） |
| **KDF 越界 `m=4 TiB`** | `E_FORMAT` | `E_FORMAT` | OK（**证明硬校验先于 Argon2id，绝不分配 4TiB**，F14/N3） |
| 改 1 bit 对象 chunk | `E_CORRUPT` | `E_CORRUPT` | OK |
| 删 1 个对象 | `E_MISSING` | `E_MISSING` | OK |
| 截断 chunk | `E_CORRUPT`/`STRUCT_ANOMALY` | `E_CORRUPT` | OK |
| 制造孤儿对象 | `NO_ERROR`（警告非致命） | `NO_ERROR` | OK |
| 空对象 `chunk_count=0` 伪造 | `E_CORRUPT`/`E_FORMAT` | `E_CORRUPT` | OK（F15/N4） |
| 1 bit 翻转 manifest | `E_MANIFEST` | `E_MANIFEST` | OK（内容仍可由 `export --object` 取回，R13） |
| 错误口令 | `E_AUTH_FAILED` | `E_AUTH_FAILED` | OK |

---

## 5. PoC 逼出、已回写规范的 3 类文字问题（v1.1.1，均不改已定案算法）

| # | 问题 | 处置 |
|---|---|---|
| ① | §3.3 规则⑤「最短转义」与控制字符一律 `\u00XX` 在 `\n` 上冲突，会产出不同 versionMac 字节 | 措辞勘误：转义集合封闭（§3.3 + crypto-review §12.2） |
| ② | `device_id` 一名两义：多处写「绝不写入容器/绝不从容器读回」，却又在 manifest 记录里列它、用它做并发比对、§9 要固定向量 | 两义消歧：①设备当前身份只存设备本地、整盘复制不携带、绝不从容器读回；②写入者溯源标签=每条加密记录写入瞬间的本地 id 快照（是加密记录内容、非设备状态）。§1/§2/§5.3/§6.3/§9 + crypto-review §12.3 已同步 |
| ③ | **FINDING-4 字节级拼接未冻结** + **FINDING-5 恢复槽 KDF 输入未逐字定义**：`‖`/`info`/AAD 只写语义，两个正确实现也可能逐字节不一致；RK 是否过 Argon2id、用哪个 salt 没写死 | **新增 [vault-format-spec.md](vault-format-spec.md) §2.1「字节级拼接约定（冻结）」**：u32 big-endian / `jti`(ascii) / 16B 原始 id / 32B 固定 header / chunk 布局 / HKDF info / chunk & manifest AAD / manifest 明文无外层包装 / 两 secret 同参数同 salt 各自 Argon2id；§9 指向 test-vectors.json；crypto-review §12.5 登记。**改动须升 `format` 版本** |

---

## 6. 交付物与「还差什么」（诚实边界）

**本轮新增/更新的文件**：

| 文件 | 作用 |
|---|---|
| [poc/verify_primitives.py](poc/verify_primitives.py) / [poc/verify-primitives.json](poc/verify-primitives.json) | 权威 KAT 逐字节校验各库调用（26/26 PASS） |
| [poc/mylock_poc.py](poc/mylock_poc.py) | 主链路 + 恢复槽 + 13 负向向量的一次性脚手架 |
| [poc/test-vectors.json](poc/test-vectors.json) | **§9 唯一逐字节契约**（13499 B，30+ 派生输出字段 + 13 负向结果） |
| [poc/vault_poc/](poc/vault_poc/) / [poc/usb_copy/](poc/usb_copy/) / [poc/export_out/](poc/export_out/) | 源库 / 移动硬盘副本 / 明文导出（可现场核验） |
| [vault-format-spec.md](vault-format-spec.md) **v1.1.1** | 新增 §2.1 字节级冻结、§9 契约指针、§3.3 勘误、§5.3 device_id 消歧 |
| [crypto-review-m1.md](crypto-review-m1.md) **§12.5** | 登记 FINDING-4/5、KAT 结果、技术栈实测、诚实边界 |
| [产品分析与概要设计.md](产品分析与概要设计.md) **§7.1** | 点名 `argon2` crate 必须 parallelism=4、禁用 libsodium crypto_pwhash |
| 本报告 [poc-report-m1.md](poc-report-m1.md) | 运行方法、结论、回写清单 |

**明确未满足、不得越界宣称的**：

- ❌ **G4「五端 + 恢复 CLI 逐字节一致」尚未验收**：PoC 是**单一 Python 实现**产出的参考向量并跑通主链路。要满足条件，须等 M2 的 Rust 内核、Win/mac/Linux/Android/iOS 各端、独立恢复 CLI 都实现后，逐一对照 `test-vectors.json` 的 `derived_outputs` 复现字节。本轮只交付「契约」与「参考实现的自洽证明」。
- ❌ **不是产品代码 / 不是 M2 开发**：全部 `poc/` 下文件是一次性验证脚手架；`mylock_poc.py` 用 Python 拼装，仅用于验证格式与算法链路，不是可发布内核。
- ❌ **不是第三方审计 / 不是安全认证**：上线前硬门槛 5（第三方密码学审计）仍必须做；`verify` 绿 ≠ 未被恶意替换；内部评审 + 内部 PoC 不等于外部背书。
- ❌ **不覆盖 R1/R2/R9 等残余风险**：三者全失不可恢复、解锁态库外明文持久副本、整盘回放旧状态不可检出——这些是文档已登记的诚实边界，PoC 不试图「解决」它们。

---

## 7. 清掉的硬门槛与下一步建议

对照 [vault-format-spec.md](vault-format-spec.md) §12 上线前 5 条硬门槛：

| 门槛 | 状态 |
|---|---|
| ① §9 正负向向量实测通过、五端 + 恢复 CLI 逐字节一致 | **部分**：正负向向量 13/13 + 主链路在参考实现上通过 ✅；五端 + 恢复 CLI 逐字节一致 ⏳ 待 M2 实现后比对 |
| ② threat-model.md §6 K1–K10 回填 | ✅ 已在 v1.1 回填（早于本 PoC） |
| ③ 主文档同步 v1.2 | ✅ 已同步（本轮再补 §7.1 选型实测） |
| ④ §11 裁定全部有记录 | ✅ 已记录（本轮补 §12.5 FINDING-4/5） |
| ⑤ 上线前第三方密码学审计 | ⏳ 未开始，仍必须做 |

**下一步建议（均需用户另行拍板，不自动进入）**：
1. **（倾向）进入 M2 内核实现**：以 `test-vectors.json` 为验收契约，先做 Rust 加密 crate + 独立恢复 CLI，跑通「Rust 端逐字节复现 PoC 向量」，这是清掉门槛①最短的路径。
2. **先补跨平台 KAT 校验**：把 `verify_primitives.py` 的同一组 KAT 在目标端语言/库上各跑一遍，提前暴露库差异（如各端 XChaCha20 实现是否一致）。
3. **把 PoC 归档为 `tests/vectors/` 并加 CI 门禁**：任何格式改动若使 `derived_outputs` 变化，CI 直接失败——把「冻结」变成机器强制。

> 在用户明确选择之前，**停留在当前状态**：概念确立 + M1 评审 + M1 PoC 全部完成，不写产品代码、不启动 M2。
