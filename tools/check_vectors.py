#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
G4 向量契约完整性校验器（跨平台门禁，Windows/macOS/Linux 通用）。

用途
----
`poc/test-vectors.json` 是 vault-format-spec.md v1.1.1 §9 冻结的**唯一逐字节契约**。
`tests/vectors/test-vectors.json` 是它的归档副本（CI 与各端实现只读这一份）。
本脚本在每次 CI 构建、每次本地发布前运行，判定：

  1. 两份文件 SHA-256 都等于冻结常量 VECTOR_SHA256；
  2. 两份文件字节完全相同（防「只改了一份」造成的分叉）；
  3. `tests/vectors/SHA256SUMS` 两条记录与实测一致；
  4. **EOL 未被归一化**：原文件是 CRLF 190 行、无裸 LF。这是真实跨平台风险——
     Git 默认在 Linux/macOS 上会把 CRLF 转 LF，字节一变 SHA-256 就变，派生值
     全部失效。`.gitattributes` 已把两个副本标为 `-text`，本检查是第二道防线；
  5. JSON 结构完整（顶层键、`_meta.byte_conventions`、30 个 derived_outputs、
     13 条 negative_vectors、summary 标记）。

判定原则
--------
任何一项不满足 → 非零退出并打印 `FAIL: ...`。**不得**自动修复、不得静默重生锚。
要变更向量（例如升 `format` 版本），必须人工同时更新：
  `poc/test-vectors.json`、`tests/vectors/test-vectors.json`、
  `tests/vectors/SHA256SUMS`、`tools/check_vectors.py` 的 VECTOR_SHA256、
  `core/crates/mylock-core/tests/conformance.rs` 的 EXPECTED_SHA256，
  并在变更记录里写明原因；这条「多点同时改」的摩擦力是刻意的。

运行：  python3 tools/check_vectors.py        （必须从仓库根目录运行）
        & 'C:\\Python312\\python.exe' tools\\check_vectors.py
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

# 冻结锚：poc/test-vectors.json 的 SHA-256（2026-11-14 实测，13499 B）。
# 与 tests/vectors/SHA256SUMS、core/crates/mylock-core/tests/conformance.rs 保持一致。
VECTOR_SHA256 = "07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939"

# 冻结的 EOL 形态：CRLF 行数、LF 总行数必须相等且等于该值（即无裸 LF）。
FROZEN_CRLF_LINES = 190

REPO_REL = {
    "poc": Path("poc") / "test-vectors.json",
    "archive": Path("tests") / "vectors" / "test-vectors.json",
    "sums": Path("tests") / "vectors" / "SHA256SUMS",
}

TOP_LEVEL_KEYS = {
    "_meta", "fixed_inputs", "derived_outputs", "verify_phase1_json",
    "verify_phase2", "export", "negative_vectors", "summary",
}
BYTE_CONVENTION_KEYS = {
    "integers", "jti", "object_id/manifest.id", "HKDF info",
    "chunk AAD", "manifest AAD", "two secrets",
}


def fail(msg: str) -> "NoReturn":
    print(f"FAIL: {msg}")
    raise SystemExit(1)


def find_root() -> Path:
    """从 cwd 向上找同时含 core/Cargo.toml 与 tests/vectors/ 的仓库根。

    `MYLOCK_VECTOR_ROOT` 可显式覆盖（用于「门禁自检」：在临时目录里造一个被篡改的
    最小仓库树，验证本校验器确实会 FAIL，而不是只测好路径）。
    """
    override = __import__("os").environ.get("MYLOCK_VECTOR_ROOT")
    if override:
        return Path(override).resolve()
    p = Path.cwd().resolve()
    for cand in (p, *p.parents):
        if (cand / "core" / "Cargo.toml").is_file() and (
            cand / "tests" / "vectors" / "test-vectors.json"
        ).is_file():
            return cand
    fail("repo root not found (run from the repository root)")


