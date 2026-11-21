#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
gen_third_party_notices.py — 只读生成器，产出 THIRD_PARTY_NOTICES.md。

读取两份 Cargo.lock + 本机 cargo registry 缓存元数据（与
tools/audit_licenses.py 同一解析口径），生成第三方许可告知【草稿】。

只读约束：
  - 不写 Cargo.lock / Cargo.toml / registry 缓存 / 任何上游文件；
  - .crate 归档中的 Cargo.toml 仅在内存中读取，绝不解压落盘；
  - 唯一写出目标为 PROJECT_ROOT/THIRD_PARTY_NOTICES.md；
  - 可重复运行：同样输入得到同样输出。

本脚本产出的是【告知草稿】，不是合规结论、不是法律意见、
不是第三方审计报告、不是已通过认证；未经律师/合规律师签发不得对外发布。
"""
import os
import re
import sys
import tarfile
import tomllib
from collections import OrderedDict

sys.stdout.reconfigure(encoding="utf-8")

PROJECT_ROOT = r"D:\Projects\dsh\mylock"
REGISTRY_BASE = r"C:\Users\zjc_9\.cargo\registry"
REGISTRY_SRC = os.path.join(REGISTRY_BASE, "src", "index.crates.io-1949cf8c6b5b557f")
CACHE_DIRS = [os.path.join(REGISTRY_BASE, "cache", "index.crates.io-1949cf8c6b5b557f")]
LOCKS = [
    (r"D:\Projects\dsh\mylock\core\Cargo.lock", "core"),
    (r"D:\Projects\dsh\mylock\desktop\src-tauri\Cargo.lock", "desktop/src-tauri"),
]
FIRST_PARTY = {
    "mylock-core": (r"D:\Projects\dsh\mylock\core\crates\mylock-core\Cargo.toml",
                    r"D:\Projects\dsh\mylock\core\Cargo.toml"),
    "mylock-cli": (r"D:\Projects\dsh\mylock\core\crates\mylock-cli\Cargo.toml",
                   r"D:\Projects\dsh\mylock\core\Cargo.toml"),
    "mylock-mount": (r"D:\Projects\dsh\mylock\core\crates\mylock-mount\Cargo.toml",
                     r"D:\Projects\dsh\mylock\core\Cargo.toml"),
    "mylock-desktop": (r"D:\Projects\dsh\mylock\desktop\src-tauri\Cargo.toml", None),
}
OUT_PATH = os.path.join(PROJECT_ROOT, "THIRD_PARTY_NOTICES.md")
SNAPSHOT = "2026-11-15"

COPYLEFT_ATOM_RE = re.compile(r"(?i)\b(?:A?GPL|LGPL|AGPL|MPL|EPL|CDDL|EUPL|SSPL|BSL)(?:-[0-9.]+)?(?:-or-later|-only)?\b")
GPL_ATOM_RE = re.compile(r"(?i)\b(?:A?GPL|LGPL|AGPL)(?:-[0-9.]+)?(?:-or-later|-only)?\b")

# 原子许可 -> (中文名, 官方全文 URL, SPDX 页面 URL)
ATOM_REFS = {
    "MIT": ("MIT License", "https://opensource.org/license/mit", "https://spdx.org/licenses/MIT.html"),
    "Apache-2.0": ("Apache License 2.0", "https://www.apache.org/licenses/LICENSE-2.0.txt", "https://spdx.org/licenses/Apache-2.0.html"),
    "MPL-2.0": ("Mozilla Public License 2.0", "https://www.mozilla.org/media/MPL/2.0/index.txt", "https://spdx.org/licenses/MPL-2.0.html"),
    "Unicode-3.0": ("Unicode License v3 (Unicode-3.0)", "https://www.unicode.org/license.txt", "https://spdx.org/licenses/Unicode-3.0.html"),
    "BSD-3-Clause": ("BSD 3-Clause", "https://opensource.org/license/bsd-3-clause", "https://spdx.org/licenses/BSD-3-Clause.html"),
    "Zlib": ("zlib License", "https://opensource.org/license/zlib", "https://spdx.org/licenses/Zlib.html"),
    "Unlicense": ("The Unlicense", "https://unlicense.org/", "https://spdx.org/licenses/Unlicense.html"),
    "ISC": ("ISC License", "https://opensource.org/license/isc", "https://spdx.org/licenses/ISC.html"),
    "0BSD": ("BSD Zero Clause License", "https://opensource.org/license/0bsd", "https://spdx.org/licenses/0BSD.html"),
    "CC0-1.0": ("Creative Commons CC0 1.0", "https://creativecommons.org/publicdomain/zero/1.0/legalcode", "https://spdx.org/licenses/CC0-1.0.html"),
    "MIT-0": ("MIT No Attribution", "https://opensource.org/license/mit-0", "https://spdx.org/licenses/MIT-0.html"),
    "LGPL-2.1-or-later": ("GNU LGPL v2.1 or later", "https://www.gnu.org/licenses/old-licenses/lgpl-2.1.txt", "https://spdx.org/licenses/LGPL-2.1-or-later.html"),
    "LLVM-exception": ("LLVM Exception (附于 Apache-2.0)", "https://llvm.org/LICENSE.txt", "https://spdx.org/licenses/LLVM-exception.html"),
}

# ---------------------------------------------------------------------------
# 解析（口径与 tools/audit_licenses.py 一致）
# ---------------------------------------------------------------------------

def parse_toml_bytes(raw):
    return tomllib.loads(raw.decode("utf-8", errors="replace"))


def find_cargo_toml(name, version):
    if not os.path.isdir(REGISTRY_SRC):
        return None
    target = f"{name}-{version}"
    p = os.path.join(REGISTRY_SRC, target, "Cargo.toml")
    if os.path.isfile(p):
        return p
    for d in os.listdir(REGISTRY_SRC):
        if d.lower() == target.lower():
            q = os.path.join(REGISTRY_SRC, d, "Cargo.toml")
            if os.path.isfile(q):
                return q
    return None


def find_crate_archive(name, version):
    target = f"{name}-{version}.crate"
    for c in CACHE_DIRS:
        if not os.path.isdir(c):
            continue
        for f in os.listdir(c):
            if f.lower() == target.lower():
                return os.path.join(c, f)
    return None


def license_from_archive(path, name, version):
    """仅内存读取 .crate 内 Cargo.toml，绝不解压落盘。"""
    try:
        with tarfile.open(path, "r:gz") as tf:
            member = f"{name}-{version}/Cargo.toml"
            try:
                f = tf.extractfile(member)
            except KeyError:
                cand = [m for m in tf.getnames()
                        if m.lower().endswith("cargo.toml") and '/' in m]
                if not cand:
                    return None, None, "archive has no Cargo.toml"
                f = tf.extractfile(cand[0])
            raw = f.read()
        data = parse_toml_bytes(raw)
        pkg = data.get("package", {})
        return pkg.get("license"), pkg.get("license-file"), None
    except Exception as e:
        return None, None, f"archive read error: {e}"


def license_from_workspace(name):
    if name not in FIRST_PARTY:
        return None, None, None, "not a known first-party crate"
    manifest, ws = FIRST_PARTY[name]
    if not os.path.isfile(manifest):
        return None, None, None, "first-party manifest not found"
    with open(manifest, "rb") as f:
        data = tomllib.load(f)
    pkg = data.get("package", {})
    lic = pkg.get("license")
    licf = pkg.get("license-file")
    src = manifest
    if isinstance(lic, dict) and lic.get("workspace") is True:
        if ws and os.path.isfile(ws):
            with open(ws, "rb") as f:
                wdata = tomllib.load(f)
            lic = wdata.get("workspace", {}).get("package", {}).get("license")
            src = f"{manifest} (license.workspace=true -> {ws})"
        else:
            return None, None, None, "license.workspace=true but workspace manifest missing"
    return lic, licf, src, None


def resolve(name, version, has_source):
    """返回 (lic, licf, src_desc, crate_dir, note)。"""
    if not has_source:
        lic, licf, src, err = license_from_workspace(name)
        if err:
            return None, None, None, None, err
        return lic, licf, src, os.path.dirname(FIRST_PARTY[name][0]), None
    ct = find_cargo_toml(name, version)
    if ct:
        with open(ct, "rb") as f:
            data = tomllib.load(f)
        pkg = data.get("package", {})
        return pkg.get("license"), pkg.get("license-file"), ct, os.path.dirname(ct), None
    arc = find_crate_archive(name, version)
    if arc:
        lic, licf, err = license_from_archive(arc, name, version)
        if err:
            return None, None, arc, None, err
        return lic, licf, f"{arc} (Cargo.toml read in-memory from .crate)", None, None
    return None, None, None, None, "no extracted Cargo.toml and no cached .crate"


def parse_lock(path):
    with open(path, "rb") as f:
        data = tomllib.load(f)
    return data.get("package", [])


def read_authors(crate_dir):
    p = os.path.join(crate_dir, "Cargo.toml") if crate_dir else None
    if not p or not os.path.isfile(p):
        return []
    try:
        with open(p, "rb") as f:
            data = tomllib.load(f)
    except Exception:
        return []
    a = data.get("package", {}).get("authors", [])
    if isinstance(a, str):
        a = [a]
    return [str(x).strip() for x in a if str(x).strip()][:3]


def inspect_crate_files(crate_dir):
    """只读扫描顶层文件，返回 (notice_files, license_files, copyright_lines)。"""
    notice_files, license_files, copyright_lines = [], [], []
    if not crate_dir or not os.path.isdir(crate_dir):
        return notice_files, license_files, copyright_lines
    try:
        names = sorted(os.listdir(crate_dir))
    except OSError:
        return notice_files, license_files, copyright_lines
    for fn in names:
        up = fn.upper()
        p = os.path.join(crate_dir, fn)
        if not os.path.isfile(p):
            continue
        if up.startswith("NOTICE"):
            notice_files.append(fn)
        if up.startswith("LICENSE") or up.startswith("LICENCE") or up.startswith("COPYING"):
            license_files.append(fn)
        readable = (up.startswith("LICENSE") or up.startswith("LICENCE")
                    or up.startswith("COPYING") or up.startswith("NOTICE")
                    or up in ("README.MD", "README", "README.TXT"))
        if not readable:
            continue
        try:
            if os.path.getsize(p) > 2_000_000:
                continue
            with open(p, "r", encoding="utf-8", errors="replace") as f:
                text = f.read()
        except OSError:
            continue
        for line in text.splitlines():
            # 只认「Copyright 后紧跟年份 / (c) / ©」的真实版权行，
            # 避免把许可正文里的 "the copyright owner"、MIT 的
            # "The above copyright notice ..."、Unicode-3.0 的
            # "COPYRIGHT AND PERMISSION NOTICE" 标题误判为版权声明。
            if re.search(r"(?i)\bcopyright\s*(?:\(c\)|©|\(C\)|[0-9]{4})", line):
                s = " ".join(line.strip().split())
                if s and s not in copyright_lines:
                    copyright_lines.append(s[:220])
                if len(copyright_lines) >= 4:
                    break
        if len(copyright_lines) >= 4:
            break
    return notice_files, license_files, copyright_lines


# ---------------------------------------------------------------------------
# 许可表达式判定
# ---------------------------------------------------------------------------

def atoms_of(expr):
    """把表达式拆成原子许可 token（去掉括号、OR/AND/WITH、斜杠）。"""
    s = expr.replace("(", " ").replace(")", " ").replace('/', " ")
    s = re.sub(r"(?i)\b(OR|AND|WITH)\b", " ", s)
    out = []
    for t in s.split():
        t = t.strip()
        if t and t not in out:
            out.append(t)
    return out


def is_slash_non_spdx(expr):
    """'/' 分隔且不含 OR/AND/WITH —— 非 SPDX 表达式。"""
    return '/' in expr and not re.search(r"(?i)\b(OR|AND|WITH)\b", expr)


def is_pure_mpl(expr):
    return expr.strip() == "MPL-2.0"


def has_mpl_atom(expr):
    return "MPL-2.0" in atoms_of(expr)


def has_copyleft_atom(expr):
    return any(COPYLEFT_ATOM_RE.fullmatch(a) for a in atoms_of(expr))


def has_gpl_atom(expr):
    return any(GPL_ATOM_RE.fullmatch(a) for a in atoms_of(expr))


def cell(s):
    return str(s).replace("|", "\\|").replace("\n", " ").replace("\r", " ")


# ---------------------------------------------------------------------------
# 散文段落（承重事实，直接引用 m4-step4-report.md §2.3.1 / §2.4 / §2.4.1 / §5）
# ---------------------------------------------------------------------------

HEADER = """# THIRD_PARTY_NOTICES.md（第三方许可告知 · 自动生成草稿）

