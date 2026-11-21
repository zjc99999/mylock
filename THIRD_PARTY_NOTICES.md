# THIRD_PARTY_NOTICES.md（第三方许可告知 · 自动生成草稿）

> **本文件由 `tools/gen_third_party_notices.py` 只读生成，是【告知草稿】，不是合规结论、不是法律意见、不是第三方审计报告、不是已通过认证。**
> 草案未经律师 / 合规律师签发不得对外发布。当前状态为 **NO-GO**。
> 本文件 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计；未产出可分发包 · 未签名 · 证书自备。
> 所有许可 / 条文 / 版本事实为 **2026-11-15 抓取时点值**，签发前须复核。
> 本文件不给出任何「已合规」「已通过认证」结论，也**绝不说「没有任何 copyleft」**。


---

## 0. 冻结诚实边界（原样沿用，不得改写）

- 本轮准备包**非合规结论 / 非第三方审计报告 / 非法律意见 / 非已通过认证**。
- 草案**未经律师签发不得对外发布**。
- 当前 **NO-GO**；**≠ 产品发布 ≠ 五端验收 ≠ 第三方审计**。
- **未产出可分发包 · 未签名 · 证书自备**。
- **推迟 ≠ 豁免**：EAR 出口分类、PIPL/GDPR controller 定性、MPL 源码可得义务**不因规模小或免费消失**。
- 合规事实为 **2026-11-15 抓取时点值**。
- **准确措辞是「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」，绝不能说「没有任何 copyleft」。**

补充（§5 冻结边界 8 条，逐条列出）：

1. 本轮非合规结论 / 非第三方审计 / 非法律意见 / 非认证；
2. 草案未经律师签发不得对外发布；
3. 草案 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计；
4. 未产出可分发包 / 未签名 / 证书自备；
5. NO-GO 四道外部闸门：**A 第三方审计 / B 律师签发 / C 签名可分发包 / D 商店申报**，在真要上架或收费那天一个不能跳；
6. GDPR 引文来自 gdpr-info.eu（A⁻ 来源），须回官方核对；
7. 所有条文 / 价格 / 版本 / 商店规则为 2026-11-15 抓取时点值，签发前须复核；§2.3 加密出口事实不构成法律意见或合规结论，书面分类仍须律师 / 合规签发；
8. `NOT RETRIEVED` 不得当作已核验事实引用。


---

## 1. 数据来源与生成方式

- 生成脚本：`tools/gen_third_party_notices.py`（只读、可重复运行；唯一写出目标为本文件）。
- 输入：`core/Cargo.lock`、`desktop/src-tauri/Cargo.lock`（`tomllib` 解析 `[[package]]`）。
- 元数据：本机 cargo registry 已解包源码 `C:\Users\zjc_9\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\<name>-<version>\Cargo.toml` 的 `package.license` / `package.license-file` 字段；若未解包，则在内存中读取缓存 `.crate` 内的 `Cargo.toml`（**绝不解压落盘**）；无 `source` 的第一方包走 workspace manifest（支持 `license.workspace = true`）。
- 解析口径与 `tools/audit_licenses.py`（只读审计脚本，本轮未改动）完全一致。
- 版权声明 / 许可全文获取方式：逐包扫描顶层 `LICENSE*` / `LICENCE*` / `COPYING*` / `NOTICE*` / `README*` 中的 `Copyright` 行，并读 `Cargo.toml` 的 `package.authors`；未检出显式 Copyright 行时给出官方全文 URL 作为「可获取方式」。
- **脚本只读、不改动上游**：不写 Cargo.lock、不写 Cargo.toml、不写 registry 缓存、不解压任何归档到磁盘。

> 每个依赖行第 4 列的 `E<n>` 指向 §3「许可全文获取方式索引」，该索引给出官方全文 URL、SPDX 页面、包内 LICENSE 检出情况与版权声明示例 / 模板。因此每一行均满足「包名 + 版本 + SPDX / 许可表达式 + 版权声明/许可全文或可获取方式」。


---

## 2. 核验口径与总览（2026-11-15 时点值）

| workspace | lock 内包数 | 可核验 | NOT VERIFIABLE |
| --- | ---: | ---: | ---: |
| core | 147 | 147 | 0 |
| desktop/src-tauri | 466 | 466 | 0 |
| **合计** | **613** | **613** | **0** |

- 不同许可表达式（含 NOT VERIFIABLE / license-file 形式）：**28** 组。
- **NOT VERIFIABLE：0** 个（fetch 前为 core 13 + desktop 166 = **179**；`cargo fetch --locked` 补齐元数据后降为 0）。凡无法核验者一律标 `NOT VERIFIABLE`，绝不猜测许可。
- 带上游 `NOTICE*` 文件的 crate 数：**0**（详见 §6）。
- 纯 `MPL-2.0` 依赖行：**7**；MPL 仅作为非 SPDX '/' 表达式选项的依赖行：**2**（详见 §5.3 / §8）。

**准确措辞（§0 原样）**：无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft；绝不能说「没有任何 copyleft」。

**fetch 后新增事实（如实提示，不改写冻结边界）**：`cargo fetch --locked` 使原先 179 个不可核验包全部可核验，其中 `r-efi 5.3.0` / `r-efi 6.0.0` 的 `license = "MIT OR Apache-2.0 OR LGPL-2.1-or-later"` 含 `LGPL-2.1-or-later` 可选原子，使审计脚本的 GPL-expression hits 由 fetch 前 core 0 / desktop 0 变为 fetch 后 core 1 / desktop 2。报告「可核验范围内 GPL 家族 0」是 fetch 前口径；OR 表达式下另有 MIT / Apache-2.0 可择一，是否触发 LGPL 义务须逐依赖确认（§8.3）。本文件不据此改写 §0 冻结边界，也不宣称存在 GPL 家族强 copyleft 义务。
**r-efi 目标门控（与 §8.3 一致）**：`r-efi` 是 `getrandom` 在 `[target.'cfg(all(target_os = "uefi", getrandom_backend = "efi_rng"))'.dependencies.r-efi]` 下的 target-specific 传递依赖；对 `x86_64-pc-windows-gnu` 目标不参与编译，仅作为 Cargo.lock 的 target-specific 条目存在，不作「已编译进产物」的断言。

---

## 3. 许可全文获取方式索引（按许可表达式分组）

每组给出：表达式原样、出现次数、原子许可的官方全文 URL 与 SPDX 页面、本机包内 LICENSE 检出情况、版权声明示例 / 模板或 `package.authors`。

### E1 · `MIT OR Apache-2.0`

- 出现次数：**305** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**295/305** 个依赖行；`NOTICE*` 检出：**0/305** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2019-2026 The RustCrypto Project Developers`；`Copyright (c) 2019 MobileCoin, LLC`；`Copyright (c) 2018-2024 The RustCrypto Project Developers`。
- `package.authors` 示例（最多 3 条）：`RustCrypto Developers`；`Nicolas Silva <nical@fastmail.com>`；`Marshall Pierce <marshall@mpierce.org>`。

### E2 · `MIT`

- 出现次数：**131** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**120/131** 个依赖行；`NOTICE*` 检出：**0/131** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2018 Carl Lerche`；`Copyright (c) 2015-2020 Julien Cretin`；`Copyright (c) 2017-2020 Google Inc.`。
- `package.authors` 示例（最多 3 条）：`Carl Lerche <me@carllerche.com>`；`Sean McArthur <sean@seanmonstar.com>`；`Bartłomiej Kamiński <fizyk20@gmail.com>`。

### E3 · `Apache-2.0 OR MIT`

- 出现次数：**56** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**55/56** 个依赖行；`NOTICE*` 检出：**0/56** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2016 Alex Crichton`；`Copyright (c) 2017 The Tokio Authors`；`Copyright (c) 2018 Josh Stone`。
- `package.authors` 示例（最多 3 条）：`Stjepan Glavina <stjepang@gmail.com>`；`Contributors to futures-rs`；`Josh Stone <cuviper@gmail.com>`。

### E4 · `Unicode-3.0`

- 出现次数：**36** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Unicode-3.0`（Unicode License v3 (Unicode-3.0)）：官方全文 <https://www.unicode.org/license.txt>；SPDX 页面 <https://spdx.org/licenses/Unicode-3.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**36/36** 个依赖行；`NOTICE*` 检出：**0/36** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright © 2020-2024 Unicode, Inc.`。
- `package.authors` 示例（最多 3 条）：`The ICU4X Project Developers`；`Manish Goregaokar <manishsmail@gmail.com>`。

### E5 · `Zlib OR Apache-2.0 OR MIT`

- 出现次数：**17** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Zlib`（zlib License）：官方全文 <https://opensource.org/license/zlib>；SPDX 页面 <https://spdx.org/licenses/Zlib.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/17** 个依赖行；`NOTICE*` 检出：**0/17** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2019 Daniel "Lokathor" Gee.`。
- `package.authors` 示例（最多 3 条）：`Lokathor <zefria@gmail.com>`；`Mads Marquart <mads@marquart.dk>`；`Mary <mary@mary.zone>`。

### E6 · `MIT/Apache-2.0`

- 出现次数：**13** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
- **注意：该表达式为 '/' 分隔非 SPDX 表达式**，须逐依赖明确择一（§8），本索引仅列出各原子全文，不代表已作出选择。
- 本机包内 `LICENSE*` / `COPYING*` 检出：**11/13** 个依赖行；`NOTICE*` 检出：**0/13** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2017-2018 Sergio Benitez`；`Copyright (c) 2014 The Rust Project Developers`；`Copyright (c) 2016 The roaring-rs developers.`。
- `package.authors` 示例（最多 3 条）：`Sergio Benitez <sb@sergio.bz>`；`The Rust Project Developers`；`Steven Fackler <sfackler@gmail.com>`。

### E7 · `Unlicense OR MIT`

- 出现次数：**10** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Unlicense`（The Unlicense）：官方全文 <https://unlicense.org/>；SPDX 页面 <https://spdx.org/licenses/Unlicense.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**10/10** 个依赖行；`NOTICE*` 检出：**0/10** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2015 Andrew Gallant`；`Copyright (c) 2017 Andrew Gallant`。
- `package.authors` 示例（最多 3 条）：`Andrew Gallant <jamslam@gmail.com>`；`bluss`。

