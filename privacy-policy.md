# MyLock 隐私政策（草案 · 待法务复核）

> **性质声明（务必先读）**：本文是**隐私政策草案**，供上架前**律师/法务复核**使用，**不是已生效的对外法律文件、不是法律意见、不是合规结论**。生效前必须由具资质法务按最终分发地区、分发渠道、公司主体信息逐条核定。
> 版本：v0.1（草案） · 日期：2026-11-15 · 授权：用户 m03444「继续」（M4 第 4 点：上架前准备）
> 数据流依据：[security-whitepaper.md](security-whitepaper.md) §0/§3/§4、[产品分析与概要设计.md](产品分析与概要设计.md)（反定位 No Cloud·No AI·No Telemetry·No Account）、[threat-model.md](threat-model.md)

---

## 0. 待法务填写/核定的占位项（生效前必须补全，不得留空对外）

| 占位 | 说明 |
|---|---|
| `[运营主体名称]` | 公司/开发者法定名称 |
| `[注册地址 / 联系地址]` | 法定地址 |
| `[隐私事务联系邮箱]` | 用户行使权利/问询的渠道 |
| `[生效日期]` | 政策生效日 |
| `[适用地区]` | 最终分发地区（中国 / 美国 / 欧盟 / 其他，逐地区核定） |
| `[分发渠道]` | 应用商店（Microsoft Store / Mac App Store / 移动商店）或官网直接下载 —— **尚未决定** |
| `[儿童/未成年人条款]` | 若目标年龄含未成年人，需按当地法补 COPPA / 未成年人保护条款 |
| `[数据主体权利响应流程]` |  GDPR/PIPL 下即便「零收集」也可能需说明如何响应访问/更正/删除请求 |

> 在补齐上述占位并经律师签发前，本文件**不得**作为对外隐私政策发布。

---

## 1. 一句话结论

**MyLock 不收集、不传输、不存储、不共享你的任何个人数据。** 你的文件、口令、密钥自始至终只存在于**你自己的设备/你自己的存储介质**上；我们没有服务器、没有账号、没有能力看到它们。

---

## 2. 我们如何处理数据（真实数据流，逐条如实）

| 数据 | 发生位置 | 是否离开你的设备 | 我们能否访问 |
|---|---|---|---|
| 你的文件内容 | 你的设备本地（加密后写入你指定的 vault 目录） | **否** | **否** |
| 主口令 / Recovery Key / 派生密钥 | 你的设备内存（仅解锁会话内），并即时从内存清除 | **否** | **否** |
| 文件名 / 目录结构 | 加密后存于 vault 的 `manifest`（密文） | **否** | **否** |
| 解锁后的明文 | 你的设备内存 + 本机 `127.0.0.1` loopback 挂载点 | **否**（不出本机） | **否** |
| 使用数据 / 崩溃日志 / 遥测 / 埋点 | **不存在**（无此类代码） | — | — |
| 账号 / 标识符 / 设备指纹 | **不存在**（无账号体系、无设备 ID 采集） | — | — |

- **无对外网络请求**：本产品 App 自身不发起对外网络连接、不监听外部网络接口、无服务端、无云同步、无更新检查回传、无 AI 调用、无第三方分析/广告 SDK。唯一监听是**本机回环地址 `127.0.0.1:<port>`**（默认 4918）的 WebDAV 服务，用于把解密后的 vault 映射为本地文件夹；该端口**仅本机可访问，不对外发包**（[m3-architecture.md](m3-architecture.md)）。⚠ loopback 仍是本机网络接口上的通信，**不绝对化说「无任何网络流量」**；操作系统更新、商店更新、系统级崩溃报告等可能产生与 App 无关的流量，不属本产品行为。
- **无第三方数据处理者**：我们未接入任何会代表我们收集数据的第三方服务。
- **口令与密钥**：Argon2id 派生、AES-KW 包裹、XChaCha20-Poly1305 加密全部在你本机完成。我们无法为你找回 forgotten 主口令 —— Recovery Key 是唯一兜底，且同样只由你持有。

---

## 3. 关于「收集」的口径（对齐 Apple 一手定义）

