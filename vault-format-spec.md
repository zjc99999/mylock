# Vault 容器格式规范 v1.1.1（可审计规范 · 字段冻结 · 对抗裁定回填 · M1 PoC 实测契约）

> 状态：**M1 密码学评审通过（附条件）· 设计定案 v1.1.1**。字段与算法组合已冻结，可被独立审计；**不是实现、不是安全认证、不是第三方审计通过**。不得对外宣称「已支持 / 已实现 / 已认证」。
> 评审依据：[crypto-review-m1.md](crypto-review-m1.md) **v1.1 + §12 补遗**（A/A⁻ 级标准论据 + P1–P10 / F1–F20 定案 + §11 对抗性审阅逐项裁定 + §12 M1 PoC 实测回写）。威胁模型见 [threat-model.md](threat-model.md)。上位文档 [产品分析与概要设计.md](产品分析与概要设计.md) §6.3 / §6.6 / §6.8。
> 日期：2026-11-13 初稿 / 2026-11-14 v1.1 定案 / **2026-11-14 v1.1.1（M1 PoC 实测契约）**。载体：**「载体无关、自描述、可独立取回」**——让「拷到移动硬盘仍是保险柜」与「不装本 App 也能取回」成为可验证的工程事实。
> 设计红线：**不依赖任何 OS 或文件系统特性**——不用 xattr / NTFS ADS / 硬链接 / 符号链接 / 稀疏文件 / 权限位 / 大小写敏感性 / 时间戳做正确性判断。
> 版本变化：**v0.1→v1.0** 修掉 6 处结构性问题（§0.2 D1–D6）；**v1.0→v1.1** 回填独立对抗性审阅的 9 组裁定（§0.3 N1–N9），其中最重要的是 **N2 零吊销 → `rotate-keys` 升为一等命令**；**v1.1→v1.1.1**（M1 PoC 实测回写，见 [crypto-review-m1.md](crypto-review-m1.md) §12 与 [poc-report-m1.md](poc-report-m1.md)）不改动任何已定案算法/参数/字段集，只做三件事：①§3.3 规则⑤「转义集合封闭」措辞勘误；②§5.3 `device_id` 两义消歧；③**新增 §2.1「字节级拼接约定（冻结）」与 §9 的 [test-vectors.json](poc/test-vectors.json) 实测契约指针**，把 AAD / HKDF info / 整数编码 / 恢复槽 KDF 输入逐字节钉死。
> 维护约定：本文（**v1.1.1**）与 `crypto-review-m1.md` v1.1+§12、`threat-model.md` v1.1、`产品分析与概要设计.md` v1.9 四处版本号与参数总表必须一致；任何一处改算法，四处同改。**§9 固定测试向量以 [poc/test-vectors.json](poc/test-vectors.json) 为唯一逐字节契约**（由被审计过的库产出，M1 PoC 一次性验证脚手架，非产品代码）。

---

## 0. 总览

### 0.1 一句话

`<vault-root>/` 是一个**纯静态普通文件目录**：`vault.meta`（公开头 + 被包裹的主密钥 + 全头认证标签）+ `manifest`（加密清单）+ `d/`（每文件独立密文对象池）+ `README.txt`。整目录用户数据零明文、自描述、自校验，**拷到任何文件系统、任何 OS 都仍是同一个保险柜**，但**透明性不随盘走**（读明文须在装了解锁器的设备上输口令）。

### 0.2 v1.0 相对 v0.1 的 6 项结构性修正（评审定案）

| # | v0.1 的问题 | v1.0 定案 |
|---|---|---|
| D1 | 「无需口令即可验 versionMac」自相矛盾（MAC 密钥须先解包） | versionMac 移到**解锁后**；解锁前只有非密码学结构检查 |
| D2 | manifest AAD 绑定 versionMac → 换口令重算 versionMac 会让整个库读不出 | AAD 改绑稳定标识；删 `manifest_ref.mac`；用 `manifest_seq` 双向绑定 |
| D3 | 「名称用 SIV」与全链路 XChaCha 冲突（RFC 5297 证明 SIV 是 AES 模式，无 XChaCha-SIV） | 删除名称 SIV；名称 = AEAD 加密 manifest 内普通字段 |
| D4 | BLAKE3 checksum 与每 chunk Poly1305 tag 冗余，且存在加密 manifest 内→解锁前读不到 | 删除 BLAKE3；完整性只由每 chunk AEAD tag + 明文 header 长度承担 |
| D5 | PHC 字符串保存 `<hash-b64>` 落盘 = 免费离线口令验证器 | `kdf` 只存参数 + salt，**不存任何派生结果** |
| D6 | 运行时 `AES-256-GCM` / `XChaCha20` 双 cipher 协商面 | 单一 `XCHACHA20-POLY1305`；AES 仅用于 RFC 3394 密钥包裹；FIPS 走 `format=2` |

### 0.3 v1.1 相对 v1.0 的 9 组裁定（独立对抗性审阅，详见 crypto-review-m1.md §11）

| # | v1.0 未覆盖的真问题 | v1.1 定案 |
|---|---|---|
| N1 | NIST SP 800-38F §B.2 / §A.3 亲证：AES-KW **无 AAD**、64-bit 完整性在离线反复改盘下累积 | **保留标准件**（批准、有测试向量），补齐 F13：`KEK_slot` 域分离 + `key_epoch` + versionMac **全头绑定** 作 256-bit 外层补偿；拒绝自造 XChaCha wrap；AES-SIV 列为 `format=2` 候选 |
| N2 | **换口令 = 零吊销**：持有旧 `vault.meta`+旧口令者仍解开同一主密钥读当前全部数据 | **`rotate-keys` 升为一等命令**（新 Master/MAC/salt/jti/`key_epoch++`/新 RK + 全量重加密 + 销毁旧介质）；`change-password`/`rotate-recovery` 明确标注零吊销；RK 带 `key_epoch`（F17） |
| N3 | KDF 参数盘上可控 → `m=4 TiB` OOM DoS | 解锁前硬校验 `m/t/p` 上下界，越界 `E_FORMAT`，绝不先跑 Argon2id（F14） |
| N4 | `chunk_count=0`（空文件）零 tag，明文 header 可伪造 | 强制 `chunk_count ≥ 1`，空明文也发一个带 AAD 的空 chunk（F15） |
| N5 | 解锁前 verify 被当 oracle（早停 / 错误码 / id 枚举） | 全量扫描不早停、定长输出、预认证失败统一 `E_AUTH_FAILED`、遍历全 slot、常量时间（F16） |
| N6 | `device_id` 存库内被整盘复制 → 两份自认同一写者 → 静默丢更新 | **`device_id` 分两义（见 §5.3 术语消歧）**：当前设备身份只存设备本地、绝不从容器读回（整盘复制不携带它）；每条加密记录带写入瞬间的本地 id **快照**作溯源标签用于并发判定；记录加 `base_version` 判并发；拒绝全量版本向量，保留 N=2（F18） |
| N7 | `zeroize` 只提名字、Rust `String` 不可清零 | 具体化 `SecretVec`/`Zeroizing` + `mlock` + 禁核心转储 + 带上限 LRU（F19） |
| N8 | `created` 无用且泄漏创建时间；「延时减速」对离线爆破不适用 | **删 `created` 字段**；口径改「Argon2id 仅抬高单次成本，**口令熵为唯一上限**」（F20） |
| N9 | 挂载期明文在**库外**留持久副本口径过弱 | 降级为「仅未解锁态防得住 / 解锁态仅缓解」，补 VSS/Time Machine/AV-EDR；**拒绝在同步根内挂载**（F20） |

> 另有 5 项**拒绝**并已在评审报告 §11 记明理由：明文 BLAKE3 catalog（【4】）、全确定式 nonce（【9】，G4 系误读）、deterministic CBOR 分帧（【12①】）、per-record AEAD 日志（【12②】，列为 v0.2+ 扩容路径）、全量版本向量/OR-Set（【6】）。

### 0.4 设计目标与验收条件

