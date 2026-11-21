# -*- coding: utf-8 -*-
"""Focused API dump — write to file to avoid BrokenPipe from Select-Object."""
import io, re, tarfile, urllib.request

CRATES = {
    "argon2": "0.6.0",
    "chacha20poly1305": "0.11.0",
    "aead": "0.6.0",
    "aes-kw": "0.3.1",
    "hkdf": "0.13.0",
    "hmac": "0.13.0",
    "sha2": "0.11.1",
    "base64": "0.23.1",
    "data-encoding": "2.11.1",
    "zeroize": "1.9.1",
}
OUT = []
def p(s=""): OUT.append(str(s))

def fetch(name, ver):
    url = f"https://static.crates.io/crates/{name}/{name}-{ver}.crate"
    with urllib.request.urlopen(url, timeout=60) as r:
        return r.read()

def files(data):
    tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
    out = {}
    for m in tf.getmembers():
        if m.isfile() and m.name.endswith(".rs"):
            out[m.name.split("/", 1)[-1]] = tf.extractfile(m).read().decode("utf-8", "replace")
    return out

def grab(name, ver, needles, maxhits=6):
    p("=" * 78); p(f"### {name} {ver}")
    try:
        fs = files(fetch(name, ver))
    except Exception as e:
        p("  FETCH FAIL", e); return
    for needle in needles:
        hits = 0
        for fn, body in fs.items():
            for m in re.finditer(needle, body):
                ln = body.count("\n", 0, m.start()) + 1
                ctx = "\n".join(body.splitlines()[max(0, ln-2):ln+8])
                p(f"  -- [{fn}:{ln}] --"); p(ctx)
                hits += 1
                if hits >= maxhits: break
            if hits >= maxhits: break
        if hits == 0: p(f"  (no hit for {needle})")

grab("argon2", "0.6.0", [
    r"pub fn new\(&?self\)|pub const fn new\(",
    r"pub fn hash_password_into\(",
    r"pub fn hash_password_into_with_memory\(",
    r"pub struct Params\b",
    r"pub enum Algorithm",
    r"pub enum Version",
])
grab("chacha20poly1305", "0.11.0", [
    r"pub type XChaCha20Poly1305",
    r"impl KeyInit for XChaCha|pub struct XChaCha20Poly1305",
    r"pub use.*XChaCha",
])
grab("aead", "0.6.0", [
    r"pub trait Aead\b",
    r"fn encrypt\(",
    r"fn decrypt\(",
    r"pub struct Payload",
])
grab("aes-kw", "0.3.1", [
    r"impl.*AesKw<",
    r"pub fn wrap_key\(",
    r"pub fn unwrap_key\(",
    r"impl.*KeyInit.*AesKw",
])
grab("hkdf", "0.13.0", [
    r"pub fn from_prk\(",
    r"pub fn expand\(",
    r"pub struct Hkdf\b",
])
grab("hmac", "0.13.0", [
    r"pub type Hmac\b",
    r"pub struct Hmac\b",
    r"pub struct SimpleHmac",
])
grab("base64", "0.23.1", [
    r"pub const STANDARD\b",
    r"pub trait Engine",
    r"pub fn encode\(",
])
grab("data-encoding", "2.11.1", [
    r"pub const BASE32\b",
    r"pub fn encode\(",
    r"pub fn decode\(",
])

open("D:/Projects/dsh/mylock/tools/crate-api-dump.txt", "w", encoding="utf-8").write("\n".join(OUT))
print("wrote crate-api-dump.txt", len(OUT), "lines")
