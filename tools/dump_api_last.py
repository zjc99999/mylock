import io, re, tarfile, urllib.request

CRATES = {
    "getrandom": "getrandom-0.4.3.crate",
    "chacha20poly1305": "chacha20poly1305-0.11.0.crate",
    "aead": "aead-0.6.1.crate",
    "base64": "base64-0.23.1.crate",
    "data-encoding": "data-encoding-2.11.1.crate",
    "hmac": "hmac-0.13.0.crate",
    "subtle": "subtle-2.6.1.crate",
}
NEEDLES = [
    r"pub fn fill\b", r"pub fn getrandom\b", r"pub use", r"pub type XNonce",
    r"pub type XChaCha20Poly1305", r"pub trait Aead\b", r"pub struct Payload",
    r"pub const STANDARD", r"pub trait Engine", r"pub const BASE32_NOPAD",
    r"pub fn encode\b", r"pub fn decode\b", r"trait ConstantTimeEq",
    r"pub use digest", r"pub trait Mac", r"fn new_from_slice",
]
out = []
for name, fn in CRATES.items():
    url = "https://static.crates.io/crates/%s/%s" % (name, fn)
    with urllib.request.urlopen(url, timeout=60) as r:
        data = r.read()
    tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
    out.append("=" * 70)
    out.append("### " + name)
    seen = set()
    for m in tf.getmembers():
        if not m.isfile() or not m.name.endswith("src/lib.rs"):
            continue
        b = tf.extractfile(m).read().decode("utf-8", "replace")
        lines = b.splitlines()
        for i, l in enumerate(lines):
            if any(re.search(n, l) for n in NEEDLES):
                key = (m.name, i // 30)
                if key in seen:
                    continue
                seen.add(key)
                out.append("  --- L%d ---" % (i + 1))
                out.extend("  " + x for x in lines[max(0, i - 1):i + 7])
with open(r"D:\Projects\dsh\mylock\tools\api-last-dump.txt", "w", encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote", len(out), "lines")
