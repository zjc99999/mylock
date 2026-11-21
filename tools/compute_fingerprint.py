#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
G4 指纹独立门禁（M4 第 1 点，CI 门禁 4）。

从归档的逐字节契约算出 G4 指纹，并**断言**它等于冻结常量。与
conformance.rs::this_platform_reproduces_g4_fingerprint（在 Rust 侧跑真实
derive_outputs）互为交叉验证：一个从契约 JSON 侧、一个从实现侧，都命中同一
冻结值。任何向量漂移、条目增减、键名变化都会让本门禁 FAIL（非零退出）。

变更向量须同步改这里的 EXPECTED_* 常量（见 tests/vectors/CONTRACT.md §5）。
"""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

# 冻结常量（与 conformance.rs / check_vectors.py / CONTRACT.md §1 一致）
EXPECTED_FINGERPRINT = "d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa"
EXPECTED_ENTRIES = 30
EXPECTED_INPUT_BYTES = 8254

ROOT = Path(__file__).resolve().parent.parent
VECTOR = ROOT / "tests" / "vectors" / "test-vectors.json"


def fail(msg: str) -> int:
    print(f"FAIL: {msg}")
    return 1


def main() -> int:
    if not VECTOR.is_file():
        return fail(f"missing {VECTOR}")
    v = json.loads(VECTOR.read_text(encoding="utf-8"))
    d = v.get("derived_outputs")
    if not isinstance(d, dict):
        return fail("derived_outputs is missing or not an object")
    s = "".join(f"{k}={d[k]}\n" for k in sorted(d))
    payload = s.encode("utf-8")
    fp = hashlib.sha256(payload).hexdigest()

    print(f"ENTRIES={len(d)}")
    print(f"INPUT_BYTES={len(payload)}")
    print(f"G4_FINGERPRINT={fp}")

    if len(d) != EXPECTED_ENTRIES:
        return fail(f"derived_outputs count drifted: got {len(d)}, expected {EXPECTED_ENTRIES}")
    if len(payload) != EXPECTED_INPUT_BYTES:
        return fail(f"fingerprint input bytes drifted: got {len(payload)}, expected {EXPECTED_INPUT_BYTES}")
    if fp != EXPECTED_FINGERPRINT:
        return fail(f"G4 fingerprint drifted: got {fp}, expected {EXPECTED_FINGERPRINT}. "
                    "If the vector was intentionally changed, update ALL frozen anchors "
                    "(see tests/vectors/CONTRACT.md §5) in the SAME commit with a reason.")

    print("OK: G4 fingerprint matches frozen contract")
    return 0


if __name__ == "__main__":
    sys.exit(main())