def sha256(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def main() -> int:
    root = find_root()
    paths = {k: root / v for k, v in REPO_REL.items()}

    for k, p in paths.items():
        if not p.is_file():
            fail(f"missing {k}: {p}")

    raw = {k: paths[k].read_bytes() for k in ("poc", "archive")}
    digests = {k: sha256(v) for k, v in raw.items()}

    # 1) 冻结锚
    for k in ("poc", "archive"):
        if digests[k] != VECTOR_SHA256:
            fail(
                f"{k} SHA-256 drifted: got {digests[k]}, "
                f"expected {VECTOR_SHA256} ({len(raw[k])} B). "
                "If the vector was intentionally changed, update ALL anchors "
                "(SHA256SUMS, check_vectors.py, conformance.rs) with a written reason."
            )

    # 2) 两份副本必须字节相同
    if raw["poc"] != raw["archive"]:
        fail("poc/ and tests/vectors/ copies diverged byte-for-byte")

    # 3) EOL 漂移（Linux/macOS 上 Git 自动转 LF 会命中这里）
    b = raw["archive"]
    crlf = b.count(b"\r\n")
    lf = b.count(b"\n")
    if crlf != FROZEN_CRLF_LINES or lf != FROZEN_CRLF_LINES:
        fail(
            f"EOL normalization detected: CRLF={crlf}, LF_total={lf}, "
            f"expected both == {FROZEN_CRLF_LINES}. "
            "Check .gitattributes (-text) and core.autocrlf."
        )
    if b"\x00" in b:
        fail("NUL byte in vector file (binary corruption)")

    # 4) SHA256SUMS 两条记录
    sums_lines = [
        ln.strip() for ln in paths["sums"].read_text(encoding="utf-8").splitlines()
        if ln.strip()
    ]
    if len(sums_lines) != 2:
        fail(f"SHA256SUMS must have exactly 2 records, got {len(sums_lines)}")
    parsed: dict[str, str] = {}
    for ln in sums_lines:
        parts = ln.split()
        if len(parts) != 2 or len(parts[0]) != 64:
            fail(f"malformed SHA256SUMS line: {ln!r}")
        parsed[parts[1].replace("\\", "/")] = parts[0].lower()
    for rel in ("poc/test-vectors.json", "tests/vectors/test-vectors.json"):
        if parsed.get(rel) != VECTOR_SHA256:
            fail(f"SHA256SUMS record for {rel} is missing or wrong")

    # 5) JSON 结构
    try:
        v = json.loads(b.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as e:
        fail(f"vector file is not valid UTF-8 JSON: {e}")
    missing = TOP_LEVEL_KEYS - set(v)
    if missing:
        fail(f"top-level keys missing: {sorted(missing)}")
    conv = v["_meta"].get("byte_conventions", {})
    mc = BYTE_CONVENTION_KEYS - set(conv)
    if mc:
        fail(f"_meta.byte_conventions missing: {sorted(mc)}")
    n_derived = len(v["derived_outputs"])
    if n_derived != 30:
        fail(f"derived_outputs count = {n_derived}, expected 30")
    n_neg = len(v["negative_vectors"])
    if n_neg != 13:
        fail(f"negative_vectors count = {n_neg}, expected 13")
    if not all(x.get("pass") is True for x in v["negative_vectors"]):
        fail("a negative_vectors entry is not marked pass:true in the contract")
    if v["summary"].get("negative_vectors_matched") != "13/13":
        fail("summary.negative_vectors_matched != '13/13'")
    fixed = v["fixed_inputs"]
    if fixed.get("kdf", {}).get("p") != 4:
        fail("fixed_inputs.kdf.p != 4 (parallelism=4 is a hard constraint, "
             "libsodium crypto_pwhash p=1 cannot reproduce these vectors)")
    if fixed.get("format") != 1:
        fail("fixed_inputs.format != 1 (a format bump requires new anchors + review)")

    print("OK: vector archive integrity")
    print(f"    sha256      = {VECTOR_SHA256}")
    print(f"    bytes       = {len(b)}")
    print(f"    eol         = CRLF x{crlf} (no bare LF), -text enforced")
    print(f"    derived     = {n_derived} outputs")
    print(f"    negative    = {n_neg} vectors")
    print("    copies      = poc/ == tests/vectors/ (byte-identical)")
    print("    SHA256SUMS  = 2/2 records verified")
    return 0


if __name__ == "__main__":
    sys.exit(main())
