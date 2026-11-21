# M4 第 3 点 · WinFsp 增强路径评估与决策

> 版本：v1.0 · 日期：2026-11-15 · 上游：[产品分析与概要设计.md](产品分析与概要设计.md) v1.9（M4 第 3 点当时为 v1.7；M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step4-report.md](m4-step4-report.md)）/ [m3-architecture.md](m3-architecture.md) §6
> 授权：用户 m03177「继续第 3 点」= M4 第 2 点汇报「逐项进行」第 3 条「**WinFsp 增强路径（需另评 GPL-3.0 法务）**」
> 性质：**评估与决策文档，不是实现**。本轮不写挂载代码、不引入 WinFsp 依赖、不实现驱动级挂载。

---

## 0. 结论先行

**决策：本阶段不采用 WinFsp 驱动级挂载增强路径**；M3 已落地的 **loopback-only WebDAV** 仍是唯一已实现的挂载形态，产品代码与依赖树保持零 GPL。

决定性原因（**A 级一手事实**）：

- WinFsp 是 **GPLv3 + 仅面向 FLOSS 的特殊例外**，不是 LGPL、不是「带专有链接例外」。其例外条款逐字规定被授权软件 **"Is not linked or distributed with proprietary (non-FLOSS) software. [You cannot mix FLOSS and proprietary software while using WinFsp under this special exception.]"**（[License.txt](https://api.github.com/repos/winfsp/winfsp/license)）。→ 闭源/专有产品**无法依赖该例外**。
- 上游明确表态：issue #32 "**Closed-source projects would still require a commercial license.**"；作者 "**I would still want to keep the license GPL rather than LGPL.**"（[issues/32](https://api.github.com/repos/winfsp/winfsp/issues/32) + comments 274910261）。
- 商业许可是一道**有标价、有期限的法律门槛**：组织 **6000 USD / 3 年**（单个具名团队 ≤10 开发者、可无限链接/分发闭源产品、无版税、**不含支持**）；个人 **2400 USD / 3 年**（但逐字 "**cannot be used for a single individual within an organization; an organization license must be purchased instead**"）。**3 年非永久**（[winfsp.dev/com/](https://winfsp.dev/com/)）。
- **第二层 copyleft**：Rust 绑定 `winfsp 0.13.2+winfsp-2.1` 与 `winfsp-sys 0.12.1+winfsp-2.1` 本身即 **GPL-3.0**；而商业许可授权的是 "the WinFsp source code and binaries"，绑定是否被覆盖**无上游澄清**（C 级判断，须法务/上游书面确认）。
- **部署侧新增依赖**：已签名内核驱动 MSI；自行构建需 **EV 证书 + Partner Center for Windows Hardware 账号**，否则 `bcdedit -set testsigning on`；驱动安装需提权（上游**无明确声明，NOT RETRIEVED**）。
- **本地依赖树实测**：core `Cargo.lock` 147 包 / desktop `Cargo.lock` 466 包，**可核验范围内 GPL 家族 0**，WinFsp 未被任何源码引用。引入 WinFsp 会引入**首个强 copyleft**，是性质变化，不是「再加一个 copyleft」。

**重新评估触发条件**见 §6。本轮**没有**把任何 GPL 代码引入产品树。

---

## 1. 范围、方法与可信度

| 项 | 内容 |
|---|---|
| 任务边界 | 只做「WinFsp 增强路径」的**许可 / 驱动签名 / 安装成本 / 技术适配 / 替代方案**评估并落决策；不实现、不采购、不签署 |
| 分级 | **A** = 父代理亲自抓取一手页 HTTP 200 并逐字引用；**A⁻** = 子代理经官方 API（api.github.com / crates.io /api/v1 / docs.rs / spdx.org）或官网 JSON 取回；**B** = 趋势或未取到可引用一手页；**C** = 本项目判断；**NOT RETRIEVED** = 取不到，不臆造 |
| 纪律 | 沿用主文档 §9.3：「已修复/已支持/已弃用/已认证」类表述必须对应 A 或 A⁻，B 级不进硬论据；价格/版本/星数注明「2026-11-15 抓取时点」 |
| 三路独立来源 | ① 父代理亲抓 A 级一手页 ② subagent「WinFsp licensing and driver facts」27 行分级事实表 ③ subagent「Windows driverless mount alternatives」替代品判定 ④ subagent「Local dependency license audit」+ 父代理独立复跑（`tools/audit_licenses.py`，EXIT=0，数字一致） |
| 抓取时点 | 2026-11-15（WinFsp v2.1 / winfsp-2.1.25156.msi，发布 2025-06-05） |

---

## 2. A / A⁻ 级事实（决策载荷，逐字保留）

### 2.1 许可性质（A⁻，api.github.com 官方端点）

- [api.github.com/repos/winfsp/winfsp/license](https://api.github.com/repos/winfsp/winfsp/license) 返回 `License.txt` 36658 B（sha `372164f96a3c976286f082eb4cece6c2287c60a5`），开头逐字：
  > "The WinFsp project is Copyright (C) Bill Zissimopoulos. It is licensed under the terms of the GPLv3."
  >
  > "As a special exception to GPLv3, Bill Zissimopoulos grants additional permissions to Free/Libre and Open Source Software ("FLOSS") without requiring that such software is covered by the GPLv3."

  例外**仅授予两项权限**：① 链接平台特定版本 WinFsp DLL（`winfsp-a64.dll` / `winfsp-x64.dll` / `winfsp-x86.dll` / `winfsp-msil.dll`）；② 分发未修改的 WinFsp 安装器二进制发布。**三项条件**逐字：
  > "1. Is distributed under a license that satisfies the Free Software Definition Version 1.141 ... or the Open Source Definition Version 1.9"
  > "2. Includes the copyright notice "WinFsp - Windows File System Proxy, Copyright (C) Bill Zissimopoulos" and a link to the WinFsp repository in its user-interface and any user-facing documentation."
  > "3. **Is not linked or distributed with proprietary (non-FLOSS) software. [You cannot mix FLOSS and proprietary software while using WinFsp under this special exception.]**"
  > "Commercial licensing options are also available: Please contact Bill Zissimopoulos <billziss at navimatics.com>."

  其后为标准 "GNU GENERAL PUBLIC LICENSE Version 3, 29 June 2007" 全文。
- [winfsp.dev/](https://winfsp.dev/)（**A**）："WinFsp is available under the **GPLv3 license with a special exception for Free/Libre and Open Source Software**. A commercial license is also available."
- [api.github.com/repos/winfsp/winfsp](https://api.github.com/repos/winfsp/winfsp)：`license {"key":"other","name":"Other","spdx_id":"NOASSERTION"}`，topics 含 `gplv3`/`kernel`/`driver`（GitHub 无法自动分类，因自定义例外）。
- **不是 LGPL / 不是 FUSE 式专有链接例外**：issue #32 comments 274910261（作者）"I would still want to keep the license GPL rather than LGPL."
- **per-component 许可拆分 NOT RETRIEVED**：`License.txt` 是项目级（"The WinFsp project"），例外只点名 4 个 DLL 文件名；内核驱动 `src/sys`、DLL `src/dll`、FUSE `src/dll/fuse`+`fuse3`、.NET `src/dotnet`、headers `inc/*` 目录布局已确认，但**无上游声明**把驱动单独区别对待（C 级：不假设驱动可与 DLL 分开处理）。

### 2.2 商用许可条款（A，[winfsp.dev/com/](https://winfsp.dev/com/)）

> "Entities that wish to use WinFsp without the restrictions imposed by the GPLv3, may alternatively purchase a commercial license."

| 维度 | 组织许可 | 个人许可 |
|---|---|---|
| 主体 | "a single named team with up to 10 developers" | "a single named individual" |
| 权限 | "unlimited linking and distribution of an organization's **closed-source software products** with WinFsp"；可开发多个闭源产品 | 同类（个人） |
| 期限 | "up to **3 years** from the date in the Agreement" | 3 年 |
| 价格 | **6000 USD every 3 years**，无版税 | **2400 USD every 3 years**，无版税 |
| 限制 | 多团队 / >10 开发者另议 | "**cannot be used for a single individual within an organization; an organization license must be purchased instead**" |
| 支持 | "**does not include any support**"；公共支持 = GitHub issues / Google group；优先支持另签年度合同 | 同左 |
| 联系 | Bill Zissimopoulos <billziss at navimatics.com> | 同左 |

- 上游对闭源产品的立场（**A⁻**，issue #32 正文 + comments）："**Closed-source projects would still require a commercial license.**"；2017 年提议的头部措辞 "Unless you have a commercial license you may not: (1) **use WinFsp from a non-free software project/component**, and (2) distribute WinFsp with a non-free software project/component."；作者 "Yes, I do intend to provide commercial licensing for proprietary projects."
- 商业许可才允许 rebranding（**A**，[WinFsp-Rebranding](https://winfsp.dev/doc/WinFsp-Rebranding/)）："If you have a commercial WinFsp license and wish to produce a rebranded version of WinFsp..."

### 2.3 「只加载 DLL / 让用户自行安装」的规避论 —— 上游未裁决（NOT RETRIEVED）

- **没有任何第一方 WinFsp 声明**明确裁决「仅运行时 delay-load DLL、不分发 WinFsp、让用户自己装」是否构成衍生作品（[FAQ](https://winfsp.dev/doc/Frequently-Asked-Questions/) 无许可章节；issue 搜索无上游背书）。
- 上游商业许可措辞是 "linking and distribution of ... closed-source software products **with** WinFsp"，issue #32 甚至用 "**use** WinFsp from a non-free software project/component"，**均未以「是否 convey」为条件**。
- 开放 issue #341（2020-12-30，仍 OPEN，A⁻）显示例外条款 3 对 OS DLL 的歧义连上游都承认未修：作者 "Good catch. I agree that linking with Windows system DLLs and/or system/language runtimes ... should be allowed. Do you have any suggestion for wording that would make this clearer?"
- GPLv3 文本本身（**A**，[spdx.org/licenses/GPL-3.0.json](https://spdx.org/licenses/GPL-3.0.json)）："You may make, run and propagate covered works that you do not convey, without conditions"；"Inclusion of a covered work in an aggregate does not cause this License to apply to the other parts of the aggregate" —— 但这是**通用 GPL 论点，不是 WinFsp 上游背书**；且 WinFsp 自定义例外对「linking with proprietary software」的限制不依赖 convey。
- **判定（C 级，非法律意见）**：把该规避论当作「已解决的安全港」不可靠。若真要走 WinFsp 且不想买商业许可，须取得上游书面澄清或律师意见。**本报告不把它当作可执行路径。**

### 2.4 驱动 / 签名 / 安装成本（A / A⁻）

| 事实 | 级别 | 来源 |
|---|---|---|
| "WinFsp is released in the form of an **MSI installer that includes a signed driver**" | A | [winfsp.dev/rel/](https://winfsp.dev/rel/) |
| 自行构建 "requires an **EV certificate and a Partner Center for Windows Hardware account**"；否则未签名驱动 Windows 拒载，须 `bcdedit -set testsigning on` | A | [WinFsp-Building](https://winfsp.dev/doc/WinFsp-Building/) |
| MSI 静默安装 `msiexec /i winfsp-x.x.x.msi /qn INSTALLLEVEL=1000`（省略 INSTALLLEVEL 只装 core） | A⁻ | issues/332 comments 722706133（作者） |
| "You should **not need to reboot**, unless WinFsp was already running on your system." | A | winfsp.dev/rel/ |
| 支持 "Windows 7 to Windows 11 and the x86, x64 and ARM64 architectures" | A⁻ | README（api.github.com contents） |
| 运行时文件系统进程**无需管理员**（"Start a non-admin Command Prompt"） | A | winfsp.dev/rel/ |
| **驱动安装需提权：NOT RETRIEVED**（无上游明确声明；仅第三方 issue #673 "run as Administrator" = B 级；C 级推断驱动安装通常需提权，不作事实） | — | — |
| DLL 故意不装进系统目录；推荐 **delay load + `FspLoad`**；默认装到 `C:\Program Files (x86)\WinFsp` | A | FAQ / [WinFsp-Tutorial](https://winfsp.dev/doc/WinFsp-Tutorial/) |
| 最新 release v2.1 "WinFsp 2025"（2025-06-05），`winfsp-2.1.25156.msi`（sha256 `073a70e0…f7a`，725055 downloads） | A⁻ | api.github.com releases/latest |
| Win7 缺 SHA-2 代码签名支持会显示驱动未签名；FUSE 支持 2.8 / 3.2 API；WinFsp-FUSE 用 reparse point 挂目录、不能挂非空目录 | A | FAQ |

### 2.5 Rust 绑定（A⁻，crates.io /api/v1 + docs.rs + 上游 README）

| crate | 版本 | 许可 | 备注 |
|---|---|---|---|
| `winfsp` | 0.13.2+winfsp-2.1 | **GPL-3.0** | 42 版本全 GPL-3.0；2026-10-20 更新；110690 downloads；repo `SnowflakePowered/winfsp-rs`；edition 2024 |
| `winfsp-sys` | 0.12.1+winfsp-2.1 | **GPL-3.0** | 11 版本全 GPL-3.0；raw FFI |
| `winfsp-rs` | — | — | **crate 不存在**（crates.io 404）；只是 GitHub 仓库名 |
| `dokany` | — | — | **crate 不存在**（crates.io 404） |
| `dokan` | 0.3.1+dokan206 | **MIT** | wrapper；最后发布 2022-12-08（≈4 年陈旧） |
| `dokan-sys` | 0.3.1+dokan206 | **MIT** | raw FFI；`lib_links="dokan"`；同期 |

- `winfsp-rs` README「## Legal」逐字："**winfsp-rs is licensed under the terms of the GNU General Public License version 3** as published by the Free Software Foundation."；并警示绑定 "there is still a chance these bindings are unsound"。
- [docs.rs/winfsp](https://docs.rs/winfsp/latest/winfsp/)：**"WinFSP only supports delayloading of its library. You must emit the required compile flags in `build.rs` with `winfsp_link_delayload`."**（即 `build.rs` 必须调 `winfsp::build::winfsp_link_delayload()`）。
- **C 级判断**：GPL-3.0 绑定是在 WinFsp 之上的**第二层 copyleft 面**；商业许可逐字只授权 "the WinFsp source code and binaries"，未明确覆盖第三方 Rust 绑定 —— 这个错配须律师/上游澄清，不能默认被覆盖。

### 2.6 抓取失败记录（不臆造）

- `https://winfsp.net/` → `TypeError: fetch failed`（用 `winfsp.dev`，可访问）。
- `github.com` HTML 页 / `raw.githubusercontent.com` 直取 → `TypeError: fetch failed`；改用 `api.github.com/repos/...` 与 `/contents`、`/license` 端点（HTTP 200，base64 解码）。
- `https://raw.githubusercontent.com/wiki/winfsp/winfsp/Frequently-Asked-Questions.md` → fetch failed（用 winfsp.dev/doc 的 FAQ 页代替）。
- cfapi 某文档路径 404；ProjFS 版本/edition 精细限制 NOT RETRIEVED；WSL Linux 内核许可 NOT RETRIEVED。

---

## 3. 替代方案对比（Windows 真·本地文件语义）

判定框架两条硬约束：**① 不引入 GPL-3.0 内核驱动 / 强 copyleft ② 真 RW 本地文件语义（非网络重定向）**。来源：subagent「Windows driverless mount alternatives」+ 父代理 A 级复核。

| 方案 | 许可 | 驱动/部署 | 真 RW 本地语义 | 判定 |
|---|---|---|---|---|
| **WinFsp**（本点评估对象） | **GPLv3 + FLOSS-only 例外**（闭源须 6000 USD/3yr 商业许可） | 已签名内核驱动 MSI；自构建需 EV 证书+Partner Center；用户机提权安装 | ✅（NTFS 级语义、缓存、锁、删除/重命名） | ❌ 本阶段不采用（§0） |
| **Dokany / Dokan** | 驱动与用户库 **LGPLv3**；`dokanctl.exe`+samples MIT；Rust `dokan`/`dokan-sys` **MIT** | **非零驱动**：管理员装已签名内核驱动（DokanSetup.exe / MSI / choco / winget）；活跃维护 v2.3.1.1000（2025-09-28），Win7 SP1–Win11/Server 2022，x86/x64/ARM/ARM64 | ✅ 完整 Win32 FS（CreateFile/ReadFile/WriteFile → 用户态回调） | 🟡 **唯一同时满足两条硬约束的备选**；代价=失去零驱动 + LGPLv3 仍有义务（提供库源码/允许重链接）**须法务复核** + Rust 绑定 2022 年陈旧（≈4 年） |
| **ProjFS**（ProjectedFSLib） | Windows 内置可选组件（OS 许可，不随产品分发） | 默认关闭；须提权 `Enable-WindowsOptionalFeature -Online -FeatureName Client-ProjFS -NoRestart`、可能重启；最低 Win10 1809 [desktop apps only] | ❌ 投影/缓存于既有 NTFS virtualization root；写入使文件变 "full file"、"no longer a cache of its state in the provider's store"（copy-up 分叉）→ **明文落到 NTFS，对加密 vault 不利** | ❌ 不适用 |
| **CfAPI**（cldapi.dll / Cloud Files） | Win10 1709 起内置，`cldflt.sys` inbox minifilter（不随产品分发） | 须 sync-root 注册（Windows.Storage.Provider + 注册表 CopyHook/ShareHandler/SearchHandlerFactory）+ Desktop Bridge 打包 + "Cloud sync engines must be implemented in desktop apps"；"Cldflt.sys currently only supports NTFS volumes" | ❌ 占位/水合/脱水三态（Placeholder/Full/Pinned full）、hydration policy `max(app, provider)`、系统可自动脱水 | ❌ C 级：纯本地加密 vault 的过度设计 |
| **WSL `\\wsl$` / plan9** | 不自带驱动 | 需 WSL 特性 + distro | ❌ 非 Win32 真 RW（微软 "We recommend against working across operating systems with your files"） | ❌ 不适用 |
| **SMB loopback** | OS 原生零驱动 | 零驱动 | ❌ 仍 network-redirector / oplock / leasing 语义（与现行 WebDAV 同类） | ❌ 不带来升级 |
| **rclone mount on Windows** | rclone 本体 MIT | **"you will need to download and install WinFsp"** → 继承 GPLv3 驱动 | ⚠️ 默认 `--vfs-cache-mode off` 时 "Files can't be opened for both read AND write"、不能 seek、O_APPEND/O_TRUNC 被忽略；仅 `writes`/`full` 支持全操作 | ❌ 继承 WinFsp GPLv3，被排除（除非买商业许可） |
| **现行 loopback-only WebDAV**（M3 已实现） | dav-server 0.11.0 Apache-2.0 等宽松许可 | **零驱动**、OS 原生 `net use` | ❌ 网络重定向语义、loopback 无 per-request 认证（诚实边界已冻结） | ✅ **保留为默认**：已验证（26/0 测试 + 真实网络冒烟），零 copyleft、零驱动 |

**ONE-LINE**：若两硬约束同时成立 → **Dokany 是唯一选项**（但仍需法务复核 LGPLv3 与陈旧绑定）；若「零驱动」不可谈判 → **保留 WebDAV/SMB loopback**。ProjFS / CfAPI / WSL / SMB / rclone 均不满足。

---

## 4. 本地依赖树许可审计（A 级，父代理独立复跑 EXIT=0）

工具：[tools/audit_licenses.py](tools/audit_licenses.py)（只读，解析 `Cargo.lock` → 本地展开的 `Cargo.toml` `license`/`license-file` 字段；日志 [tools/license-audit.log](tools/license-audit.log)）。subagent 与父代理两次独立运行数字一致。

| workspace | 锁文件包数 | 本地可核验 | 许可分组 | **GPL 家族** | 弱 copyleft（MPL-2.0） | 不可核验 |
|---|---|---|---|---|---|---|
| `core/Cargo.lock` | **147**（3 第一方 path + 144 crates.io） | 134 | 9 组 | **0** | 1（`htmlescape 0.3.1` = "Apache-2.0 / MIT / MPL-2.0"，三许可含宽松替代） | 13（**全部** wasm32/wasi/uefi/unix/android 非 Windows 分支） |
| `desktop/src-tauri/Cargo.lock` | **466**（3 第一方 path + 463 crates.io） | 300 | 22 组 | **0** | 6（`cssparser 0.37.0`、`cssparser-macros 0.7.1`、`dtoa-short 0.3.5`、`option-ext 0.2.0`、`selectors 0.38.0`、`htmlescape 0.3.1`） | 166（含 Linux GTK/WebKit、macOS、Android、wasm；**部分 Windows 相关**：winapi 0.3.9 / windows-sys 0.45.0 / windows-targets 0.42.2 家族 / winreg 0.56.0 / vswhom* / reqwest 0.13.5 / syn 1.0.109 / thiserror 1.0.69 / bitflags 1.3.2 等） |

关键结论：

- **GPL / LGPL / AGPL / SSPL / BSL / EPL / CDDL / CC-BY-SA 表达式：0**（134 可核验 core 包 + 300 可核验 desktop 包）；`Cargo.lock` 与仓库源码文本中 **0 处 WinFsp / GPL 引用**。
- 独立短语扫描 1924 个展开文本文件：**0 个 `Cargo.toml` 含 GPL 短语**；4 个 LICENSE 文件含 "GNU General Public License" 字样，但**全部是 MPL-2.0 §1.12「Secondary License」定义**，不是 GPL 授权（逐字核对 `cssparser-0.37.0\LICENSE:67-71`："1.12. "Secondary License" means either the GNU General Public License, Version 2.0, ..."）—— **朴素 grep 会在此误报**。
- 第一方 crate 许可：`core/Cargo.toml:8` 与 `desktop/src-tauri/Cargo.toml:7` 均 `license = "MIT OR Apache-2.0"`。
- **诚实边界**：现行树不是「纯宽松许可」—— MPL-2.0 弱 copyleft / 文件级 copyleft 已存在（1 core + 6 desktop）；但**与 GPL-3.0 内核驱动有性质差别**。179 个包（13 + 166）本地不可核验（无展开 `Cargo.toml` 也无缓存 `.crate`，sparse index 无 `license` 键），**对它们不作任何许可声明**；最终法务签发前须 `cargo fetch` / 读 crates.io 元数据补齐。**不能据此宣称「全图无 GPL」**，只能说「可核验范围内 GPL 家族 0」。
- **引入 WinFsp 会引入首个强 copyleft**（WinFsp GPLv3 + winfsp/winfsp-sys GPL-3.0 绑定），是性质变化而非「再加一个 copyleft」。

---

## 5. 决策与理由

**决策：M4 第 3 点 = 评估完成，结论「不采用 WinFsp 增强路径」，保留 loopback-only WebDAV 为默认。** 不写挂载代码、不加 WinFsp 依赖、不采购商业许可。

| # | 理由 | 级别 |
|---|---|---|
| 1 | FLOSS 例外**逐字禁止与专有软件混用**；MyLock 当前按 `MIT OR Apache-2.0` 双许可组织，未承诺全量 FLOSS，无法安全依赖该例外 | A |
| 2 | 上游明确「闭源项目仍需商业许可」，且**曾明确拒绝改 LGPL**；商业许可 6000 USD/3yr（组织）是真实、可标价、**非永久**的门槛 | A / A⁻ |
| 3 | 「只 delay-load、让用户自装、不分发」的规避论 **NOT RETRIEVED**（无上游背书），开放 issue #341 连 OS DLL 歧义都未修 → 未解决的法律风险，非安全港 | A⁻ / C |
| 4 | `winfsp`/`winfsp-sys` 绑定本身 GPL-3.0，商业许可是否覆盖**无上游澄清** → 第二层 copyleft 错配 | A⁻ / C |
| 5 | 驱动级方案新增**已签名内核驱动 + 用户机提权安装 + 自构建 EV 证书/Partner Center** 的部署与运维成本，与产品「零驱动、零服务端、零安装负担」反定位相冲突 | A / C |
| 6 | 本地实测依赖树**可核验范围内 GPL 家族 0**；引入 WinFsp 会**首次引入强 copyleft**，把现行「宽松 + 少量 MPL 弱 copyleft」的干净姿态换成 GPL 面 | A |
| 7 | 现行 WebDAV 已被 26/0 测试 + 真实网络冒烟验证，零驱动、零 copyleft、零安装负担；**没有用户痛点证据**表明必须升级到驱动级语义才成立 | C |

**明确拒绝的三条捷径**（防止后续被误当成已解决）：

- ❌ 「用 `winfsp` crate 但不开源产品」→ 绑定 GPL-3.0，第二层 copyleft 未澄清。
- ❌ 「不分发 WinFsp、让用户自己装、我们只 delay-load」→ 上游未裁决（NOT RETRIEVED），issue #341 歧义未修。
- ❌ 「改用 rclone mount 绕开」→ rclone on Windows 仍需装 WinFsp，继承 GPLv3。

---

## 6. 重新评估的触发条件（activation triggers，非当前承诺）

| 触发条件 | 可走路径 | 前置动作 |
|---|---|---|
| 产品决定**整体转为 FLOSS**（按 OSD 1.9 / FSD 1.141 兼容许可分发），且接受 UI/文档署名 + 仓库链接 + 不与专有软件混用 | 可依 FLOSS 例外使用 WinFsp（免商业许可） | 法务确认全产品许可兼容；仍须处理 `winfsp` crate GPL-3.0 绑定与自身 GPL 化一致 |
| 公司**采购商业许可**（预算 6000 USD/3yr 组织许可，注意每 3 年续期、不含支持），并取得上游**书面澄清**覆盖 Rust 绑定 | WinFsp 增强路径可立项 | 预算 + 合同 + EV 证书/Partner Center + 驱动分发与签名的运维方案；须用户新授权 |
| 「真本地文件语义」成为硬需求、**且法务接受 LGPLv3** | **Dokany**（LGPLv3 驱动 + MIT Rust 绑定）为唯一备选 | 复核 Dokan C 库/驱动许可、LGPLv3 义务、Rust 绑定 2022 陈旧度、提权安装 UX；须用户新授权 + 法务签字 |
| 以上均不成立 | **保留 loopback-only WebDAV**（现状） | 无 |

> 任何一条路径都不改变当前诚实边界：非 WinFsp 驱动级挂载、loopback 无 per-request 认证、拒绝云同步根内挂载。

---

## 7. 诚实边界（本点唯一冻结措辞）

- 本轮是**评估与决策**，**不是法律意见**；许可结论均标注 A/A⁻/B/C 与 NOT RETRIEVED，须由执业律师做最终签发。
- **未把任何 GPL 代码引入产品树**；`core/` 与 `desktop/` 源码、两个 `Cargo.lock` 均不含 WinFsp / GPL-3.0 引用。
- **未实现驱动级挂载**：`mount` 在独立恢复 CLI 仍明确拒绝；桌面壳仍只 spawn `mylock-serve`（loopback WebDAV）。
- **未产出可分发安装包 / 未代码签名 / 证书自备**（沿用 [desktop/SIGNING.md](desktop/SIGNING.md)）。
- **不是 G4 五端验收通过**（Android/iOS 未实现，macOS/Linux CI 已配置未实跑）；**内核/桌面壳 ≠ 产品发布 ≠ 五端验收 ≠ 第三方安全审计**。
- 依赖审计结论仅覆盖 **134 可核验 core 包 + 300 可核验 desktop 包**；179 个本地不可核验包**不作任何许可声明**，最终法务签发前须 `cargo fetch` 补齐；MPL-2.0 弱 copyleft 已存在，不得称「纯宽松许可」。
- 价格 / 版本 / 星数均为 **2026-11-15 抓取时点**，会变动。

---

## 8. 交付物与文档接口

| 交付物 | 说明 |
|---|---|
| [m4-step3-report.md](m4-step3-report.md)（本文） | WinFsp 增强路径评估与决策（唯一冻结措辞 §7） |
| [tools/audit_licenses.py](tools/audit_licenses.py) | 只读依赖许可审计脚本（解析两个 Cargo.lock → 本地展开 Cargo.toml 的 license 字段） |
| [tools/license-audit.log](tools/license-audit.log) | 父代理独立复跑日志（EXIT=0；core 147 / desktop 466 / GPL 0 / MPL 7 / 179 不可核验） |
| 主文档 [产品分析与概要设计.md](产品分析与概要设计.md) **v1.9**（M4 第 3 点当时为 v1.7；M4 第 4 点后升 v1.8、M5 后升 v1.9） | 表头 / 配套文件 / §7.1 虚拟文件系统行 / §8.2 里程碑（新增 M4 第 3 点行）/ §9.3 维护约定回填 |
| [m3-architecture.md](m3-architecture.md) §6 第 2 条 | 「WinFsp 增强路径：另行评估」→「已评估，结论=不采用；Dokany 列备选需新授权」 |

复现依赖审计：

```powershell
& 'C:\Python312\python.exe' 'D:\Projects\dsh\mylock\tools\audit_licenses.py'
```

一手来源（全部 HTTP 200，2026-11-15）：[winfsp.dev](https://winfsp.dev/) / [/com/](https://winfsp.dev/com/) / [/rel/](https://winfsp.dev/rel/) / FAQ / WinFsp-Building / WinFsp-Tutorial / WinFsp-Rebranding / Native-API-vs-FUSE；[api.github.com/repos/winfsp/winfsp](https://api.github.com/repos/winfsp/winfsp)（+ `/license`、`/issues/32`、`/issues/341`、`/issues/332`、`/releases/latest`、`/contents/README.md`）；[crates.io/api/v1/crates/winfsp](https://crates.io/api/v1/crates/winfsp) / `winfsp-sys` / `dokan` / `dokan-sys`；[docs.rs/winfsp](https://docs.rs/winfsp/latest/winfsp/)；[spdx.org/licenses/GPL-3.0.json](https://spdx.org/licenses/GPL-3.0.json)；[learn.microsoft.com ProjFS](https://learn.microsoft.com/en-us/windows/win32/projfs/projected-file-system) / CfAPI；[dokan-dev.github.io](https://dokan-dev.github.io/)；[rclone.org/commands/rclone_mount](https://rclone.org/commands/rclone_mount/)。NOT RETRIEVED：winfsp.net、直取 github.com/raw.githubusercontent.com、wiki FAQ、cfapi 某 404 路径。
