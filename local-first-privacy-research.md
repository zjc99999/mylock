# 本地优先 / 离线优先个人隐私与加密存储 · 竞品对比（2026-11）

> 方法：`web_fetch` 抓官网/官方文档 + `api.github.com/repos` 核实协议/平台/星数/最新 release。GitHub HTML 与 raw 域名在本环境被跨域拦截，故元数据统一走 `https://api.github.com/repos/<owner>/<repo>`（及 `/releases/latest`）。★=stars、版本号均为 2026-11-13 抓取时点值。凡抓取失败或页面截断者标「未核实」，不补常识、不编造。
> 缩写：**协议** G2=GPLv2+ · G3=GPL-3.0 · A3=AGPL-3.0 · M=MIT · B3=BSD-3-Clause · Ap2=Apache-2.0 · MP2=MPL-2.0 · 闭=闭源商业 · ?=GitHub 标 NOASSERTION/未核实。**平台** W/M/L/A/i=Windows/macOS/Linux/Android/iOS。**模式** 免=免费开源 · 捐=捐赠 · 订=订阅 · 买=买断 · 自=自托管。

## 一、结论先行
- **完全离线加密保险库已有成熟好用的开源方案**：`KeePass`/`KeePassXC`（单文件 `.kdbx`，GPL）与 `pass`（GPG+git，G2）是事实标准，桌面端审计充分（KeePassXC 2.7.9 获法国 ANSSI CSPN 一级认证）。但成熟≠无痛点，移动端与「无云多端一致性」仍空。
- 两大范式：**单文件密库 + 外部搬运**（KeePass/Cryptomator/age/VeraCrypt）vs **E2EE + 服务器中继**（Bitwarden/Standard Notes/Ente）。前者最贴合「无云」，后者用账号换多端体验。
- 密码学趋同：对称 `AES-256` / `XChaCha20-Poly1305`；KDF 由 `PBKDF2` 转向 `Argon2(id/d)`/`scrypt`；认证普遍 `Encrypt-then-MAC`/AEAD。

## 二、密码 / 机密保险库

| 项目 | 协议·★ | 平台 | 加密 / KDF | 联网·账号 | 模式 | 卖点 / 可直接借鉴 |
|---|---|---|---|---|---|---|
| KeePass 2.x | G2 [1][2] | W（M/L 经 Mono） | 库加密 AES-256/ChaCha20/Twofish；KDF 可选 AES-KDF 或 **Argon2d**；KDBX4 强制 HMAC-SHA-256 头认证 + Encrypt-then-MAC（1MB 块 HmacBlockStream）[3] | 完全离线·无账号 | 免·捐 | 单文件 `.kdbx` 即全部数据；主密码+密钥文件+Win 用户账户三重解锁、插件（passkeys/浏览器填充）、进程内存保护 [4]。借鉴：**单文件自包含 + 可换 KDF/套件 + 插件生态** |
| KeePassXC **2.8.0** | G3（官网；GitHub ?）·★29,959 | W/M/L | 兼容 KDBX4，同上 | 完全离线·无云/无广告 | 免·捐 | 社区 C++ 移植。2.8.0（2026-10-21）：Qt5→Qt6、外部工具远程库同步、Wayland Auto-Type、Win ARM64；修 CVE-2026-69150 / CVE-2026-118673。借鉴：**把云同步降级为可选外部工具**、生物识别快速解锁 |
| Bitwarden | GitHub ? [5]·server★20,626 / clients★14,219 | W/M/L/A/i+Web/CLI/扩展 | **未核实**（加密白皮书/`/help` 页截断） | 需账号·可自托管 | 免+订 | 全平台 + 家族/企业共享。借鉴：开源客户端 + 可替换服务端 |
| Vaultwarden **1.37.5** | A3·Rust·★70,651 [6] | 服务端（配任意 BW 客户端） | 同 BW（未核实） | 自托管·无需官方账号 | 免·自 | 轻量兼容服务端。1.37.5（2026-11-01）含 2 个 Medium 修复（GHSA-x9cq-f669-vrr4 / GHSA-9pjj-f9g5-3x4c）+ web vault v2026.10.1。借鉴：**协议兼容 + 自建** |
| 1Password | 闭 | W/M/L/A/i/CLI | **Secret Key**：34 字符、128 bit 熵，本机生成、永不上传，与约 40 bit 主密码合成 128+ bit 密钥 [7] | 需账号 SaaS | 订（价格未核实） | 借鉴：**高熵设备侧密钥材料 + Emergency Kit 恢复**，专抗服务器拖库暴破 |
| pass **1.7.4** | G2 [8] | Unix；A/i 靠社区客户端 | 每条=一个 GPG 加密文件，文件名即站点名 | 离线；`git push/pull` 才联网 | 免 | Unix 哲学 + git 版本历史。借鉴：**明文目录结构 + git diff 审计 + 生态客户端**（Android-Password-Store/passforios/browserpass） |
| gopass | M·Go·★7,321 | W/M/L | 兼容 pass + GPG/git | 离线 / 团队 git | 免 | 借鉴：团队多 store、CLI 对开发者友好 |
| Enpass / Proton Pass | 未核实 | 未核实 | 未核实 | — | — | 官网抓取失败/截断，不臆测 |