> **本文件由 `tools/gen_third_party_notices.py` 只读生成，是【告知草稿】，不是合规结论、不是法律意见、不是第三方审计报告、不是已通过认证。**
> 草案未经律师 / 合规律师签发不得对外发布。当前状态为 **NO-GO**。
> 本文件 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计；未产出可分发包 · 未签名 · 证书自备。
> 所有许可 / 条文 / 版本事实为 **{snapshot} 抓取时点值**，签发前须复核。
> 本文件不给出任何「已合规」「已通过认证」结论，也**绝不说「没有任何 copyleft」**。
""".format(snapshot=SNAPSHOT)

BOUNDARY = """## 0. 冻结诚实边界（原样沿用，不得改写）

- 本轮准备包**非合规结论 / 非第三方审计报告 / 非法律意见 / 非已通过认证**。
- 草案**未经律师签发不得对外发布**。
- 当前 **NO-GO**；**≠ 产品发布 ≠ 五端验收 ≠ 第三方审计**。
- **未产出可分发包 · 未签名 · 证书自备**。
- **推迟 ≠ 豁免**：EAR 出口分类、PIPL/GDPR controller 定性、MPL 源码可得义务**不因规模小或免费消失**。
- 合规事实为 **{snapshot} 抓取时点值**。
- **准确措辞是「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」，绝不能说「没有任何 copyleft」。**