| # | 目标 | 可验证的验收条件 |
|---|---|---|
| G1 | **用户数据零明文** | 对整个 vault 目录做字符串/熵扫描，除 `vault.meta` 显式公开字段与 README，找不到任何用户内容/文件名/目录名（口径见 §5.4） |
| G2 | 形状泄漏被显著压缩 | 文件名/目录名/目录结构/标签/精确大小不泄漏；**仍可能推断对象数量与大致大小**（不承诺 100% 消除） |
| G3 | 载体无关 | 同一 vault 在 NTFS / exFAT / FAT32 / APFS / ext4 往返拷贝后 `vault verify` 全绿、可解锁 |
| G4 | 跨平台一致 | 五端（Win/mac/Linux/Android/iOS）+ 恢复 CLI 对**固定测试向量**产出逐字节相同密文；**只约束固定向量，不要求运行时确定性密文** |
| G5 | 可独立取回 | 不装主 App，仅用公开恢复 CLI + 主口令/Recovery Key 即可导出全部明文 |
| G6 | 篡改可检出 | 改 1 bit meta / 任一对象 → 解锁后 `verify` 报对应错误码，绝不返回未认证明文 |
| G7 | 不完整可检出 | 只拷子集 / 半途断电 / 旧版覆盖 → 检出缺失/孤儿/版本不一致（**整盘回放旧三元组 = R9，盘内不可检出，写进诚实声明**） |
| G8 | 增量友好 | 改 1 个文件只重写该对象密文 + `manifest`；不触碰其他对象 |

---

## 1. 目录布局

```
<vault-root>/                     ← 目录名可任意改名/移动/拷贝，不参与任何密码学计算
├── vault.meta                    ← 唯一「半公开」文件：格式头 + 参数 + 被包裹的密钥 + 全头认证标签
├── vault.meta.bak                ← 同样受 versionMac 保护的上一份 meta（防半写入损坏，§6.4）
├── manifest                     ← 加密并认证的对象清单
├── d/                           ← 密文对象池（扁平 + 固定 2 层分片，不映射用户路径）
│   ├── <2-char>/                ← 固定分片目录，取自对象 id 的前 2 个 base32 字符
│   │   └── <base32-id>          ← 单个密文对象（若干 chunk 串接）
│   └── ...
└── README.txt                   ← 纯文本说明（公开元数据，不含用户数据）：格式版本、恢复 CLI 获取方式、
                                      「请勿单独删除本目录内文件」的提示
```

- **路径不含任何用户信息**：`d/<2-char>/<base32-id>` 由随机 id 决定，与用户原始文件名/目录树无关。用户目录树作为**加密后的一条记录**存在 `manifest` 里，不映射成磁盘目录层级。
- **固定 2 层分片**只为规避单目录文件数上限，是定长结构，不违反扁平化。
- **不写临时明文文件、不写索引缓存、不写隐藏状态文件**；一切状态由三者当前内容决定。`vault.meta` 自身改写用「临时文件 + fsync + 原子 rename + bak」（§6.4），临时文件属格式状态机内部、非用户明文。
- **设备本地状态不计入容器**：设备的**当前** `device_id`、可选回滚锚点 last-seen `head/seq`、生物识别硬件 slot 均存**设备本地**、绝不从 `<vault-root>/` 读回作为设备身份（§5.3 / §6.3）。（区分：每条加密 manifest 记录里那条 `device_id` 是写入瞬间的本地 id **快照/溯源标签**，不是设备身份状态；二者同名但语义不同，见 §5.3 消歧。）

---

## 2. 密码学总表（实现冻结值 · v1.1）

| 层 | 取值 | 依据 |
|---|---|---|
| 口令 → KEK | `Argon2id(m=65536 KiB, t=3, p=4, salt=128-bit, tag=256-bit, memory-wiping)` → **`KEK_slot = HKDF-Expand(out, "wrap/" ‖ slot, 32)`**，`slot ∈ {password, recovery}`；移动端保底 `m≥19456 KiB, t=2, p=1` | RFC 9106 §4 SECOND / OWASP / **F13** |
| **KDF 参数硬校验** | 解锁前校验 `19456 ≤ m ≤ 1048576 KiB`、`2 ≤ t ≤ 10`、`1 ≤ p ≤ 16`；越界 `E_FORMAT`，**绝不先跑 Argon2id** | **F14 / N3** |
| 口令编码 | 完整 Unicode，保留 NULL 字节，不做 trim；显式 NFC（写入时记录）；**不落盘任何派生结果** | OWASP / D5 |
| 密钥包裹 | `AES-256-KW` (RFC 3394, **NIST SP 800-38F 批准**)，输入 64 B = 8 块，ICV `A6A6A6A6A6A6A6A6`，输出 72 B；解包失败 MUST 报错不返回密钥（2^-64 首道过滤） | RFC 3394 / A10 |
| 主密钥 | `Master(32 B) ∥ MAC_Key(32 B)`，CSPRNG 生成，永不落盘明文 | C |
| **key_epoch** | u32，初值 **0**，**仅 `rotate-keys` 递增**；纳入 versionMac 覆盖集与 RK 编码 | **F17 / N2** |
| 全头认证/防降级标签 | `versionMac = HMAC-SHA256(HKDF-Expand(MAC_Key, "meta-version-mac", 32), canonical(meta 全字段，排除自身))`；**解锁后**常量时间比对；**256-bit 外层补偿 AES-KW 的 64-bit 与无 AAD 局限** | RFC 5869 / **F3 / F13** |
| 子密钥派生 | **仅 HKDF-Expand**（Master 已是 256-bit 均匀密钥，跳 Extract），`info = context ‖ id`，**禁止 PRK 直接当 OKM** | RFC 5869 §3.3 |
| HKDF contexts | `"meta-version-mac"` / `"manifest"‖manifest.id` / `"object"‖object_id` / **`"wrap/"‖slot`**；**无 `"name"`** | F13 |
| 内容 / manifest AEAD | **XChaCha20-Poly1305**，192-bit(24 B) 随机 nonce / chunk，128-bit(16 B) tag | draft-xchacha-03（expired，非 RFC） |
| 单一 cipher | `XCHACHA20-POLY1305` 唯一值；**无运行时协商**；不简写为 `CHACHA20-POLY1305`；FIPS 走 `format=2` | D6 / F12d |
| chunk AAD | `magic ‖ format ‖ jti ‖ object_id ‖ chunk_index ‖ chunk_count ‖ header_hash` | F9 |
| manifest AAD | `magic ‖ format ‖ jti ‖ manifest.id ‖ manifest_seq`（稳定，**不绑 versionMac**） | D2 |
| 对象 id | 128-bit CSPRNG → base32（RFC 4648，大写无填充，26 字符），前 2 字符分片；**每次写入全新、永不复用**（copy-on-write 不变量） | F18 |
| **空对象** | `chunk_count ≥ 1`；0 字节明文也发一个带 tag 的空 chunk | **F15 / N4** |
| 分块 | 1 MiB / chunk | P4 |
| 完整性 | 每 chunk Poly1305 tag（解锁后）+ 明文 header 长度字段（解锁前结构检查）；**无 BLAKE3、无 checksum、无明文 catalog** | D4 |
| 序列化 | canonical JSON，**必须实现 §3.3 冻结 profile（8 条规则）** | F12c-1 |
| 名称 | AEAD 加密 manifest 内的普通字段；**无独立名称加密、无 SIV** | D3 |
| `vault.meta` 写入 | 临时文件 + fsync + 原子 rename + `vault.meta.bak`（各自带 versionMac）+ `gc_pending` 崩溃标记 | F12b / F18 |
| **`created`** | **删除**（v1.1） | F20 / N8 |
| Recovery Key | 256-bit → base32(52) + 4 校验位 + **`key_epoch` 前缀**，13×4 分组；不引 BIP39 | F12c / F17 |
| **device_id** | 两义（§5.3）：设备**当前**身份 16B CSPRNG 只存设备本地、绝不从容器读回；每条加密记录里的 `device_id` 是写入瞬间读该本地值写进记录的**溯源标签快照**（密文，非设备状态） | **F18 / N6** |
| 冲突 | 每对象 N=2 + `base_version` 并发判定 + 双留 + **禁 LWW** + `vault gc --confirm` 水位回收 | F6 / F18 |
| 内存 | `SecretVec`/`Zeroizing` 覆盖口令/KEK/Master/MAC/manifest 明文/chunk 缓冲；`mlock` + 禁核心转储 + 带上限 LRU | **F19 / N7** |
| verify 阶段 1 | 全量扫描不早停、定长输出、预认证统一 `E_AUTH_FAILED`、遍历全 slot、常量时间 | **F16 / N5** |