## 三、文件 / 目录加密

| 项目 | 协议·★ | 平台 | 加密 / KDF | 联网 | 模式 | 卖点 / 可借鉴 |
|---|---|---|---|---|---|---|
| Cryptomator **1.19.6** | G3·Java·★16,499 [9] | W/M/L/A/i | 文件与文件名 AES-256；**scrypt(N=2^15,r=8)** 派生 KEK + RFC3394 AES Key Wrap 包主密钥；format 8，vault 配置为 JWT [10] | 无账号·仅需 vault 密码 | 桌面免/捐·移动买·Hub 订 | 虚拟盘（WinFsp/macFUSE，回退本地 WebDAV）、逐文件加密便于云同步。借鉴：**「密文目录 + 虚拟盘」解耦云厂商** |
| VeraCrypt **1.26.31** | 开源（SPDX 未核实）[11] | W/M/L·**无官方移动版** | 容器/分区/系统盘；系统盘 PBKDF2-RIPEMD160 200,000 迭代、标准容器 500,000（PIM 可调）；1.26.29 起非系统卷支持 Argon2id | 完全离线 | 免 | 透明实时 + 隐藏卷/隐藏 OS 的似是而非否认。借鉴：**可否认加密** |
| age / rage | B3 / Ap2·★24,317 [12] | W/M/L/FreeBSD+多发行版 | 现代小密钥、无配置项、UNIX 可组合；v1.3.0+ 内置后量子 **ML-KEM-768+X25519**（`age-keygen -pq`）；SSH 公钥、多收件人、YubiKey 插件 | 完全离线 | 免 | 借鉴：**极简格式 + 公钥/口令双模 + 管道组合**，最适合做开发者密钥分发底座 |
| GnuPG **2.5.24** | GPL [13] | 全平台（Win=Gpg4win 5.1.x） | RFC4880 OpenPGP，亦支持 S/MIME/SSH | 离线 | 免 | 1997 起标准；2.5.13–2.5.16 严重漏洞 T8044，2.5.17 修复。借鉴：成熟 Web-of-Trust，但 UX 重，宜做「可选后端」 |
| Picocrypt | G3·Go·★1,549 [14] | 便携单文件 | XChaCha20/Serpent + Argon2 + Reed-Solomon | 离线 | 免 | ⚠️ 仓库**已归档**（末次 push 2025-09-07），不宜作新项目依赖 |
| Hat.sh / AxCrypt / NordLocker / Tresorit | 未核实 | 未核实 | 未核实 | — | 商业 | 官网跨域/空正文/抓取失败 |

## 四、离线 / P2P 同步与传输（与「密库跨设备搬运」的关系）

| 项目 | 协议·★ | 平台 | 机制 | 账号·联网 | 模式 | 关系 |
|---|---|---|---|---|---|---|
| Syncthing **v2.1.7** | MP2·Go·★90,415 [15] | W/M/L/FreeBSD 等 | P2P 持续文件同步、TLS 设备认证 | 无中心账号·可纯 LAN | 免 | **最优解**：把 `.kdbx`/vault 当普通文件同步，密文端到端、零知识；v2.1.7（2026-10-21，Go 1.27.3） |
| LocalSend **v1.18.2** | Ap2·Dart·★98,700 [16] | W/M/L/A | 局域网 AirDrop 式近场互传 | 无账号·纯 LAN | 免 | 一次性「把密库甩给另一台设备」；1.18.2 修复「不跟随对端 HTTP 重定向」 |
| Resilio Sync | 未核实 | 未核实 | 未核实（商业 P2P） | — | 商业 | 未核实 |
| Tailscale | B3·Go·★39,012 [17] | 全平台 | WireGuard 组网 + 2FA | 需账号（控制面） | 免+订 | **仅作对比**：解决「设备可达」而非密文同步，控制面依赖账号，不属纯离线 |