### E8 · `MPL-2.0`

- 出现次数：**7** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MPL-2.0`（Mozilla Public License 2.0）：官方全文 <https://www.mozilla.org/media/MPL/2.0/index.txt>；SPDX 页面 <https://spdx.org/licenses/MPL-2.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**5/7** 个依赖行；`NOTICE*` 检出：**0/7** 个（§4(d) 触发边界，见 §6）。
- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。
- `package.authors` 示例（最多 3 条）：`Simon Sapin <simon.sapin@exyr.org>`；`Xidorn Quan <me@upsuper.org>`；`Simon Ochsenreither <simon@ochsenreither.de>`。

### E9 · `Apache-2.0`

- 出现次数：**4** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/4** 个依赖行；`NOTICE*` 检出：**0/4** 个（§4(d) 触发边界，见 §6）。
- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。
- `package.authors` 示例（最多 3 条）：`Miquel van Smoorenburg <mike@langeraar.net>`；`messense <messense@icloud.com>`；`Actyx AG <developer@actyx.io>`。

### E10 · `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`

- 出现次数：**4** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `LLVM-exception`（LLVM Exception (附于 Apache-2.0)）：官方全文 <https://llvm.org/LICENSE.txt>；SPDX 页面 <https://spdx.org/licenses/LLVM-exception.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**4/4** 个依赖行；`NOTICE*` 检出：**0/4** 个（§4(d) 触发边界，见 §6）。
- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。
- `package.authors` 示例（最多 3 条）：`The Cranelift Project Developers`；`Alex Crichton <alex@alexcrichton.com>`。

### E11 · `Apache-2.0/MIT`

- 出现次数：**3** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- **注意：该表达式为 '/' 分隔非 SPDX 表达式**，须逐依赖明确择一（§8），本索引仅列出各原子全文，不代表已作出选择。
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/3** 个依赖行；`NOTICE*` 检出：**0/3** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright 2014-2018 David Henningsson <diwic@ubuntu.com> and other contributors`；`Copyright (c) 2014-2018 David Henningsson <diwic@ubuntu.com> and other contributors`。
- `package.authors` 示例（最多 3 条）：`Eric Kidd <git@randomhacks.net>`；`David Henningsson <diwic@ubuntu.com>`。

### E12 · `MIT OR Apache-2.0 OR LGPL-2.1-or-later`

- 出现次数：**3** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `LGPL-2.1-`：本索引未收录参考 URL；许可全文按上游包内 LICENSE 文件 / crates.io 源码包取得（获取方式），不臆造。
  - `-later`：本索引未收录参考 URL；许可全文按上游包内 LICENSE 文件 / crates.io 源码包取得（获取方式），不臆造。
- 本机包内 `LICENSE*` / `COPYING*` 检出：**0/3** 个依赖行；`NOTICE*` 检出：**0/3** 个（§4(d) 触发边界，见 §6）。
- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。

### E13 · `(MIT OR Apache-2.0) AND Unicode-3.0`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `Unicode-3.0`（Unicode License v3 (Unicode-3.0)）：官方全文 <https://www.unicode.org/license.txt>；SPDX 页面 <https://spdx.org/licenses/Unicode-3.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright © 1991-2023 Unicode, Inc.`。
- `package.authors` 示例（最多 3 条）：`David Tolnay <dtolnay@gmail.com>`。

### E14 · `Apache-2.0 / MIT / MPL-2.0`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `MPL-2.0`（Mozilla Public License 2.0）：官方全文 <https://www.mozilla.org/media/MPL/2.0/index.txt>；SPDX 页面 <https://spdx.org/licenses/MPL-2.0.html>
- **注意：该表达式为 '/' 分隔非 SPDX 表达式**，须逐依赖明确择一（§8），本索引仅列出各原子全文，不代表已作出选择。
- 本机包内 `LICENSE*` / `COPYING*` 检出：**0/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。
- `package.authors` 示例（最多 3 条）：`Viktor Dahl <pazaconyoman@gmail.com>`。

### E15 · `BSD-3-Clause`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `BSD-3-Clause`（BSD 3-Clause）：官方全文 <https://opensource.org/license/bsd-3-clause>；SPDX 页面 <https://spdx.org/licenses/BSD-3-Clause.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2016 Dropbox, Inc.`。
- `package.authors` 示例（最多 3 条）：`Daniel Reiter Horn <danielrh@dropbox.com>`。

### E16 · `BSD-3-Clause OR MIT OR Apache-2.0`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `BSD-3-Clause`（BSD 3-Clause）：官方全文 <https://opensource.org/license/bsd-3-clause>；SPDX 页面 <https://spdx.org/licenses/BSD-3-Clause.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2018, Daniel Wagner-Hall`。
- `package.authors` 示例（最多 3 条）：`Daniel Wagner-Hall <dawagner@gmail.com>`；`Daniel Henry-Mantilla <daniel.henry.mantilla@gmail.com>`；`Vincent Esche <regexident@gmail.com>`。

### E17 · `MIT OR Apache-2.0 OR Zlib`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `Zlib`（zlib License）：官方全文 <https://opensource.org/license/zlib>；SPDX 页面 <https://spdx.org/licenses/Zlib.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2019 Osspial`；`Copyright (c) 2020 Osspial`；`Copyright (c) 2021 Chris Denton.`。
- `package.authors` 示例（最多 3 条）：`Osspial <osspial@gmail.com>`。

### E18 · `MIT OR Zlib OR Apache-2.0`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Zlib`（zlib License）：官方全文 <https://opensource.org/license/zlib>；SPDX 页面 <https://spdx.org/licenses/Zlib.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright 2013-2014 RAD Game Tools and Valve Software`；`Copyright 2010-2014 Rich Geldreich and Tenacious Software LLC`；`Copyright (c) 2017 Frommi`。
- `package.authors` 示例（最多 3 条）：`Frommi <daniil.liferenko@gmail.com>`；`oyvindln <oyvindln@users.noreply.github.com>`；`Rich Geldreich richgel99@gmail.com`。

### E19 · `Unlicense/MIT`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Unlicense`（The Unlicense）：官方全文 <https://unlicense.org/>；SPDX 页面 <https://spdx.org/licenses/Unlicense.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- **注意：该表达式为 '/' 分隔非 SPDX 表达式**，须逐依赖明确择一（§8），本索引仅列出各原子全文，不代表已作出选择。
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2017 Andrew Gallant`；`Copyright (c) 2015 Andrew Gallant`。
- `package.authors` 示例（最多 3 条）：`Andrew Gallant <jamslam@gmail.com>`。

### E20 · `Zlib`

- 出现次数：**2** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Zlib`（zlib License）：官方全文 <https://opensource.org/license/zlib>；SPDX 页面 <https://spdx.org/licenses/Zlib.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**2/2** 个依赖行；`NOTICE*` 检出：**0/2** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2024 Orson Peters`。
- `package.authors` 示例（最多 3 条）：`Orson Peters <orsonpeters@gmail.com>`。

### E21 · `0BSD OR MIT OR Apache-2.0`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `0BSD`（BSD Zero Clause License）：官方全文 <https://opensource.org/license/0bsd>；SPDX 页面 <https://spdx.org/licenses/0BSD.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (C) Jonas Schievink <jonasschievink@gmail.com>`。
- `package.authors` 示例（最多 3 条）：`Jonas Schievink <jonasschievink@gmail.com>`；`oyvindln <oyvindln@users.noreply.github.com>`。

### E22 · `Apache-2.0 / MIT`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- **注意：该表达式为 '/' 分隔非 SPDX 表达式**，须逐依赖明确择一（§8），本索引仅列出各原子全文，不代表已作出选择。
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2017 Contributors`。
- `package.authors` 示例（最多 3 条）：`Alex Crichton <alex@alexcrichton.com>`。

### E23 · `Apache-2.0 AND MIT`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2018 Jorge Aparicio`；`Copyright © 2005-2020 Rich Felker, et al.`；`Copyright © 1993,2004 Sun Microsystems or`。

### E24 · `Apache-2.0 WITH LLVM-exception`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
  - `LLVM-exception`（LLVM Exception (附于 Apache-2.0)）：官方全文 <https://llvm.org/LICENSE.txt>；SPDX 页面 <https://spdx.org/licenses/LLVM-exception.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。
- `package.authors` 示例（最多 3 条）：`Dan Gohman <sunfish@mozilla.com>`。

### E25 · `BSD-3-Clause AND MIT`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `BSD-3-Clause`（BSD 3-Clause）：官方全文 <https://opensource.org/license/bsd-3-clause>；SPDX 页面 <https://spdx.org/licenses/BSD-3-Clause.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2016 Dropbox, Inc.`；`Copyright (c) 2009, 2010, 2013-2016 by the Brotli Authors.`。
- `package.authors` 示例（最多 3 条）：`Daniel Reiter Horn <danielrh@dropbox.com>`；`The Brotli Authors`。

### E26 · `BSD-3-Clause/MIT`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `BSD-3-Clause`（BSD 3-Clause）：官方全文 <https://opensource.org/license/bsd-3-clause>；SPDX 页面 <https://spdx.org/licenses/BSD-3-Clause.html>
  - `MIT`（MIT License）：官方全文 <https://opensource.org/license/mit>；SPDX 页面 <https://spdx.org/licenses/MIT.html>