> **诚实标注（必须随规范传播）**：XChaCha20-Poly1305 基于 **expired IRTF Internet-Draft**（draft-irtf-cfrg-xchacha-03，2020 过期，"not endorsed by the IETF and has no formal standing"）+ WireGuard / libsodium / Tink / Go / LibreSSL 广泛实现，**不是 IETF 标准、不是 RFC 8439 定义的 ChaCha20-Poly1305**（后者用 96-bit nonce 且明写 "MUST NOT be randomly generated"，RFC 8439 全文检索 XChaCha/HChaCha/192-bit 命中 0 次）。随机 24B nonce 的合法性**仅由 XChaCha draft 的 192-bit 性质支撑**，不得拿 RFC 8439 背书。
> **AES-KW 标注**：其「被批准」地位来自 **NIST SP 800-38F**（"approved ... protection of the confidentiality and integrity of cryptographic keys"），**不是** Informational 的 RFC 3394 本身；SP 800-38F §B.2 亦列出「不能认证关联数据」「不能在不解包下验真」、§A.3 「重复尝试累积」——这些缺口由 **versionMac 全头绑定**补偿（F13）。

### 2.1 字节级拼接约定（v1.1.1 冻结 · 由 M1 PoC 实测钉死，FINDING-4 / FINDING-5）

> v1.1 的 `‖`、HKDF `info`、AAD 只写到「语义拼接」，**没有冻结字节**，导致两个独立实现即使算法全对也可能逐字节不一致 → G4 的「五端逐字节一致」无法验收。M1 PoC（[poc/mylock_poc.py](poc/mylock_poc.py)）用被审计过的库把以下约定跑通并产出 [test-vectors.json](poc/test-vectors.json)，现冻结为规范的一部分；**改动这些字节必须升 format 版本**。

| 项 | 冻结字节约定 |
|---|---|
| 整数编码 | `format` / `key_epoch` / `manifest_seq` / `chunk_index` / `chunk_count` / `ciphertext_len` 一律 **u32 big-endian（4 B）**；`obj_version` / `cipher_id` 为 **u8（1 B）** |
| `jti` | uuid-v4 字符串的 **ASCII 字节**（即 meta 里那个字符串本身） |
| `object_id` / `manifest.id` | **16 字节原始值**参与 HKDF info 与 AAD；base32 字符串仅用于磁盘文件名 / JSON 展示 / 记录 id |
| 对象 header | 固定 **32 B**：`magic(4)=b"MYOB"` \| `obj_version(1)=1` \| `object_id(16)` \| `chunk_size(4)` \| `chunk_count(4)` \| `cipher_id(1)=1` \| `reserved(2)=0`；`header_hash = SHA-256(header)`（整 32 B header）供 AAD，不另存 |
| chunk 布局 | `nonce(24)` \| `ciphertext_len(4)=len(ct)` \| `ct` \| `tag(16)`；密文 = AEAD 输出 `ct‖tag` 合并 |
| HKDF-Expand `info` | 固定 ASCII 标签（或标签 + 16 B 原始 id）：`KEK_password=HKDF-Expand(IKM_pw, b"wrap/password")`、`KEK_recovery=HKDF-Expand(IKM_rk, b"wrap/recovery")`、`vmac_key=HKDF-Expand(MAC_Key, b"meta-version-mac")`、`manifest_key=HKDF-Expand(Master, b"manifest"+manifest_id_raw)`、`object_key=HKDF-Expand(Master, b"object"+object_id_raw)`；`length=32` |
| chunk AAD | `b"MYVAULT" + u32(format) + jti.ascii + object_id_raw(16) + u32(chunk_index) + u32(chunk_count) + header_hash(32)` |
| manifest AAD | `b"MYVAULT" + u32(format) + jti.ascii + manifest_id_raw(16) + u32(manifest_seq)` |
| manifest 明文 | canonical JSON 记录数组（§3.3），**无外层包装对象**；外层即 `[{...},{...}]` |
| **恢复槽 KDF 输入（FINDING-5）** | **两个 secret 输入都走同一组 `Argon2id(m,t,p,salt)`**：主口令 = 完整 Unicode UTF-8 字节（保留 NULL）；Recovery Key = **其解码后的原始 32 B**。各自得 `IKM_pw` / `IKM_rk`，再经 `wrap/<slot>` 域分离得两个 KEK，包裹**同一** `Master∥MAC_Key`。v1.1 的单一 `kdf` 块对两槽共用，不做第二套参数 |

> **FINDING-5 说明**：v1.1 §3.4 派生图把 `slot ∈ {password, recovery}` 只挂在 HKDF 行，未逐字写「Recovery Key 也过 Argon2id 且用哪个 salt」。PoC 采用「同参数同 salt、两个 secret 各自 KDF」的直读，并经 [4b] 恢复槽端到端解密成功 + [4c] RK 喂错槽 `E_AUTH_FAILED` 验证。此约定现已冻结；若未来要改（如 RK 不过 KDF 直接作 KEK），必须升 `format` 并给新向量。

---

## 3. `vault.meta` 结构（冻结 · v1.1）

### 3.1 字段

采用 **canonical JSON**（§3.3），人可读、可手工核对、便于第三方实现。JSON 内**不得出现任何用户数据**。

```jsonc
{
  "magic":    "MYVAULT",              // 固定 7 字节标识
  "format":   1,                      // 格式版本；升级须走 §8 迁移流程；禁止静默降级
  "jti":      "<uuid-v4>",            // vault 实例 id；★ rotate-keys / fork 必须轮换
  "key_epoch": 0,                     // ★ u32，仅 rotate-keys 递增；写入 RK 编码与 versionMac
  "cipher":   "XCHACHA20-POLY1305",   // 唯一合法值（单一 cipher，无协商）
  "kdf": {
    "alg":    "argon2id",
    "m":      65536,                 // KiB；移动端保底 19456；★ 解锁前硬校验上下界（§3.5）
    "t":      3,                     // passes；移动端保底 2
    "p":      4,                     // lanes；移动端保底 1
    "salt":   "<16-byte random, base64>"   // 128-bit，每库唯一；★ rotate-keys 换新 salt
    // ★ 不存任何派生结果 / 口令哈希 / PHC <hash> —— D5
  },
  "wrapped": [
    { "slot": "password", "alg": "RFC3394-AES-KW", "blob": "<base64 72B>" },  // KEK_password 包裹 Master∥MAC_Key
    { "slot": "recovery", "alg": "RFC3394-AES-KW", "blob": "<base64 72B>" }   // KEK_recovery 独立包裹同一主密钥
  ],
  "manifest_ref": { "id": "<base32-id>", "seq": 1 },  // ★ 无 mac；seq 用于双向绑定
  "versionMac": "<HMAC-SHA256 over canonical(meta 全字段，排除本字段), base64>",  // 全头认证/防降级
  "gc_pending": false                 // ★ 崩溃恢复标记（F18）：GC 中断后下次启动续做
  // ★ v1.1 删除 created 字段 —— F20/N8
}
```

### 3.2 字段纪律（实现约束）

- `versionMac`：
  - 校验密钥 = `HKDF-Expand(MAC_Key, "meta-version-mac", 32)`（`MAC_Key` 来自解包）。
  - 覆盖 **`canonical(magic | format | jti | key_epoch | cipher | kdf | wrapped | manifest_ref | gc_pending)`**，即**除 versionMac 自身外全字段**。
  - **属解锁后校验**（先解包得到 MAC_Key）；改动 `format` / `key_epoch` / `jti` / `wrapped` / `kdf` 必然导致校验失败 → 防降级 + 全头绑定（**这是 AES-KW 无 AAD 的 256-bit 外层补偿，F13**）。
  - 常量时间比对；失败即 `E_DOWNGRADE`，绝不继续。
  - **它是全头认证标签，「防降级」只是效果之一**；不能检整盘回放旧三元组（R9）。
