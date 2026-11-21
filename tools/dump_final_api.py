import io, re, tarfile, urllib.request

CRATES = {
    "argon2": ("argon2-0.6.0.crate", ["argon2-0.6.0/src/lib.rs", "argon2-0.6.0/src/params.rs"]),
    "aes-kw": ("aes-kw-0.3.1.crate", ["aes-kw-0.3.1/src/lib.rs"]),
    "chacha20poly1305": ("chacha20poly1305-0.11.0.crate", ["chacha20poly1305-0.11.0/src/lib.rs"]),
    "hkdf": ("hkdf-0.13.0.crate", ["hkdf-0.13.0/src/lib.rs"]),
    "data-encoding": ("data-encoding-2.11.1.crate", ["data-encoding-2.11.1/src/lib.rs"]),
}
NEEDLES = [
    r"pub fn hash_password_into\b", r"pub fn new_with_secret", r"pub fn new\(algorithm",
    r"pub const BASE32", r"BASE32_NOPAD", r"pub fn from_prk", r"pub fn expand\b",
    r"pub type KwAes256", r"pub type XChaCha20Poly1305", r"impl.*KeyInit",
    r"pub use.*KeyInit", r"pub use.*Aead", r"pub struct Params", r"pub fn new\(m_cost",
]
out = []
for name, (fn, files) in CRATES.items():
    url = "https://static.crates.io/crates/%s/%s" % (name, fn)
    with urllib.request.urlopen(url, timeout=60) as r:
        data = r.read()
    tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
    out.append("=" * 70)
    out.append("### " + name)
    for m in tf.getmembers():
        if not m.isfile():
            continue
        if not any(m.name.endswith(f) for f in files):
            continue
        b = tf.extractfile(m).read().decode("utf-8", "replace")
        lines = b.splitlines()
        out.append("-- file: " + m.name)
        for i, l in enumerate(lines):
            if any(re.search(n, l) for n in NEEDLES):
                out.append("  --- L%d ---" % (i + 1))
                out.extend("  " + x for x in lines[max(0, i - 2):i + 10])
with open(r"D:\Projects\dsh\mylock\tools\final-api-dump.txt", "w", encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote", len(out), "lines")
