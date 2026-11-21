# MyLock 上架前核验清单（草案 · go/no-go 判据）

> **性质声明（务必先读）**：本文是**上架前准备包与核验清单**，不是合规结论、不是法律意见、不是第三方审计报告、不是已通过的认证。它把「必须由外部主体完成、本仓库无法代劳」的闸门逐项列成可执行清单与 go/no-go 判据。
> 版本：v0.1（草案） · 日期：2026-11-15 · 授权：用户 m03444「继续」（M4 第 4 点：上架前准备）
> 配套：[security-whitepaper.md](security-whitepaper.md) · [privacy-policy.md](privacy-policy.md) · [m4-step4-report.md](m4-step4-report.md)（合规事实调研，A/A⁻/B/C 分级） · [desktop/SIGNING.md](desktop/SIGNING.md) · [tools/audit_licenses.py](tools/audit_licenses.py)

---

## 0. 总览：四道外部闸门 + 一道内部闸门

| 闸门 | 由谁完成 | 当前状态 | 能否由本仓库代劳 |
|---|---|---|---|
| A. 第三方安全审计 | 具资质第三方安全公司 | 🔴 未启动 | ❌ 否 |
| B. 法律意见 / 律师签发 | 具资质律师（加密出口、隐私法、商店合规、许可告知） | 🔴 未启动 | ❌ 否 |
| C. 代码签名 + 可分发安装包 | 开发者（自备 EV/OV 证书）+ 构建流水线 | 🔴 未产出（仅 `bundle.active=true` 配置） | ⚠️ 配置已就绪，证书/签名不可代劳 |
| D. 商店上架申报 | 开发者账号 + 法务结论输入 | 🔴 未启动（渠道未定） | ❌ 否 |
| E. 内部工程就绪 | 本仓库 | 🟡 部分（见 §1） | ✅ 是（本机可验） |

**总 go/no-go 判据**：A–E **全部为绿**且 B 出具书面法律意见后，方可对外分发。当前 E 中仍有未达成项（G4 五端验收、179 包许可核验），故**总判据当前为 NO-GO**。

---

## 1. 内部工程就绪（E，本机可验，非外部闸门）

| # | 项 | 判据 | 当前证据 | 状态 |
|---|---|---|---|---|
| E1 | 全量测试零回退 | `cargo test --workspace --locked` = 26 passed / 0 failed | `core\m4s3-final-test.log` 等 | ✅ |
| E2 | 向量契约门禁 | `check_vectors.py` OK + 指纹命中 `d45b8659…02fa` + `gate_selftest.py` 5/5 | 锚 `07af0e8b…9939`、CRLF×190、30 derived/13 negative | ✅ |
| E3 | 桌面壳编译 | `cargo check`（desktop/src-tauri）EXIT=0 | `check-m4s3-final.log` | ✅ |
| E4 | 依赖许可只读审计 | `audit_licenses.py` EXIT=0、可核验范围 GPL 家族 0 | core 147/134、desktop 466/300、MPL 1+6 | ✅（但见 E5/E6） |
| E5 | **179 个不可核验包** | `cargo fetch` 补齐 + 逐包许可核验 + 完整清单 | 当前 13+166 未核验 | 🔴 未完成 |
| E6 | **G4 五端 + 恢复 CLI 逐字节一致** | Windows/macOS/Linux/Android/iOS 全绿 | 仅 Rust/Windows 单端；Android/iOS 未实现 | 🔴 未完成 |
| E7 | 第三方审计报告已出具并闭环 | 闸门 A 完成 + 高危项修复复验 | 未启动 | 🔴 未完成 |
| E8 | 代码签名 + 可分发包 | `tauri build` 产出 .msi/NSIS + `signtool verify /pa` 通过 | 未产出、未签名 | 🔴 未完成 |

> E5/E6/E7/E8 任一为红 → 总判据 NO-GO。这与「上架前」阶段定位一致：本阶段交付的是**准备包**，不是成品发布。

---

## 2. 闸门 A：第三方安全审计准备包（交给审计方的材料清单）

审计方应收到并可复现以下材料（全部已在仓库）：