- 文件系统 mtime / 任何时间戳 **一律不参与完整性判断**；v1.1 已删 `created`。
- `wrapped` 可未来扩展第三个 slot（硬件密钥本机副本），但**硬件 slot 只能是本机便利副本，不得成为唯一解包途径**（口令兜底）。
- 主口令 / Recovery Key **永不落盘**；`wrapped.password` 与 `wrapped.recovery` 包裹**同一 Master∥MAC_Key**，但用**不同的 `KEK_slot`**（§3.4，F13 域分离）→ 两 blob 互换在解包时确定性地被 AES-KW 完整性拒绝。
- `vault.meta.bak` 与 `vault.meta` 结构相同、各自带自己的 versionMac；`verify` 主文件解析失败时尝试 bak 并提示（§6.4）。

### 3.3 canonical JSON profile（冻结，F12c-1）

`versionMac` 是**跨实现验证的硬依赖**——两端序列化差 1 字节 HMAC 即失败。所有实现必须严格遵守：

1. UTF-8 编码，无 BOM。
2. 对象键按 **Unicode 码点升序**排列（递归）。
3. 无冗余空白（无空格/换行/缩进）。
4. 整数用十进制，不用浮点、不用指数；字节串一律 base64 字符串。
5. 字符串转义集合**封闭**：仅 `\"`、`\\`，以及控制字符一律 `\u00XX`（小写 hex，含 `\n`/`\t`/`\r` 也必须用 `\u00XX`，不得用 `\n` 等短转义）；不转义非 ASCII。（措辞勘误 v1.1.1：「最短转义」与「控制字符一律 \u00XX」在 `\n` 上冲突，逐字节契约以枚举集合为准。）
6. 禁止重复键（解析即拒绝）。
7. 布尔/null/数组字面量固定小写。
8. 字段集固定，禁止未知字段（解析即拒绝）；缺字段视为 `E_FORMAT`。

> 依据 RFC 8949 §4.2 的启示：确定性编码「须由应用协议自行定义」，合法编码允许多种序列化。JSON 同理——只写「键排序」不够，必须冻结完整 profile。（RFC 8949 §1.1 亦明写合法 CBOR 允许多种编码，故采用 CBOR 同样要另写 profile，相对 JSON 无优势。）

### 3.4 密钥派生流程（冻结顺序 · v1.1）

```
passphrase (完整 Unicode, 保留 NULL 字节; 显式 NFC)
   ① 解析 meta 公开头 + ② KDF 参数硬校验 (F14: 19456≤m≤1048576, 2≤t≤10, 1≤p≤16)
    └─ Argon2id(m, t, p, salt, tag=32B, memory-wiping)   → IKM (32 B, 均匀)
         └─ KEK_slot = HKDF-Expand(IKM, "wrap/" ‖ slot, 32)   ★ F13 域分离
              └─ RFC 3394 AES-256-KW unwrap wrapped[slot].blob
                   ├─ 解包完整性失败 (2^-64 首道过滤) → E_AUTH_FAILED（统一，F16）；MUST NOT 返回任何密钥
                   └─ → Master (32 B) ∥ MAC_Key (32 B)          [CSPRNG 生成, 永不落盘明文]
                        ├─ HKDF-Expand(MAC_Key, "meta-version-mac", 32)      → versionMac 校验密钥
                        ├─ HKDF-Expand(Master,  "manifest" ‖ manifest.id, 32) → manifest 密钥
                        └─ HKDF-Expand(Master,  "object"   ‖ object_id,   32) → 每对象内容密钥
```

- **解锁顺序（不可颠倒）**：①解析 meta 公开头 → ②**KDF 参数硬校验**（越界 `E_FORMAT`，绝不分配内存）→ ③Argon2id 得 IKM → ④`KEK_slot` 域分离 → ⑤AES-KW 解包得 `Master∥MAC_Key`（首道 2^-64 过滤）→ ⑥派生 versionMac 校验密钥 → ⑦常量时间比对 versionMac（256-bit 全头绑定）→ ⑧不符 `E_DOWNGRADE` 且绝不继续 → ⑨派生 manifest 密钥解密清单。
- **预认证统一失败（F16）**：任一步预认证失败（口令错 / slot blob 篡改 / 任一 slot 解包失败）统一 `E_AUTH_FAILED` + 常量时间 + **遍历全部 slot 后统一失败**，绝不区分是哪个 slot、也绝不区分「口令错」与「blob 篡改」。
- **HKDF 只走 Expand**：Master/IKM 是 256-bit 均匀密钥，按 RFC 5869 §3.3 可跳 Extract；但**禁止把 PRK 直接当 OKM**（§3.3 "NOT RECOMMENDED"），必须经 `info` 绑定上下文（slot / manifest.id / object_id）。口令场景必须先 Argon2id（HKDF「cannot amplify entropy」）。

### 3.5 KDF 参数硬校验（F14 / N3，解锁前）

读 `meta` 后、跑 Argon2id **之前**校验，防止攻击者把 `m` 改成 4 TiB 造成 OOM DoS：

| 参数 | 下界 | 上界 | 越界 |
|---|---|---|---|
| `m` (KiB) | 19456（OWASP / RFC 9106 最低） | 1048576（1 GiB） | `E_FORMAT` |
| `t` (passes) | 2 | 10 | `E_FORMAT` |
| `p` (lanes) | 1 | 16 | `E_FORMAT` |

移动端达不到下限时**拒绝在该设备建库并明确提示**，不做静默降档（P3）。校验失败绝不进入 Argon2id，绝不分配 KDF 内存。

---

## 4. 密文对象格式（`d/<2-char>/<base32-id>`）

### 4.1 对象 id 与命名

- `object_id` = **128-bit CSPRNG** → **base32（RFC 4648，大写、无填充、26 字符）**。
  - 选 base32 而非 base64：`+ / =` 与大小写折叠在 FAT / exFAT / 默认 APFS 上会冲突；base32 大小写不敏感安全、全为合法文件名字符、26 字符远低于 255 字节上限。
- 分片目录 = id 前 2 字符 → 最多 1024 子目录，规避单目录文件数上限。
- **copy-on-write 不变量**：每次写入分配**全新 object_id**，永不复用。这是「确定性 nonce 备选」成立的前提（本规范主线用随机 nonce，不依赖此不变量做 nonce 安全，但复用 id 仍会破坏冲突检测与回滚语义）。

### 4.2 对象二进制布局

```
┌────────────────────────────────────────────────────────────┐
│ header (固定长度, 明文但不含用户数据)                       │
│   magic(4) | obj_version(1) | object_id(16) | chunk_size(4)│
│   chunk_count(4) | cipher_id(1) | reserved(n)               │
│   ★ header_hash = SHA-256(header) 供 AAD 绑定（不另存）    │
├────────────────────────────────────────────────────────────┤
│ chunk[0] : nonce(24) | ciphertext_len(4) | ciphertext | tag(16) │
│ chunk[1] : ...                                             │
│ ...   ★ chunk_count ≥ 1，空明文也发一个空 chunk (F15)      │
└────────────────────────────────────────────────────────────┘
```

