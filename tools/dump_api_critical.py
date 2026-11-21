import io, re, tarfile, urllib.request

targets = {
    "getrandom-0.4.3.crate": ("getrandom", [r"pub fn fill\b", r"sys_rng", r"default ="]),
    "aes-kw-0.3.1.crate": ("aes-kw", [r"impl.*KeyInit", r"impl.*InnerInit", r"pub struct AesKw", r"fn new_from_slice"]),
    "base64-0.23.1.crate": ("base64", [r"pub const STANDARD", r"fn encode\b", r"fn decode\b", r"pub trait Engine"]),
    "hmac-0.13.0.crate": ("hmac", [r"fn new_from_slice", r"fn verify_slice", r"fn finalize"]),
}
out = []
for fn, (name, needles) in targets.items():
    url = "https://static.crates.io/crates/%s/%s" % (name, fn)
    with urllib.request.urlopen(url, timeout=60) as r: data = r.read()
    tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
    out.append("=" * 70); out.append("### " + name)
    for m in tf.getmembers():
        if not m.isfile() or not m.name.endswith((".rs", "Cargo.toml")): continue
        b = tf.extractfile(m).read().decode("utf-8", "replace")
        lines = b.splitlines()
        for i, l in enumerate(lines):
            if any(re.search(n, l) for n in needles):
                out.append("  --- %s L%d ---" % (m.name, i + 1))
                out.extend("  " + x for x in lines[max(0, i - 1):i + 8])
with open(r"D:\Projects\dsh\mylock\tools\api-critical-dump.txt", "w", encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote", len(out), "lines")