| 材料 | 路径 | 用途 |
|---|---|---|
| 安全白皮书（设计说明） | [security-whitepaper.md](security-whitepaper.md) | 理解威胁模型、密码学设计、诚实边界 |
| 格式规范（字段冻结） | [vault-format-spec.md](vault-format-spec.md) v1.1.1 | 逐字节契约、AAD/header/canonical 定义 |
| 内部密码学评审 + 对抗裁定 | [crypto-review-m1.md](crypto-review-m1.md) | 6 处结构性修正 D1–D6、N1–N9、十条诚实声明 |
| 威胁模型 | [threat-model.md](threat-model.md) | 防得住/防不住边界 |
| 固定测试向量 + 门禁 | [tests/vectors/](tests/vectors/) + [CONTRACT.md](tests/vectors/CONTRACT.md) | 跨实现逐字节复现基准 |
| 各阶段报告 | [m2-report.md](m2-report.md) / [m3-report.md](m3-report.md) / [m4-report.md](m4-report.md) / [m4-step2-report.md](m4-step2-report.md) / [m4-step3-report.md](m4-step3-report.md) | 实现与验证历史 |
| 源码与构建 | [core/](core/)（mylock-core / mylock-mount / mylock-cli）、[desktop/src-tauri/](desktop/src-tauri/) | 代码审读、复现测试 |
| 依赖清单 | `core\Cargo.lock` / `desktop\src-tauri\Cargo.lock` + [tools/audit_licenses.py](tools/audit_licenses.py) | 供应链审读 |

**审计范围建议（交审计方核定，非替其定范围）**：
1. 密码学实现正确性：Argon2id p=4 绑定、AES-KW 包裹、HKDF 域分离、XChaCha20-Poly1305 nonce 唯一性、AAD 绑定、versionMac 全头认证、KDF 硬校验先于分配。
2. 密钥生命周期：内存中 `Master`/`MAC_Key`/`IKM` 的清零（Zeroizing / mlock / 禁转储）、解锁态明文驻留、Recovery Key 编码与域分离。
3. 写回崩溃安全：严格 copy-on-write、原子 manifest 替换、孤儿清理、`keep` 集合不误删复用对象、增量写回不泄漏旧明文。
4. 降级/篡改/回滚攻击：负向向量 13 条覆盖之外的组合、`verify` 绿 ≠ 最新版的残余风险、key_epoch 与 rotate-keys 真吊销。
5. 桌面层：loopback 无 per-request 认证的暴露面、`is_sync_root` 拒绝是否可绕过、Tauri sidecar 解析路径、CSP/权限配置。
6. G4 五端一致性（若届时已实现）。

**交付要求（写入审计委托，go 判据）**：正式报告 + 高危/中危分级 + 修复建议 + 修复后复验通过证明。**仅「内部评审通过」不构成闸门 A 完成。**

---

## 3. 闸门 B/C/D：法务与上架核验清单（事实结论见 [m4-step4-report.md](m4-step4-report.md)）

> 下列每一项的「规则事实」由 [m4-step4-report.md](m4-step4-report.md) 以 A/A⁻/B/C 分级给出并附一手 URL；本清单只列**必须核验的动作**与 **go 判据**，不重复事实、不给法律意见。

### 3.1 加密出口管制（自研 XChaCha20-Poly1305 是重点）

> 事实结论见 [m4-step4-report.md](m4-step4-report.md) §2.3（US EAR / Apple / Google Play / Microsoft Store / 中国 / EU 2021/821，A/A⁻/B/C 分级 + 一手 URL）；本清单只列动作与 go 判据，不给法律意见。**最反直觉的两条**：① US「免费/开源 ≠ 自动豁免」——§740.17(b) 明写公开可得的 mass-market 加密软件在完成分类/自我分类前仍受 EAR 约束；② EU 2021/821 原文无「公开可得/开源豁免」字样、GENERAL SOFTWARE NOTE 明确「不释放 Category 5 Part 2」，且 MyLock 以信息安全为主要功能，不能套用大众市场排除。