- **分块 1 MiB**（P4）：①规避 FAT32 单文件 4GB 上限；②增量同步只传变化的 chunk；③大文件随机读无需整文件解密。
- **空对象强制一个 chunk（F15 / N4）**：`chunk_count ≥ 1` 是格式不变量；0 字节明文也写一个 `plaintext_len=0` 的 chunk（仍带 24B nonce + 16B tag + 含 `header_hash` 的 AAD）→ 明文 header 不再是「零 tag 可任意伪造」。
- **单一正确做法（F9）**：**一个对象一个密钥** `HKDF-Expand(Master, "object"‖object_id, 32)` + **每 chunk 一个 CSPRNG 24B 随机 nonce**；XChaCha20-Poly1305；**不**派生 per-chunk 密钥、**不**对 chunk 用 SIV、**不**做 nonce 计数器。
  - 依据 draft-xchacha-03："users can safely generate a random 192-bit nonce for each message"、"random 192-bit nonces should experience a single collision (with probability 50%) after roughly 2^96 messages"。
  - **拒绝全确定式 nonce**（对抗审阅【9】裁定）：G4「逐字节相同」只约束固定测试向量，正常运行用随机 nonce；把 nonce 安全性全押在「object_id 永不复用」比 CSPRNG 192-bit 更脆弱（RFC 8439 明写 nonce 复用 "reveals the XOR of the plaintexts"）。
- **chunk AAD** = `magic ‖ format ‖ jti ‖ object_id ‖ chunk_index ‖ chunk_count ‖ header_hash` → chunk 顺序、归属、数量、header 都被认证，防重排/截断/跨对象拼接；**`jti` 绑定使 `rotate-keys`/`fork` 轮换 jti 后旧主密钥无法解开新数据**。
- 密文长度 = 明文长度 + 16 B tag/每 chunk；**不做长度填充时文件大小略泄漏**（§5.4）。
- 对象内**不含文件名、不含路径、不含时间戳**——这些都在 `manifest` 的加密记录里。

---

## 5. 加密范围

### 5.1 内容
每对象用 `HKDF-Expand(Master, "object"‖object_id, 32)` 派生子密钥 → XChaCha20-Poly1305。**单对象密钥泄露不扩散到其他对象**（G8 收益，修 LocnLock L6）。

### 5.2 目录树 / 文件名 / 元数据
用户目录树、文件名、类型、大小、排序、标签等，全部序列化为 `manifest` 内的**加密记录**（§5.3），**不落到磁盘路径上**。解锁定后由 App 在**内存**中重建目录树，**不建持久索引**。

### 5.3 `manifest`（加密 + 认证的对象清单）

```
manifest_key = HKDF-Expand(Master, "manifest" ‖ manifest.id, 32)
manifest = XChaCha20-Poly1305(
             key = manifest_key,
             nonce = CSPRNG 24 B,   // manifest 自身也是一个 AEAD 密文
             plaintext = canonical JSON [
               { id, parent_id, name, kind, size, chunk_count,
                 base_version, device_id, updated_seq, content_version,
                 manifest_seq }, ...
             ],
             AAD = magic ‖ format ‖ jti ‖ manifest.id ‖ manifest_seq   // ★ 稳定，不绑 versionMac（D2）
           )
```

- **名称 = 普通加密字段（D3）**：`name` 直接是 AEAD 加密 manifest 里的一条记录，**不再做 SIV 确定性加密**。理由：①RFC 5297 证明 SIV 是 AES 模式，不存在「XChaCha20-SIV」，引入它会变成第二套 cipher 原语；②manifest 本就整体 AEAD 加密，「按名查找无需全表解密」的收益不成立。同名冲突由用户在解锁态处理。
- `parent_id` 用**稳定随机 `object_id`**（非路径派生）→ 目录**重命名 / 移动无需重加密子树**（消解 P9 顾虑）。
- **无 checksum 字段（D4）**：完整性由每 chunk Poly1305 tag 承担；解锁前结构检查靠对象 header 的 `chunk_count / ciphertext_len` 对比实际文件长度。**拒绝明文 BLAKE3 catalog**（对抗审阅【4】裁定：只检随机损坏、解密前读不到、且主动暴露精确大小与 id 列表）。
- **并发字段（F18 / N6）**：
  - `device_id` = 每安装 16B CSPRNG，**设备自身当前身份只存设备本地状态，绝不从容器读取、也绝不把「当前设备身份」写成容器里的全局字段**；每条 manifest 记录在写入瞬间把该本地 id 作为**溯源标签**写进加密记录（`writer` 语义，见 §6.3）。→ 整盘复制后两台设备本地 id 不同，各自新写入的记录带不同标签，不再自认同一写者。
  - `base_version` = 本次写入所基于的 `manifest_seq`；并发判定 = 「`base_version` 相同且 `(device_id, updated_seq)` 不同」→ 两份双留，**绝不 LWW**。
  - `content_version`（每对象 Lamport 时钟）/ `updated_seq` 供冲突检测（§6.3）。
  - v0.2+ 可选 `content_hash = SHA-256(明文)` 做同内容自动去重（**用已标准化的 SHA-256，不用 BLAKE3**）。
  - **★ `device_id` 术语消歧（v1.1.1 勘误，由 M1 PoC 实测暴露的规格自相矛盾）**：本名有两个语义，混用会让实现者「既不能写容器又必须写记录」——①**设备当前身份** = 每安装 16B CSPRNG，只存设备本地状态，**绝不从容器读取**、整盘复制不携带它；②**写入者溯源标签** = 每条 manifest 记录在写入瞬间读该本地值、作为一条**加密字段**写进记录本身（即上面 plaintext 里的 `device_id`），供解锁态并发比对。N6 / §1 的「绝不写入容器 / 设备本地状态不计入容器」原话只针对①；②是加密记录内容、非设备状态，二者不冲突。v1.1 原文把①笼统写成「绝不写入容器」却又把②列进记录字段，属措辞矛盾；v1.1.1 按两义拆开定案，§9 固定输入向量里的「固定 `device_id`」指②的测试快照值。
- `manifest_seq` 递增；解锁时断言 `manifest.seq == vault.meta.manifest_ref.seq`（前者由 AEAD tag 认证、后者由 versionMac 认证）→ **meta ↔ manifest 双向认证绑定**（D2）。
- `manifest_ref = { id, seq }`，**无 mac**；`manifest.id` 建库时一次 CSPRNG **全库固定**，每写只递增 `seq` 并换 manifest 的 AEAD nonce/tag（消解对抗审阅【12⑤】的「固定文件名却每次换 id」矛盾）。
- `manifest` 是唯一随每次写入重写的对象（G8 代价，文件很小，可接受）。**单体 manifest 的残留风险 R13**（1 bit 翻转使清单整体不可读、每写 O(n)）由 `vault.meta.bak` + `export --object` 兜底；per-record AEAD 日志列为 v0.2+ 扩容路径。

### 5.4 大小 / 数量的形状泄漏（诚实边界）

| 泄漏 | 现状 | 缓解（P6：MVP 不做） |
|---|---|---|
| 对象数量、大致总大小 | 可见（目录里有多少文件） | 可选：定长填充对象 / 定期 compaction 合并小对象 |
| 单文件精确大小 | 密文长度 ≈ 明文长度 + 每 chunk 16 B tag | 可选：按 1 MiB 粒度填充到 chunk 边界 |
| 「有同步/写入行为」 | 开启 BYO 同步时观察者可知 | 不可消除，写进安全边界 |

> **对外口径冻结（与 threat-model.md §5、crypto-review-m1.md §8 一致）**：「**用户数据零明文**」——磁盘上看不到内容、文件名、目录名、目录结构、标签；`vault.meta` 字段名、`MYVAULT`、`d/xx/` 分片、README 是格式自描述的**公开元数据**。「**形状泄漏被显著压缩，但未 100% 消除**」——仍可能推断对象数量与大致大小。**同时承认可见**：`kdf` + `wrapped` 即完整离线爆破输入、存在 Recovery slot 是公开事实、`jti` 跨介质可关联。**禁止**写成「零元数据」「连形状都不泄漏」。

---

## 6. 完整性与恢复

### 6.1 `verify` 两阶段模型（F7 + F16/N5：防 oracle）

