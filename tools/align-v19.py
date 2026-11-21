# -*- coding: utf-8 -*-
"""One-shot doc version alignment: v1.8 -> v1.9 (M5 zero-cost closeout).

Two kinds of occurrences, kept distinct on purpose:
  (A) "current upstream reference" -> bump v1.8 to v1.9 (with an M5 note)
  (B) "historical version-chain note" -> keep v1.8, append "M5 后升 v1.9"
Every replacement asserts its expected hit count so a silent miss fails loudly.
Read/write UTF-8, preserve original newlines.
"""
import io, sys

BASE = r"D:\Projects\dsh\mylock"

# (relative path, [(old, new, expected_count), ...])
PLAN = {
    r"m3-report.md": [
        ("v1.8（M3 当时为 v1.4", "v1.9（M3 当时为 v1.4", 1),
        ("M4 第 4 点后升 v1.8，见 [m4-step3-report.md]",
         "M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step3-report.md]", 1),
    ],
    r"m3-architecture.md": [
        ("v1.8（M3 当时为 v1.4", "v1.9（M3 当时为 v1.4", 1),
        ("M4 第 4 点后升 v1.8，见 [m4-step3-report.md]",
         "M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step3-report.md]", 1),
    ],
    r"m2-report.md": [
        ("M4 第 4 点后升 v1.8，见 [m3-report.md]",
         "M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m3-report.md]", 2),
    ],
    r"m4-step3-report.md": [
        (") v1.8（M4 第 3 点当时为 v1.7", ") v1.9（M4 第 3 点当时为 v1.7", 1),
        ("M4 第 4 点后升 v1.8，见 [m4-step4-report.md]",
         "M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step4-report.md]", 1),
        ("**v1.8**（M4 第 3 点当时为 v1.7；M4 第 4 点后升 v1.8）",
         "**v1.9**（M4 第 3 点当时为 v1.7；M4 第 4 点后升 v1.8、M5 后升 v1.9）", 1),
    ],
    r"m4-step2-report.md": [
        ("M4 第 4 点后升 v1.8，见 [m4-step4-report.md]",
         "M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step4-report.md]", 2),
    ],
    r"m4-report.md": [
        ("**v1.8**（M4 第 1 点当时为 v1.5", "**v1.9**（M4 第 1 点当时为 v1.5", 1),
        ("v1.8（M4 第 1 点当时为 v1.5", "v1.9（M4 第 1 点当时为 v1.5", 1),
        ("M4 第 4 点后升 v1.8", "M4 第 4 点后升 v1.8、M5 后升 v1.9", 3),
        ("M4 第 4 点后升 **v1.8**", "M4 第 4 点后升 **v1.8**、M5 后升 **v1.9**", 1),
    ],
    r"m4-step4-report.md": [
        (") v1.8 / [m4-step3-report.md](m4-step3-report.md) · 授权",
         ") v1.9（M4 第 4 点当时为 v1.8；M5 后升 v1.9，见 [m5-report.md](m5-report.md)）/ [m4-step3-report.md](m4-step3-report.md) · 授权", 1),
        ("**v1.8**（表头 / 配套文件 / v1.8 变更块",
         "**v1.9**（M4 第 4 点当时为 v1.8；M5 后升 v1.9。表头 / 配套文件 / v1.8 变更块", 1),
    ],
    r"desktop\SIGNING.md": [
        ("（v1.8）", "（v1.9）", 1),
    ],
    r"crypto-review-m1.md": [
        ("v1.8 四处版本号", "v1.9 四处版本号", 1),
    ],
    r"vault-format-spec.md": [
        ("v1.8 四处版本号", "v1.9 四处版本号", 2),
        ("M4 第 4 点后升 v1.8）", "M4 第 4 点后升 v1.8，M5 后升 v1.9）", 1),
    ],
    r"poc-report-m1.md": [
        ("M4 第 4 点后升 v1.8，见 [m2-report.md]",
         "M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m2-report.md]", 1),
    ],
    r"tests\vectors\CONTRACT.md": [
        ("产品分析与概要设计.md](../../产品分析与概要设计.md) v1.8、",
         "产品分析与概要设计.md](../../产品分析与概要设计.md) v1.9、", 1),
    ],
}

import os
errors = []
touched = []
for rel, pairs in PLAN.items():
    path = os.path.join(BASE, rel)
    with io.open(path, "r", encoding="utf-8", newline="") as f:
        txt = f.read()
    orig = txt
    for old, new, exp in pairs:
        n = txt.count(old)
        if n != exp:
            errors.append("%s | expected %d got %d for %r" % (rel, exp, n, old[:50]))
        txt = txt.replace(old, new)
    if txt != orig:
        with io.open(path, "w", encoding="utf-8", newline="") as f:
            f.write(txt)
        touched.append(rel)

print("TOUCHED=%d" % len(touched))
for t in touched:
    print("  " + t)
if errors:
    print("ERRORS=%d" % len(errors))
    for e in errors:
        print("  " + e)
    sys.exit(1)
print("ALIGN_V19_OK")
