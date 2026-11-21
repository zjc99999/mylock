# 移动端本地隐私/加密存储调研报告
（面向「本地优先加密保险库」概要设计 · 2026-11-13）

> **核实说明**：本环境 `developer.android.com`、`source.android.com`、`support.apple.com` 正文均不可达或被截断；Android 侧以 AOSP / AndroidX **源码**为准，iOS 侧以 Apple 官方 JSON 文档为准，产品许可取自 GitHub API。凡未取得一手来源者标注 **未核实**，不作推断。来源编号 [S1]… 见文末。

## 1. 平台安全能力（关键事实与版本）

### 1.1 Android

- **Keystore**：`KeyGenParameterSpec` 决定密钥授权用途（是否需用户认证、允许哪些运算、有效期），密钥按 alias 存入 Android Keystore；源码明确 `The key material of the generated symmetric and private keys is not accessible`——对称/私钥密钥材料不可导出 [S1]。
- **StrongBox**：`Builder.setIsStrongBoxBacked(boolean)`，Javadoc "Sets whether this key should be protected by a StrongBox security chip."，配套 `isStrongBoxBacked()` [S1]。引入版本（常见说法 API 28）**未核实**。
- **认证绑定 / BiometricPrompt**：`setUserAuthenticationRequired(true)` 要求已设安全锁屏；原文 `The key will become irreversibly invalidated once the secure lock screen is disabled ... or when the secure lock screen is forcibly reset`，且增删生物特征默认也会不可逆失效，可用 `setInvalidatedByBiometricEnrollment(false)` 关闭。每次使用都需认证时，官方做法是把 `Cipher/Signature/Mac` 包成 `BiometricPrompt.CryptoObject` 再 `BiometricPrompt.authenticate` [S1]。
- **生物识别分级**（`androidx.biometric` `BiometricManager.java`，文件头 Apache-2.0）[S2]：`BIOMETRIC_STRONG = 0x000F` = "Class 3 (formerly Strong), as defined by the Android CDD"；`BIOMETRIC_WEAK = 0x00FF` = Class 2（是 STRONG 超集）；`DEVICE_CREDENTIAL = 1<<15`（PIN/图案/密码）；`IDENTITY_CHECK = 1<<16`（**API 36** 新增，高风险环境强制 Class 3 且丢弃密码回退）。CDD 对 Class 3 的原文定义 **未核实**。
- **密钥认证**：`setAttestationChallenge(byte[])` 触发 Key Attestation；源码 NOTE `If attestation is not requested ... generated certificate may be self-signed.` [S1]。版本矩阵 **未核实**。
- **Direct Boot / 解锁设备要求**：`setUnlockedDeviceRequired(boolean)` — "authorized to be used only while the device is unlocked"；源码注 `In Android 11 (API level 30) and lower, encryption and verification operations with symmetric keys weren't restricted either.` [S1]。Direct Boot 的 CE/DE 存储细节 **未核实**。
- **FLAG_SECURE**：`WindowManager.LayoutParams.FLAG_SECURE = 0x00002000` — "treat the content of the window as secure, preventing it from appearing in screenshots or from being viewed on non-secure displays" [S3]；是 Android 防截图、防最近任务快照、防投屏录制的主开关。
- **EncryptedSharedPreferences / EncryptedFile：官方已弃用**。`androidx.security:security-crypto` **整个模块全部 API** 弃用（commit `3724241a`，2023-09-25 "Deprecate all APIs in security-crypto"，Bug 301997816）[S4]；源码 `@deprecated Use android.content.SharedPreferences instead` / `Use java.io.File instead` [S5][S6]；最新稳定版 **1.1.0** [S7]。内部基于 Tink（key `AES256_SIV`、value `AES256_GCM`、主密钥存 Keystore；文件用 `StreamingAead`）[S5][S6]。**致命坑**：源码 WARNING `should NOT be backed up with Auto Backup ... the key used to encrypt it will no longer be present`，须用 backup rules 排除，连静默生成的 `__androidx_security_crypto_encrypted_file_pref__` 也要排除 [S5][S6]。→ 新项目不要依赖它，直接用 Tink 或 SQLCipher。
- **SQLCipher / Room**：SQLCipher 社区版开源（256-bit AES 全库加密，BSD-3-Clause）[S8]，官方称"每个 SQLCipher 数据库都可在任意设备打开，适合多平台应用" [S9]；Android 集成 `sqlcipher-android-4.20.0.aar` + `androidx.sqlite 2.7.0`（`net.zetetic.database.sqlcipher`）[S9]；Commercial $999/年起 [S9]。**Room 本身不加密**，需 SQLCipher 承接或 DAO 层自加密（Room+SQLCipher 接法 **未核实**）。
- **Scoped Storage / 应用沙盒 / Auto Backup 与 E2EE 备份**：官方页不可达 → **未核实**。