**阶段 1 · 解锁前（无口令，`vault verify <root> [--json]`）** —— 只做**非密码学结构检查**：
- 解析 `magic/format/jti/key_epoch/cipher/kdf` 公开头（**含 §3.5 KDF 参数硬校验**）；遍历 `d/` 命名与 2 层分片；用每对象**明文 header** 的 `chunk_count / ciphertext_len` 对比实际文件长度判截断（含 `chunk_count=0` 违规）；统计对象数与总字节。
- **全量扫描、不早停、定长输出**；输出收敛为 `{ "phase":"pre-unlock", "result":"OK" | "STRUCT_ANOMALY", "objects":N, "bytes":M }`。N/M 列目录本就可得，不构成新泄漏；耗时不随「首个失败位置」变化。
- **禁止**在此阶段区分/报告具体对象错误、禁止 id 枚举；`vault info` 不列对象 id；`export --object` 在预认证阶段不区分「id 不存在」与「口令错」。
- **任何预认证失败统一 `E_AUTH_FAILED`** + 常量时间比较 + 遍历全部 slot 后统一失败（F16）。

**阶段 2 · 解锁后（持口令/Recovery Key，`vault verify <root> --password [--json]`）** —— 完整密码学验证，细粒度错误码：

| 码 | 含义 | 检查 |
|---|---|---|
| `E_FORMAT` | 非本格式 / 版本未知 / **KDF 参数越界** / canonical JSON profile 违规 | 公开头 + §3.5 + §3.3 |
| `E_AUTH_FAILED` | **所有 slot 解包失败（统一，不区分哪个 slot、不区分口令错与 blob 篡改）** | F16 |
| `E_TAMPER` | 授权态下解包/标签异常的细分 | RFC 3394 unwrap |
| `E_DOWNGRADE` | versionMac 不符 / `format` 或 `key_epoch` 被降级 | HMAC-SHA256 常量时间比对 |
| `E_MANIFEST` | manifest AEAD tag 失效或 `manifest.seq != manifest_ref.seq` | XChaCha20-Poly1305 |
| `E_MISSING` | manifest 引用的对象不存在（只拷子集 / 半途断电） | 清单比对 |
| `E_CORRUPT` | 任一 chunk 的 AEAD tag / AAD 不符（含重排、跨对象拼接、截断、空对象伪造） | 逐 chunk |
| `E_ORPHAN` | `d/` 中存在未被清单引用的对象（**警告，非致命**） | 目录比对 |
| `E_CONFLICT` | 同 `base_version` 下两份 `(device_id, updated_seq)`（需解锁态人工决定） | 清单比对 |

> 设计要点：拿到一块十年没插过的移动硬盘，阶段 1 先知道「结构是否还完损」，再决定要不要输口令跑阶段 2。**完整性认证只在解锁后**，因为认证的密钥来自解锁。
> **`verify` 绿不能证明是最新版**（R9）：无服务端/可信单调计数器，整盘回放旧 `(meta,manifest,d/)` 三元组、旧版本覆盖在密码学上不可检出；解锁前 `verify` **只检随机损坏/不完整，不检恶意篡改**。降级/篡改/冲突/陈旧判定一律解锁后 —— 除非显式开启「库外本地锚点」（§6.3），且锚点丢失即失效。

### 6.2 独立恢复 CLI（M1 必备，兑现「不锁定」）

```
vault info     <root>                                    # 打印 format/cipher/kdf/key_epoch，不解密、不列对象 id
vault verify   <root> [--json]                          # 阶段1 结构检查（全量扫描、收敛输出）
vault verify   <root> --password [--json]               # 阶段2 完整密码学验证（细粒度错误码）
vault export   <root> --out <dir> [--password|--recovery]        # 导出全部明文 + 原始目录树
vault export   <root> --object <base32-id> --out <file> [--recovery]  # 单对象兜底取回
vault change-password  <root>                          # 重包裹 password slot（★零吊销，UI 明说）
vault rotate-recovery  <root>                          # 新 RK 重包裹，旧 RK 失效（★仍零吊销旧口令持有者）
vault rotate-keys      <root>                          # ★真吊销：新 Master/MAC/salt/jti/key_epoch + 全量重加密
vault fork     <root> <new-root>                       # v0.2+：克隆并轮换 jti（等价 rotate-keys 作用于副本）
vault gc       <root> --confirm                        # 回收孤儿 / 已解决冲突副本（水位触发）
vault mount    <root> <mountpoint>                     # 可选；唯一依赖 OS 虚拟文件系统的命令；★拒绝同步根内挂载
```

- CLI **与核心 crate 一起开源**、可复现构建、签名发布，**不依赖主 App、不依赖账号、不联网**。
- `export` 输出**原始明文文件 + 原始目录树**，不是专有格式 → 用户取回后可自由迁移。
- `export --object` 兜底：manifest 损坏时仍能凭 id 取回内容（名称可能丢失，须在文档说明）；预认证阶段统一 `E_AUTH_FAILED`。
- `mount` 是**唯一**依赖 OS 的命令，其余全为纯文件操作 → 再次印证「载体无关」卖点；命中已知同步目录则**拒绝挂载**（N9）。
- P7（age 互操作桥）**非 M1**，v0.2+。

### 6.3 冲突处理（绝不静默覆盖，F6 + F18 收敛策略）

- 并发判定 = 「两条记录 `base_version` 相同且 `(device_id, updated_seq)` 不同」→ **两份密文对象都保留**，manifest 保留两条记录并标 `E_CONFLICT`，解密后的内存视图中文件名后缀带 `device_id + updated_seq`（后缀不落磁盘路径）。**绝不 LWW、绝不静默覆盖**。这里的 `device_id` 指每条记录里那条**写入者溯源标签快照**（§5.3，随记录加密存入 manifest，比对时从解密后的记录读到）。
- 设备的**当前** `device_id` 来自**设备本地状态**（整盘复制后两台设备各自不同），写新记录时把它作为快照写进该记录；**绝不从容器读回一个全局「当前设备身份」**（N6 消歧后的准确语义）。
- **每对象冲突副本上限 N=2**；超出时拒绝新写入并强制先解决。解锁态强制引导用户处理，解决后从 manifest 删除对应记录；显式 `vault gc --confirm` 回收孤儿与已解决副本。GC 触发水位：孤儿数 > 活跃对象 20%，或每 N 次写入。`vault.meta.gc_pending` 作崩溃恢复标记（不新增隐藏文件）。**绝不无限保留**。
- **可选回滚检测增强（F18）**：设备本地保存 last-seen `head/seq` 锚点，可检出「当前 manifest 比上次见过的更旧」。**锚点丢失即失效、绝不写入容器** → 不破坏 §6.8 载体无关；属显式开启功能，非默认。
- 沿用 LocnLock「还原前校验 + 保留当前 vault 副本 + 失败自动回滚」的谨慎基因。

### 6.4 `vault.meta` 原子写入（F12b）

- 每次改写：`写临时文件 → fsync → 原子 rename 覆盖 vault.meta → 保留上一份为 vault.meta.bak（同样带自己的 versionMac）`。
- `verify` 阶段 1 主文件解析失败时，尝试 `vault.meta.bak` 并提示「检测到半写入，已从 bak 恢复」。应对 U 盘热插拔 / 断电。
- **诚实残留 R9**：攻击者若能整盘写（E5），可整体回放一组旧的合法 `(meta, manifest, d/)` 三元组实现状态回滚。纯本地、无账号、无外部状态的密库无法在盘内根除回滚（Cryptomator 快照回滚同此限制）。登记为已知限制，**不宣称解决**；仅库外本地锚点（§6.3）可部分检出，且锚点丢失即失效。

### 6.5 时序 / 侧信道（F11 + F19/N7）

- Argon2id 启用 memory-wiping（RFC 9106 §9）；口令入口即 `SecretVec`/`Zeroizing`（**不用 Rust `String`**）；覆盖 KEK / Master / MAC_Key / manifest 明文 / 每 chunk 缓冲（随机读缓冲带**硬上限 LRU**，逐出即清零）；`mlock` 锁定密钥页、按平台禁核心转储。
- versionMac 常量时间比对；解锁前结构检查全量不早停（F16）。
- **明确不受控**：swap / 休眠 / 冷启动 / 剪贴板历史与云剪贴板 / 他进程已读内容、明文导出后的生命周期 —— 不可撤销，写进对外声明。
- 挂载期间明文「第二份副本」被 OS 索引/缩略图/云同步/VSS/Time Machine/AV-EDR 采集的泄漏面，见 [threat-model.md](threat-model.md) §3.1；**仅未解锁态防得住 / 解锁态仅缓解**，**不承诺 100% 阻止**，绝对隔离改用「单文件解密导出」而非透明挂载。

