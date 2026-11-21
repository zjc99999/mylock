# M1 密码学评审报告 v1.1（对抗性审阅裁定 · 定案）

> 状态：**评审完成 · 逐项定案（v1.1 为最终版）**。本文裁定 [vault-format-spec.md](vault-format-spec.md) v0.1 的 P1–P10、12 项结构性议题，并对一份独立对抗性审阅的 16+ 项发现逐项裁定「接受 / 部分接受 / 拒绝（附理由）」。
> 性质：**设计评审产物**（A 级标准论据 + C 级工程定案），**不是实现、不是安全认证、不是第三方审计**。规范升 v1.1 仅表示「字段与算法组合已冻结、可被独立审计」；上线前仍必须委托第三方密码学审计。
> 日期：2026-11-14　归属：[产品分析与概要设计.md](产品分析与概要设计.md) §6.3 / §6.6 / §6.8。配套：[threat-model.md](threat-model.md)、[vault-format-spec.md](vault-format-spec.md)。
> 纪律：评审定案前不得进入实现；本文所有「已定案」指**设计决策**，不得对外宣称「已支持 / 已实现 / 已通过审计」。用户 m00241 明确选择「先做 M1 密码学评审」，**不写产品代码、不进入开发阶段**。
> **v1.1 变更**：①登记三路独立复核全部回传（§10）；②新增 A10–A12 一手论据（NIST SP 800-38F / RFC 3394 定位 / RFC 9106 措辞）；③新增 §11 对抗性审阅逐项裁定；④据裁定新增 F13–F20 与 N1–N9 修正，刷新 §0/§3/§4/§5/§6/§7/§8/§9；⑤[vault-format-spec.md](vault-format-spec.md) 同步升 v1.1。

---

## 0. 结论摘要

v0.1 草案的密码学骨架（Argon2id → 包裹主密钥 → HKDF 子密钥 → 分块 AEAD → 密文文件名）方向正确。评审分两轮：

**第一轮（父代理 + 标准复核）**：发现 **6 处结构性阻断问题 D1–D6**，已在 v1.0 修正。

| # | v1.0 定案 | 触发的问题 |
|---|---|---|
| D1 | `versionMac` 移到**解锁后**校验；解锁前只做非密码学结构检查 | 「无需口令即可验 versionMac」自相矛盾（MAC 密钥须先解包） |
| D2 | manifest AAD **不绑 versionMac**，改绑稳定标识；删 `manifest_ref.mac` | 换口令重算 versionMac 会连带让 manifest AEAD 失效（换口令即坏库） |
| D3 | **删除名称 SIV**；名称 = AEAD 加密 manifest 内普通字段 | RFC 5297 证明 SIV 是 AES 模式，不存在「XChaCha20-SIV」；manifest 本就整体加密 |
| D4 | **删除 BLAKE3 checksum**；完整性只由每 chunk Poly1305 tag 承担 | checksum 在加密 manifest 内→解锁前读不到；与 AEAD tag 冗余；BLAKE3 一手规范未取到 |
| D5 | PHC **不落盘派生结果**，只存参数 + salt | 把 KEK/口令哈希落盘 = 免费离线爆破验证器 |
| D6 | 单一 cipher = **XChaCha20-Poly1305**，取消运行时 AES-GCM 协商 | 双 cipher 协商面 = 降级/歧义面 |

**第二轮（独立对抗性审阅 dca3264f + 标准复核 c8835859）**：对抗审阅针对的是 **v0.1**，因此其多数【高】级发现（versionMac 循环 / AAD 绑 versionMac / 双 cipher / SIV / 零明文口径 / verify oracle）**已在 v1.0 解决**，本轮裁定记为「已在 v1.0 解决」。但它另外发现 **5 项 v1.0 确实未覆盖的真问题**，构成本 v1.1 的实质修订：

| # | 新发现（v1.0 未覆盖） | v1.1 处置 |
|---|---|---|
| **N1** | RFC 3394 AES-KW **无 AAD**，且 64-bit 完整性在离线反复改盘下会累积（NIST SP 800-38F §B.2 / §A.3 亲证） | **部分接受**：保留标准件 AES-256-KW（NIST 批准、有测试向量），补齐 `key_epoch` + slot 域分离 + versionMac 全头绑定；**拒绝**改用自造 XChaCha wrap |
| **N2** | 「换口令不重加密」= **零吊销**：持有旧 `vault.meta` + 旧口令者仍能解开同一主密钥读当前全部数据 | **接受（最重要）**：`rotate-keys` 升为规范内一等命令（新 Master/MAC/salt/jti/key_epoch + 全量重加密），加 `key_epoch`，写进对外声明第 5 条 |
| **N3** | KDF 参数盘上可控 → 改 `m=4 TiB` 即 OOM DoS | **接受**：解锁前硬校验 `m/t/p` 上下界，越界 `E_FORMAT`，绝不先跑 Argon2id |
| **N4** | `chunk_count=0`（空文件）零 tag，明文 header 可伪造 | **接受**：强制 `chunk_count ≥ 1`，空明文也发一个带 AAD 的空 chunk |
| **N5** | 解锁前 verify 被当 oracle（早停/错误码/对象枚举） | **接受**：全量扫描不早停、定长输出、预认证失败统一 `E_AUTH_FAILED`、遍历全部 slot 后统一失败 |
| **N6** | `device_id` 存库内被整盘复制 → 两份自认同一写者 → 静默丢更新 | **部分接受**：`device_id` 改为每安装本地生成、**绝不从容器读取**；记录加 `base_version` 判并发；**拒绝**全量版本向量/OR-Set（MVP 保留 N=2） |
| **N7** | `zeroize` 只提名字、Rust `String` 不可清零 | **接受**：具体化 `Zeroizing`/`SecretVec` + `mlock` + 禁核心转储 + 带上限 LRU |
| **N8** | `created` 无用且泄漏创建时间；「延时减速」对离线爆破不适用 | **接受**：删 `created`；口径改「Argon2id 仅抬高单次成本，口令熵为唯一上限」 |
| **N9** | 挂载期明文在**库外**留持久副本的口径过弱 | **接受**：降级为「仅未解锁态防得住 / 解锁态仅缓解」，补 VSS/Time Machine/AV-EDR，拒绝在同步根内挂载 |

**总判定：通过（附条件）** —— 按本文定案把规范升到 **v1.1** 后，格式层冻结、可进入后续 M2 实现排期；但在任何对外发布前必须完成 §9 的第三方审计与负向向量实测，本文不构成安全背书。

---

## 1. 评审所依据的一手论据（A 级）

