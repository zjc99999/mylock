"""一次性探针：dav-server 0.11.0 的 DavHandler::handle 精确签名、hyper example 全文、GuardedFileSystem trait 定义、MemFs 的 do_open/create_dir/remove_file/rename 实现。非产品代码。"""
import io, re, tarfile, urllib.request

OUT = r"D:\Projects\dsh\mylock\tools\dav-api-dump4.txt"
VER = "0.11.0"
data = urllib.request.urlopen(f"https://static.crates.io/crates/dav-server/dav-server-{VER}.crate", timeout=60).read()
tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")


def src(name):
    return tf.extractfile("dav-server-0.11.0/" + name).read().decode("utf-8", "replace")


lines = []
lines.append("========== examples/hyper.rs (full) ==========")
lines.append(src("examples/hyper.rs"))

lines.append("\n========== examples/auth.rs (full) ==========")
lines.append(src("examples/auth.rs"))

lines.append("\n========== GuardedFileSystem trait definition (fs.rs) ==========")
fs = src("src/fs.rs")
i = fs.find("pub trait GuardedFileSystem")
lines.append(fs[i - 400:i + 900])

lines.append("\n========== DavHandler impl + handle (davhandler.rs) ==========")
dh = src("src/davhandler.rs")
for mm in re.finditer(r"(?m)^.*(pub fn handle|async fn handle|pub async fn handle|impl<C.*DavHandler|pub struct DavHandler).*$", dh):
    lines.append("[match] " + mm.group(0).strip())
i = dh.find("pub fn handle")
if i < 0:
    i = dh.find("fn handle")
lines.append("---- around handle ----")
lines.append(dh[max(0, i - 600):i + 900])

lines.append("\n========== memfs.rs do_open / create_dir / remove_file / rename (signatures) ==========")
memfs = src("src/memfs.rs")
for kw in ["fn do_open", "fn create_dir", "fn remove_file", "fn remove_dir", "fn rename", "fn copy", "fn metadata", "fn open<", "fn read_dir<"]:
    for mm in re.finditer(r"(?m)^.*" + re.escape(kw) + r".*$", memfs):
        lines.append("[memfs] " + mm.group(0).strip())

lines.append("\n========== DavPath methods (davpath.rs) ==========")
dp = src("src/davpath.rs")
for mm in re.finditer(r"(?m)^\s*pub fn .*$", dp):
    lines.append("[DavPath] " + mm.group(0).strip())

with open(OUT, "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
print("written", OUT)