---

## 7. 写入 / 解锁 / 轮换流程（格式层视角 · v1.1）

| 操作 | 格式层动作 |
|---|---|
| 新建 vault | 生成 Master∥MAC_Key + Recovery Key（带 `key_epoch=0`）+ `jti` + `salt` → 写 `vault.meta`（含 versionMac）→ 写空 `manifest` → 建 `d/` → **强制引导离线保存 Recovery Key + 跑一次 `verify` + `export` 回读校验** |
| 写文件 | 分配**全新** object_id → 分块 AEAD 写 `d/xx/<id>`（空文件也发 1 chunk）→ 更新 `manifest`（加密记录，带 `base_version`/本地 `device_id`，manifest_seq++）→ 更新 `vault.meta.manifest_ref` + 重算 versionMac。**全程无明文临时文件** |
| 解锁 | 按 §3.4 冻结顺序：解析头 → **KDF 参数硬校验** → Argon2id → `KEK_slot` 域分离 → AES-KW 解包 → 验 versionMac → 解密 manifest → 内存重建目录树 → 启动空闲计时 |
| 锁定 | `Zeroizing` 清空主密钥、manifest 明文、所有缓冲 → 清剪贴板 → 磁盘状态回到「只有密文」 |
| **change-password** | **仅重写 `wrapped[password].blob` + 重算 versionMac；不重加密。★ 零吊销**（UI 明说：「不会使旧 Recovery Key 失效，**也不会撤销任何持有旧 `vault.meta` 副本的人**」） |
| **rotate-recovery** | 生成**新 Recovery Key** → 重包裹 `wrapped[recovery].blob` → 重算 versionMac；旧 RK 失效，**但对持有旧 `vault.meta`+旧口令者仍零吊销**。不重加密数据 |
| **rotate-keys**（★真吊销，v1.1 一等命令） | 生成**全新 `Master∥MAC_Key` + 新 salt + 新 `jti` + `key_epoch++` + 新 RK** → 全量重加密所有对象与 manifest（旧主密钥解不开新 jti 绑定的 AAD）→ `verify` 双绿 → 用户确认后 `gc` 旧对象/旧 bak → **必须物理销毁所有旧介质副本**。旧 epoch 的 RK 报「已轮换」 |
| **fork**（v0.2+） | 把整库克隆到第二介质时**轮换 `jti`**（等价对副本执行一次 rotate-keys）→ 消除 jti 跨介质可关联 / 克隆血缘不可分；代价同为全量重加密 |
| 换设备 / 拷到移动硬盘 | 拷整个 `<vault-root>/` → `vault verify`（阶段1）→ 用同一主口令或 Recovery Key 解锁。硬件 slot 只影响本机生物识别，**不参与跨端**；`device_id` 从新设备本地生成 |
| 升级 KDF 参数 | 重新派生 KEK（经 §3.5 校验）→ 重包裹 `wrapped[]` → 重算 versionMac；**不重加密数据**（§8） |

> **零吊销是架构事实**（N2）：在「包裹主密钥」架构下，数据加密密钥不变则**任何**凭据轮换都不构成吊销。这是 §8 第 5 条与 R12 的对外措辞来源，不是实现缺陷。

---

## 8. 版本与迁移

- `format` 单调递增；`versionMac` 绑定 `format` 与 `key_epoch`，**禁止静默降级**。恢复 CLI 另用**本地策略钉死最低支持 `format`**（CLI 版本策略，不依赖盘内字段）。
- 升级流程：旧版可读 → `export` → 新建新版 vault → 导入 → `verify` 双绿 → 用户确认后删旧版。**绝不原地改写不兼容格式**。
- 恢复 CLI **必须永久支持读取所有历史 `format`**（「十年后还取得出来」的硬要求）；新版 CLI 可写新格式，但读旧格式的能力不可撤销。
- 每次格式变更须附**测试向量**（§9）与迁移说明，作为开源规范一部分发布。
- FIPS 需求（如 AES-256-GCM）走 **`format=2` 新版本**（含独立测试向量与迁移路径），**不在同一库内做 cipher 协商**（D6）。**AES-SIV（RFC 5297，drop-in 替换 RFC3394、支持 AAD、128-bit 认证）列为 `format=2` 候选**：若未来第三方审计要求 ≥128-bit 包裹完整性再评估，v1.1 不引入第二套 cipher。

---

## 9. 测试向量与一致性要求（G4）

> **★ 唯一逐字节契约（v1.1.1 / M1 PoC 实测产出）**：[poc/test-vectors.json](poc/test-vectors.json)（由 [poc/mylock_poc.py](poc/mylock_poc.py) 用被审计过的库产出，每个原语先经 [poc/verify_primitives.py](poc/verify_primitives.py) 权威 KAT 26/26 校验）。该文件含 `_meta.byte_conventions`（§2.1 冻结约定的机器可读版）、`fixed_inputs`（固定 passphrase 含非 ASCII + NULL、固定 16 B salt、Master/MAC_Key/RK hex、固定 `jti`、`key_epoch=0`、`object_id`、`manifest.id`、`device_id`、固定 24 B nonce×3、`base_version=0`、37 B 明文 + **0 B 空对象明文**、`manifest_seq=1`）、`derived_outputs`（IKM/KEK/wrapped.blob/versionMac/object_key/对象文件 hex/manifest hex/AAD hex/meta canonical hex/RK display 等 30+ 字段）、`verify_phase1_json`、`verify_phase2`、`export`、`negative_vectors`（13/13）、`summary`。**五端 + 恢复 CLI 的实现必须逐字节复现 `derived_outputs`**；[poc-report-m1.md](poc-report-m1.md) 记录运行方法与结论。
>
> **M1 PoC 实测结论**：主链路（建库 → 整盘复制到「移动硬盘」副本 → 不输口令的 phase-1 结构校验 → 口令解锁 → phase-2 细粒度 verify → 明文 export）**ALL PASS**；**恢复槽端到端**用 RK 原始 32 B 解出同样明文（F13 域分离被真正跑通），RK 喂 password 槽确定性 `E_AUTH_FAILED`；**13/13 负向向量**期望码全部命中（含 KDF 越界 `m=4 TiB` 在 Argon2id 之前就 `E_FORMAT`、空对象 `chunk_count=0` 伪造 `E_CORRUPT`、slot 互换 `E_AUTH_FAILED`、1 bit meta/chunk/manifest 分别 `E_DOWNGRADE`/`E_CORRUPT`/`E_MANIFEST`）；orphan 对象为**警告非致命**（`NO_ERROR`）。**这是内部一次性验证脚手架，不是产品代码、不是第三方审计**；上线前硬门槛 5（第三方审计）仍必须做。

**固定输入向量（M1 必须发布、五端 + 恢复 CLI 逐字节一致）**：固定 passphrase（含非 ASCII）、固定 16 B salt、固定 Master/MAC_Key(hex)、固定 Recovery Key、固定 `jti`、固定 `key_epoch=0`、固定 `object_id`、**固定每 chunk 24 B nonce**、固定 `base_version=0`、固定 `device_id`、固定明文（**含 0 字节空对象向量**）、固定 `manifest_seq=1`、固定 `manifest.id`。

**固定输出**：`IKM(hex)`、`KEK_password(hex)`、`KEK_recovery(hex)`、`wrapped.password.blob(base64)`、`wrapped.recovery.blob(base64)`、`versionMac(base64)`、对象密文(hex)（含空对象）、manifest 密文(hex)、`verify --json` 输出。

> **必须写进规范**：该向量**固定了 nonce 才逐字节一致**；正常运行使用随机 nonce，**重复加密同一文件不会得到相同密文**（这是安全属性，不是缺陷）。G4 只约束「固定输入的测试向量」，不要求运行时确定性密文（对抗审阅【9】裁定为误读）。