| # | 来源 | 关键原文 / 事实 | 用在 |
|---|---|---|---|
| A1 | **RFC 9106** https://www.rfc-editor.org/rfc/rfc9106.txt | "Argon2id MUST be supported"；§4 FIRST `t=1,p=4,m=2^21 (2 GiB)`、SECOND（内存受限）**`t=3,p=4,m=2^16 (64 MiB)`, 128-bit salt, 256-bit tag**；§7.3 "1 pass maximizes the attack costs for the constant defender time"；§9 "enable the memory-wiping option"；v=0x13；m∈[8p,2^32-1] KiB | P3 / F11 |
| A2 | **RFC 3394** https://www.rfc-editor.org/rfc/rfc3394.txt | 输入 64-bit 块整数倍且 `n≥2`；默认 IV `A6A6A6A6A6A6A6A6`；输出 `(n+1)` 块；"chance that the key data is corrupt is **2^-64**"；失败 "MUST return an error, and it MUST NOT return any key data"；KEK 128/192/256-bit | F3 / P1 |
| A3 | **RFC 5297** https://www.rfc-editor.org/rfc/rfc5297.txt | SIV 是 **AES 模式**（S2V=AES-CMAC, CTR=AES-CTR）；"SIV can be used as a drop-in replacement for any specification that uses [RFC3394]"；§6 同密钥调用上限 2^48 | D3 / P9 / N1 |
| A4 | **draft-irtf-cfrg-xchacha-03** https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-03 | 页面自标 **"Expired Internet-Draft ... not endorsed by the IETF and has no formal standing"**；**192-bit(24B) nonce**；"random 192-bit nonces ... single collision (with probability 50%) after roughly 2^96 messages"；2^-32 下允许 2^80 条/单密钥；WireGuard/libsodium/Tink/Go/LibreSSL 实现；附 A.3.1 测试向量 | F9 / §8 |
| A5 | **RFC 5869** https://www.rfc-editor.org/rfc/rfc5869.txt | §3.2 `info` "bind the derived key material to application- and context-specific information"；§3.3 均匀强密钥 "can skip the extract part"；直接用 PRK 当 OKM "is NOT RECOMMENDED"；§4 "cannot amplify entropy" | F1 / F13 / P2 |
| A6 | **OWASP Password Storage Cheat Sheet** https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html | Argon2id 最低 `m=19456 KiB, t=2, p=1`；work factor 随硬件升级；PHC 存算法与参数；完整 Unicode / NULL 字节 | P3 / N3 |
| A7 | **RFC 8439** https://www.rfc-editor.org/rfc/rfc8439.txt | ChaCha20-Poly1305 = **96-bit nonce + 32-bit counter**；**全文检索 XChaCha/HChaCha/192-bit 命中 0 次**；"16-byte tag"/"128-bit tag"；nonce 复用 "reveals the XOR of the plaintexts"；原构造 "MUST NOT be randomly generated" | F9 / F12d |
| A8 | **RFC 8949** https://www.rfc-editor.org/rfc/rfc8949.txt | §4.2 "Those protocols are free to define what they mean by a 'deterministic format'"；§1.1 "no requirement that all data formats be uniquely encoded"；§4.2.1 map key 字节字典序 + 最短编码 + 禁 indefinite-length | F12c-1 / P8 |
| A9 | OWASP（子代理独立复核，与 A6 一致） | "minimum configuration of 19 MiB of memory, an iteration count of 2, and 1 degree of parallelism"；NULL 字节；PHC 存参数 | P3 |
| **A10** | **NIST SP 800-38F** https://csrc.nist.gov/pubs/sp/800/38/f/final （web_fetch 200）+ PDF https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-38F.pdf （web_fetch 406，子代理用浏览器 Node `https` 取得 200 / application/pdf / 289262 bytes，解压 FlateDecode 流取正文 → **PDF 正文按 A⁻ 计**，CSRC 落地页按 A 计） | 摘要 "cryptographic methods that are **approved for 'key wrapping,' i.e., the protection of the confidentiality and integrity of cryptographic keys**"；§1 "approved methods for key wrapping"；"KW, KWP, and TKW were designed to protect the confidentiality and the authenticity/integrity of cryptographic keys"；§4.3 "ICV1 The 64-bit default ICV for KW: 0xA6A6A6A6A6A6A6A6"；**§A.3** "strong, but not absolute, assurance"、随机串被接受概率 "exactly 1 in 2^64 for KW"、"**Given repeated attempts, of course, the adversary can increase the probability**"、"should monitor and, if necessary, limit the number of unsuccessful verification attempts for each key"；**§B.2** "a digital signature or a MAC is **not necessary** to authenticate the data that is protected by a key-wrap algorithm"，但同节列出三条差异，其中 (2) "Many authenticated-encryption algorithms ... provide an efficient means of **authenticating associated data that is not confidential**"、(3) "**for KW, KWP, and TKW, there are no separate authentication tags ... the authenticity of the data cannot be verified without invoking the authenticated-decryption function**"；§A.2 "The cryptographic strength of the protected key is limited implicitly by the resistance of the KEK to brute force search"；§A.4 Kelsey 长消息伪造观察 | **N1 裁定的决定性论据** / F3 / §4 |
| **A11** | RFC 3394 定位（子代理核实） | "Category: **Informational**"；"does not specify an Internet standard of any kind"；全文检索 `800-38F`/`SP 800` **零匹配**（**已核实的否定结论**：RFC 3394 不引用 SP 800-38F）；源自 2001 NIST AES-WRAP 草案 → **「AES-KW 被批准」这一地位来自 SP 800-38F，不是 RFC 3394 本身** | §8 / N1 |
| **A12** | RFC 9106 措辞（子代理核实） | 全文**无 "GPU" 字面词**，只有 "maximizes adversarial costs on dedicated brute-force hardware" 与 ASIC 电路深度表述；「抗侧信道」有逐字原文 → **不得把「抗 GPU」写成 RFC 9106 引文** | 措辞纪律 |

**未取到一手、明确不得当 A 级**：BLAKE3 规范（GitHub 页面外壳，正文截断）、libsodium 文档（404 / 跨域）、NIST SP 800-38D PDF（406）、Windows Recall 链接（404）、standardnotes.com（fetch failed）。BLAKE3 因此**不进入 v1.1**（D4）。

---

## 2. 结构性修正（F1–F20 定案）

### A 类 · 结构性自相矛盾（v1.0）

**F1　versionMac 循环依赖（阻断）**：v0.1 把「验 versionMac」排在「Argon2id 派生 KEK」之前，而 versionMac 密钥须由解包得到的 MAC_Key 派生 → 顺序不可能成立。**定案**：解锁顺序冻结为 ①解析 meta 公开头 → ②**解锁前硬校验 KDF 参数上下界（F14）** → ③Argon2id 得 KEK → ④AES-KW 解包得 `Master∥MAC_Key`（2^-64 首道过滤，失败 `E_TAMPER`）→ ⑤`HKDF-Expand(MAC_Key,"meta-version-mac",32)` → ⑥常量时间比对 versionMac → ⑦不符 `E_DOWNGRADE` 绝不继续 → ⑧派生 manifest 密钥解密清单。**versionMac 属解锁后检查**。

**F2　manifest AAD 绑定 versionMac 导致换口令坏库（阻断）**：**定案**：①manifest AAD 改绑**稳定标识** `magic‖format‖jti‖manifest.id‖manifest_seq`；②删 `manifest_ref.mac`，收敛为 `{ id, seq }`；③manifest 明文内含 `manifest_seq`，解锁时断言 `manifest.seq == meta.manifest_ref.seq`（前者由 AEAD tag 认证、后者由 versionMac 认证）→ **双向绑定**；④`manifest.id` 变更必须显式迁移。**诚实残留**：整盘回放旧 `(meta,manifest,d/)` 三元组的状态回滚盘内不可根除 → R9。

**F12a　PHC `<hash-b64>` 落盘 = 免费离线验证器（阻断）**：`kdf` 只存 `{ alg, m, t, p, salt }`，**不存任何派生结果**；口令正确性由 AES-KW 解包完整性 + versionMac 共同判定。

### B 类 · 密码学细节（v1.0）

**F3　AES-KW 仅 64-bit 完整性**：A2 + **A10 §A.3**（"Given repeated attempts ... the adversary can increase the probability"）证明离线反复改盘会侵蚀 2^-64。**定案**：保留 RFC 3394（NIST 批准的标准件，§B.2 "a MAC is not necessary"），其 2^-64 作**首道过滤**；由 **versionMac（HMAC-SHA256，覆盖 canonical(meta 全字段)）作 256-bit 强绑定**，恰好补上 §B.2 列出的「KW 无法认证非机密关联数据」缺口 → 即标准结论所建议的「外层另加 MAC」。不循环（versionMac 用解包得到的 MAC_Key；解包正确性由 AES-KW 2^-64 先筛）。

**F4　BLAKE3 checksum 冗余且前提错误**：删除。完整性由每 chunk AEAD tag（解锁后）+ 明文对象 header 长度字段（解锁前结构检查）承担。

**F9　随机 nonce + 每 chunk 独立密钥是否多余**：A4 证明 192-bit nonce 可安全随机生成。**定案（单一正确做法）**：**一个对象一个密钥** `HKDF-Expand(Master,"object"‖object_id,32)` + **每 chunk 一个 CSPRNG 24B 随机 nonce**；不派生 per-chunk 密钥、不对 chunk 用 SIV、不做 nonce 计数器。AAD 绑定 `magic‖format‖jti‖object_id‖chunk_index‖chunk_count‖header_hash`。

### C 类 · 元数据与侧信道（v1.0）

**F8　「全目录零明文」口径过宽**：统一收窄为「**用户数据零明文**」——磁盘上不出现任何用户内容/文件名/目录名/目录层级/标签；`vault.meta` 字段名与 README 为公开自描述元数据。形状泄漏口径与 [threat-model.md](threat-model.md) §5 一致：**「仍可能推断对象数量与大致大小，不承诺完全消除」**。禁止写「零元数据」「连形状都不泄漏」。

**F11　时序 / 侧信道 / 挂载期明文第二份**：Argon2id memory-wiping（A1 §9）；versionMac 常量时间比对；错误码按 F7 收敛；`threat-model.md` §3.1 已列挂载期第二份明文（**v1.1 按 N9 补 VSS / Time Machine / AV-EDR 并降级口径**），不承诺 100% 阻止；绝对隔离改用单文件解密导出。登记为 R2。

### D 类 · 可实现性与收敛策略（v1.0）

**F6　冲突副本导致 manifest 无限膨胀**：`device_id` = 每安装随机 id（**v1.1 按 N6 改为设备本地生成、绝不入库/绝不由容器读取**）；`updated_seq` = 每对象 Lamport 时钟；**每对象冲突副本上限 N=2**；解锁态强制引导处理；`vault gc --confirm` 回收孤儿与已解决副本。**绝不静默覆盖、绝不无限保留、禁 LWW**。

**F7　verify 被当 oracle**：两阶段模型。**解锁前**只输出收敛结果 `OK(结构) | STRUCT_ANOMALY` + 对象数/总字节（列目录本就可得）；**解锁后**才输出完整细粒度错误码。**v1.1 按 N5 加严**：全量扫描不早停、定长输出、预认证失败统一 `E_AUTH_FAILED` + 常量时间 + 遍历全部 slot 后统一失败；`export --object` 预认证阶段不区分「id 不存在」与「口令错」。

**F10　换口令与 Recovery Key 轮换语义**：两 slot 包裹**同一 Master/MAC_Key**。`rotate-password` = 重包裹 password slot + 重算 versionMac，Recovery Key 保持有效。**v1.1 按 N2 重写**：见 F17。

**F12b　vault.meta 单点写入损坏**：临时文件 + fsync + 原子 rename + 受 versionMac 保护的 `vault.meta.bak`；`verify` 检测半写入。

**F12c　Recovery Key 编码（P10）与序列化（P8）**：RK = 256-bit CSPRNG → base32(52) + 4 校验位，13×4 分组；**不引入 BIP39**；**v1.1 按 N2 在编码内加入 `key_epoch`**。序列化全链路 **canonical JSON**，不采用 CBOR（A8：确定性格式须应用协议自定，CBOR 无优势且增解析面）。校验位仅拦截抄写错误，不作密码学认证。

