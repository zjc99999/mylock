import io, tarfile, urllib.request

def fetch(name, ver):
    url = "https://static.crates.io/crates/%s/%s-%s.crate" % (name, name, ver)
    with urllib.request.urlopen(url, timeout=60) as r: data = r.read()
    return tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")

out = []
for name, ver in [("aead","0.6.1"), ("crypto-common","0.2.2"), ("hybrid-array","0.4.5")]:
    try:
        tf = fetch(name,ver)
    except Exception as e:
        out.append("skip %s %s: %r" % (name,ver,e)); continue
    for m in tf.getmembers():
        if not m.name.endswith(".rs"): continue
        b = tf.extractfile(m).read().decode("utf-8","replace")
        lines = b.splitlines()
        for i,l in enumerate(lines):
            if ("pub use" in l and ("array" in l.lower() or "Array" in l)) or \
               "fn from_slice" in l or "impl<T, N" in l and "From<[T" in "\n".join(lines[i:i+3]) or \
               "pub type Array" in l or "pub use hybrid" in l:
                out.append("=== %s / %s L%d ===" % (name, m.name, i+1))
                out.extend("L%d: %s" % (j+1, lines[j]) for j in range(max(0,i-1), min(i+6,len(lines))))
# also chacha20poly1305 lib.rs top 40
tf = fetch("chacha20poly1305","0.11.0")
for m in tf.getmembers():
    if m.name.endswith("src/lib.rs"):
        lines = tf.extractfile(m).read().decode("utf-8","replace").splitlines()
        out.append("=== chacha20poly1305 lib.rs 1-40 ===")
        out.extend("L%d: %s" % (i+1,l) for i,l in enumerate(lines[:40]))
with open(r"D:\Projects\dsh\mylock\tools\array-api-dump.txt","w",encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote",len(out),"lines")