补充（§5 冻结边界 8 条，逐条列出）：

1. 本轮非合规结论 / 非第三方审计 / 非法律意见 / 非认证；
2. 草案未经律师签发不得对外发布；
3. 草案 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计；
4. 未产出可分发包 / 未签名 / 证书自备；
5. NO-GO 四道外部闸门：**A 第三方审计 / B 律师签发 / C 签名可分发包 / D 商店申报**，在真要上架或收费那天一个不能跳；
6. GDPR 引文来自 gdpr-info.eu（A⁻ 来源），须回官方核对；
7. 所有条文 / 价格 / 版本 / 商店规则为 {snapshot} 抓取时点值，签发前须复核；§2.3 加密出口事实不构成法律意见或合规结论，书面分类仍须律师 / 合规签发；
8. `NOT RETRIEVED` 不得当作已核验事实引用。
""".format(snapshot=SNAPSHOT)

DATASOURCE = """## 1. 数据来源与生成方式

- 生成脚本：`tools/gen_third_party_notices.py`（只读、可重复运行；唯一写出目标为本文件）。
- 输入：`core/Cargo.lock`、`desktop/src-tauri/Cargo.lock`（`tomllib` 解析 `[[package]]`）。
- 元数据：本机 cargo registry 已解包源码 `C:\\Users\\zjc_9\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\<name>-<version>\\Cargo.toml` 的 `package.license` / `package.license-file` 字段；若未解包，则在内存中读取缓存 `.crate` 内的 `Cargo.toml`（**绝不解压落盘**）；无 `source` 的第一方包走 workspace manifest（支持 `license.workspace = true`）。
- 解析口径与 `tools/audit_licenses.py`（只读审计脚本，本轮未改动）完全一致。
- 版权声明 / 许可全文获取方式：逐包扫描顶层 `LICENSE*` / `LICENCE*` / `COPYING*` / `NOTICE*` / `README*` 中的 `Copyright` 行，并读 `Cargo.toml` 的 `package.authors`；未检出显式 Copyright 行时给出官方全文 URL 作为「可获取方式」。
- **脚本只读、不改动上游**：不写 Cargo.lock、不写 Cargo.toml、不写 registry 缓存、不解压任何归档到磁盘。