- **注意：该表达式为 '/' 分隔非 SPDX 表达式**，须逐依赖明确择一（§8），本索引仅列出各原子全文，不代表已作出选择。
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright (c) 2016 Dropbox, Inc.`。
- `package.authors` 示例（最多 3 条）：`Daniel Reiter Horn <danielrh@dropbox.com>`；`The Brotli Authors`。

### E27 · `CC0-1.0 OR MIT-0 OR Apache-2.0`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `CC0-1.0`（Creative Commons CC0 1.0）：官方全文 <https://creativecommons.org/publicdomain/zero/1.0/legalcode>；SPDX 页面 <https://spdx.org/licenses/CC0-1.0.html>
  - `MIT-0`（MIT No Attribution）：官方全文 <https://opensource.org/license/mit-0>；SPDX 页面 <https://spdx.org/licenses/MIT-0.html>
  - `Apache-2.0`（Apache License 2.0）：官方全文 <https://www.apache.org/licenses/LICENSE-2.0.txt>；SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。
- `package.authors` 示例（最多 3 条）：`Kornel <kornel@geekhood.net>`。

### E28 · `ISC`

- 出现次数：**1** 个依赖行（core + desktop 合计）。
- 原子许可与全文获取方式：
  - `ISC`（ISC License）：官方全文 <https://opensource.org/license/isc>；SPDX 页面 <https://spdx.org/licenses/ISC.html>
- 本机包内 `LICENSE*` / `COPYING*` 检出：**1/1** 个依赖行；`NOTICE*` 检出：**0/1** 个（§4(d) 触发边界，见 §6）。
- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：`Copyright © 2015, Simonas Kazlauskas`。
- `package.authors` 示例（最多 3 条）：`Simonas Kazlauskas <libloading@kazlauskas.me>`。


---

## 4. 全部第三方依赖清单（逐依赖）

表头：`E<n>` 指向 §3 索引；「包内 LICENSE」为 `有`/`无`；「NOTICE」为 `有`/`无`（§4(d) 边界）；「版权/获取」给出包内检出的首条 Copyright 行或 `package.authors`，未检出则指向 §3 的官方全文 URL（即「可获取方式」）。第一方包一并列出但标注「第一方」，不计入第三方依赖。

### 4.1 core（147 个包）

| 包名 | 版本 | 许可表达式（Cargo.toml 原样） | 类别 | 版权声明 / 许可全文或可获取方式 |
| --- | --- | --- | --- | --- |
| `aead` | `0.6.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 The RustCrypto Project Developers |
| `aes` | `0.9.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2024 The RustCrypto Project Developers |
| `aes-kw` | `0.3.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2022 RustCrypto Developers |
| `android_system_properties` | `0.1.6` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2016 Nicolas Silva |
| `argon2` | `0.6.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2026 The RustCrypto Project Developers |
| `atomic-waker` | `1.1.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `autocfg` | `1.5.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 Josh Stone |
| `base64` | `0.22.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Alice Maz |
| `base64` | `0.23.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2025 Alice Maz, Marshall Pierce |
| `base64ct` | `1.8.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Steve "Sc00bz" Thomas (steve at tobtu dot com) |
| `blake2` | `0.11.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017-2026 The RustCrypto Project Developers |
| `block-buffer` | `0.10.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2019 The RustCrypto Project Developers |
| `block-buffer` | `0.12.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2025 The RustCrypto Project Developers |
| `bumpalo` | `3.20.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Nick Fitzgerald |
| `bytes` | `1.12.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 Carl Lerche |
| `cc` | `1.11.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `cfg-if` | `1.0.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `chacha20` | `0.10.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 The RustCrypto Project Developers |
| `chacha20poly1305` | `0.11.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 The RustCrypto Project Developers |
| `chrono` | `0.4.45` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Apache 2.0 License [2]. Copyright (c) 2014--2026, Kang Seonghoon and |
| `cipher` | `0.5.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016-2025 RustCrypto Developers |
| `cmov` | `0.5.4` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2026 The RustCrypto Project Developers |
| `const-oid` | `0.10.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2026 The RustCrypto Project Developers |
| `core-foundation-sys` | `0.8.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `cpubits` | `0.1.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023-2026 The RustCrypto Project Developers |
| `cpufeatures` | `0.2.17` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2025 The RustCrypto Project Developers |
| `cpufeatures` | `0.3.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2026 The RustCrypto Project Developers |
| `crypto-common` | `0.1.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 RustCrypto Developers |
| `crypto-common` | `0.2.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2026 RustCrypto Developers |
| `ctutils` | `0.4.8` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2025-2026 The RustCrypto Project Developers |
| `data-encoding` | `2.11.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2020 Julien Cretin |
| `dav-server` | `0.11.0` | `Apache-2.0` | 第三方 | E9 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Miquel van Smoorenburg <mike@langeraar.net>; messense <messense@icloud.com> |
| `derive-where` | `1.7.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 Roland Fredenhagen |
| `digest` | `0.10.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Artyom Pavlov |
| `digest` | `0.11.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017-2025 RustCrypto Developers |
| `displaydoc` | `0.2.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Jane Lusby <jlusby@yaah.dev> |
| `dyn-clone` | `1.0.20` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `find-msvc-tools` | `0.1.15` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `form_urlencoded` | `1.2.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2016 The rust-url developers |
| `futures-channel` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-core` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-macro` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-task` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-util` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `generic-array` | `0.14.7` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Bartłomiej Kamiński |
| `getrandom` | `0.4.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2026 The rust-random Project Developers |
| `headers` | `0.4.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2025 Sean McArthur |
| `headers-core` | `0.3.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2023 Sean McArthur |
| `hex` | `0.4.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2014 The Rust Project Developers. |
| `hkdf` | `0.13.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2018 Vlad Filippov |
| `hmac` | `0.13.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Artyom Pavlov |
| `htmlescape` | `0.3.1` | `Apache-2.0 / MIT / MPL-2.0` | 第三方 | E14 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Viktor Dahl <pazaconyoman@gmail.com> |
| `http` | `1.5.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2017 http-rs authors |
| `http-body` | `1.1.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 Sean McArthur & Hyper Contributors |
| `http-body-util` | `0.1.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 Sean McArthur & Hyper Contributors |
| `httparse` | `1.10.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2025 Sean McArthur |
| `httpdate` | `1.0.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Pyfisch |
| `hybrid-array` | `0.4.15` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2026 The RustCrypto Project Developers |
| `hyper` | `1.12.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2026 Sean McArthur |
| `hyper-util` | `0.1.21` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023-2025 Sean McArthur |
| `iana-time-zone` | `0.1.65` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2020 Andrew Straw |
| `iana-time-zone-haiku` | `0.1.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2020 Andrew Straw |
| `icu_collections` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_locale_core` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_normalizer` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_normalizer_data` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_properties` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_properties_data` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_provider` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `idna` | `1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2025 The rust-url developers |
| `idna_adapter` | `1.2.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) The rust-url developers |
| `inout` | `0.2.6` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2025 The RustCrypto Project Developers |
| `itoa` | `1.0.19` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `js-sys` | `0.3.107` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `libc` | `0.2.191` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) The Rust Project Developers |
| `litemap` | `0.8.3` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `log` | `0.4.35` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `memchr` | `2.8.3` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `mime` | `0.3.18` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Sean McArthur |
| `mime_guess` | `2.0.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Austin Bonander |
| `mio` | `1.2.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Carl Lerche and other MIO contributors |
| `mylock-cli` | `0.1.0` | `MIT OR Apache-2.0` | 第一方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `mylock-core` | `0.1.0` | `MIT OR Apache-2.0` | 第一方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `mylock-mount` | `0.1.0` | `MIT OR Apache-2.0` | 第一方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `num-traits` | `0.2.19` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `password-hash` | `0.6.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2026 RustCrypto Developers |
| `percent-encoding` | `2.3.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2025 The rust-url developers |
| `phc` | `0.6.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2026 The RustCrypto Project Developers |
| `pin-project-lite` | `0.2.17` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `poly1305` | `0.9.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2026 The RustCrypto Project Developers |
| `potential_utf` | `0.1.6` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `proc-macro2` | `1.0.107` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com>; Alex Crichton <alex@alexcrichton.com> |
| `quote` | `1.0.47` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `r-efi` | `6.0.0` | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | 第三方 | E12 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `rand_core` | `0.10.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2026 The Rand Project Developers |
| `rustversion` | `1.0.23` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `serde` | `1.0.229` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Erick Tryzelaar <erick.tryzelaar@gmail.com>; David Tolnay <dtolnay@gmail.com> |
| `serde_core` | `1.0.229` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Erick Tryzelaar <erick.tryzelaar@gmail.com>; David Tolnay <dtolnay@gmail.com> |
| `serde_derive` | `1.0.229` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Erick Tryzelaar <erick.tryzelaar@gmail.com>; David Tolnay <dtolnay@gmail.com> |
| `serde_json` | `1.0.154` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `sha1` | `0.10.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2006-2009 Graydon Hoare |
| `sha2` | `0.11.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016-2026 The RustCrypto Project Developers |
| `shlex` | `2.0.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2015 Nicholas Allegra (comex). |
| `slab` | `0.4.12` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Carl Lerche |
| `smallvec` | `1.16.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 The Servo Project Developers |
| `socket2` | `0.6.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `stable_deref_trait` | `1.2.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Robert Grosse |
| `syn` | `2.0.119` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `syn` | `3.0.10` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `synstructure` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2016 Nika Layzell |
| `tinystr` | `0.8.5` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `tokio` | `1.54.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Tokio Contributors |
| `tokio-macros` | `2.7.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Yoshua Wuyts |
| `typenum` | `1.20.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2014 Paho Lurie-Gregg |
| `unicase` | `2.10.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2026 Sean McArthur |
| `unicode-ident` | `1.0.27` | `(MIT OR Apache-2.0) AND Unicode-3.0` | 第三方 | E13 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 1991-2023 Unicode, Inc. |
| `universal-hash` | `0.6.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2025 RustCrypto Developers |
| `url` | `2.5.8` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2025 The rust-url developers |
| `utf8_iter` | `1.1.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Henri Sivonen <hsivonen@hsivonen.fi> |
| `utf_types` | `1.0.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Henri Sivonen <hsivonen@hsivonen.fi> |
| `uuid` | `1.29.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `version_check` | `0.9.5` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017-2018 Sergio Benitez |
| `wasi` | `0.11.1+wasi-snapshot-preview1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | 第三方 | E10 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The Cranelift Project Developers |
| `wasm-bindgen` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-bindgen-macro` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-bindgen-macro-support` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-bindgen-shared` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `windows-core` | `0.62.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-implement` | `0.60.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-interface` | `0.59.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-link` | `0.2.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-result` | `0.4.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-strings` | `0.5.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-sys` | `0.61.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `writeable` | `0.6.4` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `xml` | `1.4.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Vladimir Matveev |
| `xml-rs` | `1.0.0` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Vladimir Matveev <vmatveev@citrine.cc>; Kornel (https://github.com/kornelski) |
| `xmltree` | `0.12.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Chin |
| `yoke` | `0.8.4` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `yoke-derive` | `0.8.4` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerofrom` | `0.1.8` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerofrom-derive` | `0.1.8` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zeroize` | `1.9.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2026 The RustCrypto Project Developers |
| `zerotrie` | `0.2.5` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerovec` | `0.11.9` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerovec-derive` | `0.11.7` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zmij` | `1.0.23` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |

### 4.2 desktop/src-tauri（466 个包）

| 包名 | 版本 | 许可表达式（Cargo.toml 原样） | 类别 | 版权声明 / 许可全文或可获取方式 |
| --- | --- | --- | --- | --- |
| `adler2` | `2.0.1` | `0BSD OR MIT OR Apache-2.0` | 第三方 | E21 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (C) Jonas Schievink <jonasschievink@gmail.com> |
| `aead` | `0.6.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 The RustCrypto Project Developers |
| `aes` | `0.9.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2024 The RustCrypto Project Developers |
| `aes-kw` | `0.3.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2022 RustCrypto Developers |
| `aho-corasick` | `1.1.5` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `alloc-no-stdlib` | `3.0.0` | `BSD-3-Clause` | 第三方 | E15 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Dropbox, Inc. |
| `alloc-stdlib` | `0.3.0` | `BSD-3-Clause` | 第三方 | E15 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Daniel Reiter Horn <danielrh@dropbox.com> |
| `android_system_properties` | `0.1.6` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2016 Nicolas Silva |
| `anyhow` | `1.0.104` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `argon2` | `0.6.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2026 The RustCrypto Project Developers |
| `atk` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `atk-sys` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `atomic-waker` | `1.1.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `autocfg` | `1.5.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 Josh Stone |
| `base64` | `0.21.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Alice Maz |
| `base64` | `0.22.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Alice Maz |
| `base64` | `0.23.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2025 Alice Maz, Marshall Pierce |
| `base64ct` | `1.8.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Steve "Sc00bz" Thomas (steve at tobtu dot com) |
| `bit-set` | `0.11.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 The Rust Project Developers |
| `bit-set` | `0.8.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 The Rust Project Developers |
| `bit-vec` | `0.10.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 The Rust Project Developers |
| `bit-vec` | `0.8.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 The Rust Project Developers |
| `bitflags` | `1.3.2` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `bitflags` | `2.13.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `blake2` | `0.11.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017-2026 The RustCrypto Project Developers |
| `block-buffer` | `0.10.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2019 The RustCrypto Project Developers |
| `block-buffer` | `0.12.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2025 The RustCrypto Project Developers |
| `block2` | `0.6.2` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Mads Marquart <mads@marquart.dk> |
| `brotli` | `9.1.1` | `BSD-3-Clause AND MIT` | 第三方 | E25 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Dropbox, Inc. |
| `brotli-decompressor` | `6.3.1` | `BSD-3-Clause/MIT` | 第三方 | E26 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Dropbox, Inc. |
| `bs58` | `0.5.1` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 The roaring-rs developers. |
| `bumpalo` | `3.20.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Nick Fitzgerald |
| `bytemuck` | `1.25.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Daniel "Lokathor" Gee. |
| `byteorder` | `1.5.0` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `bytes` | `1.12.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 Carl Lerche |
| `cairo-rs` | `0.18.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `cairo-sys-rs` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `camino` | `1.2.6` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Without Boats <saoirse@without.boats>; Ashley Williams <ashley666ashley@gmail.com… |
| `cargo-platform` | `0.1.9` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `cargo_metadata` | `0.19.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Oliver Schneider <git-spam-no-reply9815368754983@oli-obk.de> |
| `cargo_toml` | `1.0.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Kornel <kornel@geekhood.net> |
| `cc` | `1.11.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `cesu8` | `1.1.0` | `Apache-2.0/MIT` | 第三方 | E11 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Eric Kidd <git@randomhacks.net> |
| `cfb` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Matthew D. Steele |
| `cfg-expr` | `0.15.8` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Embark Studios |
| `cfg-if` | `1.0.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `chacha20` | `0.10.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 The RustCrypto Project Developers |
| `chacha20poly1305` | `0.11.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 The RustCrypto Project Developers |
| `chrono` | `0.4.45` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Apache 2.0 License [2]. Copyright (c) 2014--2026, Kang Seonghoon and |
| `cipher` | `0.5.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016-2025 RustCrypto Developers |
| `cmov` | `0.5.4` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2026 The RustCrypto Project Developers |
| `combine` | `4.6.9` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Markus Westerlind |
| `const-oid` | `0.10.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2026 The RustCrypto Project Developers |
| `cookie` | `0.18.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2017 Sergio Benitez |
| `core-foundation` | `0.10.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `core-foundation-sys` | `0.8.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `core-graphics` | `0.25.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `core-graphics-types` | `0.2.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `cpubits` | `0.1.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023-2026 The RustCrypto Project Developers |
| `cpufeatures` | `0.2.17` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2025 The RustCrypto Project Developers |
| `cpufeatures` | `0.3.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2026 The RustCrypto Project Developers |
| `crc32fast` | `1.5.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 Sam Rijs, Alex Crichton and contributors |
| `crossbeam-channel` | `0.5.17` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 The Crossbeam Project Developers |
| `crossbeam-utils` | `0.8.23` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 The Crossbeam Project Developers |
| `crypto-common` | `0.1.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 RustCrypto Developers |
| `crypto-common` | `0.2.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2026 RustCrypto Developers |
| `cssparser` | `0.37.0` | `MPL-2.0` | 第三方 | E8 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Simon Sapin <simon.sapin@exyr.org> |
| `cssparser` | `0.38.0` | `MPL-2.0` | 第三方 | E8 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Simon Sapin <simon.sapin@exyr.org> |
| `cssparser-macros` | `0.7.1` | `MPL-2.0` | 第三方 | E8 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Simon Sapin <simon.sapin@exyr.org> |
| `ctor` | `1.0.14` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Matt Mastracci <matthew@mastracci.com> |
| `ctutils` | `0.4.8` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2025-2026 The RustCrypto Project Developers |
| `darling` | `0.24.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Ted Driggs |
| `darling_core` | `0.24.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Ted Driggs |
| `darling_macro` | `0.24.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Ted Driggs |
| `data-encoding` | `2.11.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2020 Julien Cretin |
| `dav-server` | `0.11.0` | `Apache-2.0` | 第三方 | E9 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Miquel van Smoorenburg <mike@langeraar.net>; messense <messense@icloud.com> |
| `dbus` | `0.9.12` | `Apache-2.0/MIT` | 第三方 | E11 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2014-2018 David Henningsson <diwic@ubuntu.com> and other contributors |
| `deranged` | `0.5.9` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2024 Jacob Pratt et al. |
| `derive-where` | `1.7.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 Roland Fredenhagen |
| `derive_more` | `2.1.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Jelte Fennema |
| `derive_more-impl` | `2.1.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Jelte Fennema |
| `digest` | `0.10.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Artyom Pavlov |
| `digest` | `0.11.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017-2025 RustCrypto Developers |
| `dirs` | `7.0.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2019 dirs-rs contributors |
| `dirs-sys` | `0.5.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2019 dirs-rs contributors |
| `dispatch2` | `0.3.1` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Mads Marquart <mads@marquart.dk>; Mary <mary@mary.zone> |
| `displaydoc` | `0.2.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Jane Lusby <jlusby@yaah.dev> |
| `dlopen2` | `0.8.2` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Szymon Wieloch <szymon.wieloch@gmail.com>; Ahmed Masud <ahmed.masud@saf.ai> |
| `dlopen2_derive` | `0.4.3` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Szymon Wieloch <szymon.wieloch@gmail.com>; OpenByte <development.openbyte@gmail.c… |
| `dom_query` | `0.28.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 Mykola Humanov |
| `dom_query` | `0.29.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 Mykola Humanov |
| `dpi` | `0.1.2` | `Apache-2.0 AND MIT` | 第三方 | E23 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 Jorge Aparicio |
| `dtoa` | `1.0.11` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `dtoa-short` | `0.3.5` | `MPL-2.0` | 第三方 | E8 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Xidorn Quan <me@upsuper.org> |
| `dunce` | `1.0.5` | `CC0-1.0 OR MIT-0 OR Apache-2.0` | 第三方 | E27 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Kornel <kornel@geekhood.net> |
| `dyn-clone` | `1.0.20` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `embed-resource` | `3.0.12` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 nabijaczleweli |
| `embed_plist` | `1.2.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020 Nikolai Vazquez |
| `equivalent` | `1.0.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016--2023 |
| `erased-serde` | `0.4.11` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `fastrand` | `2.5.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Stjepan Glavina <stjepang@gmail.com> |
| `fdeflate` | `0.3.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The image-rs Developers |
| `field-offset` | `0.3.6` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016-2021 Diggory Blake, and other contributors. |
| `find-msvc-tools` | `0.1.15` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `flate2` | `1.1.11` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2026 Alex Crichton |
| `fnv` | `1.0.7` | `Apache-2.0 / MIT` | 第三方 | E22 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Contributors |
| `foldhash` | `0.2.0` | `Zlib` | 第三方 | E20 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2024 Orson Peters |
| `foreign-types` | `0.5.0` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 The foreign-types Developers |
| `foreign-types-macros` | `0.2.4` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 The foreign-types Developers |
| `foreign-types-shared` | `0.3.1` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 The foreign-types Developers |
| `form_urlencoded` | `1.2.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2016 The rust-url developers |
| `futures-channel` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-core` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-executor` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-io` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-macro` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-sink` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-task` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `futures-util` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Alex Crichton |
| `gdk` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gdk-pixbuf` | `0.18.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gdk-pixbuf-sys` | `0.18.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gdk-sys` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gdkwayland-sys` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gdkx11` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gdkx11-sys` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `generic-array` | `0.14.7` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Bartłomiej Kamiński |
| `getrandom` | `0.3.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2025 The rust-random Project Developers |
| `getrandom` | `0.4.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2026 The rust-random Project Developers |
| `gio` | `0.18.4` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gio-sys` | `0.18.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `glib` | `0.18.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `glib-macros` | `0.18.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `glib-sys` | `0.18.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `glob` | `0.3.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `gobject-sys` | `0.18.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gtk` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gtk-sys` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `gtk3-macros` | `0.18.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `hashbrown` | `0.12.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Amanieu d'Antras |
| `hashbrown` | `0.17.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Amanieu d'Antras |
| `headers` | `0.4.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2025 Sean McArthur |
| `headers-core` | `0.3.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2023 Sean McArthur |
| `heck` | `0.4.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 The Rust Project Developers |
| `heck` | `0.5.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 The Rust Project Developers |
| `hex` | `0.4.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2014 The Rust Project Developers. |
| `hkdf` | `0.13.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2018 Vlad Filippov |
| `hmac` | `0.13.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Artyom Pavlov |
| `html5ever` | `0.39.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The html5ever Project Developers |
| `html5ever` | `0.40.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The html5ever Project Developers |
| `htmlescape` | `0.3.1` | `Apache-2.0 / MIT / MPL-2.0` | 第三方 | E14 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Viktor Dahl <pazaconyoman@gmail.com> |
| `http` | `1.5.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2017 http-rs authors |
| `http-body` | `1.1.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 Sean McArthur & Hyper Contributors |
| `http-body-util` | `0.1.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2026 Sean McArthur & Hyper Contributors |
| `httparse` | `1.10.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2025 Sean McArthur |
| `httpdate` | `1.0.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Pyfisch |
| `hybrid-array` | `0.4.15` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2026 The RustCrypto Project Developers |
| `hyper` | `1.12.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2026 Sean McArthur |
| `hyper-util` | `0.1.21` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023-2025 Sean McArthur |
| `iana-time-zone` | `0.1.65` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2020 Andrew Straw |
| `iana-time-zone-haiku` | `0.1.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2020 Andrew Straw |
| `ico` | `0.6.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 Matthew D. Steele |
| `icu_collections` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_locale_core` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_normalizer` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_normalizer_data` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_properties` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_properties_data` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `icu_provider` | `2.4.0` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `ident_case` | `1.0.1` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Ted Driggs <ted.driggs@outlook.com> |
| `idna` | `1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2025 The rust-url developers |
| `idna_adapter` | `1.2.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) The rust-url developers |
| `indexmap` | `1.9.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016--2017 |
| `indexmap` | `2.14.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016--2017 |
| `infer` | `0.22.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Bojan |
| `inout` | `0.2.6` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2025 The RustCrypto Project Developers |
| `ipnet` | `2.12.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2017 Juniper Networks, Inc. |
| `itoa` | `1.0.19` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `javascriptcore-rs` | `1.1.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2021, The Gtk-rs Project Developers. |
| `javascriptcore-rs-sys` | `1.1.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2017, The Gtk-rs Project Developers. |
| `jiff` | `0.2.43` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `jiff-core` | `0.1.5` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `jiff-static` | `0.2.43` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `jiff-tzdb` | `0.1.9` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `jiff-tzdb-platform` | `0.1.3` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `jni` | `0.21.1` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Prevoty, Inc. and jni-rs contributors |
| `jni-sys` | `0.3.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 The rust-jni-sys Developers |
| `jni-sys` | `0.4.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 The rust-jni-sys Developers |
| `jni-sys-macros` | `0.4.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Robert Bragg <robert@sixbynine.org> |
| `js-sys` | `0.3.107` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `json-patch` | `4.2.0` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Ivan Dubrov |
| `jsonptr` | `0.7.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2024 Chance Dinkins |
| `keyboard-types` | `0.8.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Pyfisch |
| `libappindicator` | `0.9.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017-2021 qDot |
| `libappindicator-sys` | `0.9.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `libc` | `0.2.191` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) The Rust Project Developers |
| `libdbus-sys` | `0.2.7` | `Apache-2.0/MIT` | 第三方 | E11 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2014-2018 David Henningsson <diwic@ubuntu.com> and other contributors |
| `libloading` | `0.7.4` | `ISC` | 第三方 | E28 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2015, Simonas Kazlauskas |
| `libredox` | `0.1.25` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 4lDO2 |
| `link-section` | `0.19.4` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Matt Mastracci <matthew@mastracci.com> |
| `litemap` | `0.8.3` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `lock_api` | `0.4.14` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 The Rust Project Developers |
| `log` | `0.4.35` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `markup5ever` | `0.39.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The html5ever Project Developers |
| `markup5ever` | `0.40.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The html5ever Project Developers |
| `memchr` | `2.8.3` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `memoffset` | `0.9.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Gilad Naaman |
| `mime` | `0.3.18` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Sean McArthur |
| `mime_guess` | `2.0.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Austin Bonander |
| `miniz_oxide` | `0.8.9` | `MIT OR Zlib OR Apache-2.0` | 第三方 | E18 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2013-2014 RAD Game Tools and Valve Software |
| `miniz_oxide` | `0.9.1` | `MIT OR Zlib OR Apache-2.0` | 第三方 | E18 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2013-2014 RAD Game Tools and Valve Software |
| `mio` | `1.2.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Carl Lerche and other MIO contributors |
| `muda` | `0.20.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2022 Tauri Programme within The Commons Conservancy |
| `mylock-core` | `0.1.0` | `MIT OR Apache-2.0` | 第一方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `mylock-desktop` | `0.1.0` | `MIT OR Apache-2.0` | 第一方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `mylock-mount` | `0.1.0` | `MIT OR Apache-2.0` | 第一方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `ndk` | `0.9.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: The Rust Mobile contributors |
| `ndk-context` | `0.1.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: The Rust Windowing contributors |
| `ndk-sys` | `0.6.0+11769913` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: The Rust Windowing contributors |
| `new_debug_unreachable` | `1.0.6` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Jonathan Reem |
| `num-conv` | `0.2.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Jacob Pratt |
| `num-traits` | `0.2.19` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `num_enum` | `0.7.6` | `BSD-3-Clause OR MIT OR Apache-2.0` | 第三方 | E16 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018, Daniel Wagner-Hall |
| `num_enum_derive` | `0.7.6` | `BSD-3-Clause OR MIT OR Apache-2.0` | 第三方 | E16 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018, Daniel Wagner-Hall |
| `objc2` | `0.6.5` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Mads Marquart <mads@marquart.dk> |
| `objc2-app-kit` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-cloud-kit` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-core-data` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-core-foundation` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-core-graphics` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-core-image` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-core-location` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-core-text` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-encode` | `4.1.0` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Mads Marquart <mads@marquart.dk> |
| `objc2-exception-helper` | `0.1.1` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Mads Marquart <mads@marquart.dk> |
| `objc2-foundation` | `0.3.2` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-io-surface` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-quartz-core` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-ui-kit` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-user-notifications` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `objc2-web-kit` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `once_cell` | `1.21.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Aleksey Kladov <aleksey.kladov@gmail.com> |
| `option-ext` | `0.2.0` | `MPL-2.0` | 第三方 | E8 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Simon Ochsenreither <simon@ochsenreither.de> |
| `pango` | `0.18.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `pango-sys` | `0.18.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The gtk-rs Project Developers |
| `parking_lot` | `0.12.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 The Rust Project Developers |
| `parking_lot_core` | `0.9.12` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 The Rust Project Developers |
| `password-hash` | `0.6.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2026 RustCrypto Developers |
| `percent-encoding` | `2.3.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2025 The rust-url developers |
| `phc` | `0.6.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021-2026 The RustCrypto Project Developers |
| `phf` | `0.13.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_codegen` | `0.13.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_codegen` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_generator` | `0.13.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_generator` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_macros` | `0.13.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_macros` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_shared` | `0.13.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `phf_shared` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi |
| `pin-project-lite` | `0.2.17` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `pkg-config` | `0.3.34` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `plist` | `1.10.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Edward Barnard |
| `png` | `0.18.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 nwin |
| `poly1305` | `0.9.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2026 The RustCrypto Project Developers |
| `portable-atomic` | `1.15.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `portable-atomic-util` | `0.2.8` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `potential_utf` | `0.1.6` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `powerfmt` | `0.2.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2023 Jacob Pratt et al. |
| `precomputed-hash` | `0.1.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Emilio Cobos Álvarez |
| `proc-macro-crate` | `1.3.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Bastian Köcher <git@kchr.de> |
| `proc-macro-crate` | `2.0.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Bastian Köcher <git@kchr.de> |
| `proc-macro-crate` | `3.5.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Bastian Köcher <git@kchr.de> |
| `proc-macro-error` | `1.0.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2019-2020 CreepySkeleton <creepy-skeleton@yandex.ru> |
| `proc-macro-error-attr` | `1.0.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2019-2020 CreepySkeleton <creepy-skeleton@yandex.ru> |
| `proc-macro2` | `1.0.107` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com>; Alex Crichton <alex@alexcrichton.com> |
| `quick-xml` | `0.42.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Johann Tuffe |
| `quote` | `1.0.47` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `r-efi` | `5.3.0` | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | 第三方 | E12 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `r-efi` | `6.0.0` | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | 第三方 | E12 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `rand_core` | `0.10.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2026 The Rand Project Developers |
| `raw-window-handle` | `0.6.2` | `MIT OR Apache-2.0 OR Zlib` | 第三方 | E17 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Osspial |
| `redox_syscall` | `0.5.18` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Redox OS Developers |
| `redox_users` | `0.5.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Jose Narvaez |
| `ref-cast` | `1.0.27` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `ref-cast-impl` | `1.0.27` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `regex` | `1.13.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `regex-automata` | `0.4.18` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `regex-syntax` | `0.8.11` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `reqwest` | `0.13.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2016 Sean McArthur |
| `rustc-hash` | `2.1.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The Rust Project Developers |
| `rustc_version` | `0.4.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 The Rust Project Developers |
| `rustversion` | `1.0.23` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `same-file` | `1.0.6` | `Unlicense/MIT` | 第三方 | E19 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Andrew Gallant |
| `schemars` | `0.8.22` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Graham Esau |
| `schemars` | `0.9.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Graham Esau |
| `schemars` | `1.2.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Graham Esau |
| `schemars_derive` | `0.8.22` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Graham Esau |
| `scopeguard` | `1.2.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016-2019 Ulrik Sverdrup "bluss" and scopeguard developers |
| `selectors` | `0.38.0` | `MPL-2.0` | 第三方 | E8 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: The Servo Project Developers |
| `selectors` | `0.41.0` | `MPL-2.0` | 第三方 | E8 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: The Servo Project Developers |
| `semver` | `1.0.28` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `serde` | `1.0.229` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Erick Tryzelaar <erick.tryzelaar@gmail.com>; David Tolnay <dtolnay@gmail.com> |
| `serde-untagged` | `0.1.9` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `serde_core` | `1.0.229` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Erick Tryzelaar <erick.tryzelaar@gmail.com>; David Tolnay <dtolnay@gmail.com> |
| `serde_derive` | `1.0.229` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Erick Tryzelaar <erick.tryzelaar@gmail.com>; David Tolnay <dtolnay@gmail.com> |
| `serde_derive_internals` | `0.29.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Erick Tryzelaar <erick.tryzelaar@gmail.com>; David Tolnay <dtolnay@gmail.com> |
| `serde_json` | `1.0.154` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `serde_repr` | `0.1.21` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `serde_spanned` | `0.6.9` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `serde_spanned` | `1.1.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `serde_with` | `3.24.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 |
| `serde_with_macros` | `3.24.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 |
| `serialize-to-javascript` | `0.1.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 Chip Reed |
| `serialize-to-javascript-impl` | `0.1.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 Chip Reed |
| `servo_arc` | `0.4.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The Servo Project Developers |
| `servo_arc` | `0.5.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The Servo Project Developers |
| `sha1` | `0.10.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2006-2009 Graydon Hoare |
| `sha2` | `0.10.9` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2006-2009 Graydon Hoare |
| `sha2` | `0.11.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016-2026 The RustCrypto Project Developers |
| `shlex` | `2.0.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2015 Nicholas Allegra (comex). |
| `simd-adler32` | `0.3.10` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) [2021] [Marvin Countryman] |
| `siphasher` | `1.0.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2012-2016 The Rust Project Developers. |
| `slab` | `0.4.12` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Carl Lerche |
| `smallvec` | `1.16.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018 The Servo Project Developers |
| `socket2` | `0.6.5` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `softbuffer` | `0.4.8` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2022 Kirill Chibisov |
| `soup3` | `0.5.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2017, The Gtk-rs Project Developers. |
| `soup3-sys` | `0.5.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2017, The Gtk-rs Project Developers. |
| `stable_deref_trait` | `1.2.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Robert Grosse |
| `static_vcruntime` | `3.0.0` | `MIT OR Apache-2.0 OR Zlib` | 第三方 | E17 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 Chris Denton. |
| `string_cache` | `0.11.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `string_cache` | `0.9.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `string_cache_codegen` | `0.11.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `string_cache_codegen` | `0.6.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2012-2013 Mozilla Foundation |
| `strsim` | `0.11.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Danny Guo |
| `swift-rs` | `1.0.9` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2023 The swift-rs developers |
| `syn` | `1.0.109` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `syn` | `2.0.119` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `syn` | `3.0.10` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `sync_wrapper` | `1.0.2` | `Apache-2.0` | 第三方 | E9 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Actyx AG <developer@actyx.io> |
| `synstructure` | `0.14.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2016 Nika Layzell |
| `system-deps` | `6.2.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Guillaume Desmottes <guillaume.desmottes@collabora.com>; Josh Triplett <josh@josh… |
| `tao` | `0.37.2` | `Apache-2.0` | 第三方 | E9 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Tauri Programme within The Commons Conservancy; The winit contributors |
| `tao-macros` | `0.1.4` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022 - Present Tauri Apps Contributors |
| `target-lexicon` | `0.12.16` | `Apache-2.0 WITH LLVM-exception` | 第三方 | E24 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Dan Gohman <sunfish@mozilla.com> |
| `tauri` | `2.12.5` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 - Present Tauri Apps Contributors |
| `tauri-build` | `2.7.4` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 - Present Tauri Apps Contributors |
| `tauri-codegen` | `2.7.4` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 - Present Tauri Apps Contributors |
| `tauri-macros` | `2.7.4` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 - Present Tauri Apps Contributors |
| `tauri-runtime` | `2.12.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 - Present Tauri Apps Contributors |
| `tauri-runtime-wry` | `2.12.3` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 - Present Tauri Apps Contributors |
| `tauri-utils` | `2.10.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 - Present Tauri Apps Contributors |
| `tauri-winres` | `0.3.7` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2023 - Present Tauri Apps Contributors |
| `tendril` | `0.5.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Keegan McAllister |
| `thiserror` | `1.0.69` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `thiserror` | `2.0.21` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `thiserror-impl` | `1.0.69` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `thiserror-impl` | `2.0.21` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `time` | `0.3.55` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Jacob Pratt et al. |
| `time-core` | `0.1.9` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Jacob Pratt et al. |
| `time-macros` | `0.2.32` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Jacob Pratt et al. |
| `tinystr` | `0.8.5` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `tinyvec` | `1.13.3` | `Zlib OR Apache-2.0 OR MIT` | 第三方 | E5 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Daniel "Lokathor" Gee. |
| `tokio` | `1.54.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Tokio Contributors |
| `tokio-macros` | `2.7.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Yoshua Wuyts |
| `tokio-util` | `0.7.20` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Tokio Contributors |
| `toml` | `0.8.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `toml` | `1.2.1+spec-1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `toml_datetime` | `0.6.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `toml_datetime` | `1.1.3+spec-1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `toml_edit` | `0.19.15` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `toml_edit` | `0.20.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `toml_edit` | `0.25.19+spec-1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `toml_parser` | `1.1.5+spec-1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `toml_writer` | `1.2.0+spec-1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Individual contributors |
| `tower` | `0.5.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Tower Contributors |
| `tower-http` | `0.6.11` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2021 Tower Contributors |
| `tower-layer` | `0.3.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Tower Contributors |
| `tower-service` | `0.3.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Tower Contributors |
| `tracing` | `0.1.44` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Tokio Contributors |
| `tracing-core` | `0.1.36` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 Tokio Contributors |
| `tray-icon` | `0.25.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2022-2022 Tauri Programme within The Commons Conservancy |
| `try-lock` | `0.2.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2023 Sean McArthur |
| `typeid` | `1.0.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |
| `typenum` | `1.20.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2014 Paho Lurie-Gregg |
| `unicase` | `2.10.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014-2026 Sean McArthur |
| `unicode-ident` | `1.0.27` | `(MIT OR Apache-2.0) AND Unicode-3.0` | 第三方 | E13 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 1991-2023 Unicode, Inc. |
| `unicode-segmentation` | `1.13.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 The Rust Project Developers |
| `universal-hash` | `0.6.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019-2025 RustCrypto Developers |
| `url` | `2.5.8` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2013-2025 The rust-url developers |
| `urlpattern` | `0.6.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2021 the Deno authors |
| `utf8_iter` | `1.1.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Henri Sivonen <hsivonen@hsivonen.fi> |
| `utf_types` | `1.0.0` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Henri Sivonen <hsivonen@hsivonen.fi> |
| `uuid` | `1.29.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The Rust Project Developers |
| `version-compare` | `0.2.1` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Tim Visée |
| `version_check` | `0.9.5` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017-2018 Sergio Benitez |
| `vswhom` | `0.1.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 nabijaczleweli |
| `vswhom-sys` | `0.1.3` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2019 nabijaczleweli |
| `walkdir` | `2.5.0` | `Unlicense/MIT` | 第三方 | E19 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Gallant |
| `want` | `0.3.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2019 Sean McArthur |
| `wasi` | `0.11.1+wasi-snapshot-preview1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | 第三方 | E10 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: The Cranelift Project Developers |
| `wasip2` | `1.0.4+wasi-0.2.12` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | 第三方 | E10 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `wasm-bindgen` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-bindgen-futures` | `0.4.80` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-bindgen-macro` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-bindgen-macro-support` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-bindgen-shared` | `0.2.130` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `wasm-streams` | `0.5.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Mattias Buelens <mattias@buelens.com> |
| `web-sys` | `0.3.107` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Alex Crichton |
| `web-time` | `1.1.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright 2023 dAxpeDDa |
| `web_atoms` | `0.2.6` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The html5ever Project Developers |
| `web_atoms` | `0.3.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 The html5ever Project Developers |
| `webkit2gtk` | `2.0.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com> |
| `webkit2gtk-sys` | `2.0.2` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com> |
| `webview2-com` | `0.39.1` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `webview2-com-macros` | `0.8.1` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `webview2-com-sys` | `0.39.1` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `winapi` | `0.3.9` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015-2018 The winapi-rs Developers |
| `winapi-i686-pc-windows-gnu` | `0.4.0` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Peter Atashian <retep998@gmail.com> |
| `winapi-util` | `0.1.11` | `Unlicense OR MIT` | 第三方 | E7 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2017 Andrew Gallant |
| `winapi-x86_64-pc-windows-gnu` | `0.4.0` | `MIT/Apache-2.0` | 第三方 | E6 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Peter Atashian <retep998@gmail.com> |
| `window-vibrancy` | `0.8.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2022 Tauri Programme within The Commons Conservancy |
| `windows` | `0.62.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-collections` | `0.3.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-core` | `0.62.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-future` | `0.3.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-implement` | `0.60.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-interface` | `0.59.3` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-link` | `0.2.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-numerics` | `0.3.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-result` | `0.4.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-strings` | `0.5.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-sys` | `0.45.0` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-sys` | `0.61.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-targets` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-threading` | `0.2.1` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows-version` | `0.1.7` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows_aarch64_gnullvm` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows_aarch64_msvc` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows_i686_gnu` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows_i686_msvc` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows_x86_64_gnu` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows_x86_64_gnullvm` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `windows_x86_64_msvc` | `0.42.2` | `MIT OR Apache-2.0` | 第三方 | E1 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) Microsoft Corporation. |
| `winnow` | `0.5.40` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `winnow` | `1.0.5` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `winreg` | `0.56.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Igor Shaula |
| `wit-bindgen` | `0.57.1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | 第三方 | E10 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: Alex Crichton <alex@alexcrichton.com> |
| `writeable` | `0.6.4` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `wry` | `0.57.2` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2020-2023 Ngo Iok Ui & Tauri Programme within The Commons Conservancy |
| `x11` | `2.21.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: daggerbot <daggerbot@gmail.com>; Erle Pereira <erle@erlepereira.com> |
| `x11-dl` | `2.21.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: daggerbot <daggerbot@gmail.com>; Erle Pereira <erle@erlepereira.com> |
| `xml` | `1.4.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2014 Vladimir Matveev |
| `xml-rs` | `1.0.0` | `MIT` | 第三方 | E2 · 包内LICENSE:无 · NOTICE:无 · 版权/获取:authors: Vladimir Matveev <vmatveev@citrine.cc>; Kornel (https://github.com/kornelski) |
| `xmltree` | `0.12.0` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2015 Andrew Chin |
| `yoke` | `0.8.4` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `yoke-derive` | `0.8.4` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerofrom` | `0.1.8` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerofrom-derive` | `0.1.8` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zeroize` | `1.9.1` | `Apache-2.0 OR MIT` | 第三方 | E3 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright (c) 2018-2026 The RustCrypto Project Developers |
| `zerotrie` | `0.2.5` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerovec` | `0.11.9` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zerovec-derive` | `0.11.7` | `Unicode-3.0` | 第三方 | E4 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:Copyright © 2020-2024 Unicode, Inc. |
| `zlib-rs` | `0.6.8` | `Zlib` | 第三方 | E20 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:见 §3 索引官方全文 URL |
| `zmij` | `1.0.23` | `MIT` | 第三方 | E2 · 包内LICENSE:有 · NOTICE:无 · 版权/获取:authors: David Tolnay <dtolnay@gmail.com> |


