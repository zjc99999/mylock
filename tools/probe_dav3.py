"""一次性探针：dav-server 0.11.0 的 Cargo.toml 依赖、GuardedFileSystem 泛型实现、DavHandler::handle 签名、fs.rs 头部 imports。非产品代码。"""
import io, re, tarfile, urllib.request

OUT = r"D:\Projects\dsh\mylock\tools\dav-api-dump3.txt"
VER = "0.11.0"
data = urllib.request.urlopen(f"https://static.crates.io/crates/dav-server/dav-server-{VER}.crate", timeout=60).read()
tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
lines = []
lines.append("========== Cargo.toml ==========")
lines.append(tf.extractfile("dav-server-0.11.0/Cargo.toml").read().decode("utf-8", "replace"))

lines.append("\n========== fs.rs header (first 60 lines) ==========")
fs = tf.extractfile("dav-server-0.11.0/src/fs.rs").read().decode("utf-8", "replace")
lines.append("\n".join(fs.splitlines()[:60]))

lines.append("\n========== GuardedFileSystem occurrences ==========")
for name in tf.getnames():
    if name.endswith(".rs"):
        s = tf.extractfile(name).read().decode("utf-8", "replace")
        for mm in re.finditer(r"(?m)^.*GuardedFileSystem.*$", s):
            lines.append("[%s] %s" % (name, mm.group(0).strip()))

lines.append("\n========== DavHandler::handle signature ==========")
dh = tf.extractfile("dav-server-0.11.0/src/davhandler.rs").read().decode("utf-8", "replace")
for mm in re.finditer(r"(?ms)pub fn handle[^\{]*\{", dh):
    lines.append(mm.group(0).strip())
lines.append("---- impl DavHandler block (first 90 lines around handle) ----")
idx = dh.find("pub fn handle")
lines.append(dh[max(0, idx-1200):idx+400])

lines.append("\n========== DavMethodSet / methods ==========")
for name in ["dav-server-0.11.0/src/util.rs", "dav-server-0.11.0/src/davhandler.rs"]:
    s = tf.extractfile(name).read().decode("utf-8", "replace")
    for mm in re.finditer(r"(?m)^.*(DavMethodSet|pub const HEAD|allow_method|methods\().*$", s):
        t = mm.group(0).strip()
        if len(t) < 200:
            lines.append("[%s] %s" % (name, t))

with open(OUT, "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
print("written", OUT)
