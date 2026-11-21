#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
门禁负向自检：在临时目录里造 4 种被篡改的最小仓库树，验证 check_vectors.py
确实会 FAIL。只测好路径的门禁等于没有门禁。

用法（从仓库根运行）：
    python3 tools/gate_selftest.py
    & 'C:\\Python312\\python.exe' tools\\gate_selftest.py
"""
from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "tools" / "check_vectors.py"
ARCHIVE = ROOT / "tests" / "vectors" / "test-vectors.json"
SUMS = ROOT / "tests" / "vectors" / "SHA256SUMS"
RAW = ARCHIVE.read_bytes()
SUMS_TEXT = SUMS.read_text(encoding="utf-8")
PY = sys.executable

CASES = [
    ("single-byte tamper", lambda b: b[:100] + bytes([b[100] ^ 1]) + b[101:]),
    ("CRLF -> LF normalization", lambda b: b.replace(b"\r\n", b"\n")),
    ("truncated archive", lambda b: b[:1000]),
    ("stale SHA256SUMS", None),  # 专门改 sums 文件
]


def run(tmp: Path, name: str, mutate) -> bool:
    """返回 True 表示该用例按预期被门禁拒绝（FAIL + 非零退出）。"""
    work = tmp / name.replace(" ", "_").replace("->", "to")
    (work / "tests" / "vectors").mkdir(parents=True)
    (work / "poc").mkdir()
    (work / "core").mkdir()
    (work / "core" / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    (work / "poc" / "test-vectors.json").write_bytes(RAW)
    if mutate is None:
        (work / "tests" / "vectors" / "test-vectors.json").write_bytes(RAW)
        bad = SUMS_TEXT.replace(
            "07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939",
            "0" * 64,
        )
        (work / "tests" / "vectors" / "SHA256SUMS").write_text(bad, encoding="utf-8")
    else:
        (work / "tests" / "vectors" / "test-vectors.json").write_bytes(mutate(RAW))
        (work / "tests" / "vectors" / "SHA256SUMS").write_text(SUMS_TEXT, encoding="utf-8")

    r = subprocess.run(
        [PY, str(CHECKER)],
        cwd=str(work),
        env={**__import__("os").environ, "MYLOCK_VECTOR_ROOT": str(work)},
        capture_output=True,
        text=True,
    )
    out = (r.stdout + r.stderr).strip().replace("\n", " | ")
    rejected = r.returncode != 0 and "FAIL:" in out
    print(f"  [{'PASS' if rejected else 'BROKEN'}] {name}")
    print(f"         exit={r.returncode}  {out[:220]}")
    return rejected


def main() -> int:
    import tempfile

    tmp = Path(tempfile.mkdtemp(prefix="mylock-gate-selftest-"))
    try:
        results = [run(tmp, n, m) for n, m in CASES]
        # 好路径也要确认：真实仓库必须 OK
        r = subprocess.run([PY, str(CHECKER)], cwd=str(ROOT), capture_output=True, text=True)
        good = r.returncode == 0 and "OK: vector archive integrity" in r.stdout
        print(f"  [{'PASS' if good else 'BROKEN'}] untampered repo passes")
        ok = all(results) and good
        print(f"\nGATE_SELFTEST={'PASS' if ok else 'FAIL'}")
        return 0 if ok else 1
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