---

## 5. MPL-2.0 专门段（文件级弱 copyleft）

**本项目准确措辞：无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft。**

### 5.1 条款要点（引自 m4-step4-report.md §2.4 / §2.4.1）

- **§3.1**：Covered Software 的发行须 "under the terms of this License"，且须以 Source Code Form 提供 §3.2 所要求的源码。
- **§3.2**：源码须「以合理方式及时可得」；原文措辞 "reasonable means in a timely manner, at a charge **no more than the cost of distribution**"（收费不超过分发成本）。FAQ Q26：任何增加成本与复杂度的机制（如 courier）**"is not reasonable"**。
- **§3.3**：可执行形式可按自选许可发行，"under terms of Your choice, **provided that You also comply with the requirements of this License for the Covered Software**"。
- **§3.4**：不得移除或改动 Covered Software 中的许可 / 版权声明。
- **Exhibit A**：标准 Source Code Form License Notice 头（"This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/."），须保留。
- **§1.12 "Secondary License"** 定义字面含 GPL / LGPL / AGPL 字样；本机全局扫描的 LICENSE 文件中命中 GPL 短语属该定义文字，**不是**本项目引入 GPL 家族依赖的证据。
- **FAQ Q1**："The MPL's **"file-level" copyleft**" 允许与其他许可（**open or proprietary**）的代码组合。
- **FAQ Q8**："You must **inform the recipients where they can get the source** for the MPLed code … (i.e., you must comply with **Section 3.2**). You may distribute any executables … under a license of your choosing, as long as that license does not interfere with the recipients' rights to the source…"
- **FAQ Q11**："**new files containing no MPL-licensed code are not Modifications**"，静态链接到 "**larger proprietary** piece of software" 不使专有文件变成 Modifications。
- **FAQ Q12**："MPL: The copyleft applies to **any files containing MPLed code**"。