### 1.2 iOS

- **Keychain Services**：加密数据库，存密码、信用卡、短笔记、密钥/证书等小块数据 [S10]。
- **可访问性等级**：`kSecAttrAccessibleWhenUnlockedThisDeviceOnly`（iOS 4.0+）— `Items with this attribute do not migrate to a new device. Thus, after restoring from a backup of a different device, these items will not be present.` [S11]；`kSecAttrAccessibleAfterFirstUnlock`（iOS 4.0+）首次解锁后可访问至下次重启、推荐后台项、**随加密备份迁移到新设备** [S12]。二者对"备份/跨设备"是相反语义。
- **Data Protection**（`FileProtectionType`）[S13]：`complete`（iOS 4.0+，磁盘加密，锁屏或开机时不可读写）[S14]、`completeUnlessOpen`（关闭后加密，允许后台续写）、`completeUntilFirstUserAuthentication`（iOS 5.0+，首次解锁后即使再锁屏仍可访问）[S15]、`none`（无保护）。
- **Secure Enclave** [S16]：隔离于主处理器的硬件密钥管理器；仅 A7+ iOS 设备 / M1+ Mac；仅 NIST P-256（签名、ECDH→对称加密）；**`Can't encode preexisting keys`**——私钥不可导入导出，App 永不接触明文，只拿运算输出。推荐 `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` + `.privateKeyUsage`（可叠加 `.biometryAny`）。→ 与"跨设备可移植主密钥"直接冲突，只能用来本机包裹（wrap）可导出的库密钥。
- **CryptoKit**：`AES.GCM` [S17]、`ChaChaPoly` [S18] 均 iOS 13.0+ / macOS 10.15+，提供 `seal/open`、`SealedBox`、`Nonce`、AAD 认证。
- **Face ID / LocalAuthentication**：`LAPolicy.deviceOwnerAuthentication`（iOS 9.0+，生物识别优先、失败回退设备密码）[S19]；App 只指定 policy 与提示语、只拿到 Boolean，**永不接触指纹图像/生物数据** [S20]。
- **截图/录屏钩子**：`UIScreen.isCaptured`（iOS 11.0+，iOS 27.0 弃用→`UITraitCollection.sceneCaptureState`）可感知录屏/镜像/AirPlay 并主动遮罩 [S21]；`UIApplication.userDidTakeScreenshotNotification`（iOS 7.0+）`posts after the screenshot is taken`——**只能事后告警，无法阻止截图** [S22]。
- **App Sandbox / iCloud Keychain / iCloud 备份 E2EE（ADP）/ 文件 App「在本机上」/ Lockdown Mode / app switcher 快照遮挡官方 API**：support.apple.com 正文截断 → **未核实**。

## 2. 移动端真实痛点：OS 已解决 vs App 自理