**负向向量**（对应 §6.1 错误码，稳定命中、永不返回未认证明文）：

| 输入扰动 | 期望码 |
|---|---|
| 改 1 bit meta（versionMac 覆盖字段） | `E_DOWNGRADE` |
| 降级 `format` / 改 `key_epoch` | `E_DOWNGRADE` |
| 改 1 bit `wrapped.blob`（AES-KW 解包失败） | `E_AUTH_FAILED`（预认证统一）/ 授权态 `E_TAMPER` |
| password↔recovery slot 互换 / 跨库 blob 移植 | `E_AUTH_FAILED`（F13 域分离确定性检出） |
| **KDF 参数越界**（m=4 TiB / t=0 / p=99） | `E_FORMAT`（F14，绝不先跑 Argon2id） |
| 改 1 bit 对象 chunk | `E_CORRUPT` |
| 删 1 个对象 | `E_MISSING` |
| 截断 chunk / 制造孤儿对象 | `E_CORRUPT` / `E_ORPHAN` |
| **空对象 `chunk_count=0` 伪造** | `E_CORRUPT` / `E_FORMAT`（F15） |
| 空 manifest / 1 bit 翻转清单 | `E_MANIFEST`（内容仍可由 `export --object` 取回，R13） |

**模糊测试**：随机截断、随机字节翻转、随机 chunk 重排 → **永不返回未认证的明文**，只返回错误。

---

## 10. P1–P10 定案结果（v1.1 评审闭环）

| # | 待定项 | **v1.1 定案** | 评审条目 |
|---|---|---|---|
| P1 | 主密钥包裹机制 | **RFC 3394 AES-256-KW**（唯一，NIST SP 800-38F 批准）+ 256-bit versionMac 全头绑定兜底 + `key_epoch` + **F13 slot 域分离**；AES 仅用于包裹 | F3/F13/N1 |
| P2 | 两 slot 是否同一主密钥 | **同一 Master∥MAC_Key 的两个独立、域分离包裹**（`"wrap/password"` / `"wrap/recovery"`） | F13 |
| P3 | Argon2id 参数 / 低配降档 | `m=65536, t=3, p=4`；移动端保底 `m≥19456, t=2, p=1`；**解锁前硬校验上下界**（§3.5）；不做静默降档 | A1/A9/F14 |
| P4 | chunk 大小 | **1 MiB**；`chunk_count ≥ 1`（空对象也发 1 chunk） | F15 |
| P5 | chunk nonce 机制 | **每 chunk CSPRNG 24B 随机 nonce + 单对象密钥**；取消 SIV / per-chunk 密钥 / 确定性 nonce | F9/A7 |
| P6 | 大小/数量填充 | **MVP 不做**，对外声明未 100% 消除 | F8 |
| P7 | age 互操作桥 | **非 M1**，v0.2+ | 范围控制 |
| P8 | manifest 序列化 | **canonical JSON 全链路统一 + §3.3 冻结 8 条 profile**，不采用 CBOR；G4 仅约束固定测试向量 | F12c-1 |
| P9 | 名称加密 / parent_id 绑定 | **删除名称 SIV**；名称 = AEAD 加密 manifest 普通字段；`parent_id` 用稳定随机 object_id，重命名/移动无需重加密子树 | D3 |
| P10 | Recovery Key 编码 | **base32(52) + 4 校验位 + `key_epoch` 前缀**，13×4 分组；不引入 BIP39；`rotate-recovery` 使旧 RK 失效、`rotate-keys` 轮换 epoch | F12c/F17 |

> v1.1 追加定案（归入 F13–F20，不新增编号）：`key_epoch` (F17)、`base_version` / 设备本地 `device_id` (F18)、`gc_pending` (F18)、`content_hash` 用 SHA-256 且 v0.2+ (F18)、删 `created` (F20)、KDF 硬校验 (F14)、空 chunk (F15)、`E_AUTH_FAILED` 统一 (F16)、`rotate-keys` 一等命令 (F17)、内存具体化 (F19)、同步根拒绝挂载 (F20)。

---

## 11. 与既有调研的对应关系（避免重复造轮子）

| 本规范字段 | 借鉴来源（已核实） | 我们改了什么 |
|---|---|---|
| 主密钥包裹层、versionMac 全头认证 | Cryptomator `masterkey.cryptomator`（A 级） | 更简洁 JSON + 显式 HMAC，避开 JWT 算法协商面；versionMac 解锁后校验并覆盖全头（补偿 AES-KW 无 AAD） |
| 每文件独立密钥 + 文件名/目录加密 | Cryptomator `cipherCombo=SIV_GCM` | **去掉 SIV**（避免第二套 cipher）；内容随机 nonce + 分块 |
| KDF 参数可升级 work factor | OWASP Password Storage Cheat Sheet（A 级） | 参数写 meta 并绑 versionMac；**不落盘派生结果**；解锁前硬校验防 OOM |
| 头认证 + Encrypt-then-MAC | KeePass KDBX4 `HmacBlockStream`（A⁻） | 每 chunk AAD 认证取代整库块认证 |
| 分层密钥 / recoveryKey / 凭据轮换 | Ente 架构（A⁻） | 去掉共享能力，保留分层与恢复；**补 `rotate-keys` 真吊销** |
| 标准件选型与完整性边界 | **NIST SP 800-38F**（A/A⁻） | 明确「批准」地位来自 SP 800-38F；用 versionMac 补其 §B.2 / §A.3 指出的缺口 |
| 192-bit 随机 nonce 合法性 | draft-irtf-cfrg-xchacha-03（expired，A 级） | 明确标注非 IETF 标准，不拿 RFC 8439 背书 |
| 极简可组合文件加密 / 公钥收件人 | age / rage（A⁻） | 仅列为 v0.2+ 互操作桥（P7） |

> 诚实声明：**本规范的密码学结构基本是成熟范式的组合，不是创新**（与主文档 §4.5 口径一致）。创新在 §6.8 的「载体无关 + 主动防合法采集者 + 双形态同钥」产品层，而非格式层。格式层刻意保守，是为了可审计、可独立恢复。

---

## 12. 上线前的硬门槛（不得跳过）

1. §9 正负向测试向量实测通过、五端 + 恢复 CLI 逐字节一致（含空对象向量、KDF 越界向量、slot 互换向量）。
2. `threat-model.md` §6 密码学假设 K1–K10 状态列按本规范回填（v1.1 已回填）。
3. 主文档 §6.3 / §6.6 / §7.1 / §8.2 / §9.3 同步刷新版本号与参数总表（v1.2 已刷新，M2 后升 v1.3，M3 后升 v1.4，M4 第 1 点后升 v1.5，M4 第 2 点后升 v1.6，M4 第 3 点后升 v1.7，M4 第 4 点后升 v1.8，M5 后升 v1.9）。
4. `crypto-review-m1.md` §11 对抗性审阅 16+ 项裁定全部有记录、被接受的项已落进本规范。
5. **委托第三方密码学审计** —— 本规范与 `crypto-review-m1.md` 均为内部设计评审，**不构成安全背书**；`verify` 绿 ≠ 未被恶意替换。

> 跨阶段边界（更新 2026-11-14）：用户 m00241 先选「先做 M1 密码学评审」、m00606 授权 M1 PoC；**随后用户 m00983「进入 M2 内核实现」已正式授权进入开发阶段，M2 内核实现完成**（Rust `mylock-core` + `mylock` 独立恢复 CLI，逐字节复现 `test-vectors.json`，见 [m2-report.md](m2-report.md)）。但门槛 ①「五端 + 恢复 CLI 逐字节一致」仍只完成 Rust 单端、门槛 ⑤ 第三方审计仍未做；**内核实现 ≠ 产品发布 ≠ 五端验收 ≠ 安全认证**。
> 维护约定：本文与 `crypto-review-m1.md` v1.1、`threat-model.md` v1.1、`产品分析与概要设计.md` v1.9 四处版本号与参数总表必须一致；任何一处改算法，四处同改。