### 5.2 「源码可得要约」（§3.2 / FAQ Q8）——本文件据此作出告知

对下列**未修改**的 MPL-2.0 库，本告知文件本身即为 §3.1 / FAQ Q8 所要求的告知：

1. 明确声明这些具体文件**受 MPL-2.0 管辖**；
2. 给出 MPL-2.0 全文获取方式：<https://www.mozilla.org/media/MPL/2.0/index.txt> 与 <https://spdx.org/licenses/MPL-2.0.html>；各库上游源码可由 crates.io 对应 `<name> <version>` 源码包取得（本机已解包路径见 §3 索引）；
3. 以 "**reasonable means in a timely manner, at a charge no more than the cost of distribution**"（以合理方式及时可得、收费不超过分发成本）告知受 MPL 管辖文件的 Source Code Form 获取位置；
4. 保留各库 Exhibit A 头与 §3.4 声明，不作移除或改动。

**链接未修改 MPL 库的专有 App 不须公开自身源码**（FAQ Q11 / §3.3）：本项目专有代码与仅被链接、未被修改的 MPL 库组合时，MPL 不要求公开专有 App 自身源码；可执行形式可按 §3.3 自选许可发行。

**但被修改的具体文件须提供修改后源码**：一旦修改任一 MPL 依赖文件，或新建含 Covered Software 的文件（§1.10 Modifications），**这些具体文件**须依 §3.1 以 MPL-2.0 提供修改后源码——这是**文件级**而非全项目级 copyleft（FAQ Q1 / Q12 / §2.4.1 综合判断）。本轮为只读审计与告知生成，**未修改任何上游依赖文件**；是否修改、修改了哪些文件须在实际发行前逐依赖复核并留痕。