**F12c-1 canonical JSON profile（冻结 8 条，跨实现硬依赖）**：① UTF-8 无 BOM；② 键按 **Unicode 码点升序**递归排列；③ 无冗余空白；④ 整数十进制、不用浮点/指数、字节串 base64；⑤ 字符串最短转义（仅 `\"` `\\` 与控制字符 `\u00XX` 小写 hex，不转义非 ASCII）；⑥ 禁重复键（解析即拒）；⑦ 布尔/null/数组字面量固定小写；⑧ 字段集固定、禁未知字段（缺字段 `E_FORMAT`）。`versionMac = HMAC(..., canonical(meta))` 跨实现差 1 字节即失败，故必须冻结整份 profile，不得只写「键排序」。

**F12d　XChaCha20 非 IETF 标准 + 算法标识**：A4/A7：expired draft、无正式地位、广泛实现；RFC 8439 全文未出现 XChaCha/HChaCha/192-bit，它只定义 96-bit nonce 且明写 "MUST NOT be randomly generated"。**定案**：①规范与对外说明写明「基于 expired IRTF draft + 广泛业界实现，**不是 IETF 标准、不是 RFC 8439 的 ChaCha20-Poly1305**」；②`cipher` 只能写 `XCHACHA20-POLY1305`，**不得**简写为 `CHACHA20-POLY1305`；③随机 24B nonce 的合法性**仅由 XChaCha draft 的 192-bit 性质支撑**，不得拿 RFC 8439 背书。G4「逐字节相同」**只适用于固定输入的测试向量**。

### E 类 · v1.1 新增修正（对抗性审阅裁定）

**F13　slot 域分离（N1，部分接受）**：`KEK_slot = HKDF-Expand(Argon2id(secret, salt, m/t/p, tag=32B), "wrap/" ‖ slot, 32)`，`slot ∈ {password, recovery}`。理由：A5 §3.3 允许对均匀强 IKM 跳 Extract 直接用 Expand；显式 `info` 绑定 slot，使 password↔recovery blob 互换在解包时**确定性地**被 AES-KW 2^-64 完整性拒绝（两者 KEK 必不同），不依赖「用户输入恰好不同」这一隐含假设。同时 `jti / format / kdf / slot / key_epoch` 全部纳入 versionMac 覆盖集，补上 A10 §B.2(2) 指出的「KW 不能认证关联数据」缺口。**拒绝**改用 XChaCha20-Poly1305 自造 wrap（见 §11 裁定 N1）。

**F14　KDF 参数解锁前硬校验（N3，接受）**：读 meta 后、跑 Argon2id **之前**校验 `19456 ≤ m ≤ 1048576 KiB`、`2 ≤ t ≤ 10`、`1 ≤ p ≤ 16`；越界即 `E_FORMAT`，绝不分配内存。下界取 A6/A9 OWASP 最低，上界防 `m=4 TiB` OOM DoS。移动端达不到下限时**拒绝在该设备建库并明确提示**，不做静默降档。

**F15　空对象强制一个 chunk（N4，接受）**：`chunk_count ≥ 1` 为格式不变量；0 字节明文也写一个 `plaintext_len=0` 的 chunk（仍带 24B nonce + 16B tag + 含 `header_hash` 的 AAD）→ 明文 header 不再是「零 tag 可任意伪造」。

**F16　verify 预认证统一失败（N5，接受）**：见 F7 加严。`vault info` 解锁前不打印对象 id 集合；`verify` 阶段 1 全量扫描、定长输出、不早停。细粒度错误码**仅**在 `--password` / `--recovery` 成功解包后输出（§5）。

**F17　零吊销与真吊销分离（N2，接受 —— 全文最重要的修正）**：
- `change-password`（原 `rotate-password`）：仅重包裹 password slot + 重算 versionMac，**零吊销**。UI 明说「换口令不会使旧 Recovery Key 失效，**也不会撤销任何持有旧 `vault.meta` 副本的人**」。
- `rotate-recovery`：新 Recovery Key 重包裹 recovery slot + 重算 versionMac；**仅**使旧 Recovery Key 失效，**对持有旧 `vault.meta`+旧口令者仍零吊销**。
- **`rotate-keys`（v1.1 升为规范内一等命令）** = 真吊销：生成**全新 `Master∥MAC_Key` + 新 salt + 新 `jti` + `key_epoch++` + 新 Recovery Key** → 全量重加密所有对象与 manifest → `verify` 双绿 → 用户确认后 GC 旧对象/旧 bak → **`key_epoch` 写入 RK 编码**（旧 epoch 的 RK 报「已轮换」）→ **必须物理销毁所有旧介质副本**。因为 chunk AAD 与对象密钥都绑定 `jti`/`Master`，旧 `vault.meta` 解出的旧主密钥无法解开任何新数据。
- `fork`（v0.2+）：把整库克隆到第二介质时**轮换 `jti`**（等价于对副本执行一次 `rotate-keys`），消除 jti 跨介质可关联；**代价同样是全量重加密**，故与 rotate-keys 同属显式重操作，不做静默克隆。
- `key_epoch`：u32，初值 0，**仅 `rotate-keys` 递增**，纳入 versionMac 覆盖集与 RK 编码。
> 诚实声明：在「包裹主密钥」架构下，数据加密密钥不变则**任何**凭据轮换都不构成吊销；这是架构事实，不是实现缺陷，必须写进对外边界（§8 第 5 条）。

**F18　并发模型收敛（N6，部分接受）**：①`device_id` = 每安装 16B CSPRNG，**存设备本地状态、绝不写入容器、绝不从容器读取**（整盘复制后两台设备各自用本地 id，不再自认同一写者）；②manifest 记录新增 `base_version`（本次写入所基于的 `manifest_seq`），并发判定 = 「`base_version` 相同且 `(device_id, updated_seq)` 不同」，无因果关系的两份**双留**（绝不 LWW）；③可选 `content_hash = SHA-256(明文)`（**用已标准化的 SHA-256，不用 BLAKE3**）做同内容自动去重，v0.2+；④保留 N=2 冲突上限 + `vault gc --confirm` + 水位触发（孤儿 > 活跃 20% 或每 N 次写）；⑤`vault.meta.gc_pending` 作崩溃恢复标记（不新增隐藏文件）；⑥**库外本地锚点**（last-seen `head/seq`）作为**可选**、显式开启的回滚检测增强，锚点丢失即失效、不写入容器 → 不影响 §6.8 载体无关。**拒绝**全量版本向量 / OR-Set / tombstone / `resolved_from`（对 MVP 个人库过度工程，N=2 + 显式解决已够）。

**F19　zeroize 与内存具体化（N7，接受）**：口令入口即 `SecretVec`/`Zeroizing`（**不用 Rust `String`**）；覆盖 KEK / Master / MAC_Key / manifest 明文 / 每 chunk 缓冲（随机读缓冲带**硬上限 LRU**，逐出即清零）；`mlock` 锁定密钥页、按平台禁核心转储；**声明** swap / 休眠 / 冷启动 / 剪贴板历史与云剪贴板 / 他进程已读内容不可撤销、不受控。

**F20　元数据与对外口径收窄（N8 / N9，接受）**：①**删除 `created` 字段**（无用、泄漏创建时间、v1.0 §3.2 曾为它写过别扭的完整性说明）；②§5 表格补「`kdf` + `wrapped` 即完整离线爆破输入」「存在 Recovery slot 是公开事实」「`jti` 跨介质可关联」；③「延时减速防爆破」改为「**Argon2id 仅抬高单次成本，口令熵为唯一上限**；延时减速仅对在线 UI 有效，离线有整盘故速率限制不适用」；④挂载期明文外溢口径降级为「**仅未解锁态防得住 / 解锁态仅缓解**」，补 VSS / Time Machine 快照 / AV-EDR / AI 扫描器，并把「同步目录内挂载」由「强提示建议改址」升级为「**拒绝挂载**」；⑤`README.txt` / `MYVAULT` / `d/xx/` 作为产品指纹写进可见元数据白名单。

---

## 3. P1–P10 逐项定案表（v1.1）

| # | 待定项 | **v1.1 定案** | 依据 |
|---|---|---|---|
| P1 | 主密钥包裹机制 | **RFC 3394 AES-256-KW**（唯一，NIST SP 800-38F 批准）+ 256-bit versionMac 兜底 + `key_epoch` + slot 域分离（F13）；AES **仅**用于包裹 | A2/A10/A11/F3/F13 |
| P2 | 两个 slot 是否同一主密钥 | **同一 Master∥MAC_Key 的两个独立、域分离包裹**（`info="wrap/password"` / `"wrap/recovery"`） | A5/F13 |
| P3 | Argon2id 参数与低配降档 | **m=65536 KiB, t=3, p=4, 128-bit salt, 256-bit tag**（RFC 9106 SECOND RECOMMENDED）；移动端保底 m≥19456,t=2,p=1；**解锁前硬校验上下界**（F14）；不做静默降档 | A1/A6/A9/F14 |
| P4 | chunk 大小 | **1 MiB**；`chunk_count ≥ 1`（F15） | F15 |
| P5 | chunk nonce 机制 | **每 chunk CSPRNG 24B 随机 nonce + 单对象密钥**；取消 SIV / per-chunk 密钥 / 确定性 nonce | A4/F9 |
| P6 | 大小/数量填充 | **MVP 不做**，对外声明形状泄漏未 100% 消除 | F8 |
| P7 | age 互操作桥 | **非 M1**，v0.2+ | 范围控制 |
| P8 | manifest 序列化 | **canonical JSON + F12c-1 冻结 8 条规则**，不采用 CBOR；G4 仅约束固定测试向量 | A8/F12c-1 |
| P9 | 名称加密 / parent_id 绑定 | **删除名称 SIV**；名称 = AEAD 加密 manifest 内普通字段；`parent_id` 用稳定随机 `object_id`，重命名/移动零重加密 | A3/D3/F5 |
| P10 | Recovery Key 编码 | **base32(52) + 4 校验位 + `key_epoch` 前缀**，13×4 分组；不引入 BIP39；`rotate-recovery` 使旧 RK 失效，`rotate-keys` 轮换 epoch | F12c/F17 |