| 痛点 | OS 已解决 | App 仍需自理 |
|---|---|---|
| 输入/查看长密钥 | 几乎无 | 小屏+软键盘易误输/被旁观；改用二维码/文件导入、分段显示、默认遮蔽、禁截图 |
| 文件导入导出、跨设备搬运 | Android SAF、iOS Files 提供通道（细节未核实） | **无账号无云时通道全靠 App 自建**：单文件加密容器、扫码、局域网/USB 直传、冲突合并 |
| 剪贴板泄露 | 部分（访问提示/超时清空，未核实） | 复制到系统剪贴板＝明文暴露；应不复制、用 App 内粘贴板、定时清空 |
| 截图与录屏 | Android `FLAG_SECURE`（已核实）可挡；iOS 只能检测录屏、截图仅事后通知 | 两端能力不对称，须逐端设 `FLAG_SECURE`、`isCaptured` 监听、敏感视图遮罩 |
| 应用切换预览 | Android `FLAG_SECURE` 同源压制最近任务快照 [S3] | iOS 需在失活时自行加遮挡层（官方快照 API 未核实） |
| 通知内容泄露 | 系统锁屏通知默认展示内容 | App 须关闭敏感通知预览（细节未核实） |
| 生物识别解锁 | **已解决**：Android BiometricPrompt+Keystore、iOS LA+Secure Enclave，App 拿不到生物数据/密钥明文 [S1][S20] | 只需绑定 `CryptoObject` / `SecAccessControl`，并提供口令兜底 |
| 离线可用性 | 本地优先天然可行 | 但硬件密钥不可导出，跨设备必须另带可移植主密钥 |
| 系统备份致明文外泄 | 部分：iOS `ThisDeviceOnly` 不随备份迁移 [S11]；Android Auto Backup 恢复后密钥丢失（security-crypto 源码已证）[S5][S6] | **必须**配 backup rules 排除 / `allowBackup=false`，否则密文被备份、密钥不备份＝恢复即砖 |

## 3. 现成产品梳理

> 许可/仓库属性取自 GitHub API（一手）；**除注明外定价模式均未核实**，以商店为准。

| 产品 | 平台 | 本地/云 | 开源 / 许可 | 定价 | 可借鉴点 |
|---|---|---|---|---|---|
| **Cryptomator** | 桌面+Android | 客户端加密、云盘作哑存储；本地密文 | 开源 GPL-3.0（桌面 [S23] + Android [S24]） | 桌面免费；Android 买断（未核实） | 单 vault 文件格式、虚拟文件系统、跨桌面+移动 |
| **KeePassDX** | Android | **纯本地单文件**（KeePass 格式） | 开源 GPL-3.0 [S25] | 免费（未核实） | 最贴合"无账号无云"：单文件可搬运、生物识别、自动填充、Passkey |
| **KeePassium** | iOS/macOS | 本地 KeePass 单文件 | "Other"（非标准开源）[S26] | 未核实 | iOS 侧 KeePass 兼容与 Files 集成 |
| **Strongbox** | iOS/macOS | 本地 KeePass/Password Safe（可选云同步） | 源码 AGPL-3.0 [S27] | 免费+订阅（未核实） | iOS 本地 vault + 多格式兼容 |
| **Ente** | 移动+桌面 | E2EE 云（照片/Auth） | 开源 AGPL-3.0 [S28] | 订阅（未核实） | 开源 E2EE 云架构、跨端 |
| **Standard Notes** | 全平台 | E2EE 云笔记+文件 | 开源 AGPL-3.0 [S29] | 订阅（未核实） | 端到端加密文件/笔记模型 |
| **1Password** | 全平台 | 云 E2EE（含本地 vault） | 闭源（未核实） | 订阅（未核实） | Secret Key+主密码双因子、旅行模式 |
| **Bitwarden** | 全平台 | 云 E2EE / 可自托管 | 移动 GPL-3.0 [S30]；clients "Other" [S31] | 免费+订阅（未核实） | 可自托管、可导出 |
| **Proton Pass** | 全平台 | E2EE 云 | Android GPL-3.0 [S32] | 免费+订阅（未核实） | 开源 E2EE 密码管理 |
| **Apple Passwords/钥匙串、Google Password Manager** | 系统内置 | OS 级云同步 | 闭源（未核实） | 随设备免费 | 系统级自动填充、iCloud Keychain |
| **Secure Folder / 私密空间、三星 Knox、小米/华为隐私空间** | OEM Android | 本机隔离容器 | 闭源（未核实） | 随设备免费 | 系统级"第二空间"隔离、快捷隐藏 |