- [ ] 确定分发地区与渠道，逐地区判定是否落入 EAR / 中国商用密码 / 欧盟两用物项管控。
- [ ] 对本产品做**加密自我分类**：自实现 XChaCha20-Poly1305 属专有/非标准密码（Apple 官方转述 USG 定义「未被 IETF/ISO 等认可标准组织采纳的算法」即 non-standard cryptography，见 §2.3.2），**不得简单套用「mass-market / exempt」结论**；Apple [官方页](https://developer.apple.com/tutorials/data/documentation/security/complying-with-encryption-export-regulations.json) 明写「OS 内置加密通常豁免，而 proprietary encryption 不豁免」。
- [ ] 判定 ECCN 归属（5D002 / 5E002 或自我分类改列 5D992）：**闭源专有发行不落入 §742.15(b) 的「公开可得加密源码」豁免**；§734.4 对 5E002 技术无 de minimis；若改算法，§740.17(d)(1)(iii) 触发新的分类请求。
- [ ] 若需，完成 BIS 分类/自我分类报告（§740.17(e)(3) 年度「no later than February 1」）、或取得 CCATS；Apple `ITSAppUsesNonExemptEncryption` 大概率需置 `YES` 并上传合规文档、取 `ITSEncryptionExportComplianceCode`（见 §2.3.2）。
- [ ] 中国：《商用密码管理条例》第三十一条 / 《密码法》第二十八条「**大众消费类产品所采用的商用密码不实行进口许可和出口管制制度**」为 A 级明文，但「大众消费类产品」法定定义、强制性国标符合性、两份管制清单原文 NOT RETRIEVED，须律师核定；EU 附件 I 第 5 类第 2 部分「INFORMATION SECURITY」逐条判定、法国 ANSSI 单独申报。
- [ ] 三大商店合规问卷**独立于「是否开源」**（Google Play [answer/113770](https://support.google.com/googleplay/android-developer/answer/113770?hl=en)：托管美国即视为出口、开源仍受管制）；Microsoft Store 加密申报一手陈述 NOT RETRIEVED（「在该页未见」≠「确定不存在」）。
- **go 判据**：律师/合规出具书面分类结论（含 CCATS/ERC 判定、「大众消费类产品」界定）；商店问卷「是否使用加密 / 是否受出口管制约束」据此**如实**作答（Apple `ITSAppUsesNonExemptEncryption` 等字段不得误报）。

### 3.2 隐私法适用性（PIPL / GDPR / 其他）

> 条文与判准见 [m4-step4-report.md](m4-step4-report.md) §2.1（GDPR 条文标 A⁻/gdpr-info.eu 忠实转载，**EUR-Lex 官方一手全文 NOT RETRIEVED：HTTP 202 空体 / AWS WAF challenge，正式引用前必须回 eur-lex 核对，不得宣称「已核对欧盟官方原文」**；PIPL 为 [中国人大网 A 级](http://www.npc.gov.cn/npc/c2/c30834/202108/t20210820_313088.html)）。

- [ ] 核定「纯本地处理、开发者从不接收数据」场景下，运营主体是否仍为 controller / 个人信息处理者。**不得先入为主认定「不适用」**：GDPR Art 4(7) controller 判准是「determines the purposes and means」而非「是否拿到数据」（EDPB Guidelines 07/2020 "It is not necessary that the controller actually has access to the data… to be qualified as a controller."，Recital 18 末句 "applies to controllers or processors which provide the means for processing personal data for such personal or household activities."；ICO 同）；GDPR Art 2(2)(c) 与 PIPL 第 72 条「个人/家庭事务」豁免主体是**自然人（用户本人）非公司**；Art 4(2) "processing" 明列 "storage"、PIPL 第 4 条含「存储、加工」→ 本地加密存储也可能是「处理」。PIPL 第 3 条境外「以向境内自然人提供产品/服务为目的」亦适用。
- [ ] 核定 [privacy-policy.md](privacy-policy.md) §0 全部 8 个占位项、数据主体权利流程、儿童条款；**PIPL 第 73(四) 匿名化 =「不能复原」，持口令即可解密 → 加密 ≠ 匿名化**（GDPR Recital 26 同），不得把 vault 密文声称为匿名化数据。
- [ ] PIPL 第 51 条「加密、去标识化等安全技术措施」可作为已采取的安全措施如实写入；PIPL 第 13 条合法性基础主语是「个人信息处理者」、第 17 条告知义务是否适用由律师定性。
- **go 判据**：律师签发隐私政策；措辞与十条诚实声明一致，无「零元数据 / 已审计 / 绝对安全 / 不是 controller / GDPR·PIPL 不适用 / 已完全合规 / 加密=匿名化 / 零数据=无需政策」等红线表述（见 [privacy-policy.md](privacy-policy.md) §5）。
- **NOT RETRIEVED（不得假装已核验）**：GDPR 官方全文、Art 12–14 / Art 32 逐条、EDPB/ICO 关于 "local processing" 的专门一手指引（EDPB PDF 关键词 0 命中）、PIPL 官方英译。

### 3.3 商店隐私披露与合规问卷
- [ ] Apple：**A 级已确认「零收集也必须提供隐私政策链接」**（Guideline 5.1.1(i)：App Store Connect 元数据字段 + App 内易访问位置各一处）。按 [App Privacy Details](https://developer.apple.com/app-store/app-privacy-details/) 的 "Collect" 定义如实申报（零收集 → App Store Connect 可选 "No, we do not collect data from this app"）；核定第三方 SDK 签名/隐私清单要求。
- [ ] Google Play：**A 级已确认「不访问个人数据的 App 仍须在商店页与 App 内提供隐私政策链接」**。Data safety 表单必须填写，可如实填「no user data is collected or shared」；数据删除问题须回答，但「应用内账户删除 + 网页删除链接」重义务仅在 App 支持创建账户时触发（MyLock 无账号，大概率不适用，待法务核定）。
- [ ] Microsoft Store：加密/隐私相关声明。
- [ ] 隐私政策 [privacy-policy.md](privacy-policy.md) 经律师签发后，填入各商店元数据字段与 App 内易访问位置。
- **go 判据**：各渠道问卷与隐私政策口径一致，且能证明「零第三方 SDK / 零数据收集」，且两家商店的隐私政策链接均已配置。

### 3.4 第三方许可告知义务（**最容易被误判的一项**）

> 一手条文见 [m4-step4-report.md](m4-step4-report.md) §2.4.1：MIT（SPDX/OSI）、Apache-2.0 §4(a)–(d)（apache.org 全文）、MPL-2.0 §3.1–3.4 + Exhibit A + [官方 FAQ Q1/Q8/Q11/Q12/Q13/Q17/Q22/Q26](https://www.mozilla.org/en-US/MPL/2.0/FAQ/)（A 级，2024-01-30）。

- [ ] **先 `cargo fetch` 补齐 179 个不可核验包**，生成完整依赖许可清单（勿由旧兄弟 crate 推断新版本许可）；**desktop 6 个 MPL 包本地审计只点名 5 个（cssparser 0.37.0 / cssparser-macros 0.7.1 / dtoa-short 0.3.5 / option-ext 0.2.0 / selectors 0.38.0），第 6 个须补齐后核对，不臆造**。
- [ ] 生成 `THIRD_PARTY_NOTICES.md`：**全部依赖**（包名 + 精确版本 + SPDX + 版权持有人）逐项列出，附 MIT / Apache-2.0 / MPL-2.0 / Unicode-3.0 / BSD-3-Clause / Zlib / Unlicense 各自全文或取得方式。
- [ ] **双许可择一 / 非 SPDX 表达式**：`htmlescape 0.3.1` 的 "Apache-2.0 / MIT / MPL-2.0" 是 "/" 分隔的**非 SPDX "OR" 表达式**，须明确按哪一个许可使用（直接影响是否需要 MPL 源码要约）；第一方 "MIT OR Apache-2.0" 是双许可、允许被许可方择一，仍须**逐依赖**判定实际采用的许可。
- [ ] **MPL-2.0 文件级 copyleft（FAQ Q1/Q8/Q11/Q12）**：对 core 1 个 + desktop 6 个 MPL 包，按 §3.1–3.4 / Exhibit A 落实「告知 Covered Software 受 MPL 管辖 + 给出 MPL 全文 URL + 以 reasonable means / timely / 不超过分发成本 的方式告知**未修改** MPL 库源码（Source Code Form）如何取得（§3.2(a) / Q8）+ 保留 Exhibit A 与 §3.4 声明」；可执行形式可自选许可（含专有），但不得限制受领者对源码的 MPL 权利（§3.2(b)）。**专有 App 链接未修改 MPL 库作独立模块时不需要公开自身源码**（Q11 新文件不含 MPL 代码不是 Modifications / §3.3 Larger Work 自选条款）；**若修改了任一 MPL 文件，这些具体文件须按 MPL 提供修改后源码（§3.1），是文件级而非全项目级**。核定 Larger Work §3.3 边界（**不得**声称「我们没有任何 copyleft」——MPL 弱 copyleft 已存在）。
- [ ] **Apache-2.0 §4**：随分发提供 License 副本（§4a）、保留版权/专利/商标/归属声明（§4c）、标注修改（§4b）、**逐依赖核验上游是否含 `NOTICE` 文件**（§4(d) 仅当上游含 NOTICE 时触发，触发时须把归属声明可读副本放进 NOTICE 文件 / 源码文档 / 应用内第三方声明三选一）；§6 商标条款仍在。
- [ ] **MIT**：保留版权与许可声明（"included in all copies or substantial portions"），不要求公开源码。
- [ ] 在 App 内提供可读的「开源许可 / 致谢」界面或随包 `THIRD_PARTY_NOTICES` 文件；核定 Apple 5.2（"only includes content… that you have a license to use"）、§5（"comply with all legal requirements… check with a lawyer"）与 Google [Include open source notices](https://developers.google.com/android/guides/opensource?hl=en)（"you as a developer are responsible for appropriately displaying the notices"）。**准确措辞：「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」**。
- **go 判据**：律师核定完整 `THIRD_PARTY_NOTICES.md` + SBOM；无未核验包残留；MPL 源码要约、NOTICE 传递、双许可择一逐条落实。

> **M5 追加补遗（2026-11-15，零成本最小动作已执行；追加记录、不改写 §5 已冻结的 fetch 前诚实边界）**：
> - `core` 与 `desktop/src-tauri` 两 workspace `cargo fetch --locked` 均 EXIT=0、两份 `Cargo.lock` 未改动；**不可核验包 179→0（613 包可核验全覆盖）**，记为「可核验全覆盖已达成」，但 §5「179 个不可核验包不作任何许可声明」是 fetch 前时点口径、作为历史记录保留不改写。详见 [zerocost-compliance-report.md](zerocost-compliance-report.md)。
> - 已生成 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)（613 包逐行 + MPL / Apache / MIT / 双许可专门段 + 只读幂等生成器 [tools/gen_third_party_notices.py](tools/gen_third_party_notices.py)）；registry src 带 `NOTICE*` 的 crate 数 = 0。**上方勾选框中「补齐不可核验包 / 生成 THIRD_PARTY_NOTICES」两项的本仓库工程动作已完成**，但「律师核定 + SBOM + 双许可择一逐条落实」的 **go 判据仍未达成**（属 B3 闸门）。
> - **第 6 个 MPL 已查明** = `htmlescape 0.3.1`（`Apache-2.0 / MIT / MPL-2.0`，"/" 分隔非 SPDX "OR" 三许可、MPL 仅三选一之一），与 core 那 1 个是同一个包；**不存在第 6 个纯 MPL-2.0 包、不臆造**。fetch 后 desktop 纯 MPL-2.0 由 5 增至 7（新增 `cssparser 0.38.0`、`selectors 0.41.0`），加 `htmlescape` 共 8 个 MPL 相关标志。
> - **fetch 后新观察（事实、非法律结论）**：`r-efi 5.3.0` / `6.0.0` = `MIT OR Apache-2.0 OR LGPL-2.1-or-later`（SPDX OR 表达式里的 LGPL 选项），使 [tools/audit_licenses.py](tools/audit_licenses.py) 的 GPL-expression hits 由 core 0 / desktop 0 变为 core 1 / desktop 2；`r-efi` 是 `getrandom` 在 UEFI + `efi_rng` 下的 **target-specific 传递依赖、对 `x86_64-pc-windows-gnu` 不参与编译**，不作「已编译进产物」断言。§2.4「可核验范围内 GPL 家族 0」是 **fetch 前（434 包子集）口径**。**对外措辞仍为「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」+ 补充该观察，绝不宣称「依赖树绝对没有 GPL/LGPL 字样」**。
> - **架构变更影响 §3.5**：M5 桌面壳已改为**进程内 `build_handler` + `TokioTcpListener`**，`tauri.conf.json` 的 `bundle.externalBin` 已删除 ⇒ 下方 §3.5「主程序 + sidecar `mylock-serve.exe` + 安装包三者全签」中的 **sidecar 一项对当前 M5 内测版已不适用**（恢复 CLI `mylock.exe` 仍作为独立工具随包提供）；将来若回到外部 sidecar 方案须重新按三项签名。当前 M5 内测包为**免签名 standalone exe + portable ZIP**，未产 NSIS/MSI、未签名。

### 3.5 代码签名与可分发包
- [ ] 自备代码签名证书（[desktop/SIGNING.md](desktop/SIGNING.md)：`certificateThumbprint` / `digestAlgorithm:sha256` / `timestampUrl` / `signCommand` 含 `%1`）。
- [ ] 主程序 exe + **sidecar `mylock-serve.exe`** + 安装包三者**全部签名**（只签主程序不够）。
- [ ] `cargo tauri build` 产出 .msi / NSIS .exe；`signtool verify /pa` 通过；私钥/口令**绝不入库**。
- [ ] 建立可复现构建与 CI 打包流水线（当前无 `.git` / 无打包 CI）。
- **go 判据**：可分发包产出且签名校验通过、可复现构建可重放。

---

## 4. go/no-go 总表（对外分发前逐项签字）

| 闸门 | 责任方 | go 判据 | 签字栏 |
|---|---|---|---|
| A 第三方安全审计 | 安全公司 | 正式报告 + 高危闭环 + 修复复验 | ☐ |
| B1 加密出口分类 | 律师/合规 | 书面分类结论 + 商店申报如实 | ☐ |
| B2 隐私法适用 + 政策签发 | 律师 | 政策签发 + 占位补全 + 权利流程 | ☐ |
| B3 第三方许可告知 | 律师 | THIRD_PARTY_NOTICES + SBOM + 179 包核验完成 | ☐ |
| C 代码签名 + 可分发包 | 开发者 | 三类二进制均签名 + verify 通过 + 可复现构建 | ☐ |
| D 商店问卷与上架 | 开发者 | 各渠道问卷与政策一致 | ☐ |
| E1–E4 内部工程 | 本仓库 | 26/0 + 门禁 5/5 + 桌面壳 check + 审计 EXIT=0 | ✅ 已具备 |
| E5 179 包许可核验 | 本仓库+律师 | `cargo fetch` 补齐 | ☐ |
| E6 G4 五端一致性 | 本仓库 | 五端 runner 全绿 | ☐ |
| E7/E8 报告与签名包 | 见 A/C | 见上 | ☐ |

**当前结论：NO-GO**（A、B、C、D、E5、E6、E7、E8 均未达成）。本阶段交付的是**准备包**，不是可上架成品。

---

## 5. 诚实边界（唯一冻结措辞，与 [m4-step4-report.md](m4-step4-report.md) §7 一致）

- 本清单**不是**第三方审计报告、不是法律意见、不是合规认证、不是已通过的安全结论。
- 第三方审计与律师签发**本仓库无法代劳**，只能提供准备材料与核验清单。
- 未产出可分发安装包、未签名、证书须自备；内核/桌面壳 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计。
- 非 WinFsp 驱动级挂载；loopback WebDAV 无 per-request 认证；拒绝云同步根内挂载；解锁态只缓解、库外持久副本锁定后不保证消失。
- 许可审计仅覆盖 134+300 个可核验包，**MPL-2.0 弱 copyleft 已存在**，179 个不可核验包不作任何许可声明；「全图无 GPL / 纯宽松许可」不准确。
- 合规事实为 **2026-11-15 抓取时点值**，法规/商店政策会变化，签发前须复核最新文本。

---

## 6. 文档接口

[security-whitepaper.md](security-whitepaper.md) · [privacy-policy.md](privacy-policy.md) · [m4-step4-report.md](m4-step4-report.md) · [desktop/SIGNING.md](desktop/SIGNING.md) · [产品分析与概要设计.md](产品分析与概要设计.md) · [m4-step3-report.md](m4-step3-report.md) · [tests/vectors/CONTRACT.md](tests/vectors/CONTRACT.md)