### 5.3 逐依赖 MPL 判定

- **纯 MPL-2.0（明确受 MPL-2.0 管辖）**：见 §5.4 清单；共 7 个依赖行。
- **MPL 仅作为 '/' 分隔非 SPDX 表达式中的一个选项**：`htmlescape 0.3.1` 的 `"Apache-2.0 / MIT / MPL-2.0"` **不是** SPDX "OR" 表达式，**不自动等同**可选 MPL-2.0。须**逐依赖明确按哪一个许可使用**并由律师 / 合规在签发前确认；在作出书面选择前，本文件**既不宣称其受 MPL-2.0 管辖，也不宣称其不受**，不作臆测。详见 §8。

#### 5.4 纯 MPL-2.0 依赖清单（明确受 MPL-2.0 管辖，共 7 行）

| workspace | 包名 | 版本 | 许可表达式 | 包内 LICENSE | 上游源码 / MPL 全文获取方式 |
| --- | --- | --- | --- | --- | --- |
| desktop/src-tauri | `cssparser` | `0.37.0` | `MPL-2.0` | 有 | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |
| desktop/src-tauri | `cssparser` | `0.38.0` | `MPL-2.0` | 有 | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |
| desktop/src-tauri | `cssparser-macros` | `0.7.1` | `MPL-2.0` | 有 | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |
| desktop/src-tauri | `dtoa-short` | `0.3.5` | `MPL-2.0` | 有 | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |
| desktop/src-tauri | `option-ext` | `0.2.0` | `MPL-2.0` | 有 | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |
| desktop/src-tauri | `selectors` | `0.38.0` | `MPL-2.0` | 无 | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |
| desktop/src-tauri | `selectors` | `0.41.0` | `MPL-2.0` | 无 | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |

#### 5.5 MPL 作为非 SPDX '/' 表达式选项的依赖（不臆测，见 §8）

| workspace | 包名 | 版本 | 表达式 | 判定 |
| --- | --- | --- | --- | --- |
| core | `htmlescape` | `0.3.1` | `Apache-2.0 / MIT / MPL-2.0` | '/' 分隔非 SPDX，须逐依赖明确择一；未书面选择前不宣称受 / 不受 MPL-2.0 管辖 |
| desktop/src-tauri | `htmlescape` | `0.3.1` | `Apache-2.0 / MIT / MPL-2.0` | '/' 分隔非 SPDX，须逐依赖明确择一；未书面选择前不宣称受 / 不受 MPL-2.0 管辖 |


---

## 6. Apache-2.0 §4(a)–(d) 段

引自 m4-step4-report.md §2.4.1（原文逐字，英文文本为准："**Only the English text of the Apache License is authoritative.**"）：

> **§4 Redistribution**（四项义务，逐字）：
> 1. "You must give any other recipients of the Work or Derivative Works a **copy of this License**"; and
> 2. "You must cause any **modified files** to carry prominent notices stating that You changed the files"; and
> 3. "You must **retain** … all copyright, patent, trademark, and attribution notices …"; and
> 4. "**If the Work includes a "NOTICE" text file** … any Derivative Works that You distribute must include a readable copy of the attribution notices …"

