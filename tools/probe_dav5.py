"""一次性探针：dav-server 0.11.0 DavConfig::filesystem / build_handler / DavFile / DavMetaData / OpenOptions 精确签名。非产品代码。"""
import io, re, tarfile, urllib.request

OUT = r"D:\Projects\dsh\mylock\tools\dav-api-dump5.txt"
raw = urllib.request.urlopen(
    "https://static.crates.io/crates/dav-server/dav-server-0.11.0.crate", timeout=60).read()
tf = tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz")


def src(name):
    return tf.extractfile("dav-server-0.11.0/" + name).read().decode("utf-8", "replace")


lines = []
dh = src("src/davhandler.rs")
lines.append("========== all `pub fn` in DavConfig impl ==========")
for mm in re.finditer(r"(?m)^\s*pub fn [a-z_]+[^\{]*\{", dh):
    lines.append(mm.group(0).strip())
lines.append("\n========== filesystem() context ==========")
for mm in re.finditer(r"pub fn filesystem", dh):
    i = mm.start()
    lines.append(dh[max(0, i-300):i+700])
lines.append("\n========== build_handler / new context ==========")
for kw in ["pub fn build_handler", "pub fn new() -> DavConfig", "impl<C"]:
    for mm in re.finditer(re.escape(kw), dh):
        i = mm.start()
        lines.append("---- " + kw + " ----")
        lines.append(dh[max(0,i-150):i+500])

fs = src("src/fs.rs")
lines.append("\n========== DavFile trait (full) ==========")
i = fs.find("pub trait DavFile")
j = fs.find("pub trait DavMetaData")
lines.append(fs[i:j])
lines.append("\n========== DavMetaData trait (full) ==========")
i = fs.find("pub trait DavMetaData")
j = fs.find("pub trait DavDirEntry")
lines.append(fs[i:j])
lines.append("\n========== DavDirEntry trait (full) ==========")
i = fs.find("pub trait DavDirEntry")
lines.append(fs[i:i+2200])
lines.append("\n========== OpenOptions struct ==========")
i = fs.find("pub struct OpenOptions")
lines.append(fs[i:i+1200])
lines.append("\n========== FsFuture / FsStream / DavProp type aliases ==========")
for mm in re.finditer(r"(?m)^pub type .*$", fs):
    lines.append(mm.group(0).strip())

with open(OUT, "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
print("written", OUT)
