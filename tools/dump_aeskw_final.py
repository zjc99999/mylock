import io, tarfile, urllib.request

def fetch(name, ver):
    url = "https://static.crates.io/crates/%s/%s-%s.crate" % (name, name, ver)
    with urllib.request.urlopen(url, timeout=60) as r: data = r.read()
    return tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")

out = []
tf = fetch("aes-kw", "0.3.1")
for m in tf.getmembers():
    if m.name.endswith("src/lib.rs") or m.name.endswith("src/kw.rs"):
        lines = tf.extractfile(m).read().decode("utf-8","replace").splitlines()
        out.append("=== %s (full, %d lines) ===" % (m.name, len(lines)))
        for i,l in enumerate(lines):
            out.append("L%d: %s" % (i+1,l))
# crypto-common / cipher: find blanket KeyInit for InnerInit
for name,ver in [("cipher","0.5.2"),("crypto-common","0.2.2")]:
    try:
        tf = fetch(name,ver)
    except Exception as e:
        out.append("skip %s: %r" % (name,e)); continue
    for m in tf.getmembers():
        if m.name.endswith(".rs"):
            b = tf.extractfile(m).read().decode("utf-8","replace")
            if "InnerInit" in b and "KeyInit" in b:
                out.append("=== %s ===" % m.name)
                for i,l in enumerate(b.splitlines()):
                    if "InnerInit" in l or "KeyInit for" in l or "new_from_slice" in l:
                        out.append("L%d: %s" % (i+1,l))
with open(r"D:\Projects\dsh\mylock\tools\aeskw-final-dump.txt","w",encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote", len(out), "lines")