Apple 官方 [App Privacy Details](https://developer.apple.com/app-store/app-privacy-details/) 对 "Collect" 的定义是（A 级一手引文，2026-11-15 亲抓）：

> "Collect" refers to transmitting data off the device in a way that allows you and/or your third-party partners to access it for a period longer than what is necessary to service the transmitted request in real time.

据此，MyLock 的行为**不构成「收集」**：数据不离开设备、我们与任何第三方 partner 都无法访问。因此：

- App Store Connect 隐私问卷（若走该渠道）应如实选择 **"No, we do not collect data from this app"**（[App Store Connect Help](https://developer.apple.com/help/app-store-connect/manage-app-information/manage-app-privacy/) A 级一手原句；选「否」后无需继续答题）。⚠ 字面标签 "Data Not Collected" 在这些 Apple 页上**未取到一手引文（NOT RETRIEVED）**，对外表述以问卷实际措辞为准、由法务核定。
- 商店问卷中「是否使用加密 / 是否受出口管制约束」等问题，需结合自研 XChaCha20-Poly1305 的事实单独作答，**不得**简单勾选「仅使用系统/豁免加密」——见 [prelaunch-checklist.md](prelaunch-checklist.md) 加密出口管制闸门。

> **A 级一手事实（2026-11-15 亲抓，HTTP 200）**：即便「零收集」，**仍必须提供隐私政策链接** —— Apple App Review Guideline 5.1.1(i)："All apps must include a link to their privacy policy in the App Store Connect metadata field and within the app in an easily accessible manner."；Google Play："You must link to a privacy policy on your store listing and within your app, even if your app doesn't access personal or sensitive user data or target children."；Google Play User Data policy："Apps that do not access any personal and sensitive user data **must still submit a privacy policy**."。同时两家都允许如实申报「不收集」：Apple App Store Connect 可选 "No, we do not collect data from this app"；Google Play Data safety："Even developers with apps that do not collect any user data must complete this form and provide a link to their privacy policy. In this case, the completed form and privacy policy can indicate that no user data is collected or shared."。**本草案正是为满足这一「必须提供 + 如实申报不收集」的要求而准备**；最终问卷措辞与元数据填写仍由法务核定，见 [m4-step4-report.md](m4-step4-report.md) 与 [prelaunch-checklist.md](prelaunch-checklist.md) §3.3。

### 3.1 零数据政策的强制内容清单（Google Play A 级明文，不得留白）

Google Play [User Data policy](https://support.google.com/googleplay/android-developer/answer/10144311?hl=en) 与 [Prepare your app for review](https://support.google.com/googleplay/android-developer/answer/9859455?hl=en) 要求：即使零数据，政策仍须含 —— ① **开发者信息 + 隐私联系点/咨询机制**（对应 §0 占位 `[运营主体名称]` / `[隐私事务联系邮箱]`）；② **应用名或与商店列表一致的实体名**出现在政策中；③ 数据类型披露（可写「无」，但**「无」也必须明示**）；④ 共享方（可写「无」）；⑤ 安全处理措施（本机加密/Argon2id/loopback，见 §2）；⑥ **保留与删除政策**（可写「开发者不持有数据、无保留期限、由用户在本机自行删除」，见 §4/§6）；⑦ 标题明确标注 "privacy policy"；⑧ **"Don't omit any user or device data handling" / "Don't provide misleading or inaccurate information"** —— 不得遗漏任何数据接触、不得误导。政策载体须是公开可访问、有效 SSL、全球可访问、无地域/登录限制、标准不可编辑网页、非 PDF，并在商店页与 App 内双处可访问。Apple 5.1.1(i) 另要求明确写「保留/删除政策」与「用户如何请求删除」。**上述占位未填实、载体未上线前，本草案不满足商店提交要求。**

---

## 4. 不在我们控制范围内的事项（必须如实告知，不得替用户兜底）

MyLock 保护的是**vault 容器内的密文**。以下情形产生的明文/数据暴露，源于操作系统、其他软件或用户自身行为，**不在本产品能力边界内**，我们既不控制也不承担责任（此非免责声明，而是安全边界的如实说明，最终法律责任表述由法务核定）：

1. **解锁态的明文**：用户输入口令解锁后，明文出现在设备内存与本机挂载点；任何能访问该挂载点的本机进程、操作系统索引/搜索、备份软件、恶意软件都可能读取到。锁定 vault **不会**自动清除解锁期间被复制到别处的明文副本。
2. **云同步目录**：我们**拒绝**在 OneDrive / Dropbox / Google Drive / iCloud Drive / 坚果云 等云同步根目录内挂载（`is_sync_root`）。但若用户把 vault 密文目录或其解锁后明文**手动复制**进此类目录，同步行为由用户/第三方客户端完成，与我们无关。
3. **本机 Other processes**：loopback WebDAV **无 per-request 认证**，任何本机进程都可能访问挂载点（[m3-architecture.md](m3-architecture.md) §4 公开的已知限制）。
4. **已攻陷设备**：键盘记录、内存取证、已解锁会话内的恶意进程不在防护范围内。
5. **口令强度**：口令熵是唯一安全上限；我们无法防止弱口令被离线爆破。
6. **介质丢失/物理访问**：vault 静态是密文，但拿到介质副本者可进行无限次离线口令猜测。

> 完整威胁模型与十条诚实声明见 [security-whitepaper.md](security-whitepaper.md) §1/§5 与 [threat-model.md](threat-model.md)。

---

## 5. 我们**绝不**宣称的内容（对外措辞红线）

- ❌「我们零知识所以你绝对安全」「军用级/不可破解」「100% 防泄漏」「防所有采集」——形状泄漏未 100% 消除、解锁态仅缓解。
- ❌「零元数据」——公开元数据（对象数量、大致大小、密文目录结构）可见，只承诺**用户数据零明文**。
- ❌「通过第三方安全审计 / 已获安全认证」——目前只有内部评审与本机测试，**非第三方审计**（[security-whitepaper.md](security-whitepaper.md) §8）。
- ❌「通过校验 = 文件未被篡改/是最新版」——`verify` 绿 ≠ 未被恶意替换 ≠ 最新版。
- ❌「换口令能吊销旧访问」——换口令是**零吊销**，真吊销须 `rotate-keys` + 物理销毁旧介质。
- ❌「明文只存在于加密内存、绝不留痕」——解锁态明文在本机内存/挂载点真实存在。
- ❌「我们不接收任何数据，因此 GDPR / PIPL 对我们不适用 / 我们不是控制者或处理者 / 我们已完全合规」——**这是法律定性，须律师判断，本事实调研不支持该绝对结论**。EDPB Guidelines 07/2020 明文 "It is not necessary that the controller actually has access to the data that is being processed to be qualified as a controller."；GDPR Recital 18 末句 "However, this Regulation applies to controllers or processors which provide the means for processing personal data for such personal or household activities."；PIPL 第 72 条豁免主体是「自然人」（用户本人）而非公司；判准是「谁决定处理目的与方式」而非「谁拿到数据」。见 [m4-step4-report.md](m4-step4-report.md) §2.1。
- ❌「加密后数据已匿名化 / 不再是个人信息」——PIPL 第 73(四)「匿名化」法定标准是「无法识别特定自然人且**不能复原**」，用户持口令即可解密复原；GDPR Recital 26 亦把可借助附加信息复原的假名化数据仍视为可识别信息。**加密 ≠ 匿名化**。
- ❌「零数据收集，所以不需要隐私政策」——与两家商店 A 级明文直接相反（见 §3）。

---

## 6. 数据主体权利 / 儿童 / 跨境（待法务按地区核定）

- 由于我们不持有任何个人数据，通常的访问/更正/删除/携带请求在技术上**无对象可操作**；但**是否仍需提供响应渠道与流程**，取决于 GDPR / PIPL 等对「纯本地处理、开发者不接收数据」场景的适用结论（见 [m4-step4-report.md](m4-step4-report.md) 隐私法适用性调研，标 A/A⁻/B/C 分级）。
- **无数据跨境传输**：本产品不产生跨境数据流。
- **儿童**：若定级含未成年人，须补当地未成年人条款（中国《未成年人网络保护条例》/ 美国 COPPA / GDPR-K 等）——待法务核定。
- **政策变更**：若未来引入任何网络功能/账号/遥测（当前反定位下不应发生），必须**先更新本政策并重新走法务复核**，不得静默变更数据流。

---

## 7. 联系我们

`[隐私事务联系邮箱]` · `[运营主体名称]` · `[注册地址]`

---

## 8. 复核与生效条件（go/no-go，见 [prelaunch-checklist.md](prelaunch-checklist.md)）

本草案生效前必须全部满足：

1. §0 全部占位项由法务据最终主体/地区/渠道填实；
2. 律师核定「零收集但提供政策」的形式合规、数据主体权利流程、儿童条款；
3. 加密出口管制 / 商店加密申报结论落地（自研加密不得误报豁免）；
4. 与 [security-whitepaper.md](security-whitepaper.md) 的十条诚实声明、[m4-step4-report.md](m4-step4-report.md) 的事实分级逐条一致；
5. **本文仅为草案**——未经律师签发，不得对外发布、不得作为商店隐私政策 URL 提交。

---

## 9. 文档接口

[security-whitepaper.md](security-whitepaper.md) · [prelaunch-checklist.md](prelaunch-checklist.md) · [m4-step4-report.md](m4-step4-report.md) · [产品分析与概要设计.md](产品分析与概要设计.md) · [threat-model.md](threat-model.md) · [m3-architecture.md](m3-architecture.md)
