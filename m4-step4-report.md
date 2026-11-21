# M4 第 4 点：上架前准备包（合规事实调研 + 草案 + 核验清单）

> 版本：v1.0 · 日期：2026-11-15 · 上游：[产品分析与概要设计.md](产品分析与概要设计.md) v1.9（M4 第 4 点当时为 v1.8；M5 后升 v1.9，见 [m5-report.md](m5-report.md)）/ [m4-step3-report.md](m4-step3-report.md) · 授权：用户 m03444「继续」（承接 M4 第 3 点汇报「逐项进行」第 4 条）
> **性质：合规事实调研 + 可落地的准备包 + 草案文档 + 核验清单。不是合规结论、不是第三方安全审计报告、不是法律意见、不是已通过的认证，也不代表产品已可上架。** 真正的第三方审计与律师签发本仓库无法代劳，本轮只把外部闸门如实标出。

## 0. 结论先行（给决策者的一页纸）

- **决策**：本阶段**不宣称可上架**。当前 go/no-go 总判据为 **NO-GO**（见 [prelaunch-checklist.md](prelaunch-checklist.md) §0 / §4）。四道外部闸门 A（第三方安全审计）/ B（律师签发）/ C（代码签名 + 可分发安装包）/ D（商店上架申报）**均未完成**，其中 A、B 不是软件工程能自行关闭的闸门。
- **本轮交付 5 件**：本报告、[security-whitepaper.md](security-whitepaper.md)、[privacy-policy.md](privacy-policy.md)、[prelaunch-checklist.md](prelaunch-checklist.md)、[tools/verify-m4s4.ps1](tools/verify-m4s4.ps1)；另有主文档升 v1.8 与约 12 份配套文档的版本对齐。
- **最反直觉、也最必须写进政策的 A 级事实**：**「零数据收集」的 App 仍必须提供隐私政策链接**。Apple Guideline 5.1.1(i)「All apps must include a link to their privacy policy in the App Store Connect metadata field and within the app in an easily accessible manner.」；Google Play「You must link to a privacy policy on your store listing and within your app, **even if your app doesn't access personal or sensitive user data or target children**.」；User Data policy「Apps that do not access any personal and sensitive user data **must still submit a privacy policy**.」→「我们不收集数据，所以不需要隐私政策」是**错的**，且属于商店明文违规。
- **两家商店都允许如实申报「不收集」**：Apple App Store Connect 单选「No, we do not collect data from this app」；Google Play Data safety「the completed form and privacy policy can indicate that no user data is collected or shared」。Apple 的 "collect" 定义是「transmitting data off the device …」，故纯本机 + loopback-only 处理按该定义不构成 collect。
- **绝不能宣称「开发者不是 controller / GDPR·PIPL 对我们不适用 / 已完全合规」**：EDPB Guidelines 07/2020 明文「It is **not necessary that the controller actually has access to the data** that is being processed to be qualified as a controller.」；GDPR Recital 18 末句「However, this Regulation applies to controllers or processors which **provide the means** for processing personal data for such personal or household activities.」；PIPL 第 72 条豁免主体是「自然人」（用户本人）而非公司。是否构成处理者/控制者是**法律定性，须律师判断**；「无访问权」不是排除项，判准是「谁决定目的与方式」。
- **加密 ≠ 匿名化**：PIPL 第 73(四)「匿名化」法定标准是「无法识别特定自然人且**不能复原**」，用户持口令即可解密复原；GDPR Recital 26 亦把可借助附加信息复原的假名化数据仍视为可识别信息。故不得把 vault 内密文说成「匿名信息」而主张法不适用。
- **中国商用密码（A 级一手）**：《商用密码管理条例》第三十一条「**大众消费类产品所采用的商用密码不实行进口许可和出口管制制度。**」；《密码法》第二十八条同。全文未见对普通消费产品「必须使用国密/核准算法」的强制要求；但「大众消费类产品」的界定边界与强制性国家标准符合性**仍属开放项**，须法务核定，不得据此直接下合规结论。
- **加密出口最反直觉的第二条（A/A⁻ 决定性事实）**：**「无网络 / 零数据收集 / 开源」都不是出口合规豁免依据**。Apple 官方一手 "the use of encryption that's built into the operating system … is exempt … whereas the use of **proprietary encryption is not**"；Google Play "**Even if you've open sourced your application, it may still be subject to the export regulations**"；15 CFR § 740.17(b) 注「公开可得 mass market 软件在完成分类/自我分类前仍 subject to the EAR」，§ 742.15(b) 的开源豁免**只针对源码**且须通知 BIS（非标准密码）。MyLock 自实现 XChaCha20-Poly1305 → 大概率须 Apple `ITSAppUsesNonExemptEncryption=YES` + 出口合规文档（可能含 CCATS，须律师核定）；EU (EU) 2021/821 附件 I 第 5 类（本轮经 CELLAR API 取到 A 级原文）**无逐字「开源豁免」**，GSN 明文不释放 Cat 5 Part 2 软件。详见 §2.3。
- **第三方许可告知**：依赖树并非「纯宽松许可」——MPL-2.0 弱 copyleft 已存在（core 1 + desktop 6），且 179 个包本地不可核验、不作任何许可声明。绝不能对外宣称「我们没有任何 copyleft」。发布前须 `cargo fetch` 补齐、生成 `THIRD_PARTY_NOTICES.md`、按 Apache-2.0 §4 / MPL-2.0 §3 履行义务。

---

## 1. 范围、方法与可信度分级

- **产品背景（真实数据流，草案的政策据此写）**：本地优先零知识文件保险柜；XChaCha20-Poly1305 + Argon2id；**无云、无账号、无遥测、无分析、无 AI、无服务端、无回传第三方 SDK**；开发者从不接收、看不到、无法访问用户文件/口令/密钥/使用数据；Windows 把解密 vault 暴露为 loopback-only (127.0.0.1) WebDAV 盘；分发方式未定；目标市场中国 + 可能美国/欧盟。
- **方法**：三路独立 subagent 并行调研（加密出口与商店加密申报 / 隐私法适用性与商店隐私披露 / 第三方许可告知与商店政策），父代理亲抓 A 级一手页交叉验证；本地 Rust 依赖许可由 [tools/audit_licenses.py](tools/audit_licenses.py) 只读复跑。**所有结论均附来源与分级，取不到即写 NOT RETRIEVED，绝不臆造条文号/引文/URL。**
- **分级**：**A** = 亲自 HTTP 200 抓取的一手权威页 + 精确引文；**A⁻** = 一手权威法规/指引文本带 URL 但非网页直取（官方 PDF 离线抽取、忠实转载站、GitHub API base64 解码）；**B** = 二手/解读（本轮未用 B 级内容支撑结论）；**C** = 本项目自己的判断，**明确标注、不构成法律意见**。
- **抓取时点**：2026-11-15。所有合规事实均为时点值，**律师签发前须复核最新文本**。

---

## 2. 合规事实（按域，A/A⁻/B/C 逐条标注）

### 2.1 隐私法适用性：GDPR / PIPL —— 事实与「不能说的话」

**GDPR (EU) 2016/679**（⚠ 欧盟官方 eur-lex 一手页本次 **NOT RETRIEVED**：AWS WAF 返回 HTTP 202 空体 / CloudFront challenge，官网提示 "EUR-Lex is temporarily not fully available."。**以下 Art/Recital 引文均来自 gdpr-info.eu 忠实转载，标 A⁻/转载；对外正式引用前必须回 https://eur-lex.europa.eu/eli/reg/2016/679/oj 核对，不得宣称为「已核对欧盟官方原文」**）