## 4. 三大难点与可复用开源库

### 4.1 「无账号、无云、跨桌面+移动」在移动端最大的 3 个难点

1. **跨设备搬运 + 密钥可移植性（最本质）**。已核实：Android Keystore 对称/私钥材料不可导出 [S1]、iOS Secure Enclave `Can't encode preexisting keys` [S16]，硬件密钥无法充当"跨桌面+移动"的同一把主密钥。必须自研**可移植单文件格式**（SQLCipher 库 / age 容器 / KeePass 格式）+ 口令或恢复密钥派生主密钥，并解决移动端文件导入导出与冲突合并。这与云同步 E2EE 产品（Ente、Bitwarden、1Password）靠账号绕开搬运难题形成根本差异。
2. **移动端独有的 UI 泄漏面**。截图/录屏、最近任务或 app switcher 快照、锁屏通知、剪贴板、软键盘旁观——桌面端几乎不存在。Android 有 `FLAG_SECURE` 一次挡住截图与最近任务 [S3]，iOS 只能感知录屏、截图仅事后通知 [S21][S22]，两端能力不对称；长密钥在小屏输入/查看更危险。须逐端做安全窗口标志、`isCaptured`/`sceneCaptureState` 监听、失活遮罩、通知/剪贴板脱敏。
3. **备份与认证绑定密钥的「变砖风险」（本地优先尤其致命）**。已核实：认证绑定密钥会因锁屏重置或增删生物特征**不可逆失效** [S1]，Auto Backup 恢复后加密密钥不再存在、密文变废数据 [S5][S6]。**没有账号找回渠道＝一次失效即永久丢数据**。必须：backup rules 显式排除、`allowBackup=false`、口令兜底解锁（不能只靠生物识别）、用硬件密钥仅**包裹**可导出主密钥。

### 4.2 可直接复用的开源库

| 库 | 平台 | 协议 | 用途 |
|---|---|---|---|
| **SQLCipher** (`sqlcipher/sqlcipher`) | C/跨平台（Android、iOS、桌面） | **BSD-3-Clause** [S8] | 全库 256-bit AES、单文件可跨设备打开 [S9]；结构化数据/索引首选 |
| **libsodium** (`jedisct1/libsodium`) | C/全平台 | **ISC**（LICENSE 原文核实）[S33] | secretbox/AEAD、KDF、安全内存；轻量跨平台原语 |
| **Tink** (`tink-crypto/tink`) | Java/Kotlin/多语言 | **Apache-2.0** [S34]（仓库已 archived） | AEAD/DeterministicAead + AndroidKeystoreKms；取代已弃用的 security-crypto |
| **androidx.biometric** | Android/Kotlin | **Apache-2.0**（源码文件头）[S2] | BiometricPrompt、BIOMETRIC_STRONG(Class 3)、CryptoObject 绑定 |
| **CryptoKit** | iOS/Swift（系统框架，非开源） | 系统内置 | AES.GCM / ChaChaPoly / SecureEnclave，iOS 13+ [S17][S18] |
| **age** (`FiloSottile/age`, Go, **BSD-3-Clause**) / **rage** (`str4d/rage`, Rust, **Apache-2.0**) | 跨平台 | BSD-3 / Apache-2.0 [S35][S36] | 文件/流加密、小显式密钥、可组合；适合导出容器与恢复密钥 |
| **RustCrypto/password-hashes、KDFs** | Rust | 仓库级 license 为 null，**各 crate 具体许可未核实** [S37][S38] | Argon2id/scrypt 等口令→主密钥 KDF |

### 4.3 未核实清单（汇总）