- 全文获取方式：<https://www.apache.org/licenses/LICENSE-2.0.txt>，SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>。
- **§4(d) 仅当上游含 NOTICE 时触发**：条款以 "**If the Work includes a "NOTICE" text file as part of its distribution**" 起首，因此只有当上游依赖**自带** `NOTICE` 文本文件时，才需要在衍生作品中附带其归属声明；上游没有 NOTICE 文件则不触发该条。本文件 §3 索引逐依赖给出本机 NOTICE 文件检出结果。
- 本机核验事实（**不作法律结论**）：本轮扫描两份 lock 的全部已解包依赖目录，带 `NOTICE*` 文件的 crate 数为 **0**；即本地依赖树内未检出上游 NOTICE 文件。该数字是本机时点事实，不代表上游仓库 / 其他版本绝无 NOTICE 文件，签发前须按实际上游版本复核。
- §1 逐字："**Derivative Works shall not include works that … merely link (or bind by name) to the interfaces of, the Work**"——仅按接口链接不当然构成 Derivative Works。

### 6.1 本项目据此的落地动作（告知，不作结论）

- 随可分发包附带 Apache-2.0 许可全文副本（§4(a)）；
- 若修改了任何 Apache-2.0 依赖文件，在修改文件加显著变更声明（§4(b)）；本轮只读生成，**未修改任何上游文件**；
- 保留上游全部 copyright / patent / trademark / attribution 声明（§4(c)）——版权持有人见 §3 索引逐表达式的版权声明示例与 `package.authors`；
- 逐依赖核对上游 NOTICE 文件；命中则附带可读归属声明副本（§4(d)），未命中则记录未命中（本机结果见上）。


---

## 7. MIT 归属段

引自 m4-step4-report.md §2.4.1（原文逐字）：

> "**The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.**"

- 全文获取方式：<https://opensource.org/license/mit>，SPDX 页面 <https://spdx.org/licenses/MIT.html>。
- choosealicense 归纳（引用）：MIT "conditions only requiring **preservation of copyright and license notices**"，允许 "distributed under different terms **and without source code**"。
- 落地动作：在随包告知与应用内 "Open source licenses" / "Acknowledgements" 界面中，对每个 MIT（及含 MIT 原子且按 MIT 使用的）依赖保留其版权声明与许可全文；版权持有人见 §3 索引逐表达式的 `Copyright` 行 / `package.authors`；未在本机包内检出显式 Copyright 行的，以官方全文 URL + 上游源码包作为「可获取方式」，**不臆造版权持有人**。


---

## 8. 双许可 / 非 SPDX 表达式 —— 逐依赖判定（不臆测）

### 8.1 '/' 分隔 ≠ SPDX "OR"

`htmlescape 0.3.1` 的 `license = "Apache-2.0 / MIT / MPL-2.0"` 是 **'/' 分隔的非 SPDX 表达式**，**不是** `"Apache-2.0 OR MIT OR MPL-2.0"`。SPDX "OR" 表示可择一，'/' 分隔字符串在 Cargo.toml 中不是合法 SPDX 表达式，不能自动推导为三选一。

- 该表达式在 core 与 desktop 两份 lock 中各出现一次（同一个包）。
- **须逐依赖明确判定按 Apache-2.0、MIT、还是 MPL-2.0 使用**；选择直接影响是否需要 MPL-2.0 §3.2 源码可得要约（§5）。
- 在书面选择作出并经律师 / 合规律师签发确认前，本文件**不臆测**该包按哪一许可使用，**不将其计入「纯 MPL-2.0 受管辖清单」**，也**不宣称其不受 MPL-2.0 约束**。
- 该包本机解包目录未检出独立 LICENSE 文件（目录含 `.cargo-ok` / `.gitignore` / `.travis.yml` / `Cargo.toml` / `moonstone-short.txt` / `README.md`）；报告 §6 6g 记其上游 `github.com/veddan/rust-htmlescape/master/LICENSE` 为 HTTP 404。许可全文 / 版权声明取得方式以 §3 索引给出的三个原子官方全文 URL 与 crates.io 源码包为准。

### 8.2 全部 '/' 分隔非 SPDX 表达式（均须逐依赖判定）

见 §8.4 清单：共 22 个依赖行。它们与 `htmlescape 0.3.1` 同理，均不作自动择一、不作臆测。

### 8.3 含 copyleft 原子的合法 SPDX "OR" 表达式

`r-efi 5.3.0` 与 `r-efi 6.0.0` 的 `license = "MIT OR Apache-2.0 OR LGPL-2.1-or-later"` 是**合法 SPDX OR 表达式**，其中 `LGPL-2.1-or-later` 仅为**可选原子**，另有 MIT / Apache-2.0 可择一。

- 这是 `cargo fetch --locked` 补齐元数据后新增可见的事实（fetch 前这些包不可核验）；报告「可核验范围内 GPL 家族 0」是 **fetch 前口径**。
- 本文件如实记录该 OR 选项，**不改写冻结边界**（§0），也**不据此宣称存在 GPL 家族强 copyleft 义务**：OR 表达式下可选 MIT / Apache-2.0，是否触发 LGPL 义务取决于逐依赖实际采用的许可，须在发行前逐依赖确认并留痕。
- 对外表述应避免说「绝对无 GPL/LGPL 字样」；准确措辞仍为 §0 的「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」。
- **目标门控事实（不作「已编译进产物」的断言）**：`r-efi` 是 `getrandom` 的 target-specific 传递依赖，其 Cargo.toml 声明位于 `[target.'cfg(all(target_os = "uefi", getrandom_backend = "efi_rng"))'.dependencies.r-efi]`（`getrandom-0.3.4/Cargo.toml:91`、`getrandom-0.4.3/Cargo.toml:103`）。即仅在 **UEFI 目标 + `efi_rng` 后端** 下才被拉入依赖图；对本项目当前的 `x86_64-pc-windows-gnu` 目标不参与编译，仅作为两份 `Cargo.lock` 里的 target-specific 传递条目存在。Cargo.lock 上下文：`core/Cargo.lock` getrandom 0.4.3、`desktop/src-tauri/Cargo.lock` getrandom 0.3.4（r-efi 5.3.0）与 getrandom 0.4.3（r-efi 6.0.0）。

### 8.4 判定清单（由脚本逐依赖生成）

| workspace | 包名 | 版本 | 表达式（'/' 分隔） | 判定 |
| --- | --- | --- | --- | --- |
| core | `htmlescape` | `0.3.1` | `Apache-2.0 / MIT / MPL-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| core | `version_check` | `0.9.5` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `bitflags` | `1.3.2` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `brotli-decompressor` | `6.3.1` | `BSD-3-Clause/MIT` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `bs58` | `0.5.1` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `cesu8` | `1.1.0` | `Apache-2.0/MIT` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `dbus` | `0.9.12` | `Apache-2.0/MIT` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `fnv` | `1.0.7` | `Apache-2.0 / MIT` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `foreign-types` | `0.5.0` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `foreign-types-macros` | `0.2.4` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `foreign-types-shared` | `0.3.1` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `htmlescape` | `0.3.1` | `Apache-2.0 / MIT / MPL-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `ident_case` | `1.0.1` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `jni` | `0.21.1` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `json-patch` | `4.2.0` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `libdbus-sys` | `0.2.7` | `Apache-2.0/MIT` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `same-file` | `1.0.6` | `Unlicense/MIT` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `version_check` | `0.9.5` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `walkdir` | `2.5.0` | `Unlicense/MIT` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `winapi` | `0.3.9` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `winapi-i686-pc-windows-gnu` | `0.4.0` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |
| desktop/src-tauri | `winapi-x86_64-pc-windows-gnu` | `0.4.0` | `MIT/Apache-2.0` | 非 SPDX，须逐依赖明确择一，不臆测 |


### 8.5 LGPL-OR 原子清单（由脚本逐依赖生成）

（本机核验结果：无含 GPL 家族原子的 OR 表达式依赖行。）


---

## 9. 冻结诚实边界（完整复述，与 §0 一致）


- 本轮准备包**非合规结论 / 非第三方审计报告 / 非法律意见 / 非已通过认证**。
- 草案**未经律师签发不得对外发布**。
- 当前 **NO-GO**；**≠ 产品发布 ≠ 五端验收 ≠ 第三方审计**。
- **未产出可分发包 · 未签名 · 证书自备**。
- **推迟 ≠ 豁免**：EAR 出口分类、PIPL/GDPR controller 定性、MPL 源码可得义务**不因规模小或免费消失**。
- 合规事实为 **2026-11-15 抓取时点值**。
- **准确措辞是「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」，绝不能说「没有任何 copyleft」。**

补充（§5 冻结边界 8 条，逐条列出）：

1. 本轮非合规结论 / 非第三方审计 / 非法律意见 / 非认证；
2. 草案未经律师签发不得对外发布；
3. 草案 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计；
4. 未产出可分发包 / 未签名 / 证书自备；
5. NO-GO 四道外部闸门：**A 第三方审计 / B 律师签发 / C 签名可分发包 / D 商店申报**，在真要上架或收费那天一个不能跳；
6. GDPR 引文来自 gdpr-info.eu（A⁻ 来源），须回官方核对；
7. 所有条文 / 价格 / 版本 / 商店规则为 2026-11-15 抓取时点值，签发前须复核；§2.3 加密出口事实不构成法律意见或合规结论，书面分类仍须律师 / 合规签发；
8. `NOT RETRIEVED` 不得当作已核验事实引用。


---

## 10. 应用内 "Open source licenses" / "Acknowledgements" 界面一致性

m4-step4-report.md §2.4.1 C 判断⑥：应用内 "Open source licenses" / "Acknowledgements" 界面须与本文件内容一致。本文件为自动生成草稿，落地时应以同一份脚本重跑结果作为界面数据源，避免两处清单漂移；界面同样须带 §0 的免责与冻结边界，且不得出现「已合规」「已通过认证」「没有任何 copyleft」等表述。


---

_本文件由 `tools/gen_third_party_notices.py` 于 2026-11-15 时点数据只读生成；重复运行可复现。非合规结论 / 非法律意见 / 非第三方审计报告 / 非已通过认证；未经律师签发不得对外发布。_