| 条文 | A⁻/转载 逐字要点 | 对本产品的含义 |
|---|---|---|
| Art 2(1)（[art-2](https://gdpr-info.eu/art-2-gdpr/)） | "applies to the processing of personal data wholly or partly by automated means…"；Art 2(2)(c) 豁免 "by a **natural person** in the course of a purely personal or household activity" | 豁免主体是**用户本人**，条文中没有「零数据本地软件开发者豁免」；纯本机自动化处理仍落入 Art 2(1) 字面范围 |
| Art 4(2)（[art-4](https://gdpr-info.eu/art-4-gdpr/)） | "processing" 明列 "collection, recording, organisation, structuring, **storage**, **adaptation or alteration**, retrieval…"；Art 4(7) "controller" = "which… determines the **purposes and means** of the processing" | 「本机加密/存储用户文件」字面已构成处理；controller 判准是「谁决定目的与方式」，**不是「谁拿到数据」** |
| Recital 18（[no-18](https://gdpr-info.eu/recitals/no-18/)） | 前半句家庭活动豁免；**末句承重**："However, this Regulation applies to controllers or processors which **provide the means** for processing personal data for such personal or household activities." | 不能只引前半句主张开发者整体出局 |
| Recital 26（[no-26](https://gdpr-info.eu/recitals/no-26/)） | 可借助附加信息复原的假名化数据仍属可识别自然人信息；匿名信息不适用 | **加密 ≠ 匿名化**，不得据此主张 GDPR 不适用 |
| Recital 14（[no-14](https://gdpr-info.eu/recitals/no-14/)） | 处理「法人 vs 自然人」范围 | **与本地处理问题无关**，如实记录避免误引 |

**EDPB Guidelines 07/2020（controller/processor）**（落地页 [A](https://www.edpb.europa.eu/documents/guideline/guidelines-072020-on-the-concepts-of-controller-and-processor-in-the-gdpr_en) HTTP 200；[官方 PDF](https://www.edpb.europa.eu/system/files/documents/2023-10/EDPB_guidelines_202007_controllerprocessor_final_en.pdf) 引文为离线抽取还原字距，标 **A⁻**）：逐字 "**It is not necessary that the controller actually has access to the data that is being processed to be qualified as a controller.**" / "The fact that one of the parties does not have access to personal data processed is not sufficient to exclude joint controllership." / "an organisation can still be a controller even if it does not make all the decisions as to purposes and means."
**ICO, A guide to controllers and processors**（[A，HTTP 200](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/controllers-and-processors/controllers-and-processors-a-guide/)）："The key question is – **who determines the purposes** for which the data are processed and **the means** of processing?… Controllers will be controllers **regardless of how they are described in any contract**."

**PIPL 中华人民共和国个人信息保护法**（[中国人大网，A，HTTP 200](http://www.npc.gov.cn/npc/c2/c30834/202108/t20210820_313088.html)，逐字中文原文）：

| 条文 | A 级逐字要点 | 含义 |
|---|---|---|
| 第三条 | 境内处理适用；境外处理「（一）**以向境内自然人提供产品或者服务为目的**」亦适用 | 即使开发者在境外、无数据回传，仅「向中国用户提供产品」即可能落入范围（条文字面，是否构成需法务判断，C） |
| 第四条 | 「个人信息的处理包括个人信息的收集、**存储**、使用、**加工**、传输、提供、公开、删除等」；「不包括匿名化处理后的信息」 | 「无收集/传输/提供」**不能否认「处理」存在**；加密文件不属于「匿名化后」 |
| 第十三条 | 合法性基础条款，主语「个人信息处理者」 | 「无收集」不等于「无需基础」（C） |
| 第十七条 | 「个人信息处理者在处理个人信息前，应当以显著方式…真实、准确、完整地向个人告知…」 | 若被认定为处理者，告知义务有明文 |
| 第五十一条 | 含「采取相应的**加密**、去标识化等安全技术措施」 | 加密是法定安全措施之一，可作事实写入政策，**不能等同合规结论** |
| 第七十二条 | 「**自然人因个人或者家庭事务**处理个人信息的，不适用本法」 | 豁免主体明文是「自然人」（用户本人），**不是公司/开发者** |
| 第七十三条(一)(四) | 处理者 =「**自主决定处理目的、处理方式**的组织、个人」；匿名化 =「无法识别特定自然人且**不能复原**」 | 判准同 GDPR；用户持口令即可复原 ⇒ 加密 vault 内仍是个人信息 |

**C 级事实层面回答（本项目判断，非法律意见）**：一手条文/指引中**没有**任何「零收集纯本地 App 的开发者一概不是 controller / 一概不适用 GDPR·PIPL / 一概无需隐私政策」的规定；相反 EDPB、Recital 18 末句、PIPL 第 72/73 条都指向相反方向。同时也**存在相反论据**（若开发者对用户放哪些文件、为何放、存多久完全不决定，则按 Art 4(7) / PIPL 第 73 字面，决定目的与方式的是用户本人）。**两种读法都有文本依据，本次未取到任何一手文件对「本地零知识软件开发者」给出明确定性** ⇒ 必须交由律师判断，草案中绝不写绝对结论。GDPR Art 12–14、Art 32 本轮未逐条取到，故不引条号、不下结论。

### 2.2 商店隐私披露 —— 零收集也必须做（A 级，决定性）

| 渠道 | A 级一手逐字 | 结论 |
|---|---|---|
| Apple 5.1.1(i)（[guidelines](https://developer.apple.com/app-store/review/guidelines/)） | "**All apps must include a link to their privacy policy** in the App Store Connect metadata field **and within the app** in an easily accessible manner. The privacy policy must clearly and explicitly: **Identify what data, if any**, the app/service collects… **Explain its data retention/deletion policies** and describe how a user can revoke consent and/or request deletion…" | 零数据 App 仍须在**元数据 + App 内**双处放链接；政策仍须写保留/删除政策（答案是「无数据、用户本机自行删除」也要写） |
| Apple App Privacy Details（[app-privacy-details](https://developer.apple.com/app-store/app-privacy-details/)） | "**"Collect" refers to transmitting data off the device** in a way that allows you and/or your third-party partners to access it for a period longer than what is necessary to service the transmitted request in real time." / "You need to identify all of the data you or your third-party partners collect…" | 纯本机 + loopback-only 按此定义**不构成 collect**；无第三方 SDK ⇒ 第三方共享项为「无」 |
| App Store Connect Help（[manage-app-privacy](https://developer.apple.com/help/app-store-connect/manage-app-information/manage-app-privacy/)） | "If the answer is no, select **"No, we do not collect data from this app"** and then click Save. You don't need to answer any further questions." | 「不收集」是受支持的一手选项。⚠ 字面标签 "Data Not Collected" 在这些 Apple 页上 **NOT RETRIEVED**，勿当作一手引文 |
| Google Play 准备送审（[9859455](https://support.google.com/googleplay/android-developer/answer/9859455?hl=en)） | "You must link to a privacy policy on your store listing and within your app, **even if your app doesn't access personal or sensitive user data or target children**… consult your legal representative"；政策须 "active, publicly accessible URL… valid SSL certificate… accessible worldwide without geographical restrictions, geofencing, or login requirements… standard, non-editable webpage (not a PDF)… applies specifically to your app" | 零数据仍须双处链接；政策载体有硬要求 |
| Google Play Data safety（[10787469](https://support.google.com/googleplay/android-developer/answer/10787469?hl=en)） | "**All developers must declare** how they collect and handle user data…" / "**Even developers with apps that do not collect any user data must complete this form and provide a link to their privacy policy.** In this case, the completed form and privacy policy can indicate that **no user data is collected or shared**." / "You alone are responsible for making complete and accurate declarations" | 表单必填、须先有政策链接；可如实选「无」；准确性由开发者独担 |
| Google Play User Data policy（[10144311](https://support.google.com/googleplay/android-developer/answer/10144311?hl=en)） | "Apps that do not access any personal and sensitive user data **must still submit a privacy policy**."；政策须含：开发者信息 + 隐私联系点/咨询机制、应用名或商店实体名、数据类型披露（含「无」）、共享方（含「无」）、安全处理措施、保留与删除政策、标题标注 "privacy policy"；"Don't omit any user or device data handling" / "Don't provide misleading or inaccurate information" | 零数据政策**仍有强制内容清单**；「无」也必须明示，不得留白 |
| Google Play 账号删除（[13327111](https://support.google.com/googleplay/android-developer/answer/13327111?hl=en)） | "If your app **enables account creation**, you must: provide users with an in-app path… and provide a web link resource…" / "**All developers must complete new Data deletion questions** in the Data safety form" / "**accounts that are created and operated offline are not app accounts** and do not fall within policy scope" | MyLock **无账号** ⇒ in-app/网页删除重义务按字面不触发；但 Data safety 的 Data deletion 问题仍须如实回答「无账号、数据仅在用户本机、由用户自行删除」 |
| Developer Policy Center（[play.google](https://play.google/intl/en_us/developer-content-policy/)） | 含 "User Data" / "Data safety" 分节；"all developers must provide data deletion options for their Data safety section" | ⚠ 独立 "Privacy Policy" 分节原文 NOT RETRIEVED；详细要求以 Play Console Help 为准 |

> Google Play 页 `web_fetch` 一律 `TypeError: fetch failed`、pwsh `Invoke-WebRequest` 超时，最终由 ego-browser 真实浏览器加载后取 `body.innerText`（A）。Apple "Collect" 定义由父代理 `web_fetch` 亲抓（A），与子代理独立结果一致。

### 2.3 加密出口管制与商店加密申报（独立 subagent 最终报告已并入）

> **重要更正（A，EU 原文）**：附件 I 第 5 类第 2 部分的实际标题是 **Part 2 – "INFORMATION SECURITY"**（目录 "Part VII – Category 5 Telecommunications and "information security""）；"information assurance" 并非 (EU) 2021/821 用语，下文按原文用语。全部为事实调研，不构成法律意见；C 项均为判断。

#### 2.3.1 美国 EAR / BIS（A⁻，Cornell LII 的 e-CFR 法规原文镜像 HTTP 200，非 .gov 官方站；[BIS 官网](https://www.bis.gov/) HTTP 200 A⁻ 仅确认 /regulations/ear 等入口，条文正文未从该站引到）

| 条文 | A⁻ 逐字要点 | 对本产品的含义 |
|---|---|---|
| [15 CFR § 742.15(a)(1)](https://www.law.cornell.edu/cfr/text/15/742.15) | "A license is required to export or reexport encryption items ("EI") classified under ECCN 5A002, 5A004, **5D002.a, .c.1 or .d** … or **5E002** … to all destinations, **except Canada**. … Most encryption items may be exported under … **License Exception ENC** … § 740.17 … Following classification or self-classification, items that meet … Note 3 to Category 5—Part 2 (the "**mass market**" note), are classified under **5A992 or 5D992** and are no longer subject to this Section" | 若归入 5D002，除加拿大外原则上需许可，多数可走 ENC；符合 mass market 注的经分类/自我分类改列 5D992 |
| [§ 740.17](https://www.law.cornell.edu/cfr/text/15/740.17) | "License Exception ENC authorizes export, reexport, and transfer (in-country) of … **software and technology … classified under 5D002 or 5E002** …"；"Items … that meet … the "**mass market**" note are classified under … **5D992 following self-classification or classification by BIS** and are no longer subject to "EI" and "NS" controls." | 5D002/5E002 明确在 ENC 覆盖内；存在「自我分类」与「BIS 分类」两条路径 |
| § 740.17(b) 开头注（同 URL） | "Mass market encryption software that would be considered publicly available under § 734.3(b)(3) … **remains subject to the EAR until all applicable classification or self-classification requirements … are fulfilled**." | **免费/公开下载/开源 ≠ 自动豁免**；完成分类义务前仍 subject to the EAR |
| § 740.17(b)(1)（同 URL） | "Immediate authorization… subject to submission of a **self-classification report** in accordance with § 740.17(e)(3)" | mass-market 即时授权无需事先 CCATS，但须提交年度自我分类报告 |
| § 740.17(b)(2)(i)(B)(C) / (b)(3)(ii) / (d)(1)(iii) / (d)(2)(ii)(B)（同 URL） | "Certain "**encryption source code**" … not publicly available"; "**Custom or changeable cryptography**. The cryptographic functionality … has been designed or modified to customer specification or **can be easily changed by the user**"; ""**Non-standard cryptography**" … that provide or perform "non-standard cryptography" as defined in part 772"; "A new product encryption classification request … is required if a change is made to the cryptographic functionality (e.g., **algorithms**)"; "BIS will issue a **Commodity Classification Automated Tracking System (CCATS)** to you." | **自实现/非标准密码、可变更密码功能、非公开源码**列入需 30 天正式分类（→CCATS）路径；算法一变即须重新提交分类请求 |
| § 740.17(e)(3) / (e)(1)(iii)(C)（同 URL） | "must be received by BIS and the ENC Encryption Request Coordinator **no later than February 1 the following year**"；报告排除项 "Encryption items exported … via **free and anonymous download**" | 自我分类报告最迟次年 2 月 1 日送达 BIS；免费匿名下载可从报告排除，但**不改分类义务本身** |
| [§ 742.15(b)(1)(2)](https://www.law.cornell.edu/cfr/text/15/742.15) | "Subject to the notification requirements … **publicly available … encryption source code classified under ECCN 5D002 is not subject to the EAR**"；"For publicly available encryption source code … that provides or performs "**non-standard cryptography**" … you must **notify BIS and the ENC Encryption Request Coordinator via email** of the internet location … or provide each of them a copy" | 唯一明确的「开源豁免」只针对**源码**且须满足 § 742.15(b)；涉非标准密码仍须 BIS 邮件通知源码位置；**闭源专有发行不落入此豁免** |
| [§ 734.7](https://www.law.cornell.edu/cfr/text/15/734.7) | (a)(4) "Public dissemination (i.e., unlimited distribution) … including **posting on the Internet**"；(b) "**Published encryption software … remains subject to the EAR unless** it is publicly available encryption **object code** … **and the corresponding source code** meets … § 742.15(b)" | 仅发布二进制/对象码不够，必须「对应源码」也满足 § 742.15(b) |
| [§ 734.3](https://www.law.cornell.edu/cfr/text/15/734.3) | (b)(3) "Information and "software" that … Are published …"；注 "encryption source code **in electronic form or media … remains subject to the EAR**"；"Publicly available encryption object code … is not subject to the EAR **when the corresponding source code** meets … § 742.15(b)"；(d) "Commodity classification determinations … **are not, and may not be relied upon as**, determinations that the items … are "subject to the EAR"" | 电子形式加密源码仍受 EAR；**CCATS 分类结论不能反推「是否 subject to the EAR」** |
| [§ 734.4](https://www.law.cornell.edu/cfr/text/15/734.4) | (a)(2) "Foreign produced encryption technology that incorporates U.S. origin encryption technology controlled by ECCN 5E002 is subject to the EAR **regardless of the amount of U.S. origin content**"；(b)(1)(i) 仅满足 742.15(b) 的公开源码 "does not have to be counted … in a de minimis calculation" | 对 5E002 加密技术**无 de minimis 门槛** |
| [§ 772.1](https://www.law.cornell.edu/cfr/text/15/772.1) | "Category … (5) Telecommunications and "**Information Security**""；"CCL Group … (D) Software; and (E) Technology"；"Asymmetric algorithm … A common use … is **key management**" | 5D002=软件(D 组)、5E002=技术(E 组)；Cat 5 Part 2 即 "Information Security"；MyLock 的 Argon2id + AES-256-KW 密钥封装与 XChaCha20-Poly1305 属该范畴（C） |

- **NOT RETRIEVED**：[15 CFR part 774 Supplement No. 1](https://www.law.cornell.edu/cfr/text/15/774) → HTTP 404 "Page not found"，故 **ECCN 5D002 / 5E002 条目原文（"Items Controlled" 清单、"cryptography for data confidentiality" / "described security algorithm" 定义）NOT RETRIEVED**；§ 772.1 页正文在定义条目处截断，"non-standard cryptography" 逐字定义 NOT RETRIEVED（仅有 Apple 官方页转述的 US Government 定义，见 2.3.2）。编号本身在多条 CFR 中被反复引用（A⁻）。
- **C 级判断（非法律意见）**：基于上述，① 自实现 XChaCha20-Poly1305（非 OS 加密 API）属 Apple 明确列举的 "Proprietary or non-standard encryption algorithms" 及 § 740.17(b)(3)(ii) 的 "non-standard cryptography"，**倾向于不走 (b)(1) mass-market 自我分类即时授权，而走 (b)(2)/(b)(3) 的 30 天分类请求 → CCATS 路径**；② "publicly available / 开源豁免"只在**源码**公开且满足 § 742.15(b) 时成立，且 § 740.17(b) 注明确公开可得 mass market 软件在完成分类/自我分类前仍受 EAR；③ 加密技术无 de minimis 豁免；④ 免费匿名下载仅可排除年度自我分类报告，分类义务不消失。**须由合格出口合规律师按实际分类核定。**

#### 2.3.2 Apple App Store Connect 出口合规（A，官方 JSON / Help 页）

| 条目 | A 级逐字 | 含义 |
|---|---|---|
| [`ITSAppUsesNonExemptEncryption` Info.plist 键](https://developer.apple.com/tutorials/data/documentation/bundleresources/information-property-list/itsappusesnonexemptencryption.json) | "A Boolean value indicating whether the app uses encryption."；"Set the value … to **NO** … to indicate that your app—including any third-party libraries you link against—either uses no encryption, or only uses encryption that's **exempt** … Set the value to **YES** to indicate that your app uses **non-exempt encryption**."；"If you set … YES, you typically also provide a value for the **`ITSEncryptionExportComplianceCode`** key … using a code Apple provides after successfully reviewing your export compliance documentation."；"If you don't have the … key … App Store Connect walks you through an export compliance questionnaire **every time** you upload a new version" | 必须在 Info.plist 声明；未设置则每次上传都要走问卷 |
| [Complying with Encryption Export Regulations](https://developer.apple.com/tutorials/data/documentation/security/complying-with-encryption-export-regulations.json) | "When you submit your app to TestFlight or the App Store, you upload your app to a server in the **United States**. If you distribute your app outside the U.S. or Canada, your app is **subject to U.S. export laws, regardless of where your legal entity is based**."；"Typically, the use of encryption that's **built into the operating system**—for example … HTTPS connections using URLSession—is **exempt** from export documentation upload requirements, whereas the use of **proprietary encryption is not**."；"If your app uses exempt forms of encryption, you might alternatively be required to submit a **year-end self-classification report** to the U.S. government. (If you use non-exempt encryption and provide documentation to Apple, the self-classification report isn't necessary.)" | **决定性**——MyLock 用自实现加密而非 OS 加密 API，**不能主张 OS 内置加密豁免**，落入须提供文档/合规码一侧（C） |
| [Overview of export compliance](https://developer.apple.com/help/app-store-connect/manage-app-information/overview-of-export-compliance/) | "Examples of apps requiring an export compliance determination include … **Standard encryption algorithms. Crypto functionality within Apple's operating system. Proprietary or non-standard encryption algorithms.**"；"The US Government defines "**non-standard cryptography**" as any implementation … involving the incorporation or use of **proprietary or unpublished cryptographic functionality**, including encryption algorithms or protocols that have not been adopted or approved by a duly recognized international standards body (e.g., IEEE, IETF, ISO, ITU, ETSI, 3GPP, TIA, and GSMA) and haven't otherwise been published."；"it's **your responsibility** to review the Export Administration Regulation to determine whether your app's use of encryption requires a formal classification (**CCATS**) from BIS. you're **responsible for all liabilities associated with misinterpretation of export regulations or claiming exemption inaccurately**."；"The import and export of encryption apps distributed in **France** are also controlled by the French Government. The main items of control for France are **Secure Storage**, Secure Communications, and Security Anti-Virus applications. Exemptions include Banking and Medical applications." | Apple 把「专有/非标准算法」与「标准算法/OS 加密」并列为**都需**做出口合规判定；误判豁免责任自负；法国对 "Secure Storage" 类加密 App 有独立管制（MyLock 本地文件保险柜可能接近，C） |
| [Determine and upload app encryption documentation](https://developer.apple.com/help/app-store-connect/manage-app-information/determine-and-upload-app-encryption-documentation/) | "If you're required to upload app encryption documentation … This should be completed **before submitting a build for review** … Without this information, Apple can't determine whether the app encryption documents are sufficient."；"Apple evaluates export compliance reviews **on a case-by-case basis** … approximately **two business days**. Once your documentation is approved, Apple will provide you a **key value**. You must enter this value in Xcode to avoid answering encryption questions with each app submission." | 文档须在送审前上传；批准后给 key value（即 `ITSEncryptionExportComplianceCode`）写入 Info.plist；逐案评估约 2 工作日 |

- **NOT RETRIEVED**："ERC" / "Encryption Registration Certificate" / 豁免类别清单的精确措辞在两份 Apple Help 页与已取 CFR 页中**均未出现**；"Learn about the app encryption documentation you may need to provide" 指向的具体豁免类别/表格清单页正文 NOT RETRIEVED；`https://help.apple.com/app-store-connect/#/dev88f5c7bf9` 为 HTTP 200 空体 SPA；两份 dead-end Help 页渲染 "Page Not Found - Apple Developer"。
- **C 级判断（非法律意见）**：结合 2.2「proprietary encryption is not [exempt]」+ 2.3「Proprietary or non-standard encryption algorithms」+ § 740.17(b)(3)(ii)「non-standard cryptography → 分类请求」，MyLock 在 Apple 侧**大概率需设 `ITSAppUsesNonExemptEncryption=YES`、提供出口合规文档（可能含 CCATS）并取得 `ITSEncryptionExportComplianceCode`**。此为判断，不是事实。

#### 2.3.3 Google Play（A，ego-browser 真实浏览器加载）

- [Export compliance – Play Console Help](https://support.google.com/googleplay/android-developer/answer/113770?hl=en) 逐字："Your application will be hosted on **Google servers. Google Inc. is a US company**, and the United States government considers it an **export** when someone outside the US downloads software from our servers. Because of this, your application **may be subject to US export laws** … **even if you're not located in the US** or a US national."；"**Even if you've open sourced your application, it may still be subject to the export regulations.** Your application may make use of encryption. Even if your application doesn't contain a crypto library, it may **call up crypto functionality in another program**. Under the US Commerce Department's regulations, there are different categories of encryption software (like publicly available, authentication, digital signature, **mass market**, and ancillary), and different rules apply to each type."；"Google **blocks downloads to these [embargoed] countries**." → **决定性：开源 ≠ 豁免**；类别区分规则适用；禁运国下载由 Google 屏蔽。
- [Data safety](https://support.google.com/googleplay/android-developer/answer/10787469?hl=en) 问的是「数据收集/处理与传输中加密」（"All developers must declare how they collect and handle user data … and provide details about how they protect this data through security practices like encryption"），**不是出口法申报**；MyLock 无云、无遥测、loopback-only，相关声明内容可能很少（C）。
- **NOT RETRIEVED**："Is your app subject to U.S. export laws?" / "Does your app use encryption?" 精确问卷措辞未在一手页验证（Data safety 页检索该四类措辞全部 -1）；[9859455 准备送审](https://support.google.com/googleplay/android-developer/answer/9859455?hl=en) 正文无 "export"/"encrypt" 提及。`support.google.com` 从 `web_fetch` / pwsh 均超时，**只有 ego-browser 能加载**。

#### 2.3.4 Microsoft Store（A⁻ + NOT RETRIEVED）

- [Set app pricing and availability for MSIX app](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/price-and-availability)（A⁻，Microsoft Learn 一手页 HTTP 200，Last updated 2024-07-23）：含 Markets（"over 240 countries and regions … By default, we'll offer your app in all possible markets."）、Visibility、Pricing 等；**全文未出现 encryption / export compliance 申报措辞**。
- **NOT RETRIEVED**：未取到任何 Microsoft 一手页明确陈述「App 需/不需加密出口声明」。无效入口 `.../set-app-pricing-and-availability`（含 `?source=recommendations`）→ HTTP 404；`.../windows/uwp/publish/set-app-pricing-and-availability` → 重定向到上述页。**「在该页未见」不等于「确定不存在」**。

#### 2.3.5 中国（A 级一手，已取到 + 本轮补充）

- 《商用密码管理条例》（国务院令第 760 号，2023-04-27 修订、2023-07-01 施行）[gov.cn](https://www.gov.cn/zhengce/zhengceku/202305/content_6875928.htm)：第三十一条「涉及国家安全、社会公共利益且具有加密保护功能的商用密码，列入商用密码进口许可清单…列入商用密码出口管制清单…」「**大众消费类产品所采用的商用密码不实行进口许可和出口管制制度。**」；第二条把境内「应用」纳入适用范围，商用密码定义为「采用特定变换的方法对不属于国家秘密的信息等进行加密保护、安全认证的技术、产品和服务」；第十一条「应当符合有关法律、行政法规、商用密码**强制性国家标准**，以及自我声明公开标准的技术要求」；第二十条列入网络关键设备/网络安全专用产品目录的须检测认证合格方可销售；第三十五条鼓励性条款；第三十八/三十九条强制检测认证/算法审查鉴定仅针对**关键信息基础设施**；第四十一条强制用密码义务指向「**网络运营者**」；第四十七条「…**不得要求**…披露源代码等密码相关专有信息」。
- 《中华人民共和国密码法》[oscca.gov.cn](https://www.oscca.gov.cn/sca/xxgk/2023-06/04/content_1057225.shtml)：第二十八条同「大众消费类产品…不实行进口许可和出口管制」；第八条「公民、法人和其他组织可以依法使用商用密码保护网络与信息安全」；第二十六条列目录产品须检测认证；第三十一条不得要求披露源代码；第四十四条 2020-01-01 施行。全文未见对普通消费产品强制「必须使用国密/核准算法」的要求（C）。
- 《中华人民共和国网络安全法》[oscca.gov.cn](https://www.oscca.gov.cn/sca/xxgk/2016-11/07/content_1012647.shtml)（A，**正文在第 35 条处截断**）：第二条「境内建设、运营、维护和使用网络…适用本法」；第十条「建设、运营网络或者通过网络提供服务…采取技术措施」；第二十一条等级保护制度含「（四）采取数据分类、重要数据备份和**加密**等措施」；第二十三条网络关键设备须安全认证/检测。→ MyLock 无云、无账号、无服务端、仅 127.0.0.1 loopback、非网络运营者、非关基，关联弱；但第二条「境内…使用网络」措辞宽泛，属需律师核定的边界问题（C）。
- **B（官方站转载专家解读，非法规原文）**：[陈卫东（对外经贸大学教授）解读](https://www.oscca.gov.cn/sca/xxgk/2023-06/04/content_1061070.shtml) 称「《密码法》第二十八条…大众消费类产品所采用的商用密码不实行进口许可和出口管制。由此，"大众消费类产品"获得了更为广阔的贸易自由空间」；并描述「目前我国仅对加密电话机、加密传真机、高性能密码机（密码卡）和高性能加密 VPN 设备四类…进口许可」，出口管制「范围要更宽泛…涵盖安全芯片、密码机…**密钥管理产品**…软件等测试、检查和生产设备等」。→ 解读把清单描述为覆盖「软件」并提及「密钥管理产品」；MyLock 含 AES-KW 密钥封装与 Argon2id，**是否落入须以清单原文核定**（清单原文 NOT RETRIEVED）。
- **C 级判断（非法律意见）**：纯本地、无网络、面向个人消费者的加密 App，进出口许可条款有明确「大众消费类产品」豁免、且无「必须使用国密/核准算法」强制；但「大众消费类产品」界定边界 + 强制性国家标准符合性 + 两份管制清单原文**仍属开放项**，须法务核定，不得据此直接下合规结论。
- **中国 NOT RETRIEVED**：《出口管制法》正文（npc.gov.cn 重定向到首页 / SSL 失败；gov.cn 候选 URL 一为无关量子新闻、一为 404）；《商用密码进口许可清单》《商用密码出口管制清单》原文；「大众消费类产品」法定定义/判定标准；数据安全法 / PIPL / 关基条例正文（flk.npc.gov.cn 为 JS 空壳，PIPL 另由 npc.gov.cn A 级取到）；司法部答记者问（跨域重定向未跟随）。

#### 2.3.6 欧盟 (EU) 2021/821 附件 I 第 5 类（A，本轮经 CELLAR API 新取到）

> 取法：EUR-Lex CELEX HTML 被 AWS WAF JS challenge 拦截（web_fetch / pwsh / 真实 ego-browser 一律 HTTP 202 空体或重定向到 TodayOJ 首页）→ 改走欧盟出版局 **CELLAR API**：元数据对象 `https://publications.europa.eu/resource/celex/32021R0821`（Accept: `application/xml; notice=object`，HTTP 200，561,243 B，列出 110 个 manifestation，后缀 `.0006` 为 `eng,en`）→ **英文正文** `http://publications.europa.eu/resource/cellar/d8e79ebf-ca4e-11eb-84ce-01aa75ed71a1.0006`（Accept: `application/xhtml+xml`，**HTTP 200，5,909,234 B**，对应 `OJ L 206, 2021`）〔A〕。规范链接 `https://eur-lex.europa.eu/eli/reg/2021/821/oj/eng` 本环境未成功抓取，仅作规范引用。

| 条目 | A 级逐字要点 | 含义 |
|---|---|---|
| 标题 | 目录 "Part VII - Category 5 Telecommunications and "**information security**""；正文 "**Part 2 - "INFORMATION SECURITY"**"（其后 "Note 1: Not used."、"Note 2: Category 5, Part 2 does not control products when accompanying their user for the user's **personal use**."） | 附件 I 明确管控「信息安全」类含加密软件；**"information assurance" 非本条例用语**；Note 2 给出「用户随身自用产品」不管控 |
| **5D002** | "5D002 "Software" as follows: a. "Software" specially designed or modified for the "development", "production" or "use" of … Equipment specified in **5A002** … b. … 'cryptographic activation token' … c. "Software" having the characteristics of, or performing or simulating the functions of … **1. Equipment specified in 5A002.a., 5A002.c., 5A002.d. or 5A002.e.**; **Note: 5D002.c.1. does not control "software" limited to the tasks of "OAM" implementing only published or commercial cryptographic standards.** …" | 含加密功能软件（5D002）在管控清单内；5D002.c.1 注仅豁免「OAM 且仅实现已发布/商业密码标准」的软件 |
| **5E002** | "5E002 "Technology" … for the "development", "production" or "use" of equipment specified in 5A002, 5A003, 5A004 or 5B002, or of "software" specified in 5D002.a. or 5D002.c. … **Note: 5E002 includes "information security" technical data resulting from procedures carried out to evaluate or determine the implementation of functions, features or techniques specified in Category 5, Part 2.**" | 相应技术（5E002）亦在清单内 |
| **5A002 + Cryptography Note (Note 3)** | "5A002 … a. Designed or modified to use '**cryptography for data confidentiality**' having a '**described security algorithm**', where that cryptographic capability is usable, has been activated, or can be activated by any means other than secure "cryptographic activation" …"；**Note 3** 排除须**同时**满足 "1. **Generally available to the public** by being sold, without restriction, from stock at retail selling points … 2. **The cryptographic functionality cannot easily be changed by the user**; 3. Designed for **installation by the user without further substantial support by the supplier**; and 4. … details of the goods are accessible and will be provided, upon request, to the competent authorities …"；Note 3(b) 另要求 "**"Information security" is not the primary function** … does not change any cryptographic functionality … feature set … is fixed and is not designed or modified to customer specification"；Note to the Cryptography Note "the item is of potential interest to a **wide range of individuals and businesses** … price and information about the main functionality … available before purchase" | MyLock **以「信息安全」为主要功能且密码功能是核心**，落入 Note 3(b) 排除条件反面（C）⇒ **不能简单援引 Cryptography Note**；EU 原文**没有 "mass market" 字样**（检索 -1），对应概念是 "Generally available to the public" |
| **GENERAL SOFTWARE NOTE / GISN** | "Categories 1 to 9 … do not control "software" which is … a. **Generally available to the public** … **Note: Entry a. of the General Software Note does not release "software" specified in Category 5, Part 2 ("Information Security").** … c. The minimum necessary "object code" … **Note: Entry c. … does not release "software" specified in Category 5, Part 2**."；"GENERAL "INFORMATION SECURITY" NOTE (GISN) "Information security" items or functions should be considered against … Category 5, Part 2, **even if they are components, "software" or functions of other items**."；"Controls on "technology" transfer do not apply to information "**in the public domain**" or to "**basic scientific research**"." | 与 US EAR 不同，EU 的 GSN **明文不释放** Cat 5 Part 2 软件的零售可得（entry a）与最小对象码（entry c）两项；GISN 要求一律按 Cat 5 Part 2 判定；可用去管制概念只有 "in the public domain" 与 "basic scientific research"。原文检索 "publicly available" / "open source" / "open-source" **全部 -1 ⇒ EU 该条例不存在逐字的「公开可得/开源豁免」措辞** |
| 法国（来自 [Apple 2.3.2](https://developer.apple.com/help/app-store-connect/manage-app-information/overview-of-export-compliance/)，A） | "The import and export of encryption apps distributed in France are also controlled … main items of control … **Secure Storage**, Secure Communications, and Security Anti-Virus … Exemptions include Banking and Medical applications." | MyLock 本地文件保险柜可能接近 "Secure Storage"（C），EU 商店分发须另核法国 ANSSI 管制 |

- **NOT RETRIEVED**：EU 附件 I 中 "**in the public domain**" 的逐字定义（取到引用该词的 GSN/GTN 条文，定义条目原文未单独定位）；EUR-Lex CELEX HTML/PDF 全文（本环境 AWS WAF 拦截，法规英文正文改由 CELLAR manifestation 取到）。

#### 2.3.7 对 MyLock 的 5 条决定性事实（A/A⁻ 事实 + C 判断，均不构成法律意见）

1. **US（A⁻+A）**：自实现/专有加密**不**被视为 OS 内置加密豁免。Apple 一手页 "the use of encryption that's built into the operating system … is exempt from export documentation upload requirements, whereas the use of **proprietary encryption is not**"；§ 740.17(b)(3)(ii) 把 "non-standard cryptography" 列入需 30 天分类请求（→CCATS）的路径，而 (b)(1) mass-market 自我分类即时授权须附年度自我分类报告（次年 2 月 1 日前）。→ 分发路径（商店 vs 直接下载）、开源 vs 闭源会改变美国侧义务，但**「开源即豁免」不成立**。
2. **US（A⁻）**：公开可得/开源豁免只针对**源码**且须满足 § 742.15(b)："publicly available … encryption source code … is not subject to the EAR"；但 § 734.7(b) 与 § 734.3 注明确「电子形式加密源码仍受 EAR」「仅对象码公开不够」；§ 740.17(b) 注「公开可得 mass market 软件在完成分类/自我分类前仍受 EAR」；§ 734.4(a)(2) 对 5E002 加密技术无 de minimis 豁免。→ 闭源专有发行无法援引源码豁免；若开源，仍须评估 § 742.15(b)(2) 对非标准密码的 **BIS 邮件通知义务**。
3. **商店侧（A）**：Apple 需 Info.plist 键 `ITSAppUsesNonExemptEncryption`（YES/NO）+ 可能的 `ITSEncryptionExportComplianceCode`（批准后的 key value），文档须在送审前上传、逐案审核约 2 工作日；Google Play 一手页明确 "**Even if you've open sourced your application, it may still be subject to the export regulations**" 并按加密类别区分规则；Microsoft Store 官方提交选项页未见加密出口声明字段（该声明本身 NOT RETRIEVED）。→ **三大商店的合规问卷独立于「产品是否开源」**。
4. **中国（A）**：条例第三十一条与密码法第二十八条均有同一句精确原文「大众消费类产品所采用的商用密码不实行进口许可和出口管制制度」；第十一条要求符合强制性国家标准/自我声明公开标准；第三十五/四十七条为鼓励使用检测认证合格密码、不得要求披露源代码；全文未见对普通消费级产品强制「核准/国密算法」（C）。→ 关键未决事实是「大众消费类产品」法定界定与**两份管制清单原文**（均 NOT RETRIEVED），这决定 MyLock 是否落入清单。
5. **EU（A，本轮新取到）**：附件 I 第 5 类第 2 部分标题为 "INFORMATION SECURITY"（非 "information assurance"），5D002/5E002 明确管控含加密软件与技术；GSN 明文「不释放」Cat 5 Part 2 软件的零售可得与最小对象码两项，GISN 要求一律按 Cat 5 Part 2 判定；唯一可用去管制概念是 "in the public domain" 与 "basic scientific research"；Cryptography Note (Note 3) 的排除要求「密码功能用户不可轻易更改 + 信息安全非主要功能」等条件。→ **EU 无逐字「开源豁免」**，且 MyLock 以信息安全为主要功能（C），不能简单套用大众市场排除。

#### 2.3.8 本产品必须做的分类动作（C 判断，动作清单见 [prelaunch-checklist.md](prelaunch-checklist.md) §3.1）

- **产品侧事实（供法务分类用，非法律结论）**：本产品使用**自研集成的 XChaCha20-Poly1305（非 FIPS/国密核准算法，draft-irtf-cfrg-xchacha 非 IETF 标准）**、AES-256-KW、Argon2id、HKDF-SHA256、HMAC-SHA256；含 AES-KW 密钥封装（「密钥管理」相关）。
- **不得**擅自套用 mass-market / exempt 例外；**不得**在 Apple `ITSAppUsesNonExemptEncryption` 等申报项上凭猜测填「否」；**不得**把「零数据收集 / 无网络 / 开源」当作出口合规豁免依据。
- 须由律师/合规出具**书面分类结论**：BIS ECCN 5D002 / 5E002 与 License Exception ENC 路径、是否需 CCATS、年度自我分类报告义务、Apple `ITSAppUsesNonExemptEncryption` + `ITSEncryptionExportComplianceCode`、Google Play Export compliance、EU 2021/821 附件 I 第 5 类 + 法国 ANSSI、中国「大众消费类产品」界定与强制性国标符合性、Microsoft Store 声明。
- **算法变更（例如未来替换 XChaCha20-Poly1305）即触发 § 740.17(d)(1)(iii) 的「新分类请求」义务**，不得把当前分类视为永久有效。

### 2.4 第三方许可告知义务（父代理亲抓 A 级一手原文）

- **Apache-2.0**（[LICENSE-2.0，HTTP 200](https://www.apache.org/licenses/LICENSE-2.0)，A）：§4 Redistribution 逐字四项义务 = "1. You must give any other recipients of the Work or Derivative Works a **copy of this License**; and 2. You must cause any **modified files** to carry prominent notices stating that You changed the files; and 3. You must **retain**… all copyright, patent, trademark, and attribution notices…; and 4. If the Work includes a **"NOTICE" text file**… any Derivative Works that You distribute must include a readable copy of the attribution notices contained within such NOTICE file…"；"Only the English text of the Apache License is authoritative."
- **MPL-2.0**（[mozilla.org/MPL/2.0/，HTTP 200](https://www.mozilla.org/en-US/MPL/2.0/)，A）：§1.12 "Secondary License" 定义**字面包含** "GNU General Public License, Version 2.0, the GNU Lesser General Public License, Version 2.1, the GNU Affero General Public License, Version 3.0" —— **这正是 [tools/audit_licenses.py](tools/audit_licenses.py) 全局扫描里 4 个 LICENSE 文件命中 "GPL" 字样、却被判定为误报的原因**；§3.1 源码形式分发须 "under the terms of this License" 并告知接收者如何取得许可证；§3.2 可执行形式须让源码「以合理方式及时可得」；§3.3 Larger Work "under terms of Your choice, **provided that You also comply with the requirements of this License for the Covered Software**"；§3.4 不得移除/改动许可声明；Exhibit A 标准头。
- **本地依赖审计（tools/audit_licenses.py EXIT=0 复跑）**：core `Cargo.lock` 147 包（134 可核验 / 13 不可核验，后者全为 wasm32/wasi/uefi/unix/android 非 Windows 分支）、desktop 466 包（300 可核验 / 166 不可核验）；**可核验范围 GPL 家族 0**；但 **MPL-2.0 弱 copyleft 已存在**（core 1 个 `htmlescape 0.3.1` = "Apache-2.0 / MIT / MPL-2.0"；desktop 6 个，本地核验明确点名的有 `cssparser 0.37.0`、`cssparser-macros 0.7.1`、`dtoa-short 0.3.5`、`option-ext 0.2.0`、`selectors 0.38.0` 共 5 个 —— **审计口径为 6 个但仅列出 5 个名称，第 6 个须 `cargo fetch` 补齐后核对，不臆造**）；0 个 `SPDX-License-Identifier: GPL*`；第一方四个 crate 均 `license = "MIT OR Apache-2.0"`；1924 个展开文本文件扫描、Cargo.toml 中 GPL 短语 0、LICENSE/README 样式文件中 4 个（均为 MPL §1.12 的 Secondary License 字样）。**179 个本地不可核验包不作任何许可声明，最终法务签发前须 `cargo fetch` 补齐。**

### 2.4.1 第三方许可告知 A 级条文补遗（独立 subagent 已并入）

**MIT（A）**：[SPDX MIT.html](https://spdx.org/licenses/MIT.html) / [choosealicense](https://choosealicense.com/licenses/mit/) / [OSI](https://opensource.org/license/mit) 逐字 "The above copyright notice and this permission notice shall be included in **all copies or substantial portions** of the Software."；choosealicense 补充 "conditions only requiring **preservation of copyright and license notices**. Licensed works, modifications, and larger works may be distributed under different terms **and without source code**." → 闭源分发仍须把 MIT 版权声明 + 许可全文随二进制/文档提供（典型做法 `THIRD_PARTY_NOTICES.md` 内附全文与每个 MIT 依赖的版权行），但**不要求公开源码**。

**Apache-2.0 §4(d) 的触发边界（A，[LICENSE-2.0.txt](https://www.apache.org/licenses/LICENSE-2.0.txt)）**：§4(a)(b)(c) 四项义务逐字见 §2.4；**§4(d) 仅当上游 Work 自身随分发含 `NOTICE` 文本文件时才触发**（条文以 "If the Work includes a "NOTICE" text file as part of its distribution" 起首），触发时须把归属声明可读副本放进「NOTICE 文件 / 源码文档 / 应用内第三方声明通常出现处」三选一。§1 明确 "Derivative Works shall **not** include works that remain separable from, or **merely link (or bind by name) to the interfaces** of, the Work"；§6 商标条款仍在。→ 第一方 "MIT OR Apache-2.0" 是**双许可、允许被许可方择一**，不自动让下游免于 Apache 条款，须**逐依赖**判定实际采用的许可。

**MPL-2.0 官方 FAQ（A，[mozilla.org/MPL/2.0/FAQ/](https://www.mozilla.org/en-US/MPL/2.0/FAQ/)，2024-01-30 更新，自述 "reading the FAQ should not serve as a substitute for reading the license itself, or for seeking legal advice from a lawyer."）**：
- Q1 "The MPL's **"file-level" copyleft** is designed to encourage contributors to share modifications… while still allowing them to combine your code with code under other licenses (**open or proprietary**) with minimal restrictions."
- Q8（标题即「compiled from someone else's **unchanged** MPL-licensed source code, either standalone or part of a larger work」）"You must **inform the recipients where they can get the source** for the MPLed code in the executable program or library you are distributing (i.e., you must comply with **Section 3.2**). You may distribute any executables you create under a license of your choosing, as long as that license does not interfere with the recipients' rights to the source under the terms of the MPL."
- Q11 "**new files containing no MPL-licensed code are not Modifications**, and therefore do not need to be distributed under the terms of the MPL, even if you create a Larger Work… This allows, for example, programs using MPL-licensed code to be **statically linked** to and distributed as part of a **larger proprietary** piece of software, which would not generally be possible under the terms of stronger copyleft licenses."
- Q12 "MPL: The copyleft applies to **any files containing MPLed code**. LGPL: … any library based on LGPLed code. GPL: … **all software** based on GPLed code."
- Q13 "**Yes to both.**"（MPL 与 BSD / Apache 可组合）；Q17 "'distribute' … delivery of a copy… **We do not use distribute to mean "make available"** … without delivery of a copy"（纯 SaaS 不触发）；Q22 Exhibit A 通知可放在「a LICENSE file in a relevant directory」；Q26「any mechanism (such as a courier) that adds cost and complexity… is **not reasonable**」。
- **综合（A 级条文 + C 级判断）**：专有 App 仅链接/使用**未修改**的 MPL-2.0 库作独立模块时，须 ① 告知 Covered Software 受 MPL 管辖 + 如何取得 MPL 全文（§3.1）② 以 "reasonable means in a timely manner, at a charge no more than the cost of distribution" 告知如何取得该库 **Source Code Form**（§3.2(a) / FAQ Q8）③ 保留 Exhibit A / §3.4 声明 ④ 可执行形式可自选许可（含专有），但不得限制受领者对源码的 MPL 权利（§3.2(b)）。**不需要公开专有 App 自身源码**（FAQ Q11 / §3.3 Larger Work 自选条款）。**触发放大条件**：若修改了任一 MPL 依赖文件，或新建「contains any Covered Software」的源码文件（§1.10），这些**具体文件**须按 MPL 提供修改后源码（§3.1）—— 是文件级而非全项目级。

**商店侧（A / B + NOT RETRIEVED）**：Apple [App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/) §5 "Apps must comply with **all legal requirements** in any location where you make them available (if you're not sure, check with a lawyer)"、§5.2 "Make sure your app only includes content that you created or **that you have a license to use**"、§5.2.1 / §5.2.2；**全页检索 "GPL" 0 次、'open source' 仅导航菜单 1 次 → 无专门「开源许可展示 / GPL 兼容性」条款**（C：以通用合法授权要求间接覆盖）。Google 开发者文档 [Include open source notices](https://developers.google.com/android/guides/opensource?hl=en)（A/B，属 SDK 工具文档非 Play 上架规则）逐字 "**you as a developer are responsible for appropriately displaying the notices for the open source libraries that your app uses**"、`oss-licenses-plugin` 生成标题 "Open source licenses" 的 Activity；Play Console「Prepare your app for review」的 App content 页要求 "satisfies legal requirements" 但**无专门开源许可声明条目**。[AOSP licenses](https://source.android.com/docs/setup/contribute/licenses?hl=en)（B/A，贡献者政策非 Play 规则）"Google prefer Apache 2.0 (and similar licenses such as BSD and MIT) over other licenses such as the GNU Lesser General Public License (LGPL)"、"LGPL… **many compliance problems** for downstream device makers and app developers"。NTIA [SBOM 报告](https://www.ntia.gov/report/2021/minimum-elements-software-bill-materials-sbom)（A，2021-07-12，源于 EO 14028）"An SBOM is a **formal record containing the details and supply chain relationships** of various components used in building software."（最小元素字段表 PDF NOT RETRIEVED）。

**C 级判断（非法律意见，`THIRD_PARTY_NOTICES.md` 上架前必须含）**：① 全部依赖清单（包名 + 精确版本 + SPDX 标识 + 版权持有人），core 147 / desktop 466 全部纳入而非只列 7 类许可；② MIT / Apache-2.0 / MPL-2.0 / Unicode-3.0 / BSD-3-Clause / Zlib / Unlicense 各自全文或取得方式（Apache §4a 明确 "give any other recipients… a copy of this License"）；③ 逐依赖核验 NOTICE 文件并落实 §4(d)；④ MPL 专门段：逐项声明受 MPL 管辖、给出 MPL 全文 URL、给出「未修改」MPL 库源码的 reasonable means 要约、保留 Exhibit A、说明是否修改过（修改则须提供修改后源码）；⑤ **双许可/非 SPDX 表达式择一决定**：`htmlescape 0.3.1` 的 "Apache-2.0 / MIT / MPL-2.0" 是 "/" 分隔的**非 SPDX "OR" 表达式**，须明确按哪一个许可使用（三选一直接影响是否需要 MPL 源码要约）；⑥ 应用内 "Open source licenses" / "Acknowledgements" 界面与该文件一致。**准确措辞应为「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」**，绝不能说「没有任何 copyleft」。

### 2.4.2 fetch 后全覆盖口径补遗（M5 追加，2026-11-15；**追加记录、不改写上文已冻结的 fetch 前诚实边界**）

> 性质：本节是 M5「零成本合规最小动作」对两 workspace 跑 `cargo fetch --locked` 后的**事实补遗**。§2.4 与 §5 已冻结的「可核验范围内 GPL 家族 0 / 179 个不可核验包不作声明」是 **fetch 前（2026-11-15 早间、434 包可核验子集）时点口径**，作为历史记录**保留原文、不改写**；本节只补充 fetch 后（613 包全覆盖）新显现的观察，避免读者把历史口径误读为「当前依赖树绝对没有 GPL/LGPL 字样」。详见 [zerocost-compliance-report.md](zerocost-compliance-report.md) 与 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

- **`cargo fetch` 结果（A，本地实测）**：`core` 与 `desktop/src-tauri` 两个 workspace 各自 `cargo fetch --locked` 均 EXIT=0；两份 `Cargo.lock` 未被改动（mtime 仍为 2026-11-14）。fetch 前 core 147 包（134 可核验 / 13 不可核验）+ desktop 466 包（300 / 166）= 可核验 434 / 不可核验 179；**fetch 后 core 147/147 + desktop 466/466 = 613 包可核验全覆盖、不可核验 179→0**。记为「可核验全覆盖已达成」，但**冻结边界「179 个不可核验包不作声明」是历史时点值、不改写**。
- **§2.4「desktop 6 个 MPL 但仅点名 5 个」的第 6 个已查明（A）**：= `htmlescape 0.3.1`，`license = "Apache-2.0 / MIT / MPL-2.0"`（"/" 分隔的**非 SPDX "OR" 三许可**、MPL 仅三选一之一），与 core 那 1 个 MPL 是**同一个包**。**不存在第 6 个「纯 MPL-2.0」包，不臆造**。fetch 后 desktop 纯 MPL-2.0 由 5 增至 7（新增 `cssparser 0.38.0`、`selectors 0.41.0`），加 `htmlescape` 三许可共 8 个 MPL 相关标志。registry src 中带 `NOTICE*` 文件的 crate 数 = 0（Apache-2.0 §4(d) 上游 NOTICE 触发边界的本地事实）。
- **fetch 后新显现的 LGPL-OR token（A，事实观察、非法律结论）**：`r-efi 5.3.0` / `r-efi 6.0.0`（core 含 6.0.0、desktop 含两者）`license = "MIT OR Apache-2.0 OR LGPL-2.1-or-later"` → [tools/audit_licenses.py](tools/audit_licenses.py) 的 GPL-expression hits 由 core 0 / desktop 0 变为 **core 1 / desktop 2**，全部来自这个 **SPDX OR 表达式里的 LGPL 选项**（同一表达式另有 MIT / Apache-2.0 可选）。`r-efi` 是 `getrandom` 在 UEFI + `efi_rng` 下的 **target-specific 传递依赖**，对 `x86_64-pc-windows-gnu` 目标**不参与编译**，**不作「已编译进产物」断言**。
- **fetch 后独立原文扫描（keyword scan，非许可判定）数字变化（A，本地实测）**：[tools/audit_licenses.py](tools/audit_licenses.py) 的 `INDEPENDENT RAW SCAN` 段 fetch 后为 `text files scanned: 2921` / `Cargo.toml files with GPL phrases: 2` / `LICENSE/README-style files with GPL phrases: 27`（fetch 前 §2.4 记录为 1924 个文本文件、4 个 LICENSE 文件，均系 MPL §1.12 Secondary License 字样）。典型命中含 `r-efi-5.3.0\AUTHORS` / `r-efi-6.0.0\AUTHORS`（"GNU Lesser General Public License" 出自 OR 选项说明）、`pango-0.18.3\COPYRIGHT: ['LGPL']`、`target-lexicon-0.12.16\LICENSE: ['GPLv2']`、`wasi-0.11.1+wasi-snapshot-preview1\LICENSE-Apache-2.0_WITH_LLVM-exception: ['GPLv2']`、`wit-bindgen-0.57.1\LICENSE-Apache-2.0_WITH_LLVM-exception: ['GPLv2']` 等。**这些是解压源码里的关键词命中，不是依赖声明的许可**（多为 LLVM exception 引用、Secondary License 字样、COPYRIGHT 描述文字）；本补遗**既不把它们当作 copyleft 结论，也不声称它们全是误报**——逐文件定性须律师/许可核验，属外部闸门。与「GPL-expression hits core 1 / desktop 2（全部来自 `r-efi` 的 SPDX OR 表达式）」是两个不同口径，勿混用。
- **对外措辞（冻结，沿用并补充）**：仍为「**无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft**」，并补充上述 fetch 后观察；**绝不宣称「依赖树里绝对没有 GPL/LGPL 字样」**。[tools/audit_licenses.py](tools/audit_licenses.py) 只 print SUMMARY、无失败退出码，fetch 后新增的 OR 命中不影响 `EXIT=0` 复验。`htmlescape` 三许可择一与 `r-efi` 的 LGPL-OR 逐依赖许可决定**留待律师签发**，本补遗不代替法律意见。

---

## 3. 本轮三份草案写了什么、刻意没写什么

| 文档 | 写了（基于真实实现 / A 级事实） | 刻意没写（须外部闸门） |
|---|---|---|
| [security-whitepaper.md](security-whitepaper.md) v0.1 | 如实复述已落地密码学（Argon2id m=65536,t=3,p=4 / AES-256-KW / HKDF 域分离 / 单一 XChaCha20-Poly1305 / 1 MiB 分块 / versionMac 全头 / §3.4 解锁顺序 / 两阶段 verify / 对象 header 32B 与 AAD 字节级冻结）、vault 静态格式、loopback WebDAV + 增量写回、**十条诚实声明逐字**、26/0 验证状态、依赖许可审计数字 | 不写「已通过第三方审计 / 已获认证 / 绝对安全」；明确列出 8 类未做项 |
| [privacy-policy.md](privacy-policy.md) v0.1 | 真实零数据流表（唯一监听 127.0.0.1 不出本机）、A 级「零收集也必须提供政策链接」+ 两家允许如实申报不收集 + Apple "Collect" 定义、不在控制范围 6 条、绝不宣称红线 6 条 | **8 个待法务填实占位**（运营主体/地址/邮箱/生效日/适用地区/分发渠道/儿童条款/权利响应流程）；不写「我们不是 controller / 法不适用 / 已合规」；标注**未经律师签发不得对外发布** |
| [prelaunch-checklist.md](prelaunch-checklist.md) v0.1 | 五道闸门表（A/B/C/D/E）+ **当前 NO-GO**、E1–E8 内部工程状态、审计准备包 9 类材料 + 6 项建议审计范围 + go 判据、B/C/D 动作清单 5 组、go/no-go 总表、唯一冻结诚实边界 | 不代替审计/律师/签名/商店申报；把四道外部闸门如实标 🔴 |
| [tools/verify-m4s4.ps1](tools/verify-m4s4.ps1) | 可重放 7 步复验：core `cargo test --workspace --locked` / `cargo build --workspace --locked`、向量门禁三脚本、许可审计、桌面壳 `cargo check`；后台 job `pwsh-489127be8e134e80` 实测 `VERIFY_EXIT=0` | 只读验证 + 编译/测试，**不写产品代码、不产出签名安装包** |

三份草案均把本报告 `m4-step4-report.md` 作为合规事实的唯一结论来源，不重复、不臆造事实。

---

## 4. 四道外部闸门与 go / no-go（当前 NO-GO）

| 闸门 | 谁关闭 | 本轮状态 | go 判据 |
|---|---|---|---|
| **A 第三方安全审计** | 外部审计机构 | 🔴 仅备好 9 类材料 + 6 项建议范围 | 正式审计报告 + 高危项闭环 + 修复复验；**内部评审不算** |
| **B 律师签发** | 执业律师 | 🔴 草案含 8 个待填实占位 | 隐私政策/白皮书/许可告知/加密分类经签发；GDPR·PIPL 适用性与 controller 定性有书面意见；eur-lex 官方文本核对完成 |
| **C 代码签名 + 可分发安装包** | 发布方（自备证书） | 🔴 仅 `bundle.active=true` 配置，未 `tauri build`、未签名 | 主程序 exe + sidecar + 安装包三者全签、`signtool verify /pa`、可复现构建；见 [desktop/SIGNING.md](desktop/SIGNING.md) |
| **D 商店上架申报** | 发布方 + 法务 | 🔴 未提交 | Apple 5.1.1(i) 双处政策链接 + App Privacy「No, we do not collect data」；Google Play Data safety 表单 + User Data 强制内容清单 + Data deletion 如实回答；加密申报项核定 |
| **E 内部工程** | 本仓库 | 🟡 部分（E1–E4 ✅ / E5–E8 🔴） | E5 补齐 179 包核验、E6 G4 五端绿灯、E7 审计报告、E8 签名包 |

**总判据：NO-GO。** A、B 不是软件工程能自行关闭的闸门；本轮不把任何「准备材料齐全」表述为「可上架」。

---

## 5. 唯一冻结诚实边界（对外与对内一律按此措辞）

1. 本轮是**合规事实调研 + 准备包 + 草案 + 核验清单**，**不是合规结论、不是第三方安全审计报告、不是法律意见、不是已通过的认证**。
2. **第三方安全审计与律师签发本仓库无法代劳**；三份草案**未经律师签发不得对外发布**。
3. **草案 ≠ 产品发布 ≠ 五端验收（G4 仍部分） ≠ 第三方审计**；未产出可分发安装包 / 未签名 / 证书自备。
4. 沿用既有边界：**非 WinFsp 驱动级挂载**（M4 第 3 点已决策不采用）、loopback WebDAV **无 per-request 认证**、**拒绝在云同步根目录内挂载**、仅未解锁态防合法采集者、解锁态只缓解、库外持久副本锁定后不保证消失。
5. **依赖审计只覆盖可核验包**（core 134 / desktop 300），**MPL-2.0 弱 copyleft 已存在**（core 1 + desktop 6），**179 个不可核验包不作任何许可声明**；不得宣称「全图无 GPL」「没有任何 copyleft」「纯宽松许可」。
6. **GDPR 条文引文来自 gdpr-info.eu 忠实转载（A⁻）**，欧盟官方 eur-lex 本轮 NOT RETRIEVED；正式引用前须回官方页核对。
7. 所有价格、版本、条文、商店规则均为 **2026-11-15 抓取时点值**，签发前须复核最新文本；**§2.3 的加密出口事实（A/A⁻/C）为调研结论，不构成法律意见或合规结论**，书面分类仍须律师/合规签发；凡标 NOT RETRIEVED 者不得当作已核验事实引用。
8. 现有实现仍 ≠ 产品发布 ≠ 五端一致性验收（G4 仅 Rust/Windows 单端） ≠ 第三方审计。

---

## 6. NOT RETRIEVED / 开放项清单（取不到即写，绝不臆造）

1. **欧盟官方 GDPR 一手全文**（Art 2 / 4、Recital 14/18/26）：eur-lex.europa.eu 全部 HTTP 202 空体（AWS WAF challenge / CloudFront），含 `/eli/reg/2016/679/oj`、`/legal-content/EN/TXT/HTML/?uri=CELEX:32016R0679`、PDF 变体；data.europa.eu 跨域重定向未跟随；legislation.gov.uk 条文正文不可读（仅取到 /contents 标题）。→ 条文用 gdpr-info.eu（A⁻/转载）。
2. **GDPR Art 12–14（透明度/告知义务）、Art 32（处理安全）逐条文本**未取，故不引条号、不下结论。
3. **EDPB / ICO 关于 "local processing" 的专门一手指引**未找到；EDPB Guidelines 07/2020 官方 PDF 中 household / purely personal / on the device / local 等关键词 0 命中。→ 「本地处理」定性只能依据通用条文，不能假装有专门指引。
4. **Apple 页面字面标签 "Data Not Collected"** NOT RETRIEVED（只取到 "No, we do not collect data from this app"）。
5. **Google Play Developer Policy 独立 "Privacy Policy" 分节原文** NOT RETRIEVED（详细要求来自 Play Console Help 9859455 / 10144311）。
6. **加密出口 subagent 最终报告已并入 §2.3**（三路调研全部收齐）。仍属 NOT RETRIEVED 的加密出口项，逐条列在下方 6a–6f：
   - 6a. **15 CFR part 774 Supplement No. 1（CCL）中 ECCN 5D002 / 5E002 条目原文**（[law.cornell.edu/cfr/text/15/774](https://www.law.cornell.edu/cfr/text/15/774) → HTTP 404 "Page not found"）：含 "Items Controlled" 清单及 "cryptography for data confidentiality" / "described security algorithm" 定义；编号本身在多条 CFR 中被引用（A⁻），条目正文未取到。
   - 6b. **15 CFR § 772.1 中 "non-standard cryptography" 逐字定义**（页正文在定义条目处截断）；仅有 [Apple 官方页](https://developer.apple.com/help/app-store-connect/manage-app-information/overview-of-export-compliance/) 转述的 US Government 定义（A）。
   - 6c. **BIS/eCFR.gov 官方 .gov 站上的 ENC 条文正文**：仅取到 https://www.bis.gov/ 首页（HTTP 200，A⁻）；Apple 引用的 [年度自我分类报告页](https://www.bis.doc.gov/index.php/policy-guidance/encryption/4-reports-and-reviews/a-annual-self-classification) 正文未取。本轮 CFR 条文取自 Cornell LII e-CFR 镜像（A⁻）。
   - 6d. **Apple 出口合规豁免类别清单 / CCATS 与 ERC 具体表格清单**："Learn about the app encryption documentation you may need to provide" 指向的页正文 NOT RETRIEVED；https://help.apple.com/app-store-connect/#/dev88f5c7bf9 为 HTTP 200 空体 SPA；**"ERC" 一词在所取全部一手页中均未出现**。两份 dead-end Help 页渲染 "Page Not Found - Apple Developer"。
   - 6e. **Google Play 问卷逐字措辞** "Is your app subject to U.S. export laws?" / "Does your app use encryption?"：未在任一取到的一手页验证（Data safety 页检索四类措辞全部 -1）；实际相关一手页为 [answer/113770](https://support.google.com/googleplay/android-developer/answer/113770?hl=en)。**Microsoft Store 加密出口声明的一手陈述** NOT RETRIEVED（仅取到 MSIX 定价与可用性页，无加密字样；「在该页未见」≠「确定不存在」）。
   - 6f. **中国**：《出口管制法》正文（npc.gov.cn 重定向/SSL 失败；gov.cn 候选 URL 一为无关量子新闻、一为 404）；《商用密码进口许可清单》《商用密码出口管制清单》原文；「大众消费类产品」法定定义（条例第 31 条 / 密码法第 28 条出现该短语但无定义）；《数据安全法》《关基条例》正文（flk.npc.gov.cn 为 JS 空壳）；《网络安全法》正文在第 35 条处截断。**EU**：附件 I 中 "in the public domain" 的逐字定义（GSN/GTN 引用该词，定义条目未单独定位）；EUR-Lex CELEX HTML/PDF 全文（AWS WAF 拦截，英文正文改由 CELLAR manifestation 取到 A 级，规范链接未抓取成功）。
   - 6g. **第三方许可侧**：NTIA《The Minimum Elements For a SBOM》PDF 全文与「最小元素」字段逐字表（web_fetch "unsupported content type application/pdf"）；Google Play Developer Distribution Agreement 全文（CDP 超时）；**Google Play / Apple 任何明文要求「应用内必须展示开源许可声明界面」或「GPL 兼容性审查」的商店规则条款未检索到**（结论为「未检索到明文」而非「确定不存在」）；Apple 官方 Settings bundle / Acknowledgements.plist 专门文档明文；`htmlescape 0.3.1` 上游独立 LICENSE 文件（github.com/veddan/rust-htmlescape/master/LICENSE → HTTP 404，故该 crate 三许可关系仅能从 Cargo.toml / crates.io 元数据确认）。
   - （MIT / Apache-2.0 / MPL-2.0 一手全文已由第三方许可 subagent 取到，见 §2.4.1，不再列为 NOT RETRIEVED；opensource.org 网页截断由 SPDX / choosealicense 一手页补足。）
7. **加密分类与「大众消费类产品」界定的书面结论**：§2.3 全部为事实调研 + C 判断，BIS ECCN 5D002/5E002 / CCATS / Apple `ITSAppUsesNonExemptEncryption` / EU 2021/821 / 中国「大众消费类产品」界定与强制性国标符合性 / 两份管制清单 / 法国 ANSSI，均须律师/合规出具书面分类，开放。
8. **179 个本地不可核验依赖包的许可**：未核验、不作声明；须 `cargo fetch` 补齐后重新审计。
9. **本仓库无 `.git`**：CI 工作流推送到远端启用 Actions 后才实跑；macOS / Linux 绿灯未取得；Android / iOS 未实现。
10. 抓取失败记录：`winfsp.net`、`raw.githubusercontent.com`、`github.com` HTML（截断）、`html.duckduckgo.com` 搜索、搜索引擎（Bing/百度/Sogou/DuckDuckGo/r.jina.ai 普遍不可用或返回审查后垃圾结果），一律不臆造替代内容；有效路径为官方站索引 + CELLAR API + 真实 ego-browser。

---

## 7. 复现命令与一手来源

```powershell
# 可重放最终复验（只读验证 + 编译/测试，不写产品代码）
& powershell -NoProfile -ExecutionPolicy Bypass -File D:\Projects\dsh\mylock\tools\verify-m4s4.ps1

# 单项
cd D:\Projects\dsh\mylock\core ; cargo test --workspace --locked ; cargo build --workspace --locked
& 'C:\Python312\python.exe' D:\Projects\dsh\mylock\tools\check_vectors.py
& 'C:\Python312\python.exe' D:\Projects\dsh\mylock\tools\compute_fingerprint.py
& 'C:\Python312\python.exe' D:\Projects\dsh\mylock\tools\gate_selftest.py
& 'C:\Python312\python.exe' D:\Projects\dsh\mylock\tools\audit_licenses.py
cd D:\Projects\dsh\mylock\desktop\src-tauri ; cargo check
```

**一手 URL 全表**（按域；GDPR 为 A⁻/转载，已注明）：
- PIPL：http://www.npc.gov.cn/npc/c2/c30834/202108/t20210820_313088.html （A）
- 商用密码管理条例：https://www.gov.cn/zhengce/zhengceku/202305/content_6875928.htm （A）；密码法：https://www.oscca.gov.cn/sca/xxgk/2023-06/04/content_1057225.shtml （A）
- GDPR（转载 A⁻）：https://gdpr-info.eu/art-2-gdpr/ 、/art-4-gdpr/ 、/recitals/no-14/ 、/recitals/no-18/ 、/recitals/no-26/ ；官方（NOT RETRIEVED）：https://eur-lex.europa.eu/eli/reg/2016/679/oj
- EDPB Guidelines 07/2020：https://www.edpb.europa.eu/documents/guideline/guidelines-072020-on-the-concepts-of-controller-and-processor-in-the-gdpr_en （A 落地页）/ https://www.edpb.europa.eu/system/files/documents/2023-10/EDPB_guidelines_202007_controllerprocessor_final_en.pdf （A⁻ 引文）
- ICO：https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/controllers-and-processors/controllers-and-processors-a-guide/ （A）
- Apple：https://developer.apple.com/app-store/review/guidelines/ （5.1.1(i)）、https://developer.apple.com/app-store/app-privacy-details/ 、https://developer.apple.com/help/app-store-connect/manage-app-information/manage-app-privacy/ （A）
- Google Play：https://support.google.com/googleplay/android-developer/answer/9859455?hl=en 、/answer/10787469?hl=en 、/answer/10144311?hl=en 、/answer/13327111?hl=en 、https://play.google/intl/en_us/developer-content-policy/ （A，ego-browser 真实浏览器取正文）
- Apache-2.0：https://www.apache.org/licenses/LICENSE-2.0 、https://www.apache.org/licenses/LICENSE-2.0.txt （A，§4(a)–(d) / §1 / §6）；MPL-2.0：https://www.mozilla.org/en-US/MPL/2.0/ 、https://www.mozilla.org/en-US/MPL/2.0/FAQ/ （A，§3.1–3.4 / Exhibit A / Q1·Q8·Q11·Q12·Q13·Q17·Q22·Q26）
- MIT：https://spdx.org/licenses/MIT.html 、https://choosealicense.com/licenses/mit/ 、https://opensource.org/license/mit （A）；开源声明实务：https://developers.google.com/android/guides/opensource?hl=en （A/B）、https://source.android.com/docs/setup/contribute/licenses?hl=en （B/A）、https://www.ntia.gov/report/2021/minimum-elements-software-bill-materials-sbom （A）、https://spdx.dev/learn/handling-license-info/ （A）
- **US EAR（A⁻，Cornell LII e-CFR 镜像；bis.gov 仅确认入口）**：https://www.law.cornell.edu/cfr/text/15/742.15 、/15/740.17 、/15/734.7 、/15/734.3 、/15/734.4 、/15/772.1 、/15/774 （最后一个 CCL 条目正文 HTTP 404）；https://www.bis.gov/ （A⁻ 入口）
- **Apple 加密申报（A，官方 JSON / Help）**：https://developer.apple.com/tutorials/data/documentation/bundleresources/information-property-list/itsappusesnonexemptencryption.json 、https://developer.apple.com/tutorials/data/documentation/security/complying-with-encryption-export-regulations.json 、https://developer.apple.com/help/app-store-connect/manage-app-information/overview-of-export-compliance/ 、https://developer.apple.com/help/app-store-connect/manage-app-information/determine-and-upload-app-encryption-documentation/
- **Google Play 加密出口（A，ego-browser）**：https://support.google.com/googleplay/android-developer/answer/113770?hl=en （应用托管美国即视为出口、开源仍受管制）；**Microsoft Store（A⁻/NOT RETRIEVED）**：https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/price-and-availability （无加密字样）
- **中国（A + B）**：网络安全法 https://www.oscca.gov.cn/sca/xxgk/2016-11/07/content_1012647.shtml （A，正文第 35 条处截断）；官方站转载专家解读（B）https://www.oscca.gov.cn/sca/xxgk/2023-06/04/content_1061070.shtml （「仅四类…进口许可」「出口管制范围更宽…涵盖密钥管理产品…软件」）
- **EU 2021/821（A，CELLAR API 一手正文）**：元数据 https://publications.europa.eu/resource/celex/32021R0821 （Accept: application/xml; notice=object）；英文正文 http://publications.europa.eu/resource/cellar/d8e79ebf-ca4e-11eb-84ce-01aa75ed71a1.0006 （Accept: application/xhtml+xml，OJ L 206, 2021，5,909,234 B）；规范链接 https://eur-lex.europa.eu/eli/reg/2021/821/oj/eng （NOT RETRIEVED，AWS WAF）

---

## 8. 文档接口

- 上游：[产品分析与概要设计.md](产品分析与概要设计.md) **v1.9**（M4 第 4 点当时为 v1.8；M5 后升 v1.9。表头 / 配套文件 / v1.8 变更块 / §8.2 里程碑 / §9.3 维护约定已回填）、[m4-step3-report.md](m4-step3-report.md)（M4 第 3 点 WinFsp 决策）。
- 本轮产出：[security-whitepaper.md](security-whitepaper.md) / [privacy-policy.md](privacy-policy.md) / [prelaunch-checklist.md](prelaunch-checklist.md) / [tools/verify-m4s4.ps1](tools/verify-m4s4.ps1) / 本报告；均相互引用、不重复事实。
- 依据：[vault-format-spec.md](vault-format-spec.md) v1.1.1、[crypto-review-m1.md](crypto-review-m1.md) v1.1+§12（十条诚实声明唯一冻结措辞）、[threat-model.md](threat-model.md) v1.1、[m3-report.md](m3-report.md) / [m3-architecture.md](m3-architecture.md) / [m4-report.md](m4-report.md) / [m4-step2-report.md](m4-step2-report.md)、[desktop/SIGNING.md](desktop/SIGNING.md)、[tools/audit_licenses.py](tools/audit_licenses.py)。
- **冻结措辞**：M4 第 4 点的「准备包非合规结论 / 第三方审计与律师签发无法代劳 / 草案未经签发不得对外发布 / 当前 NO-GO」以本报告 §0 / §5 与 [prelaunch-checklist.md](prelaunch-checklist.md) §0 / §4 / §5 为唯一冻结措辞；十条对外诚实声明仍以 `crypto-review-m1.md` §8 为准，本报告不新增、不矛盾。