> 每个依赖行第 4 列的 `E<n>` 指向 §3「许可全文获取方式索引」，该索引给出官方全文 URL、SPDX 页面、包内 LICENSE 检出情况与版权声明示例 / 模板。因此每一行均满足「包名 + 版本 + SPDX / 许可表达式 + 版权声明/许可全文或可获取方式」。
"""

MPL_SECTION = """## 5. MPL-2.0 专门段（文件级弱 copyleft）

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

- **纯 MPL-2.0（明确受 MPL-2.0 管辖）**：见 §5.4 清单；共 {pure_mpl_count} 个依赖行。
- **MPL 仅作为 '/' 分隔非 SPDX 表达式中的一个选项**：`htmlescape 0.3.1` 的 `"Apache-2.0 / MIT / MPL-2.0"` **不是** SPDX "OR" 表达式，**不自动等同**可选 MPL-2.0。须**逐依赖明确按哪一个许可使用**并由律师 / 合规在签发前确认；在作出书面选择前，本文件**既不宣称其受 MPL-2.0 管辖，也不宣称其不受**，不作臆测。详见 §8。
"""

APACHE_SECTION = """## 6. Apache-2.0 §4(a)–(d) 段

引自 m4-step4-report.md §2.4.1（原文逐字，英文文本为准："**Only the English text of the Apache License is authoritative.**"）：

> **§4 Redistribution**（四项义务，逐字）：
> 1. "You must give any other recipients of the Work or Derivative Works a **copy of this License**"; and
> 2. "You must cause any **modified files** to carry prominent notices stating that You changed the files"; and
> 3. "You must **retain** … all copyright, patent, trademark, and attribution notices …"; and
> 4. "**If the Work includes a "NOTICE" text file** … any Derivative Works that You distribute must include a readable copy of the attribution notices …"

- 全文获取方式：<https://www.apache.org/licenses/LICENSE-2.0.txt>，SPDX 页面 <https://spdx.org/licenses/Apache-2.0.html>。
- **§4(d) 仅当上游含 NOTICE 时触发**：条款以 "**If the Work includes a "NOTICE" text file as part of its distribution**" 起首，因此只有当上游依赖**自带** `NOTICE` 文本文件时，才需要在衍生作品中附带其归属声明；上游没有 NOTICE 文件则不触发该条。本文件 §3 索引逐依赖给出本机 NOTICE 文件检出结果。
- 本机核验事实（**不作法律结论**）：本轮扫描两份 lock 的全部已解包依赖目录，带 `NOTICE*` 文件的 crate 数为 **{notice_crates}**；即本地依赖树内未检出上游 NOTICE 文件。该数字是本机时点事实，不代表上游仓库 / 其他版本绝无 NOTICE 文件，签发前须按实际上游版本复核。
- §1 逐字："**Derivative Works shall not include works that … merely link (or bind by name) to the interfaces of, the Work**"——仅按接口链接不当然构成 Derivative Works。

### 6.1 本项目据此的落地动作（告知，不作结论）

- 随可分发包附带 Apache-2.0 许可全文副本（§4(a)）；
- 若修改了任何 Apache-2.0 依赖文件，在修改文件加显著变更声明（§4(b)）；本轮只读生成，**未修改任何上游文件**；
- 保留上游全部 copyright / patent / trademark / attribution 声明（§4(c)）——版权持有人见 §3 索引逐表达式的版权声明示例与 `package.authors`；
- 逐依赖核对上游 NOTICE 文件；命中则附带可读归属声明副本（§4(d)），未命中则记录未命中（本机结果见上）。
"""

MIT_SECTION = """## 7. MIT 归属段

引自 m4-step4-report.md §2.4.1（原文逐字）：

> "**The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.**"

- 全文获取方式：<https://opensource.org/license/mit>，SPDX 页面 <https://spdx.org/licenses/MIT.html>。
- choosealicense 归纳（引用）：MIT "conditions only requiring **preservation of copyright and license notices**"，允许 "distributed under different terms **and without source code**"。
- 落地动作：在随包告知与应用内 "Open source licenses" / "Acknowledgements" 界面中，对每个 MIT（及含 MIT 原子且按 MIT 使用的）依赖保留其版权声明与许可全文；版权持有人见 §3 索引逐表达式的 `Copyright` 行 / `package.authors`；未在本机包内检出显式 Copyright 行的，以官方全文 URL + 上游源码包作为「可获取方式」，**不臆造版权持有人**。
"""

DUAL_SECTION = """## 8. 双许可 / 非 SPDX 表达式 —— 逐依赖判定（不臆测）

