import io, re, tarfile, urllib.request
url = "https://static.crates.io/crates/argon2/argon2-0.6.0.crate"
with urllib.request.urlopen(url, timeout=60) as r: data = r.read()
tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
out = []
for m in tf.getmembers():
    if not m.isfile(): continue
    if not m.name.endswith(("src/params.rs", "src/lib.rs")): continue
    b = tf.extractfile(m).read().decode("utf-8", "replace")
    lines = b.splitlines()
    out.append("=== " + m.name + " ===")
    for i, l in enumerate(lines):
        if re.search(r"pub fn new\(|pub enum Algorithm|pub enum Version|Argon2id|V0x13|pub fn new_with_secret|impl Params\b|fn block_count|output_len", l):
            out.append("--- L%d ---" % (i + 1))
            out.extend("  " + x for x in lines[max(0, i - 1):i + 12])
with open(r"D:\Projects\dsh\mylock\tools\params-dump.txt", "w", encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote", len(out), "lines")