> v1.1 补充定案（不新增编号，归入 F13–F20）：`key_epoch` (F17)、`base_version` / 设备本地 `device_id` (F18)、`gc_pending` (F18)、`content_hash` 用 SHA-256 且 v0.2+ (F18)、删 `created` (F20)。

---

## 4. v1.1 最终参数总表（实现冻结值）

| 层 | 取值 | 依据等级 |
|---|---|---|
| 口令 → KEK | `Argon2id(m=65536 KiB, t=3, p=4, salt=128-bit, tag=256-bit, memory-wiping)` → `HKDF-Expand(out, "wrap/"‖slot, 32)` = `KEK_slot`；移动端保底 `m≥19456,t=2,p=1` | A1/A5/A6/F13 |
| **KDF 参数硬约束** | 解锁前校验 `19456 ≤ m ≤ 1048576 KiB`、`2 ≤ t ≤ 10`、`1 ≤ p ≤ 16`；越界 `E_FORMAT`，绝不先跑 Argon2 | A6/F14 |
| 口令编码 | 完整 Unicode，保留 NULL 字节，不 trim；显式 NFC；**不落盘派生结果** | A6/F12a |
| 密钥包裹 | `AES-256-KW` (RFC 3394)，输入 64 B = 8 块，ICV `A6A6A6A6A6A6A6A6`，输出 72 B；解包失败 MUST 报错不返回密钥；**NIST SP 800-38F 批准**，局限（无 AAD / 2^-64 / 离线累积）由 versionMac 补偿 | A2/A10/A11 |
| 主密钥 | `Master(32 B) ∥ MAC_Key(32 B)`，CSPRNG，永不落盘明文 | C |
| **key_epoch** | u32，初值 0，**仅 `rotate-keys` 递增**；纳入 versionMac 覆盖集与 RK 编码 | F17 |
| 防降级/全头标签 | `versionMac = HMAC-SHA256(HKDF-Expand(MAC_Key,"meta-version-mac",32), canonical(meta 全字段，排除自身))`；覆盖 `magic/format/jti/cipher/kdf/wrapped/manifest_ref/key_epoch`；**解锁后**常量时间比对 | A5/F1/F13 |
| 子密钥派生 | **仅 HKDF-Expand**（Master 均匀，跳 Extract），`info = context ‖ id`；**禁止 PRK 直接当 OKM** | A5 |
| HKDF contexts | `"meta-version-mac"` / `"manifest"‖manifest.id` / `"object"‖object_id` / `"wrap/"‖slot`；**无 `"name"`** | F13 |
| 内容 / manifest AEAD | **XChaCha20-Poly1305**，192-bit(24 B) 随机 nonce / chunk，128-bit(16 B) tag | A4 |
| 单一 cipher | `XCHACHA20-POLY1305` 唯一值；**无运行时协商**；FIPS 走 `format=2` | A4/D6 |
| chunk AAD | `magic ‖ format ‖ jti ‖ object_id ‖ chunk_index ‖ chunk_count ‖ header_hash` | F9 |
| manifest AAD | `magic ‖ format ‖ jti ‖ manifest.id ‖ manifest_seq`（稳定，**不绑 versionMac**） | F2 |
| 对象 id | 128-bit CSPRNG → base32（RFC 4648，大写无填充，26 字符），前 2 字符分片；**每次写入全新、永不复用**（copy-on-write 不变量） | F18 |
| 空对象 | `chunk_count ≥ 1`；0 字节明文也发一个带 tag 的空 chunk | F15 |
| 分块 | 1 MiB / chunk | P4 |
| manifest 记录字段 | `id, parent_id, name, kind, size, chunk_count, base_version, device_id, updated_seq, content_version, manifest_seq`（+v0.2 可选 `content_hash=SHA-256(明文)`） | F18 |
| **`device_id`** | 每安装 16B CSPRNG，**存设备本地状态、绝不写入容器、绝不从容器读取** | F18 |
| 完整性 | 每 chunk Poly1305 tag（解锁后）+ 明文 header 长度字段（解锁前结构检查）；**无 BLAKE3、无 checksum、无明文 catalog** | D4/F4 |
| 序列化 | canonical JSON，**必须实现 F12c-1 冻结 8 条规则** | F12c-1/A8 |
| 名称 | AEAD 加密 manifest 内普通字段；**无独立名称加密、无 SIV** | D3 |
| `vault.meta` 写入 | 临时文件 + fsync + 原子 rename + `vault.meta.bak`（各自带 versionMac）+ `gc_pending` 崩溃标记 | F12b/F18 |
| **`created`** | **删除该字段** | F20 |
| Recovery Key | 256-bit → base32(52) + 4 校验位 + `key_epoch` 前缀，13×4 分组 | F12c/F17 |
| 冲突 | 每对象 N=2 + `base_version` 并发判定 + 双留 + 禁 LWW + `vault gc --confirm` 水位回收 | F6/F18 |
| 内存 | `SecretVec`/`Zeroizing` 覆盖口令/KEK/Master/MAC/manifest 明文/chunk 缓冲；`mlock` + 禁核心转储 + 带上限 LRU 逐出清零 | F19 |
| verify 阶段 1 | 全量扫描不早停、定长输出、预认证统一 `E_AUTH_FAILED`、遍历全 slot、常量时间 | F16 |

---

## 5. `verify` 两阶段模型与错误码收敛（定稿接口语义）

**阶段 1 · 解锁前（无口令，`vault verify <root>`）** —— 只做非密码学结构检查：
- 解析 `magic/format/kdf/cipher` 公开头（**含 F14 参数上下界硬校验**）；遍历 `d/` 命名与 2 层分片；用每对象**明文 header** 的 `chunk_count / ciphertext_len` 对比实际文件长度判截断；统计对象数与总字节。
- **全量扫描、不早停、定长输出**；输出收敛为 `{ "phase":"pre-unlock", "result":"OK" | "STRUCT_ANOMALY", "objects":N, "bytes":M }`（N/M 列目录本就可得，不构成新泄漏）。
- **禁止**区分/报告具体对象错误、禁止 id 枚举；`export --object` 在预认证阶段不区分「id 不存在」与「口令错」，统一 `E_AUTH_FAILED`。**任何预认证失败统一 `E_AUTH_FAILED` + 常量时间比较 + 遍历全部 slot 后统一失败**（F16）。

**阶段 2 · 解锁后（持口令/Recovery Key，`vault verify <root> --password`）** —— 完整密码学验证，细粒度错误码：

| 码 | 含义 | 检查 |
|---|---|---|
| `E_FORMAT` | 非本格式 / 版本未知 / KDF 参数越界 / canonical JSON profile 违规 | public 头解析 + F14 + F12c-1 |
| `E_AUTH_FAILED` | 所有 slot 解包失败（统一，不区分哪个 slot、不区分口令错与 blob 篡改） | F16 |
| `E_TAMPER` | 解包/标签异常（授权态下的细分） | RFC 3394 unwrap |
| `E_DOWNGRADE` | versionMac 不符 / `format` 被降级 | HMAC-SHA256 常量时间比对 |
| `E_MANIFEST` | manifest AEAD tag 失效或 `manifest.seq != manifest_ref.seq` | XChaCha20-Poly1305 |
| `E_MISSING` | manifest 引用的对象不存在（只拷子集 / 拷一半断电） | 清单比对 |
| `E_CORRUPT` | 任一 chunk 的 AEAD tag / AAD 不符（含重排、跨对象拼接、截断、`chunk_count=0`） | 逐 chunk |
| `E_ORPHAN` | `d/` 中存在未被清单引用的对象（**警告，非致命**） | 目录比对 |
| `E_CONFLICT` | 同 `base_version` 下两份 `(device_id, updated_seq)`（需解锁态人工决定） | 清单比对 |

> 负向向量：改 1 bit meta / 改 1 bit 对象 / 删 1 对象 / 降级 format / 截断 chunk / 制造孤儿 / 空对象伪造 / KDF 参数越界 → 分别稳定命中上述码，且**永不返回未认证明文**。

---

## 6. 测试向量与恢复 CLI 接口（定稿）

**固定输入向量（M1 必须发布、五端 + 恢复 CLI 逐字节一致）**：固定 passphrase（含非 ASCII）、固定 16 B salt、固定 Master/MAC Key(hex)、固定 Recovery Key、固定 `jti`、固定 `object_id`、**固定每 chunk 24 B nonce**、固定 `key_epoch=0`、固定 `base_version=0`、固定 `device_id`、固定明文、固定 `manifest_seq=1`、固定 `manifest.id`。
**固定输出**：`KEK_password(hex)`、`KEK_recovery(hex)`、`wrapped.password.blob(base64)`、`wrapped.recovery.blob(base64)`、`versionMac(base64)`、对象密文(hex)（含空对象向量）、manifest 密文(hex)、`verify --json` 输出。
> **必须写进规范**：该向量**固定了 nonce 才逐字节一致**；正常运行使用随机 nonce，重复加密同一文件不会得到相同密文（安全属性，不是缺陷）。G4 只约束固定测试向量。