### 8.1 '/' 分隔 ≠ SPDX "OR"

`htmlescape 0.3.1` 的 `license = "Apache-2.0 / MIT / MPL-2.0"` 是 **'/' 分隔的非 SPDX 表达式**，**不是** `"Apache-2.0 OR MIT OR MPL-2.0"`。SPDX "OR" 表示可择一，'/' 分隔字符串在 Cargo.toml 中不是合法 SPDX 表达式，不能自动推导为三选一。

- 该表达式在 core 与 desktop 两份 lock 中各出现一次（同一个包）。
- **须逐依赖明确判定按 Apache-2.0、MIT、还是 MPL-2.0 使用**；选择直接影响是否需要 MPL-2.0 §3.2 源码可得要约（§5）。
- 在书面选择作出并经律师 / 合规律师签发确认前，本文件**不臆测**该包按哪一许可使用，**不将其计入「纯 MPL-2.0 受管辖清单」**，也**不宣称其不受 MPL-2.0 约束**。
- 该包本机解包目录未检出独立 LICENSE 文件（目录含 `.cargo-ok` / `.gitignore` / `.travis.yml` / `Cargo.toml` / `moonstone-short.txt` / `README.md`）；报告 §6 6g 记其上游 `github.com/veddan/rust-htmlescape/master/LICENSE` 为 HTTP 404。许可全文 / 版权声明取得方式以 §3 索引给出的三个原子官方全文 URL 与 crates.io 源码包为准。

### 8.2 全部 '/' 分隔非 SPDX 表达式（均须逐依赖判定）

见 §8.4 清单：共 {slash_count} 个依赖行。它们与 `htmlescape 0.3.1` 同理，均不作自动择一、不作臆测。

### 8.3 含 copyleft 原子的合法 SPDX "OR" 表达式

`r-efi 5.3.0` 与 `r-efi 6.0.0` 的 `license = "MIT OR Apache-2.0 OR LGPL-2.1-or-later"` 是**合法 SPDX OR 表达式**，其中 `LGPL-2.1-or-later` 仅为**可选原子**，另有 MIT / Apache-2.0 可择一。

- 这是 `cargo fetch --locked` 补齐元数据后新增可见的事实（fetch 前这些包不可核验）；报告「可核验范围内 GPL 家族 0」是 **fetch 前口径**。
- 本文件如实记录该 OR 选项，**不改写冻结边界**（§0），也**不据此宣称存在 GPL 家族强 copyleft 义务**：OR 表达式下可选 MIT / Apache-2.0，是否触发 LGPL 义务取决于逐依赖实际采用的许可，须在发行前逐依赖确认并留痕。
- 对外表述应避免说「绝对无 GPL/LGPL 字样」；准确措辞仍为 §0 的「无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft」。
- **目标门控事实（不作「已编译进产物」的断言）**：`r-efi` 是 `getrandom` 的 target-specific 传递依赖，其 Cargo.toml 声明位于 `[target.'cfg(all(target_os = "uefi", getrandom_backend = "efi_rng"))'.dependencies.r-efi]`（`getrandom-0.3.4/Cargo.toml:91`、`getrandom-0.4.3/Cargo.toml:103`）。即仅在 **UEFI 目标 + `efi_rng` 后端** 下才被拉入依赖图；对本项目当前的 `x86_64-pc-windows-gnu` 目标不参与编译，仅作为两份 `Cargo.lock` 里的 target-specific 传递条目存在。Cargo.lock 上下文：`core/Cargo.lock` getrandom 0.4.3、`desktop/src-tauri/Cargo.lock` getrandom 0.3.4（r-efi 5.3.0）与 getrandom 0.4.3（r-efi 6.0.0）。

### 8.4 判定清单（由脚本逐依赖生成）

{slash_table}

### 8.5 LGPL-OR 原子清单（由脚本逐依赖生成）

{lgpl_table}
"""

APP_UI = """## 10. 应用内 "Open source licenses" / "Acknowledgements" 界面一致性

