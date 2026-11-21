"""一次性探针：下载 dav-server crate 源码，抽取 DavFileSystem/DavFile/DavMetaData/DavDirEntry trait 定义。非产品代码。"""
import io, json, re, tarfile, urllib.request, os

OUT = r"D:\Projects\dsh\mylock\tools\dav-api-dump.txt"
VER = "0.11.0"
url = f"https://static.crates.io/crates/dav-server/dav-server-{VER}.crate"
print("fetching", url)
data = urllib.request.urlopen(url, timeout=60).read()
print("bytes", len(data))
tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
names = [n for n in tf.getnames() if n.endswith(".rs")]
lines = ["=== dav-server %s files ===" % VER]
lines += names
for n in names:
    src = tf.extractfile(n).read().decode("utf-8", "replace")
    for m in re.finditer(r"(?m)^[ \t]*(pub )?(async )?(trait|struct|enum|type) (DavFileSystem|DavFile|DavMetaData|DavDirEntry|DavHandler|DavConfig|Fakels|MemFs|LocalFs|FsError|DavPath|OpenOptions|ReadDirMeta|FsResult|FsFuture|FsStream)\b[^\{;]*", src):
        lines.append("--- %s :: %s ---" % (n, m.group(0).strip()))
    # trait bodies for the core traits
    for tname in ["DavFileSystem", "DavFile", "DavMetaData", "DavDirEntry"]:
        mm = re.search(r"(?ms)^[ \t]*(?:pub )?trait %s\b[^\{]*\{(.*?)^\}" % tname, src)
        if mm:
            lines.append("\n===== TRAIT %s in %s =====" % (tname, n))
            lines.append("trait %s {" % tname)
            lines.append(mm.group(1).rstrip())
            lines.append("}")
    # DavHandler builder / serve API
    for kw in ["pub fn new", "pub fn build", "pub fn with", "impl DavHandler", "pub fn handle", "pub fn memfs", "pub fn localfs"]:
        for mm in re.finditer(r"(?m)^[ \t]*.*%s.*$" % re.escape(kw), src):
            s = mm.group(0).strip()
            if len(s) < 240:
                lines.append("[%s] %s :: %s" % (kw, n, s))
with open(OUT, "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
print("written", OUT, "lines", len(lines))