Android：StrongBox 引入版本、CDD Class 3 原文、Key Attestation 版本矩阵、Direct Boot CE/DE、Scoped Storage、Auto Backup E2EE、Room+SQLCipher 接法。iOS：App Sandbox、iCloud Keychain / iCloud 备份 E2EE（ADP）、文件 App「在本机上」、Lockdown Mode、失活快照官方 API。产品：1Password、Apple/Google 密码管理器、Knox 及各厂隐私空间的开源属性与全部订阅/买断定价。库：RustCrypto 单 crate 许可。

### 来源

- S1 https://api.github.com/repos/aosp-mirror/platform_frameworks_base/contents/keystore/java/android/security/keystore/KeyGenParameterSpec.java
- S2 https://api.github.com/repos/androidx/androidx/contents/biometric/biometric/src/main/java/androidx/biometric/BiometricManager.java
- S3 https://api.github.com/repos/aosp-mirror/platform_frameworks_base/contents/core/java/android/view/WindowManager.java
- S4 https://api.github.com/repos/androidx/androidx/commits?path=security/security-crypto/src/main/java/androidx/security/crypto/EncryptedSharedPreferences.java&per_page=5
- S5 https://api.github.com/repos/androidx/androidx/contents/security/security-crypto/src/main/java/androidx/security/crypto/EncryptedSharedPreferences.java
- S6 https://api.github.com/repos/androidx/androidx/contents/security/security-crypto/src/main/java/androidx/security/crypto/EncryptedFile.java
- S7 https://dl.google.com/dl/android/maven2/androidx/security/group-index.xml
- S8 https://api.github.com/repos/sqlcipher/sqlcipher
- S9 https://www.zetetic.net/sqlcipher/sqlcipher-for-android/ （另 https://www.zetetic.net/sqlcipher/）
- S10 https://developer.apple.com/tutorials/data/documentation/security/keychain-services.json
- S11 https://developer.apple.com/tutorials/data/documentation/security/ksecattraccessiblewhenunlockedthisdeviceonly.json
- S12 https://developer.apple.com/tutorials/data/documentation/security/ksecattraccessibleafterfirstunlock.json
- S13 https://developer.apple.com/tutorials/data/documentation/foundation/fileprotectiontype.json
- S14 https://developer.apple.com/tutorials/data/documentation/foundation/fileprotectiontype/complete.json
- S15 https://developer.apple.com/tutorials/data/documentation/foundation/fileprotectiontype/completeuntilfirstuserauthentication.json
- S16 https://developer.apple.com/tutorials/data/documentation/security/protecting-keys-with-the-secure-enclave.json
- S17 https://developer.apple.com/tutorials/data/documentation/cryptokit/aes/gcm.json
- S18 https://developer.apple.com/tutorials/data/documentation/cryptokit/chachapoly.json
- S19 https://developer.apple.com/tutorials/data/documentation/localauthentication/lapolicy/deviceownerauthentication.json
- S20 https://developer.apple.com/tutorials/data/documentation/localauthentication.json
- S21 https://developer.apple.com/tutorials/data/documentation/uikit/uiscreen/iscaptured.json
- S22 https://developer.apple.com/tutorials/data/documentation/uikit/uiapplication/userdidtakescreenshotnotification.json
- S23 https://api.github.com/repos/cryptomator/cryptomator
- S24 https://api.github.com/repos/cryptomator/android
- S25 https://api.github.com/repos/Kunzisoft/KeePassDX
- S26 https://api.github.com/repos/keepassium/KeePassium
- S27 https://api.github.com/repos/strongbox-password-safe/Strongbox
- S28 https://api.github.com/repos/ente/ente
- S29 https://api.github.com/repos/standardnotes/app
- S30 https://api.github.com/repos/bitwarden/android
- S31 https://api.github.com/repos/bitwarden/clients
- S32 https://api.github.com/repos/protonpass/android-pass
- S33 https://api.github.com/repos/jedisct1/libsodium/contents/LICENSE?ref=master
- S34 https://api.github.com/repos/google/tink
- S35 https://api.github.com/repos/FiloSottile/age
- S36 https://api.github.com/repos/str4d/rage
- S37 https://api.github.com/repos/RustCrypto/password-hashes
- S38 https://api.github.com/repos/RustCrypto/KDFs
