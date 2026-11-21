# -*- coding: utf-8 -*-
"""Download .crate tarballs from static.crates.io and print the API bits we
must not guess: Cargo.toml features + relevant pub fn/type signatures.
One-off M2 prep helper, not product code."""
import io, re, tarfile, urllib.request

CRATES = {
    "argon2": "0.6.0",
    "chacha20poly1305": "0.11.0",
    "aes-kw": "0.3.1",
    "hkdf": "0.13.0",
    "hmac": "0.13.0",
    "sha2": "0.11.1",
    "base64": "0.23.1",
    "data-encoding": "2.11.1",
    "zeroize": "1.9.1",
    "subtle": "2.6.1",
}

def fetch(name, ver):
    url = f"https://static.crates.io/crates/{name}/{name}-{ver}.crate"
    with urllib.request.urlopen(url, timeout=60) as r:
        return r.read()

def files(data):
    tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
    out = {}
    for m in tf.getmembers():
        if m.isfile() and m.name.endswith((".rs", "Cargo.toml")):
            out[m.name.split("/", 1)[-1]] = tf.extractfile(m).read().decode("utf-8", "replace")
    return out

for name, ver in CRATES.items():
    print("=" * 78)
    print(f"### {name} {ver}")
    try:
        fs = files(fetch(name, ver))
    except Exception as e:
        print("  FETCH FAIL", e)
        continue
    # features
    ct = fs.get("Cargo.toml", "")
    fm = re.search(r"\[features\](.*?)(\n\[|\Z)", ct, re.S)
    if fm:
        print("  -- [features] --")
        for line in fm.group(1).strip().splitlines():
            if line.strip() and not line.strip().startswith("#"):
                print("   ", line.strip())
    # signatures
    pats = {
        "argon2": [r"pub fn new\([^)]*\)", r"pub fn hash_password_into", r"impl Params", r"pub struct Params", r"pub fn new \(m_cost"],
        "chacha20poly1305": [r"pub struct XChaCha20Poly1305", r"pub use.*XChaCha", r"XChaCha20Poly1305"],
        "aes-kw": [r"pub struct Kek", r"pub fn wrap", r"pub fn unwrap", r"pub trait", r"pub type KekAes256"],
        "hkdf": [r"pub fn expand", r"pub struct Hkdf", r"impl.*Hkdf"],
    }
    lib = fs.get("src/lib.rs", "")
    print("  -- lib.rs head (first 60 lines) --")
    for ln in lib.splitlines()[:60]:
        print("   ", ln)
    if name == "aes-kw":
        for fn, body in fs.items():
            if fn.startswith("src/"):
                for kw in ("pub fn wrap", "pub fn unwrap", "pub struct Kek", "pub type Kek"):
                    for m in re.finditer(kw + r"[^\n]*", body):
                        print(f"   [{fn}]", m.group(0)[:120])
    if name == "argon2":
        for fn, body in fs.items():
            if fn.endswith("params.rs") or fn == "src/lib.rs":
                for m in re.finditer(r"pub fn new\([^)]*m_cost[^)]*\)|pub fn hash_password_into[^\n]*", body):
                    print(f"   [{fn}]", m.group(0)[:160])
