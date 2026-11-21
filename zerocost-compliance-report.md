# 零成本合规最小动作 · 本轮报告（zerocost-compliance-report）

> **本文件不是合规结论、不是法律意见、不是第三方审计报告、不是已通过认证。**
> 草案未经律师 / 合规律师签发不得对外发布；当前状态为 **NO-GO**。
> 本文件 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计；未产出可分发包 · 未签名 · 证书自备。
> 合规事实为 **2026-11-15 抓取时点值**。
> 本文件不给出任何「已合规」「已通过认证」结论，也**绝不说「没有任何 copyleft」**。

---

## 0. 本轮定位与纪律

- 本支负责「**零成本合规最小动作**」：所有产出均为**零现金、只花时间**的动作（读文件、跑 `cargo fetch`、写脚本、起草模板 / 须知 / 报告）。
- 本轮**不**给合规结论、**不**给法律意见、**不**做第三方审计、**不**买证书、**不**签名、**不**发布、**不**上架、**不**收费。
- 只写这 5 个文件（见 §5）；严禁改动的文件（产品分析与概要设计.md、prelaunch-checklist.md、privacy-policy.md、security-whitepaper.md、m4-step4-report.md、tools/audit_licenses.py、core/**/*.rs、desktop/src-tauri/src/*、tools/verify-m4s4.ps1）本轮**一律只读引用**，未改动。
- 桌面壳架构按父代理最新事实为「**进程内引擎**」（不再 spawn mylock-serve sidecar）；`BETA-NOTICE.md` 已据此描述，未写外部 sidecar。

---

## 1. 本轮做了什么（零现金、只花时间）

1. **读背景**：只读 `tools/audit_licenses.py`（389 行）、`m4-step4-report.md`（§2.3.1 / §2.3.2 / §2.3.8 / §2.4 / §2.4.1 / §5 冻结边界），直接引用其中 A / A⁻ 一手事实，**不重复调研**。
2. **基线审计**：`C:\Python312\python.exe tools\audit_licenses.py` 在 fetch 前跑一次，记录「可核验 / 不可核验」基线。
3. **cargo fetch**：对 `core` 与 `desktop/src-tauri` 两个 workspace 分别执行 `cargo fetch --locked`（均 EXIT=0，锁文件未被改动）。环境用显式 `C:\Users\zjc_9\.cargo\bin\cargo.exe`；pwsh 把 cargo stderr 渲染成 NativeCommandError 是假象，以退出码为准。
4. **fetch 后复跑**同一只读脚本，记录数字变化，并从真实抓到的 registry 元数据核查「第 6 个 MPL 包」。
5. **写生成器 / 生成告知稿**：`tools/gen_third_party_notices.py`（只读、可重复运行、幂等）生成 `THIRD_PARTY_NOTICES.md`。
6. **起草 3 份准备文档**：`BIS-742.15b-notice-template.md`、`BETA-NOTICE.md`、本报告。

---

## 2. cargo fetch 前后数字（核心结果）

### 2.1 两个 workspace 的「可核验 / 不可核验」

| workspace | lock 包数 | fetch 前 可核验 | fetch 前 不可核验 | fetch 后 可核验 | fetch 后 不可核验 |
| --- | ---: | ---: | ---: | ---: | ---: |
| core | 147 | 134 | 13 | 147 | 0 |
| desktop/src-tauri | 466 | 300 | 166 | 466 | 0 |
| **合计** | **613** | **434** | **179** | **613** | **0** |

- **179 → 0**：`cargo fetch --locked` 补齐本机 registry 元数据后，原先 **179 个不可核验包（core 13 + desktop 166）全部可核验**，可核验范围从 **434** 扩到 **613（全覆盖）**。
- 记为「**可核验全覆盖已达成**」。但冻结边界里「**179 个不可核验不作声明**」属 **fetch 前历史时点值**，**不改写**：在补齐元数据之前，那 179 个包确实未核验、不作声明。

### 2.2 审计脚本其他口径（fetch 前 → fetch 后）

| 指标 | fetch 前 | fetch 后 | 说明 |
| --- | --- | --- | --- |
| core 许可表达式组数 | 9 | 11 | 新增可见表达式 |
| desktop 许可表达式组数 | 22 | 28 | 新增可见表达式 |
| core GPL-expression hits | 0 | 1 | 来自 r-efi 6.0.0 的 LGPL OR 选项 |
| desktop GPL-expression hits | 0 | 2 | 来自 r-efi 5.3.0 / 6.0.0 的 LGPL OR 选项 |
| core COPYLEFT 标志 | 1 | 2 | htmlescape + r-efi LGPL-OR |
| desktop COPYLEFT 标志 | 6 | 10 | 5 纯 MPL + htmlescape → 7 纯 MPL + htmlescape + 2 r-efi |
| 全局扫描文本数 | 1924 | 2921 | fetch 后本机源码变多 |
| Cargo.toml 中 GPL 短语 | 0 | 2 | r-efi 的 license 字段 |
| LICENSE/README 样式 GPL 短语 | 4 | 27 | **解压源码里的关键词命中、不是依赖声明的许可**（多为 MPL-2.0 §1.12 "Secondary License" 字面、LLVM exception 引用、COPYRIGHT 描述文字；典型 `r-efi-*/AUTHORS`、`pango-0.18.3/COPYRIGHT`、`target-lexicon-0.12.16/LICENSE`、`wasi-*/LICENSE-Apache-2.0_WITH_LLVM-exception`、`wit-bindgen-0.57.1/...`）。**既不把它们当作 copyleft 结论，也不声称它们全是误报**——逐文件定性须律师/许可核验（外部闸门）。与「GPL-expression hits core 1 / desktop 2（全部来自 `r-efi` 的 SPDX OR 表达式）」是两个不同口径，勿混用；详见 [m4-step4-report.md](m4-step4-report.md) §2.4.2 |

### 2.3 关于「可核验范围 GPL 家族 0」的口径（关键标注）

- `m4-step4-report.md` §2.4 的「**可核验范围内 GPL 家族 0**」是 **fetch 前（434 包可核验子集）口径**：当时 179 个包不可核验、不作声明，故该结论只覆盖当时能核验的 434 包。
- **fetch 后（613 包全覆盖）新增观察**：`r-efi 5.3.0` 与 `r-efi 6.0.0` 的 SPDX 表达式为 `MIT OR Apache-2.0 OR LGPL-2.1-or-later`，含 `LGPL-2.1-or-later` **可选原子**，使审计脚本的 GPL-expression token 命中 core 1 / desktop 2。
- 我们取 **MIT / Apache-2.0 选项**、不承担 LGPL 义务；但**不要**写成「依赖树绝对没有 GPL/LGPL 字样」——该字样确实在元数据中出现，只是作为 OR 可选项。是否触发 LGPL 义务须逐依赖确认并留痕（见 `THIRD_PARTY_NOTICES.md` §8.3）。
- **目标门控事实（不作「已编译进产物」的断言）**：`r-efi` 是 `getrandom` 在 `[target.'cfg(all(target_os = "uefi", getrandom_backend = "efi_rng"))'.dependencies.r-efi]`（`getrandom-0.3.4/Cargo.toml:91`、`getrandom-0.4.3/Cargo.toml:103`）下的 **target-specific 传递依赖**；仅在 **UEFI 目标 + `efi_rng` 后端** 下才被拉入依赖图，对本项目当前 `x86_64-pc-windows-gnu` 目标**不参与编译**，仅作为两份 `Cargo.lock` 里的 target-specific 条目存在。Cargo.lock 上下文：`core/Cargo.lock` getrandom 0.4.3；`desktop/src-tauri/Cargo.lock` getrandom 0.3.4（r-efi 5.3.0）与 getrandom 0.4.3（r-efi 6.0.0）。
- 以上为**如实新数据点**，**不改写冻结诚实边界**，也不宣称存在 GPL 家族强 copyleft 义务。

### 2.4 对外统一措辞（父代理指令）

> **无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft**；绝不能说「没有任何 copyleft」。

---

## 3. 「第 6 个 MPL 包」结论（从真实抓到的元数据，不臆造）

### 3.1 背景

`m4-step4-report.md` §2.4 记「desktop 6 个 MPL」，但只点名了 5 个纯 MPL-2.0：
`cssparser 0.37.0` / `cssparser-macros 0.7.1` / `dtoa-short 0.3.5` / `option-ext 0.2.0` / `selectors 0.38.0`，**第 6 个待查**。

### 3.2 结论

- 那 6 个 COPYLEFT 标志里，5 个是纯 `MPL-2.0`；**第 6 个 = `htmlescape 0.3.1`**，其 `license = "Apache-2.0 / MIT / MPL-2.0"` 是 **"/" 分隔、非 SPDX "OR" 表达式**的三许可（MPL-2.0 仅三选一之一），**不是第 6 个纯 MPL-2.0 包**。
- 它与 core 侧「1 个 MPL」是**同一个包**；fetch 前就已可核验。**绝不臆造一个「第 6 个纯 MPL-2.0 包」**。
- `htmlescape 0.3.1` 本机解包目录无独立 LICENSE 文件（含 `.cargo-ok` / `.gitignore` / `.travis.yml` / `Cargo.toml` / `moonstone-short.txt` / `README.md`）；报告 §6 6g 记其上游 `github.com/veddan/rust-htmlescape/master/LICENSE` 为 HTTP 404。三许可择一须**逐依赖书面决定**并经律师签发，未定前 `THIRD_PARTY_NOTICES.md` §8 不臆测、不计入「纯 MPL-2.0 受管辖清单」，也不宣称不受 MPL-2.0 约束。

### 3.3 fetch 后纯 MPL-2.0 的真实数量

- fetch 后 desktop 纯 `MPL-2.0` 包从 **5 增至 7**：新增 `cssparser 0.38.0`、`selectors 0.41.0`（二者 fetch 前不可核验）。
- 加上 `htmlescape 0.3.1`（MPL 仅作为非 SPDX '/' 选项，core + desktop 各 1 行），`THIRD_PARTY_NOTICES.md` §2 记：纯 `MPL-2.0` 依赖行 **7**、MPL 仅作为非 SPDX '/' 选项的依赖行 **2**。
- 真实元数据逐条 `license=` 行：`cssparser-0.37.0` / `0.38.0`、`cssparser-macros-0.7.1`、`dtoa-short-0.3.5`、`option-ext-0.2.0`、`selectors-0.38.0` / `0.41.0` = `MPL-2.0`；`htmlescape-0.3.1` = `Apache-2.0 / MIT / MPL-2.0`；`r-efi-5.3.0` / `6.0.0` = `MIT OR Apache-2.0 OR LGPL-2.1-or-later`。

### 3.4 其他核验事实

- 本机 registry src 中带 `NOTICE*` 文件的 crate 数 = **0**（`TOTAL_CRATES_WITH_NOTICE=0`）→ Apache-2.0 §4(d) 的「上游含 NOTICE 才触发」边界：本地依赖树内未见上游 NOTICE 文件；此为**本地事实**，不作法律结论。
- 两份 `Cargo.lock` 均未被 fetch 改动（core `2026/11/14 19:12:41` len=34579；desktop `2026/11/14 19:25:36` len=118735，均早于本轮 12:24）。

---

## 4. THIRD_PARTY_NOTICES.md 生成结果

- 生成器 `tools/gen_third_party_notices.py` 连续多次运行 **EXIT=0**、字节数恒定（**幂等**），唯一写出目标 `THIRD_PARTY_NOTICES.md`（本次最终 148184 字节、1179 行）。
- 只读、不改动上游：不写 Cargo.lock / Cargo.toml / registry 缓存，`.crate` 仅内存读取不解压落盘；解析口径与 `tools/audit_licenses.py` 一致。
- 生成物含：§0 冻结边界（8 条）、§1 数据来源、§2 总览（613/613/0 NOT VERIFIABLE、28 组表达式）、§3 许可全文获取方式索引（E1..E28，每表达式给官方全文 URL + SPDX 页面 + 包内 LICENSE 检出 + 真实 Copyright 行示例 + authors）、§4 全部依赖逐行清单（core 147 + desktop 466，每行带 `E<n>` 指向 §3）、§5 MPL-2.0 专门段（§3.1–3.4 / Exhibit A / FAQ Q1·Q8·Q11·Q12·Q26 + 源码可得要约 "reasonable means in a timely manner, at a charge no more than the cost of distribution" + 链接未修改 MPL 库不须公开自身源码 / 被修改具体文件须提供修改后源码）、§6 Apache-2.0 §4(a)–(d) 逐字 + §4(d) 仅上游含 NOTICE 触发、§7 MIT 归属逐字、§8 双许可非 SPDX 逐依赖判定（htmlescape + LGPL-OR + r-efi 目标门控）、§9 冻结边界复述、§10 应用内 Acknowledgements 一致性。
- 无法核验者一律标 `NOT VERIFIABLE`（本次实际为 0 个），绝不猜许可；`htmlescape 0.3.1` 的 "/" 非 SPDX "OR" 表达式逐依赖判定、不自动择一。
- 修复记录：f-string 中 `"/"` 双引号被当除法（TypeError）→ 统一改为 `'/'`；版权行匹配由宽松 `\bcopyright\b` 收紧为 `\bcopyright\s*(?:\(c\)|©|\(C\)|[0-9]{4})`，避免把许可正文 "the copyright owner" / "COPYRIGHT AND PERMISSION NOTICE" / MIT "The above copyright notice ..." 误当版权声明。

---

## 5. 产出文件清单（本轮只写这 5 个）

| # | 路径 | 性质 |
| --- | --- | --- |
| 1 | `D:\Projects\dsh\mylock\tools\gen_third_party_notices.py` | 只读、可重复运行的生成器脚本 |
| 2 | `D:\Projects\dsh\mylock\THIRD_PARTY_NOTICES.md` | 由 #1 生成的第三方许可告知草稿 |
| 3 | `D:\Projects\dsh\mylock\BIS-742.15b-notice-template.md` | §742.15(b) 通知正文**模板（未发出 / 未填写）** |
| 4 | `D:\Projects\dsh\mylock\BETA-NOTICE.md` | 中文「内测须知」准备文档 |
| 5 | `D:\Projects\dsh\mylock\zerocost-compliance-report.md` | 本文件（本轮汇总报告） |

> 全部为**准备包 / 草稿 / 模板**，未经律师签发不得对外发布；均零现金、只花时间。

---

## 6. 失败 / 未完成 / 注意事项（如实登记）

- **无业务失败**：两次 `cargo fetch --locked` 均 EXIT=0；基线 / 复跑审计均 EXIT=0；生成器多次 EXIT=0 且幂等。
- **途中已修复的技术问题**：生成器 f-string `"/"` 除法 TypeError、版权行宽松正则误报 —— 均已修复并复验（见 §4）。
- **未完成（本轮范围外 / 须签发）**：
  - BIS 模板仅骨架，所有 `[待填]` 占位为空，**未填写、未发出、未提交**；书面分类仍须律师 / 合规律师签发。
  - `BETA-NOTICE.md` 的「付费意愿比例阈值」**本轮未定具体数值**（须产品 / 负责人预先书面设定），未臆造百分比。
  - `htmlescape 0.3.1` 三许可择一、`r-efi` LGPL-OR 选项的逐依赖许可决定，**未作书面结论**（留待发行前逐依赖确认并经律师签发）。
  - 未产出可分发包、未签名、未做第三方审计、未上架、未收费 —— 这是**冻结边界**，不是遗漏。
- **口径提示**：`m4-step4-report.md` §2.4 的「GPL 家族 0」为 fetch 前（434 包子集的）口径；本轮不改写该报告，fetch 后全覆盖的新观察由本报告与 `THIRD_PARTY_NOTICES.md` §8.3 如实登记，父代理以「追加补遗」方式并入其维护的文件。

---

## 7. 冻结诚实边界（原样沿用，不得改写）

- 本轮准备包**非合规结论 / 非第三方审计报告 / 非法律意见 / 非已通过认证**。
- 草案**未经律师签发不得对外发布**。
- 当前 **NO-GO**；**≠ 产品发布 ≠ 五端验收 ≠ 第三方审计**。
- **未产出可分发包 · 未签名 · 证书自备**。
- **推迟 ≠ 豁免**：EAR 出口分类、PIPL/GDPR controller 定性、MPL 源码可得义务**不因规模小或免费消失**。
- 合规事实为 **2026-11-15 抓取时点值**，签发前须复核；A⁻ 来源（gdpr-info.eu）须回官方核对；`NOT RETRIEVED` 不得当作已核验事实引用。
- **准确措辞是「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」，绝不能说「没有任何 copyleft」。**
- 本文件**不是合规结论、不是法律意见、不是第三方审计报告、不是已通过认证**；所有条文 / 版本 / 商店规则为 **2026-11-15 抓取时点值**。