**恢复 CLI（M1 必备，与核心 crate 一起开源，不联网 / 不依赖主 App / 不依赖账号）**：
```
vault info     <root>                                    # 打印 format/cipher/kdf/key_epoch，不解密、不列对象 id
vault verify   <root> [--json]                          # 阶段1 结构检查（全量扫描、收敛输出）
vault verify   <root> --password [--json]               # 阶段2 完整密码学验证（细粒度错误码）
vault export   <root> --out <dir> [--password|--recovery]        # 导出全部明文 + 原始目录树
vault export   <root> --object <base32-id> --out <file> [--recovery]  # 单对象兜底取回
vault change-password  <root>                          # 重包裹 password slot（零吊销，UI 明说）
vault rotate-recovery  <root>                          # 新 RK 重包裹，旧 RK 失效（仍零吊销旧口令持有者）
vault rotate-keys      <root>                          # ★真吊销：新 Master/MAC/salt/jti/key_epoch + 全量重加密
vault fork     <root> <new-root>                       # v0.2+：克隆并轮换 jti（等价 rotate-keys 作用于副本）
vault gc       <root> --confirm                        # 回收孤儿 / 已解决冲突副本（水位触发）
vault mount    <root> <mountpoint>                     # 可选；唯一依赖 OS 虚拟文件系统的命令；拒绝同步根内挂载
```
> `mount` 是**唯一**依赖 OS 的命令，其余全为纯文件操作 → 印证「载体无关」核心卖点。`export --object` 在 manifest 损坏时仍可按 id 取回内容（名称可能丢失，须在文档说明）。

---

## 7. 遗留风险 / 明确不防御（与 threat-model.md 对齐）

| # | 风险 | 处置 |
|---|---|---|
| R1 | 主口令 + Recovery Key + 备份三者全失 → 永久不可恢复 | 新建库强制引导离线保存 RK + 跑一次 `verify` + `export` 回读 |
| R2 | 解锁/挂载期间明文「第二份副本」被 OS 索引/缩略图/云同步/VSS/AV-EDR 采集 | **仅未解锁态防得住；解锁态仅缓解**（F20/N9）；绝对隔离改用单文件解密导出 |
| R3 | 弱口令 | Argon2id 下限 + 建库强度提示；**口令熵为唯一上限**；无遥测故不能埋点验证假设 |
| R4 | 形状泄漏（对象数量 / 大致大小） | 显著压缩但未消除，对外诚实声明 |
| R5 | iOS 截图只能事后告警、录屏只能感知（两端不对称） | 写进安全边界 |
| R9 | **整盘回放旧 `(meta,manifest,d/)` 三元组的状态回滚** | 盘内无法根除（无外部单调锚点）；**可选**库外本地锚点 last-seen `head/seq`（F18），锚点丢失即失效；不宣称解决 |
| R10 | XChaCha20-Poly1305 非 IETF 标准 | 标注 expired draft + 广泛实现，不拿 RFC 8439 背书 |
| R11 | **本文是内部评审，不是第三方审计** | 上线前必须委托独立密码学审计 |
| **R12** | **零吊销**：任何凭据轮换（除 `rotate-keys`）都撤销不了持有旧 `vault.meta`+旧口令的人 | 架构事实；`rotate-keys` + 物理销毁旧介质 + 对外声明第 5 条 |
| **R13** | 单体 manifest：1 bit 翻转使清单整体不可读；每写 O(n) 重加密 | 由 `vault.meta.bak`（自带 versionMac）+ `export --object` 单对象兜底把影响限制在「名称可能丢失」；per-record 日志列为 v0.2+ 扩容路径（§11 裁定 12-②） |
| **R14** | `jti` 跨介质可关联 / 克隆后血缘不可分 | `fork`（=轮换 jti 的 rotate-keys，v0.2+）；默认克隆保留 jti，公开声明为可见元数据 |
| **R15** | 剪贴板历史 / 云剪贴板 / 他进程已读 / 明文导出后生命周期脱离产品 | 不可撤销，写进对外声明第 9 条 |

---

## 8. 必须写进对外安全边界的诚实声明（措辞冻结，v1.1 扩为 10 条）

1. 「**用户数据零明文**」——磁盘上看不到内容、文件名、目录名、目录结构、标签；`vault.meta` 字段名、README、`MYVAULT`、`d/xx/` 是格式自描述的**公开元数据**。**不写「零元数据」**。
2. 「**形状泄漏被显著压缩，但未 100% 消除**」——仍可能推断对象数量与大致大小（含单文件落在哪个 1 MiB 区间）。
3. 「**XChaCha20-Poly1305 基于 expired IRTF draft 与广泛业界实现，不是 IETF 标准、不是 RFC 8439 定义的 ChaCha20-Poly1305**」；`cipher` 写 `XCHACHA20-POLY1305`，不得简写为 `CHACHA20-POLY1305`。
4. 「**`verify` 绿不能证明是最新版**」——无服务端/可信单调计数器，整库回放、旧版本覆盖在密码学上不可检出；解锁前 `verify` **只检随机损坏/不完整，不检恶意篡改**；降级/篡改/冲突/陈旧判定一律解锁后（除非显式开启库外本地锚点，且锚点丢失即失效）。
5. 「**换口令 / 换 Recovery Key 均不撤销旧凭据**」——曾拿到旧 `vault.meta` 的人（旧备份、旧 U 盘、云历史）仍可用旧口令解开同一主密钥读取**当前**全部数据；真吊销须 `rotate-keys` 全量重加密 + **物理销毁所有旧介质副本**。
6. 「**仅未解锁态防合法采集者；解锁态仅缓解**」——解锁期间任何进程可读明文，OS 页缓存/swap、搜索与缩略图索引、最近文件、云同步、VSS/Time Machine 快照、AV/EDR/AI 扫描器可能在**库外留持久明文副本，锁定后不保证消失**；仅做挂载前排除（**拒绝在同步根内挂载**）、锁定 flush、尽力遮罩。
7. 「**持有容器者可离线无限次爆破**」——Argon2id **仅抬高单次成本，口令熵是唯一上限**；`kdf`+`wrapped` 本身即完整离线爆破输入；「延时减速」只对在线 UI 有效，离线有整盘故速率限制不适用。
8. 「**承认可见**」——这是一个 mylock vault、对象数量/总大小、KDF 参数与 salt、**存在 Recovery slot**、`jti` 跨介质可关联、有写入/同步行为。
9. 「**不防已攻陷设备**」——keylogger、截屏（iOS 无法阻止）、内存转储、冷启动、swap/休眠、恶意内核；`zeroize`/`mlock` 只缩小窗口。明文导出与剪贴板内容生命周期脱离本产品，云剪贴板/他进程已读**不可撤销**。主口令 + Recovery Key + 容器三者全失**任何人不可恢复**。
10. 「**格式层是成熟范式的组合，不是创新**」——创新在产品层（载体无关 + 主动防合法采集者 + 双形态同钥）。**本规范为内部设计定案，尚未经第三方密码学审计，不代表已实现 / 已认证；`verify` 绿 ≠ 未被恶意替换**。

---

## 9. 评审判定与后续

- **判定**：**通过（附条件）**。[vault-format-spec.md](vault-format-spec.md) 按本文升 **v1.1 可审计规范**；字段与算法组合冻结，可进入 M2 实现排期。
- **实现前的硬门槛**（不得跳过）：①§6 正负向测试向量实测通过、五端 + 恢复 CLI 逐字节一致（含空对象向量、KDF 越界向量）；②`threat-model.md` §6 K1–K10 状态列按本文回填（v1.1 已完成）；③主文档 §6.3 / §6.6 / §7.1 / §8.2 / §9.3 同步刷新至 v1.2；④**§11 对抗性审阅 16+ 项裁定全部有记录且被接受的项已落进规范**；⑤**上线前委托第三方密码学审计**（本文不构成安全背书）。
- **跨阶段边界不动**：用户 m00241 明确选择「先做 M1 密码学评审」，**不写产品代码、不进入开发阶段**。M2 实现须另行确认。

---

## 10. 三路独立复核交叉校验（全部回传）