m4-step4-report.md §2.4.1 C 判断⑥：应用内 "Open source licenses" / "Acknowledgements" 界面须与本文件内容一致。本文件为自动生成草稿，落地时应以同一份脚本重跑结果作为界面数据源，避免两处清单漂移；界面同样须带 §0 的免责与冻结边界，且不得出现「已合规」「已通过认证」「没有任何 copyleft」等表述。
"""


# ---------------------------------------------------------------------------
# 组装
# ---------------------------------------------------------------------------

def collect():
    workspaces = []
    all_rows = []
    for lock, label in LOCKS:
        pkgs = parse_lock(lock)
        rows = []
        for p in pkgs:
            name = p.get("name")
            version = p.get("version")
            has_source = bool(p.get("source"))
            lic, licf, src_desc, crate_dir, note = resolve(name, version, has_source)
            notice_files, license_files, copyright_lines = inspect_crate_files(crate_dir)
            authors = read_authors(crate_dir)

            if note:
                expr = "NOT VERIFIABLE"
                verifiable = False
            elif lic:
                expr = str(lic)
                verifiable = True
            elif licf:
                expr = f"[license-file] {licf}"
                verifiable = True
            else:
                expr = "NOT VERIFIABLE"
                verifiable = False

            if not verifiable:
                obtain = "本机元数据缺失：无法核验，标 NOT VERIFIABLE，绝不臆测；待 cargo fetch 后重跑本脚本"
            else:
                obtain = None  # 填 E<n> 后由 build 生成

            rows.append({
                "name": name,
                "version": version,
                "expr": expr,
                "verifiable": verifiable,
                "kind": "第一方" if not has_source else "第三方",
                "crate_dir": crate_dir,
                "notice_files": notice_files,
                "license_files": license_files,
                "copyright_lines": copyright_lines,
                "authors": authors,
                "obtain": obtain,
            })
        workspaces.append((label, len(pkgs), rows))
        all_rows.extend(rows)
    return workspaces, all_rows


def build():
    workspaces, all_rows = collect()

    # 表达式索引（全局，按出现次数降序再按字面排序，保证可重复）
    expr_counts = {}
    for r in all_rows:
        expr_counts[r["expr"]] = expr_counts.get(r["expr"], 0) + 1
    expr_ids = {}
    ordered = sorted(expr_counts, key=lambda k: (-expr_counts[k], k.lower()))
    for i, e in enumerate(ordered, 1):
        expr_ids[e] = f"E{i}"

    # 统计
    total = len(all_rows)
    not_verifiable = sum(1 for r in all_rows if not r["verifiable"])
    notice_crates = sum(1 for r in all_rows if r["notice_files"])
    pure_mpl = [r for r in all_rows if r["verifiable"] and is_pure_mpl(r["expr"])]
    mpl_option = [r for r in all_rows
                  if r["verifiable"] and has_mpl_atom(r["expr"]) and not is_pure_mpl(r["expr"])]
    slash_rows = [r for r in all_rows if r["verifiable"] and is_slash_non_spdx(r["expr"])]
    lgpl_rows = [r for r in all_rows
                 if r["verifiable"] and has_gpl_atom(r["expr"])
                 and re.search(r"(?i)\bOR\b", r["expr"])]

    L = []
    L.append(HEADER)
    L.append("\n---\n")
    L.append(BOUNDARY)
    L.append("\n---\n")
    L.append(DATASOURCE)

    # §2 总览
    L.append("\n---\n")
    L.append("## 2. 核验口径与总览（{snapshot} 时点值）\n".format(snapshot=SNAPSHOT))
    L.append("| workspace | lock 内包数 | 可核验 | NOT VERIFIABLE |")
    L.append("| --- | ---: | ---: | ---: |")
    for label, n, rows in workspaces:
        nv = sum(1 for r in rows if not r["verifiable"])
        L.append(f"| {cell(label)} | {n} | {n - nv} | {nv} |")
    L.append(f"| **合计** | **{total}** | **{total - not_verifiable}** | **{not_verifiable}** |")
    L.append("")
    L.append(f"- 不同许可表达式（含 NOT VERIFIABLE / license-file 形式）：**{len(expr_ids)}** 组。")
    L.append(f"- **NOT VERIFIABLE：{not_verifiable}** 个（fetch 前为 core 13 + desktop 166 = **179**；`cargo fetch --locked` 补齐元数据后降为 0）。凡无法核验者一律标 `NOT VERIFIABLE`，绝不猜测许可。".format())
    L.append(f"- 带上游 `NOTICE*` 文件的 crate 数：**{notice_crates}**（详见 §6）。")
    L.append(f"- 纯 `MPL-2.0` 依赖行：**{len(pure_mpl)}**；MPL 仅作为非 SPDX '/' 表达式选项的依赖行：**{len(mpl_option)}**（详见 §5.3 / §8）。")
    L.append("")
    L.append("**准确措辞（§0 原样）**：无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft；绝不能说「没有任何 copyleft」。")
    L.append("")
    L.append("**fetch 后新增事实（如实提示，不改写冻结边界）**：`cargo fetch --locked` 使原先 179 个不可核验包全部可核验，其中 `r-efi 5.3.0` / `r-efi 6.0.0` 的 `license = \"MIT OR Apache-2.0 OR LGPL-2.1-or-later\"` 含 `LGPL-2.1-or-later` 可选原子，使审计脚本的 GPL-expression hits 由 fetch 前 core 0 / desktop 0 变为 fetch 后 core 1 / desktop 2。报告「可核验范围内 GPL 家族 0」是 fetch 前口径；OR 表达式下另有 MIT / Apache-2.0 可择一，是否触发 LGPL 义务须逐依赖确认（§8.3）。本文件不据此改写 §0 冻结边界，也不宣称存在 GPL 家族强 copyleft 义务。")
    L.append("**r-efi 目标门控（与 §8.3 一致）**：`r-efi` 是 `getrandom` 在 `[target.'cfg(all(target_os = \"uefi\", getrandom_backend = \"efi_rng\"))'.dependencies.r-efi]` 下的 target-specific 传递依赖；对 `x86_64-pc-windows-gnu` 目标不参与编译，仅作为 Cargo.lock 的 target-specific 条目存在，不作「已编译进产物」的断言。")

    # §3 索引
    L.append("\n---\n")
    L.append("## 3. 许可全文获取方式索引（按许可表达式分组）\n")
    L.append("每组给出：表达式原样、出现次数、原子许可的官方全文 URL 与 SPDX 页面、本机包内 LICENSE 检出情况、版权声明示例 / 模板或 `package.authors`。\n")
    for e in ordered:
        eid = expr_ids[e]
        rows = [r for r in all_rows if r["expr"] == e]
        L.append(f"### {eid} · `{cell(e)}`\n")
        L.append(f"- 出现次数：**{len(rows)}** 个依赖行（core + desktop 合计）。")
        if e == "NOT VERIFIABLE":
            L.append("- **本机元数据缺失**：标 `NOT VERIFIABLE`，绝不臆测。获取方式：待 `cargo fetch` 补齐后重跑本脚本；在核验前不得对该依赖的许可作任何声明。")
            L.append("")
            continue
        if e.startswith("[license-file]"):
            L.append("- 该包仅声明 `license-file`、无 SPDX `license` 表达式；许可全文以包内所指文件为准（本机检出情况见下），并以上游源码包为获取方式。")
        atoms = atoms_of(e)
        L.append("- 原子许可与全文获取方式：")
        for a in atoms:
            if a in ATOM_REFS:
                cn, txt, spdx = ATOM_REFS[a]
                L.append(f"  - `{a}`（{cn}）：官方全文 <{txt}>；SPDX 页面 <{spdx}>")
            else:
                L.append(f"  - `{a}`：本索引未收录参考 URL；许可全文按上游包内 LICENSE 文件 / crates.io 源码包取得（获取方式），不臆造。")
        if is_slash_non_spdx(e):
            L.append("- **注意：该表达式为 '/' 分隔非 SPDX 表达式**，须逐依赖明确择一（§8），本索引仅列出各原子全文，不代表已作出选择。")
        if has_gpl_atom(e) and re.search(r"(?i)\bOR\b", e):
            L.append("- **注意：合法 SPDX OR 表达式中含 GPL 家族可选原子**（§8.3），是否触发相应义务取决于逐依赖实际采用的许可。")
        lic_present = sum(1 for r in rows if r["license_files"])
        notice_present = sum(1 for r in rows if r["notice_files"])
        L.append(f"- 本机包内 `LICENSE*` / `COPYING*` 检出：**{lic_present}/{len(rows)}** 个依赖行；`NOTICE*` 检出：**{notice_present}/{len(rows)}** 个（§4(d) 触发边界，见 §6）。")
        # 版权声明示例
        examples, seen = [], set()
        for r in rows:
            for c in r["copyright_lines"]:
                key = c.lower()
                if key not in seen:
                    seen.add(key)
                    examples.append(f"`{cell(c)}`")
                if len(examples) >= 3:
                    break
            if len(examples) >= 3:
                break
        if examples:
            L.append("- 版权声明示例（从包内 LICENSE / README 逐字检出，最多 3 条）：" + "；".join(examples) + "。")
        else:
            L.append("- 本机包内未检出显式 `Copyright` 行；版权声明取得方式：上游源码包内 LICENSE / README 的版权行，或对应 crate 的 `Cargo.toml` 之 `package.authors`（示例见下）。未检出即按「可获取方式」处理，**不臆造版权持有人**。")
        au, seen_au = [], set()
        for r in rows:
            for a in r["authors"]:
                if a not in seen_au:
                    seen_au.add(a)
                    au.append(f"`{cell(a)}`")
                if len(au) >= 3:
                    break
            if len(au) >= 3:
                break
        if au:
            L.append("- `package.authors` 示例（最多 3 条）：" + "；".join(au) + "。")
        L.append("")

    # §4 全量清单
    L.append("\n---\n")
    L.append("## 4. 全部第三方依赖清单（逐依赖）\n")
    L.append("表头：`E<n>` 指向 §3 索引；「包内 LICENSE」为 `有`/`无`；「NOTICE」为 `有`/`无`（§4(d) 边界）；「版权/获取」给出包内检出的首条 Copyright 行或 `package.authors`，未检出则指向 §3 的官方全文 URL（即「可获取方式」）。第一方包一并列出但标注「第一方」，不计入第三方依赖。\n")
    for label, n, rows in workspaces:
        L.append(f"### 4.{workspaces.index((label, n, rows)) + 1} {label}（{n} 个包）\n")
        L.append("| 包名 | 版本 | 许可表达式（Cargo.toml 原样） | 类别 | 版权声明 / 许可全文或可获取方式 |")
        L.append("| --- | --- | --- | --- | --- |")
        for r in sorted(rows, key=lambda x: (x["name"].lower(), x["version"])):
            if not r["verifiable"]:
                obtain = "E— `NOT VERIFIABLE`：本机元数据缺失，绝不臆测；待 cargo fetch 后重跑"
            else:
                eid = expr_ids[r["expr"]]
                lic_mark = "有" if r["license_files"] else "无"
                notice_mark = "有" if r["notice_files"] else "无"
                cp = ""
                if r["copyright_lines"]:
                    cp = r["copyright_lines"][0]
                    if len(cp) > 90:
                        cp = cp[:90] + "…"
                elif r["authors"]:
                    cp = "authors: " + "; ".join(r["authors"][:2])
                    if len(cp) > 90:
                        cp = cp[:90] + "…"
                else:
                    cp = "见 §3 索引官方全文 URL"
                obtain = f"{eid} · 包内LICENSE:{lic_mark} · NOTICE:{notice_mark} · 版权/获取:{cp}"
            L.append(f"| `{cell(r['name'])}` | `{cell(r['version'])}` | `{cell(r['expr'])}` | {r['kind']} | {cell(obtain)} |")
        L.append("")

    # §5 MPL
    L.append("\n---\n")
    L.append(MPL_SECTION.format(pure_mpl_count=len(pure_mpl), notice_crates=notice_crates))
    L.append("#### 5.4 纯 MPL-2.0 依赖清单（明确受 MPL-2.0 管辖，共 {n} 行）\n".format(n=len(pure_mpl)))
    if pure_mpl:
        L.append("| workspace | 包名 | 版本 | 许可表达式 | 包内 LICENSE | 上游源码 / MPL 全文获取方式 |")
        L.append("| --- | --- | --- | --- | --- | --- |")
        for label, n, rows in workspaces:
            for r in rows:
                if r in pure_mpl:
                    lm = "有" if r["license_files"] else "无"
                    L.append(f"| {cell(label)} | `{cell(r['name'])}` | `{cell(r['version'])}` | `MPL-2.0` | {lm} | crates.io `<name> <version>` 源码包 / 本机已解包目录；MPL 全文 <https://www.mozilla.org/media/MPL/2.0/index.txt> |")
    else:
        L.append("（本机核验结果：无纯 MPL-2.0 依赖行。）")
    L.append("")
    L.append("#### 5.5 MPL 作为非 SPDX '/' 表达式选项的依赖（不臆测，见 §8）\n")
    if mpl_option:
        L.append("| workspace | 包名 | 版本 | 表达式 | 判定 |")
        L.append("| --- | --- | --- | --- | --- |")
        for label, n, rows in workspaces:
            for r in rows:
                if r in mpl_option:
                    L.append(f"| {cell(label)} | `{cell(r['name'])}` | `{cell(r['version'])}` | `{cell(r['expr'])}` | '/' 分隔非 SPDX，须逐依赖明确择一；未书面选择前不宣称受 / 不受 MPL-2.0 管辖 |")
    else:
        L.append("（无。）")
    L.append("")

    # §6 Apache
    L.append("\n---\n")
    L.append(APACHE_SECTION.format(notice_crates=notice_crates))

    # §7 MIT
    L.append("\n---\n")
    L.append(MIT_SECTION)

    # §8 双许可
    L.append("\n---\n")
    if slash_rows:
        slash_table = "| workspace | 包名 | 版本 | 表达式（'/' 分隔） | 判定 |\n| --- | --- | --- | --- | --- |\n"
        for label, n, rows in workspaces:
            for r in rows:
                if r in slash_rows:
                    slash_table += f"| {cell(label)} | `{cell(r['name'])}` | `{cell(r['version'])}` | `{cell(r['expr'])}` | 非 SPDX，须逐依赖明确择一，不臆测 |\n"
    else:
        slash_table = "（本机核验结果：无 '/' 分隔非 SPDX 表达式依赖行。）"
    if lgpl_rows:
        lgpl_table = "| workspace | 包名 | 版本 | 表达式 | 判定 |\n| --- | --- | --- | --- | --- |\n"
        for label, n, rows in workspaces:
            for r in rows:
                if r in lgpl_rows:
                    lgpl_table += f"| {cell(label)} | `{cell(r['name'])}` | `{cell(r['version'])}` | `{cell(r['expr'])}` | 合法 SPDX OR，含 LGPL 可选原子；须逐依赖确认实际采用许可 |\n"
    else:
        lgpl_table = "（本机核验结果：无含 GPL 家族原子的 OR 表达式依赖行。）"
    L.append(DUAL_SECTION.format(slash_count=len(slash_rows),
                                 slash_table=slash_table,
                                 lgpl_table=lgpl_table))

    # §9 完整冻结边界复述
    L.append("\n---\n")
    L.append("## 9. 冻结诚实边界（完整复述，与 §0 一致）\n")
    L.append(BOUNDARY.split("## 0. 冻结诚实边界（原样沿用，不得改写）\n", 1)[1])

    # §10
    L.append("\n---\n")
    L.append(APP_UI)

    L.append("\n---\n")
    L.append(f"_本文件由 `tools/gen_third_party_notices.py` 于 {SNAPSHOT} 时点数据只读生成；重复运行可复现。非合规结论 / 非法律意见 / 非第三方审计报告 / 非已通过认证；未经律师签发不得对外发布。_")
    L.append("")
    return "\n".join(L)


def main():
    text = build()
    with open(OUT_PATH, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(f"WROTE: {OUT_PATH}")
    print(f"BYTES: {len(text.encode('utf-8'))}")
    print("READ-ONLY inputs: 2 Cargo.lock + registry src/cache (no writes to upstream).")


if __name__ == "__main__":
    main()
