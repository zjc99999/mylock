"""一次性探针：解出 memfs.rs 全文 + hyper.rs example + davhandler builder/serve 段 + fs.rs OpenOptions/FsError。非产品代码。"""
import io, re, tarfile, urllib.request

OUT = r"D:\Projects\dsh\mylock\tools\dav-api-dump2.txt"
VER = "0.11.0"
data = urllib.request.urlopen(f"https://static.crates.io/crates/dav-server/dav-server-{VER}.crate", timeout=60).read()
tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
lines = []
for n in ["dav-server-0.11.0/src/memfs.rs", "dav-server-0.11.0/examples/hyper.rs", "dav-server-0.11.0/examples/axum.rs"]:
    lines.append("\n\n========== FILE %s ==========" % n)
    lines.append(tf.extractfile(n).read().decode("utf-8", "replace"))
# davhandler.rs: builder / handle / serve signatures and their surroundings
src = tf.extractfile("dav-server-0.11.0/src/davhandler.rs").read().decode("utf-8", "replace")
lines.append("\n\n========== davhandler.rs (public methods) ==========")
for mm in re.finditer(r"(?m)^[ \t]*pub fn [a-z_]+\b[^\{]*\{?", src):
    lines.append(mm.group(0).strip())
# fs.rs: FsError enum + OpenOptions struct
fs = tf.extractfile("dav-server-0.11.0/src/fs.rs").read().decode("utf-8", "replace")
for tname in ["FsError", "OpenOptions", "ReadDirMeta"]:
    mm = re.search(r"(?ms)^[ \t]*pub (enum|struct) %s\b[^\{]*\{.*?^\}" % tname, fs)
    if mm:
        lines.append("\n========== %s ==========" % tname)
        lines.append(mm.group(0))
# lib.rs: pub use / features
lib = tf.extractfile("dav-server-0.11.0/src/lib.rs").read().decode("utf-8", "replace")
lines.append("\n\n========== lib.rs pub use lines ==========")
for mm in re.finditer(r"(?m)^[ \t]*pub (use|mod) .*$", lib):
    lines.append(mm.group(0).strip())
with open(OUT, "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
print("written", OUT)