| 子代理 | 任务 | 结果 | 带来的定案变化 |
|---|---|---|---|
| `f7654206-5cf2-46ac-9070-6245ba91fd4c` | RFC 8439 / RFC 8949 / OWASP / SP 800-38D | **已回传（m00298，finished）**，全部 web_fetch 200 | 新增 A7/A8/A9；**加强 1** RFC 8439 全文 0 次命中 XChaCha/HChaCha/192-bit → 随机 nonce 合法性只能由 XChaCha draft 背书、`cipher` 不得简写；**加强 2** RFC 8949 §4.2 → canonical JSON 也必须冻结显式 profile（F12c-1 八条）；**加强 3** Poly1305 tag 确为 16B/128-bit、nonce 复用 "reveals the XOR of the plaintexts" → 印证 F9。未核实：SP 800-38D PDF 406、libsodium 截断、「有效 CBOR ≠ 确定性 CBOR」逐字句不存在 |
| `c8835859-8356-40e1-bb9c-1922bfb99acf` | RFC 3394 / RFC 9106 / RFC 5869 / SP 800-38F | **已回传（m00329，finished）**，rfc-editor 与 CSRC 落地页 web_fetch 200；SP 800-38F PDF web_fetch 406 → 浏览器 Node `https` 取得 200 / application/pdf / 289262 bytes 并解压 FlateDecode 取正文 | 新增 A10/A11/A12。**决定性**：SP 800-38F 确认 AES-KW "approved ... protection of the confidentiality and integrity of cryptographic keys"、§B.2 "a MAC is not necessary"，但 §B.2(2)(3) 明列「不能认证关联数据」「不能在不解包下验真」、§A.3 明列「重复尝试累积、应限制失败次数」→ 直接支撑 N1 的「保留标准件 + 外层 versionMac 补偿 + 拒绝自造 wrap」裁定。A11 澄清「批准」地位来自 SP 800-38F 而非 Informational 的 RFC 3394；A12 禁止把「抗 GPU」当 RFC 9106 引文 |
| `dca3264f-b516-4237-ad92-ce155d826911` | 独立对抗性审阅 12 类问题 | **已回传（m00325，finished）** | 16+ 项发现；其中 11 项与 v1.0 已修问题重合（记为「已在 v1.0 解决」），5 项为真新增 → N1–N9 / F13–F20 / v1.1 修订。逐项裁定见 §11 |

> 维护约定：本文（v1.1 + §12 补遗）与 [vault-format-spec.md](vault-format-spec.md) **v1.1.1**（M1 PoC 后升版，§2.1 字节级拼接冻结 + §9 指向 [poc/test-vectors.json](poc/test-vectors.json)）、[threat-model.md](threat-model.md) v1.1、[产品分析与概要设计.md](产品分析与概要设计.md) v1.9 四处版本号与参数总表必须一致；任何一处改算法，四处同改。

---

## 11. 对抗性审阅逐项裁定（v1.1 新增 · 审计关键节）

> 来源：`dca3264f` 对 **v0.1** 的独立对抗性审阅（m00325）。因 v1.0 已先行修正多项，裁定分三类：**【已在 v1.0 解决】** / **【接受·v1.1 修订】** / **【拒绝·附理由】**。每条给出理由，不静默略过。