## 五、本地优先 E2EE 笔记 / 知识 / 照片

| 项目 | 协议·★ | 平台 | 加密 | 联网·账号 | 模式 | 卖点 / 可借鉴 |
|---|---|---|---|---|---|---|
| Standard Notes | A3·★6,714 [18] | W/M/L/A/i/Web | E2EE 笔记+文件 | 需账号·可自托管 | 免+订 | 借鉴：**主题/扩展与加密内核解耦** |
| Joplin **v3.7.22** | ?（GitHub 标注）·★57,155 [19] | W/M/L/A/i | 本地 Markdown + 可选 E2EE 同步 | 可纯本地；同步走 Dropbox/Nextcloud/WebDAV | 免（+Cloud 订） | 3.7.22 新增 `read_notes` MCP tool 与「把私网 AI 供应商视为本地」。借鉴：**本地文件为本 + 同步后端可插拔 + AI 时代明确隐私边界** |
| Notesnook | G3·★14,921 [20] | W/M/L/A/i/Web | E2EE | 需账号·可自托管 | 免+订 | Evernote 的开源 E2EE 替代 |
| Anytype | ?（Other，未核实）·★9,085 [21] | W/M/L（移动端另库，未核实） | topics 明示 e2ee/local-first/offline-first/p2p | 本地优先 + P2P | 免+增值 | 借鉴：**local-first + CRDT/P2P 对象化知识库** |
| Obsidian + Sync | 闭（本体免费，商业授权未核实）[22] | W/M/L/A/i | 本地 Markdown；Sync=E2EE AES-256 | 可纯离线·Sync 需账号 | 本体免；Sync **$4/月**(1 vault/1GB/5MB 单文件) 或 **$8/月**(10 vault/10GB/200MB/12 月历史) | 「离线可用、稍后合并」。借鉴：**纯本地文件 + 可选付费 E2EE 同步 + 版本历史** |
| Ente | A3·Dart·★30,077 [23] | W/M/L/A/i/Web | libsodium：**Argon2id** 派生 KEK 包 256-bit masterKey；master→collection→file 分层 256-bit；共享用 X25519 `crypto_box_seal`；密钥 XSalsa20-Poly1305、文件 `crypto_secretstream`=XChaCha20-Poly1305 [24] | 需账号（3 云厂商副本） | 免+订 | E2EE 照片/2FA。借鉴：**分层密钥 + 公钥共享 + recoveryKey + BIP39 Verification ID**，可移植到密库共享 |

## 六、平台自带能力

| 平台 | 能力 | 状态 |
|---|---|---|
| Windows | DPAPI `CryptProtectMemory`：进程/跨进程/同登录会话内存加密，长度须为 `CRYPTPROTECTMEMORY_BLOCK_SIZE` 倍数；微软明示「加密前明文已在内存，不安全」，落盘应改用 `CryptProtectData`（用户/机器绑定，为 Hello / Credential Manager 底层）[25] | 部分核实；`CryptProtectData`/Hello 参数未核实 |
| macOS Keychain / iOS Keychain+Secure Enclave+Data Protection / Android Keystore+StrongBox+EncryptedSharedPreferences | 官方页均「(Content truncated)」或 fetch failed | **未核实** |
| 跨平台库 | SQLCipher：SQLite 分支，256-bit AES 加密数据库文件，B3·C·★7,329 [26] | 已核实 |

## 七、专门回答

**Q1：完全离线加密保险库，是否已有成熟好用的开源方案？**
**有，且桌面端已相当成熟。** `KeePassXC 2.8.0`（G3）是标杆：单文件 `.kdbx`、无云无账号、KDBX4 的 Argon2d + HMAC-SHA-256 + Encrypt-then-MAC、2.7.9 过 ANSSI CSPN 认证；`pass 1.7.4` + `gopass` 以 GPG+git 做到「可 diff、可审计」；`age` 覆盖文件级、`VeraCrypt 1.26.31` 覆盖容器/隐藏卷。**但移动端是明显短板**：KeePassXC 官方无移动版、VeraCrypt 无官方移动端、pass 靠社区客户端。故结论＝**桌面成熟、移动端与「无云多端一致性」仍空白**。

