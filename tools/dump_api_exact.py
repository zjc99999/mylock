import io, re, tarfile, urllib.request

def fetch(name, ver):
    url = "https://static.crates.io/crates/%s/%s-%s.crate" % (name, name, ver)
    with urllib.request.urlopen(url, timeout=60) as r: data = r.read()
    return tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")

out = []
# aes-kw: full wrap_key / unwrap_key bodies
tf = fetch("aes-kw", "0.3.1")
for m in tf.getmembers():
    if m.name.endswith("src/kw.rs"):
        lines = tf.extractfile(m).read().decode("utf-8","replace").splitlines()
        out.append("=== aes-kw kw.rs 130-205 ===")
        out.extend("L%d: %s" % (i+1, l) for i, l in enumerate(lines[129:205], start=129))
# data-encoding: BASE32_NOPAD definition
tf = fetch("data-encoding", "2.11.1")
for m in tf.getmembers():
    if m.name.endswith("src/lib.rs"):
        lines = tf.extractfile(m).read().decode("utf-8","replace").splitlines()
        out.append("=== data-encoding BASE32_NOPAD occurrences ===")
        for i,l in enumerate(lines):
            if "BASE32_NOPAD" in l or "BASE32:" in l or "const BASE32 " in l:
                out.append("L%d: %s" % (i+1, l))
        # print the region around first definition
        for i,l in enumerate(lines):
            if l.startswith("pub const BASE32_NOPAD") or (l.startswith("pub const BASE32 ") and ":" in l):
                out.append("--- region L%d ---" % (i+1))
                out.extend("L%d: %s" % (j+1, lines[j]) for j in range(i, min(i+14,len(lines))))
                break
# hkdf from_prk / expand
tf = fetch("hkdf", "0.13.0")
for m in tf.getmembers():
    if m.name.endswith("src/lib.rs"):
        lines = tf.extractfile(m).read().decode("utf-8","replace").splitlines()
        out.append("=== hkdf lib.rs 140-215 ===")
        out.extend("L%d: %s" % (i+1, l) for i, l in enumerate(lines[139:215], start=139))
# getrandom: features + backend cfg
tf = fetch("getrandom", "0.4.3")
for m in tf.getmembers():
    if m.name.endswith("Cargo.toml"):
        b = tf.extractfile(m).read().decode("utf-8","replace")
        out.append("=== getrandom Cargo.toml [features] ===")
        lines = b.splitlines()
        inf=False
        for i,l in enumerate(lines):
            if l.startswith("[features]"): inf=True
            elif l.startswith("[") : inf=False
            if inf: out.append("L%d: %s" % (i+1,l))
    if m.name.endswith("src/lib.rs"):
        lines = tf.extractfile(m).read().decode("utf-8","replace").splitlines()
        out.append("=== getrandom lib.rs cfg/backends region (70-160) ===")
        out.extend("L%d: %s" % (i+1, l) for i, l in enumerate(lines[69:160], start=69))
with open(r"D:\Projects\dsh\mylock\tools\api-exact-dump.txt","w",encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote", len(out), "lines")