| # | 对抗性发现（摘要） | 裁定 | 理由 / 落点 |
|---|---|---|---|
| 【1】 | versionMac 无口令可验是假的、循环依赖；整份回滚旧 versionMac 仍有效 → 防降级/防回滚原理上不可实现；建议更名 headerMac、用 MAC Key、无口令态删此检查、`format` 下限由 CLI 本地策略钉死 | **部分接受 / 部分已在 v1.0 解决** | 循环依赖与「移到解锁后」已由 F1/D1 解决；「用 MAC Key（不用 Master）」v1.0 本就如此（§3.4 用 `HKDF-Expand(MAC_Key,...)`）；回滚不可检出已登记 R9 并写进 §8 第 4 条 → 与审阅主张一致。**接受**补「CLI 本地策略钉死最低支持 `format`」与「versionMac 实为全头认证标签、防降级只是效果之一」的说明（v1.0 §3.2 已列覆盖集）。**拒绝**改名为 `headerMac`：四份文档同步改名的收益低于引入不一致的风险，改由文档明确覆盖集即可 |
| 【2】 | 三重 MAC 重叠 + AAD 绑 versionMac ⇒ 换口令即锁死库；建议三者合一为 `manifest_ref.tag`、AAD 绝不绑 versionMac、凡改 wrapped/kdf/format 显式重写 manifest tag | **已在 v1.0 解决（F2/D2）** | v1.0 已删 `manifest_ref.mac`、AAD 改绑稳定标识 `magic‖format‖jti‖manifest.id‖manifest_seq`、用 `manifest.seq == manifest_ref.seq` 双向绑定。改 `wrapped`/`kdf` **不改变 AAD**（稳定标识不含它们）→ 无需重打 manifest tag，审阅担心的「换口令坏库」已消除。改 `format` 走 §8「export→新建→导入」迁移，非原地改写 → 无循环依赖 |
| 【3】 | RFC 3394 无 AAD：blob 不绑 `jti/format/kdf/slot/key_epoch` → 跨库移植 / password↔recovery slot 互换不可察；主张**彻底删 RFC3394，改用 XChaCha20-Poly1305 wrap + 显式 AAD** | **部分接受（补强）/ 拒绝（替换方案）** → F13/F3/N1 | **接受**其内核：AES-KW 确实无 AAD（A10 §B.2(2) 亲证）、64-bit 完整性在离线反复改盘下累积（A10 §A.3）。**拒绝替换**：①A10 明确 AES-KW 被 NIST "approved for key wrapping" 且 §B.2 "a MAC is not necessary"；②改用自造 XChaCha wrap 正是 D6 已否决的「自造包裹」，无标准测试向量、无 AAD 语义先例，A4 的随机 nonce 安全性适用于**内容**加密而非确定性密钥包裹；③审阅所述攻击**已被现有架构检出**：两 slot 由**不同 KEK** 包裹（F13 域分离后更确定），互换 blob → AES-KW 2^-64 完整性失败 `E_TAMPER`；跨库 blob 移植 → 用 A 库口令解 B 库 blob 必失败，或整份 meta_B 覆盖 → `d/` 在 Master_A 下的数据无法用 Master_B 解开 → `E_MANIFEST`；整库旧快照回放 = R9。④A10 结论正建议「若需把头部明文元数据纳入完整性，应在外层另加 MAC/AEAD」→ **versionMac 覆盖 canonical(meta 全字段含 wrapped/jti/kdf/key_epoch) 正是这一外层补偿**，且是 256-bit。**落点**：F13（slot 域分离 + `key_epoch` + versionMac 全头绑定）、§8 第 7 条、R12。AES-SIV（A3，RFC 5297，drop-in 替换 RFC3394、支持 AAD、128-bit 认证）列为 **`format=2` 候选**，若未来第三方审计要求 ≥128-bit 包裹完整性再评估，不在 v1.1 引入第二套 cipher |
| 【4】 | BLAKE3 keep，但重定义为**明文 catalog**（`(id,len,BLAKE3(对象文件))`，`catalog_hash` 绑 headerMac），声称唯一能检「整对象删除/换进历史合法对象」 | **拒绝** | ①审阅自认「无密钥哈希可重算 → **只检随机损坏，不检篡改**」，且 `catalog_hash` 仍绑在**解锁后**才验的 headerMac 上 → 与「解锁前可信校验」自相矛盾；②v1.0 的解锁前结构检查已用对象 header 的 `chunk_count/ciphertext_len` 对比实际长度覆盖截断，认证完整性由每 chunk AEAD tag（解锁后）覆盖，「整对象删除」由 `E_MISSING`（解锁后清单比对）覆盖；③明文 catalog 会**主动暴露精确大小与对象 id 列表**，比当前「大致大小」口径更差；④BLAKE3 一手规范未取到（§1），改用 SHA-256 又多一层代码。**折中**：把「若需解锁前发现缺失对象，可增加经 versionMac 认证的明文 catalog，但须声明仅检随机损坏且形状泄漏增加」记为 v0.2+ 可选，v1.1 维持 D4 删除 |
| 【5】 | SIV 名称加密在解不存在的问题；parent_id 若是不可变 object_id 则移动零重加密；冲突副本不该靠 name_cipher 相等区分 | **已在 v1.0 解决（D3/F2）** 并采纳其细化 | v1.0 已删 SIV、名称=AEAD 加密 manifest 内普通字段、`parent_id` 用稳定随机 `object_id`、按名查找在解锁后内存做、冲突副本用 `device_id+updated_seq` 区分。**采纳**其「确需免解密查找可加 `idx=HMAC(HKDF(Master,"name-idx"),parent_id‖name)[..16]`」作为 v0.2+ 可选索引（不进 v1.1，避免新增持久索引面） |
| 【6】 | 标量时钟无法表达并发；`device_id` 入库被整盘复制 → 两份自认同一写者静默丢更新；「不写隐藏状态文件」使陈旧/回滚 manifest 不可检出；双留导致无限膨胀；主张版本向量 + 克隆轮换 jti + OR-Set + 认证 GC + 禁 LWW | **部分接受**（F18/N6） / **部分拒绝** | **接受**：①`device_id` 改为**每安装本地生成、绝不写入容器、绝不从容器读取**（这是真 bug —— 整盘复制后两台设备必须各自用不同本地 id）；②记录加 `base_version` 使并发判定无歧义（同 base、不同 `(device_id, updated_seq)` → 双留）；③`content_hash` 用 **SHA-256**（非 BLAKE3）做同内容去重、水位触发 GC、`vault.meta.gc_pending` 崩溃标记；④**库外本地锚点** last-seen `head/seq` 作为**可选**回滚检测（明确锚点丢失即失效、不写入容器 → 不破坏 §6.8 载体无关）；⑤**禁 LWW**（v1.0 本就双留，从未 LWW）。**拒绝**全量版本向量 `map<device_id,u64>` / OR-Set / tombstone / `resolved_from` / `import-merge`：对「个人库 + 每对象 N=2 + 解锁态显式解决」的 MVP 属过度工程，收益不抵复杂度。**接受**「克隆轮换 jti」但重定义为 `fork`（v0.2+）= 对副本执行 `rotate-keys`（因 chunk AAD 绑 jti、对象密钥绑 Master，轮换 jti 必然全量重加密） |
| 【7】 | 无口令 verify 是现成 oracle（早停泄漏首个失败序号、耗时枚举数量、8 个码区分内部状态、`info` 打印对象数、`export --object` id 枚举）；主张解锁前输出收敛 `E_OK\|E_STRUCT`、全量不早停、定长、预认证统一 `E_AUTH_FAILED`、遍历全 slot | **接受（加严）**（F16/N5）+ **已在 v1.0 部分解决**（F7） | v1.0 F7 已做两阶段 + 收敛输出，但**保留了 `objects:N, bytes:M` 且未明写全量不早停/统一失败**。v1.1 加严：全量扫描不早停、定长输出、预认证失败统一 `E_AUTH_FAILED` + 常量时间 + 遍历全部 slot 后统一失败、`export --object` 预认证不区分 id 不存在/口令错、`info` 不列对象 id。**保留** `objects/bytes`：这两个数字列目录本就可得，不是新泄漏；审阅担心的「耗时 ∝ 已检数」由全量不早停消除 |
| 【8】 | 「零明文/形状不泄漏」三处口径打架；`kdf`+`wrapped` = 完整离线爆破输入；recovery slot 暴露第二凭据；`created` 无用且泄漏；`jti` 跨介质可关联；「延时减速防爆破」错（离线有整盘，速率限制不适用） | **接受**（F20/N8）+ **已在 v1.0 部分解决**（F8） | ①**删 `created` 字段**（v1.1 落实）；②§5/§8 补「`kdf`+`wrapped` 即完整离线爆破输入」「存在 Recovery slot 是公开事实」「`jti` 可关联」；③「延时减速」改为「Argon2id 仅抬高单次成本，口令熵为唯一上限；速率限制仅在线 UI 有效」；④形状口径已由 F8 统一到「用户数据零明文 / 形状未 100% 消除」。**拒绝**「slot 用不透明索引」：保留可读 slot 名利于审计，改由 §8 第 8 条公开声明其存在性。**拒绝**「删 `jti`」：jti 是 chunk/manifest AAD 的稳定绑定锚，删它要重构全部 AAD；改用 `fork` 轮换 + 公开声明 |
| 【9】 | v0.1 每对象密钥与每 chunk 密钥矛盾；**G4「固定输入→逐字节相同」与 CSPRNG 24B nonce 冲突**；主张全确定式（`K_chunk=HKDF(Master,"chunk"‖object_id‖idx)`、`nonce=HKDF(...,"nonce"‖object_id‖idx)[..24]`，写路径无 CSPRNG） | **拒绝**（G4 误读）+ **已在 v1.0 解决**（F9/F12c） | ①**G4 冲突是误读**：v1.0 §9 / F12c 已明确「**该向量固定了 nonce 才逐字节一致**；正常运行使用随机 nonce，重复加密同一文件不会得到相同密文（安全属性）」，G4 只约束固定测试向量，不要求运行时确定性密文。②v0.1 的「每对象密钥 + 每 chunk 密钥」矛盾已由 F9 统一为单对象密钥 + 每 chunk 随机 nonce。③审阅自己的「全确定式」方案**同时**派生 per-chunk 密钥**与**确定性 nonce，正是它批评 v0.1 的「并存多余」；且把 nonce 安全性全押在「object_id 永不复用」这一不变量上，比 CSPRNG 192-bit 随机更脆弱（A7 明写 nonce 复用 "reveals the XOR of the plaintexts"）。④保留 CSPRNG 有 libsodium/WireGuard 先例（A4）。**折中**：把「确定性 nonce 是合法备选，但需以 object_id 永不复用为硬不变量」记入 v1.1 §4 对象 id 行的 copy-on-write 不变量，不切换主线 |
| 【10】 | 「换口令不重加密」= **零吊销**：两 slot 包同一 Master，持有旧 `vault.meta` 者用旧口令仍能解开同一主密钥读当前全部数据；主张拆 `change-password`/`rotate-keys`、加 `key_epoch`、RK 轮换走 rotate-keys、每 slot 绑 `jti/kdf/key_epoch`、F 写明旧介质须物理销毁 | **接受（全文最重要的新发现）**（F17/N2） | 审阅完全正确，v1.0 的 F10 只说了「换口令 Recovery Key 保持有效」，**没有指出这对旧 `vault.meta` 持有者意味着零吊销**，也没把 `rekey` 从「v0.2+ 非默认」提升为一等命令。v1.1：①`change-password`/`rotate-recovery` 明确标注**零吊销**（且 rotate-recovery 仅使旧 RK 失效、对旧口令持有者仍零吊销）；②**`rotate-keys` 升为规范内一等命令**（新 Master/MAC/salt/jti/`key_epoch++`/新 RK + 全量重加密 + GC 旧对象 + 物理销毁旧介质）；③`key_epoch` u32 纳入 versionMac 覆盖集与 RK 编码，旧 epoch 报「已轮换」；④`fork` 复用同一机制；⑤写进 §8 第 5 条与 R12。**附带修正**：因 chunk AAD 绑 `jti`、对象密钥绑 Master，`rotate-keys`/`fork` 必然全量重加密 —— 这同时解决了【6】的「克隆血缘不可分」 |
| 【11①】 | KDF 参数盘上可控 → 改 `m=4 TiB` 即 OOM DoS；须解锁前硬校验参数上下界 | **接受**（F14/N3） | 真实且廉价。硬校验 `19456 ≤ m ≤ 1048576 KiB`、`2 ≤ t ≤ 10`、`1 ≤ p ≤ 16`，越界 `E_FORMAT` 绝不先跑 Argon2id |
| 【11②】 | 错误码/早停泄漏 + 解锁错误应统一、常量时间、遍历全 slot | **接受**（F16） | 并入 §5 阶段 1 与阶段 2 的 `E_AUTH_FAILED` 统一语义 |
| 【11③】 | `zeroize` 落不了地：Rust `String` 不可清零；须入口即 `SecretVec/Zeroizing`、覆盖 KEK/Master/MAC/manifest 明文/chunk 缓冲、加 `mlock`、禁 core dump、带上限 LRU | **接受**（F19/N7） | v1.0 只写「zeroize 清零」确属空话。v1.1 具体化到类型、覆盖范围、LRU 上限，并声明 swap/休眠/冷启动不受控 |
| 【11④】 | 剪贴板清除撤销不了他进程已读与剪贴板历史/云剪贴板 | **接受** | 写进 §8 第 9 条与 R15 |
| 【11⑤】 | **最大缺口**：挂载态明文外溢（§6.6 只说「任何进程可读」，未列库外持久副本）；主张 F 增列「不防解锁态明文被复制到库外、锁定后不保证消失」、产品只做首挂前排除/拒绝同步根/锁定时 flush、把「防得住」降为「仅未解锁态防得住 / 解锁态仅缓解」 | **接受**（F20/N9） | v1.0 `threat-model.md` §3.1 已列 OS 索引/缩略图/最近文件/云同步/临时/换页，但**口径仍偏强**且缺 VSS/Time Machine/AV-EDR/AI 扫描器。v1.1 降级口径、补全载体、把「同步目录内挂载」由「强提示」升级为「**拒绝挂载**」 |
| 【12①】 | MAC 输入字节未定义（`\|` 编码未定、JSON 非确定）→ G4 不可能；主张全份 deterministic CBOR + `len4LE‖CBOR‖tag32` 分帧、**先定 P8 再进 M1** | **已在 v1.0 解决**（F12c-1） / **拒绝 CBOR** | `f7654206` 的 A8 已给出同一结论（RFC 8949 §4.2「确定性格式须由应用协议自行定义」），v1.0 据此冻结 **canonical JSON 八条规则**（F12c-1），MAC 输入字节已完全定义 → 「先定 P8」已满足。**拒绝** CBOR：A8 §1.1 明写合法 CBOR 允许多种编码，采用它**同样**要另写确定性 profile，相对 JSON 无优势且增大解析面（parser-differential 风险更高）；`len4LE‖CBOR‖tag32` 分帧对纯文件容器亦无必要 |
| 【12②】 | 单体 manifest：1 bit 翻转全表报废、每写 O(n) 重加密、移动端整表驻留、叠加无限膨胀；主张拆 per-record AEAD 日志 + `head=BLAKE3(各 record tag)` 受 headerMac 认证 | **拒绝（MVP）/ 接受（v0.2+ 扩容路径）/ 已部分缓解** | ①个人库（数百–数千文件）单体 manifest 简单可审计，`vault.meta.bak`（自带 versionMac）+ `export --object` 单对象兜底把「1 bit 翻转」影响限制在「清单不可读、内容仍可取回但名称可能丢失」→ 登记 R13；②per-record 日志引入记录级密钥、head 哈希、日志 GC，复杂度显著上升，且 `head=BLAKE3(各 tag)` 又把未取到一手规范的 BLAKE3 拉回来；③「无限膨胀」已由 F6/F18（N=2 + `base_version` + 水位 GC + `content_hash` 去重）收敛。列为 v0.2+ 扩容路径，不进 v1.1 |
| 【12③】 | `cipher` 双套件重引入协商面，GCM 96-bit nonce 与随机 nonce 叠加有重用风险 | **已在 v1.0 解决（D6）** | 单一 `XCHACHA20-POLY1305`，AES 仅用于 RFC3394 包裹，FIPS 走 `format=2` |
| 【12④】 | `chunk_count=0`（空文件）零 tag，明文 header 可任意伪造且 verify 不报 `E_CORRUPT` | **接受**（F15/N4） | 强制 `chunk_count ≥ 1`，空明文也发一个带 AAD 的空 chunk（24B nonce + 16B tag）；负向向量新增「空对象伪造」 |
| 【12⑤】 | L30 manifest 是固定文件名，L64/142/153 却给它「每次写入更新」的 id 并用于派生 `HKDF(Master,"manifest"‖id)` | **已在 v1.0 解决（F2）** | v1.0 `manifest_ref = { id, seq }`：`id` 建库时一次 CSPRNG **全库固定**，每写只递增 `seq` 并换 manifest 的 AEAD nonce/tag；`manifest_key = HKDF-Expand(Master,"manifest"‖manifest.id,32)` 稳定 → 换口令/换 seq 都不改变它 |