**Q2：新产品的真空地带（差异化机会）**
1. **移动端一等公民**：缺一个「单文件密库 + 官方原生 iOS/Android + 生物识别解锁」的 GPL 方案。Cryptomator 有移动版但买断、面向文件；可借鉴其虚拟盘/Key Wrap 与 Ente 的 iOS/Android 工程。
2. **跨设备无云同步**：Syncthing（P2P）+ LocalSend（近场）已能搬 `.kdbx`，但与保险库「应用内解锁、冲突合并、版本回滚」割裂；KeePassXC 2.8.0 才刚把同步降级为「外部工具」。机会＝**把 P2P/近场同步做成密库一等内置能力，密钥永不触网、无需账号**，靠 Encrypt-then-MAC 保证传输中密文不可篡改。
3. **开发者 `.env` / 密钥工作流**：pass/gopass 偏「人读密码」、age 偏「文件」。缺面向 `.env`、CI secret、SSH/API token 的方案：借鉴 age「公钥/口令双模 + 管道可组合 + SSH 公钥收件人」，做 git 友好的密文 env、按收件人分发、一次解锁注入进程环境。
4. **AI 时代本地推理隐私边界**：Joplin 3.7.22 已「把私网 AI 供应商视为本地」，Obsidian/Anytype 在探索本地知识库喂本地 LLM。机会＝**保证「解锁态密钥/笔记也不出进程、不进模型上下文」**，借鉴 DPAPI `CryptProtectMemory` + `SecureZeroMemory` 清零与 Ente 分层密钥隔离；提供「本地推理可检索、云端推理自动脱敏」开关。
5. **可借鉴设计提纯**：①单文件自包含（KeePass/Cryptomator）；②可换 KDF/套件 + 插件（KeePass）；③高熵设备侧密钥 + 恢复包（1Password Secret Key / Ente recoveryKey）；④分层密钥 + 公钥共享（Ente）；⑤本地文件为本、同步可插拔（Joplin/Obsidian）；⑥隐藏卷/可否认加密（VeraCrypt）；⑦签名发布物 + 定期安全公告（KeePassXC `.sig/.DIGEST`、Vaultwarden GHSA、GnuPG T8044）。

## 八、来源 URL
官网/文档：[1] https://keepass.info/help/v2/license.html · [2] https://keepass.info/help/kb/kdbx_4.html · [3] 同 [2] · [4] https://keepass.info/features.html · [7] https://support.1password.com/secret-key-security/ · [8] https://www.passwordstore.org/ · [10] https://docs.cryptomator.org/security/architecture/ 与 https://cryptomator.org/ · [11] https://veracrypt.io/en/Home.html · [13] https://gnupg.org/ · [22] https://obsidian.md/sync · [24] https://ente.com/architecture/ · [25] https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectmemory · 另 KeePassXC 官网 https://keepassxc.org/
GitHub 元数据（均 `https://api.github.com/repos/<owner>/<repo>`，版本取 `/releases/latest`）：[5] bitwarden/clients、bitwarden/server · [6] dani-garcia/vaultwarden · [9] cryptomator/cryptomator · [12] FiloSottile/age、str4d/rage · [14] Picocrypt/Picocrypt · [15] syncthing/syncthing · [16] localsend/localsend · [17] tailscale/tailscale · [18] standardnotes/app · [19] laurent22/joplin · [20] streetwriters/notesnook · [21] anyproto/anytype-ts · [23] ente/ente · [26] sqlcipher/sqlcipher · KeePassXC release：api.github.com/repos/keepassxreboot/keepassxc/releases/latest · gopass：gopasspw/gopass

**未核实清单（抓取失败/截断，明确不编造）**：Enpass、Proton Pass、Hat.sh、AxCrypt、NordLocker、Tresorit、Resilio Sync 的协议/密码学/平台/定价；Bitwarden/Vaultwarden 具体加密参数（AES-CBC / PBKDF2 / Argon2 迭代数，官方白皮书与 `/help` 页均截断或 404）；1Password 订阅价格；macOS Keychain、iOS Secure Enclave/Data Protection、Android Keystore/StrongBox/EncryptedSharedPreferences 官方参数；Anytype 具体许可证；Obsidian 本体商业授权；VeraCrypt 具体 SPDX 许可证。凡 GitHub `license:"other"/NOASSERTION` 处均按「具体协议未核实」处理。