> **裁定统计**：已在 v1.0 解决 7 项（【2】【5】【9】【12①】【12③】【12⑤】 及【1】【7】【8】 的大部）；接受/部分接受并落进 v1.1 共 12 项（【1】【3】【6】【7】【8】【10】【11①–⑤】【12④】 + 【4】【12②】 的折中登记）；明确拒绝 5 项（【3】 的替换方案、【4】 的明文 catalog、【9】 的全确定式、【12①】 的 CBOR、【12②】 的 per-record 日志、【6】 的全量版本向量 —— 按主张计数）。
> **本节价值**：对抗审阅虽针对 v0.1，但它逼出了 v1.0 完全没写到的「**零吊销**」与「**AES-KW 无 AAD 的标准级确认**」两条，并把「verify oracle」「zeroize 落地」「挂载期口径」从原则变成了可验收的条款。这正是引入独立对抗审阅的目的。

---

## 12. v1.1.1 补遗（M1 PoC 实测回写，2026-11-14）

> 性质：用户 m00606 授权「做可运行 PoC 验主链路与测试向量」后，`poc/verify_primitives.py` 用**权威已发布 KAT** 逐字节校验了全部被审计库（26/26 PASS），并驱动主链路 PoC 时暴露 3 处规格文字问题。本节只更正**措辞/选型**，不改动任何已定案的算法、参数或字段集；[vault-format-spec.md](vault-format-spec.md) 已同步标为 v1.1.1。

### 12.1 原语 KAT 校验结果（新增 A 级实测，不替代 §1）

| 项 | 一手 KAT | 库 | 结果 |
|---|---|---|---|
| AES-256-KW | RFC 3394 §4.6（256/256）+ 我们 64B→72B 的 n=8 实况、1bit 篡改拒绝、错 slot KEK 拒绝 | `cryptography 50.0.2` | PASS |
| XChaCha20-Poly1305 | draft-xchacha-03 §A.3.1（ct/tag/decrypt/AAD 篡改/ct 篡改/空明文 16B tag） | `PyNaCl 1.6.2`(libsodium) | PASS |
| Argon2id | **RFC 9106 §5.3 精确 KAT（含可选 Secret+AD）** 经 `argon2-cffi` low_level.core 手工填 `argon2_context` 复现 | `argon2-cffi 25.1.0`(libargon2) | PASS |
| HKDF-SHA256 | RFC 5869 A.1 PRK/OKM + 「仅 Expand」变体 | `cryptography` | PASS |
| HMAC-SHA256 / SHA-256 | RFC 4231 TC1 / FIPS180-2 | `cryptography`+stdlib | PASS |
| canonical JSON | §3.3 八条 golden string | 本项目序列化器 | PASS（暴露措辞问题，见 12.2） |

### 12.2 实测发现 ①：§3.3 规则 ⑤ 措辞自相矛盾（已勘误）

§3.3 规则 ⑤ 原写「**最短转义**：仅 `\"` `\\` 与控制字符 `\u00XX`」——「最短」会诱导实现把 `\n`/`\t`/`\r` 写成 JSON 短转义，而枚举集合要求控制字符一律 `\u00XX`。两者对含控制字符的 `manifest.name` 会产出不同字节 → versionMac/跨实现比对失败。PoC 的 golden string（`line\nquote"`）实测触发。**裁定：枚举集合优先**，规则 ⑤ 收紧为「转义集合封闭：仅 `\"`、`\\`，控制字符一律 `\u00XX`（小写 hex，含 `\n`/`\t` 也必须用 `\u00XX`），不转义非 ASCII」。已写入 [vault-format-spec.md](vault-format-spec.md) §3.3；`poc/verify_primitives.py` 的 canonical JSON golden 同步改为 `line\u000aquote\"` 并 PASS。

### 12.3 实测发现 ②：`device_id` 术语自相矛盾（已消歧）

v1.1 多处（N6 / F18 / §1 / §2 / §5.3 / §6.3 / §9）写「`device_id` 绝不写入容器、绝不从容器读取」，却又在 §5.3 manifest 记录字段里列了 `device_id`、在 §6.3 用它做并发比对、在 §9 要「固定 `device_id`」测试向量——实现者会读成「既要写又不能写」。这不是算法缺陷，是**一个名字被用作两种语义**。裁定按两义拆开（见 [vault-format-spec.md](vault-format-spec.md) §5.3 消歧块）：

- **①设备当前身份**：每安装 16B CSPRNG，只存设备本地状态，**整盘复制不携带、绝不从容器读回**——N6「绝不写入容器」的准确对象。
- **②写入者溯源标签**：每条加密 manifest 记录在写入瞬间读①的值、作为一条**加密字段**写进记录本身，供解锁态 `(device_id, updated_seq)` 并发比对与 §9 固定向量。②是加密记录内容，不是设备状态，与①不冲突。

§1「设备本地状态不计入容器」、§2 总表、§0 的 N6 行、§6.3 并发段、§9 向量输入均已按此口径补注；本文 §11 的【6】/F18 历史裁定文字保留不改（历史记录），以本节为准。

### 12.4 实测发现 ③：技术栈必须点名 `argon2` crate 的 parallelism=4（A 级，影响 §7.1）

PoC 实测：`argon2-cffi`(libargon2) 在 `p=4` 下产出 IKM `b6393f9d…`，与 `p=1` 的 `ef99bda9…` 不同；`PyNaCl`/libsodium 的 `crypto_pwhash` **硬编码 parallelism=1**，与 libargon2 的 `p=1` 结果逐字节一致（`eq` PASS），因此**无法产出我们冻结的 `p=4` IKM**。结论：Rust 实现的口令 KDF **必须用 RustCrypto `argon2` crate 并显式 `parallelism=4`**（[产品分析与概要设计.md](产品分析与概要设计.md) §7.1 已列该 crate），**不能用 libsodium `crypto_pwhash` 替代**；XChaCha20-Poly1305 仍可继续用 libsodium / RustCrypto `chacha20poly1305`。若某端误用 p=1，跨端解锁会确定性失败（AES-KW 解包得错误 Master → `E_AUTH_FAILED`），故须纳入 §9 五端逐字节一致性向量。

### 12.5 实测发现 ④⑤：字节级拼接未冻结 + 恢复槽 KDF 输入未逐字定义（v1.1.1 已冻结）

- **FINDING-4（字节级拼接）**：v1.1 的 `‖` / HKDF `info` / AAD 只写「语义拼接」，没冻结字节；两个独立实现即使算法全对也可能逐字节不一致 → G4「五端逐字节一致」无法验收。M1 PoC 用可运行代码把整数编码（u32 big-endian）、`jti`(ascii)、`object_id`/`manifest.id`(16B 原始值)、对象 header(固定 32B)、chunk 布局、HKDF `info`(ASCII 标签+16B id)、chunk/manifest AAD、manifest 明文(无外层包装)全部钉死，现冻结为 [vault-format-spec.md](vault-format-spec.md) **§2.1**；改动须升 `format` 版本。
- **FINDING-5（恢复槽 KDF 输入）**：v1.1 §3.4 把 `slot ∈ {password, recovery}` 只挂在 HKDF 行，未逐字写「Recovery Key 也过 Argon2id、用哪个 salt」。PoC 直读为「两 secret 同参数同 salt 各自 Argon2id → 各自 `wrap/<slot>` 域分离 KEK → 包裹同一 Master∥MAC_Key」，并经 [4b] 恢复槽端到端解密成功、[4c] RK 喂错槽 `E_AUTH_FAILED` 验证。现冻结进 §2.1；未来改须升 `format` 并给新向量。
- **测试向量契约**：[poc/test-vectors.json](poc/test-vectors.json)（13499 B）为 §9 唯一逐字节契约，由 [poc/mylock_poc.py](poc/mylock_poc.py) 产出；主链路（建库→整盘复制→phase-1→口令解锁→phase-2→export）+ 恢复槽端到端 **ALL PASS**，**13/13 负向向量**命中期望码，orphan 为警告非致命。详见 [poc-report-m1.md](poc-report-m1.md)。
- **诚实边界**：PoC 是**单一 Python 实现**用被审计库产出的参考向量并跑通主链路；§12 硬门槛 1 要求的「**五端 + 恢复 CLI 逐字节一致**」**尚未满足**（需 Rust 内核与各端实现后逐一比对），不得据此宣称 G4 已验收或「已实现」。仍**不是产品代码、不是 M2 开发、不是第三方审计**。

> 纪律不变：本节全部为**设计/选型结论**；PoC 是一次性验证脚手架，不是产品代码、不是 M2 开发。上线前第三方审计（§9 硬门槛 5）仍然必须做。
