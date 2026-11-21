import io, tarfile, urllib.request, re, sys

URL = "https://static.crates.io/crates/dav-server/dav-server-0.11.0.crate"
data = urllib.request.urlopen(URL, timeout=60).read()
tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
src = None
for m in tf.getmembers():
    if m.name.endswith("src/davpath.rs"):
        src = tf.extractfile(m).read().decode("utf-8", "replace")
        break
if src is None:
    print("davpath.rs NOT FOUND"); sys.exit(1)

lines = src.splitlines()
# Print as_bytes, new, from_uri, from_str, add_prefix, as_rel_ospath bodies with context
targets = ["pub fn as_bytes", "pub fn new(", "pub fn from_uri", "pub fn as_rel_ospath",
           "pub fn as_pathbuf", "fn from_str", "percent", "decode", "pub fn file_name",
           "pub fn parent", "pub fn as_url_string", "struct DavPath"]
out = []
for i, ln in enumerate(lines):
    if any(t in ln for t in targets):
        out.append("---- L%d ----" % (i+1))
        for j in range(max(0,i-2), min(len(lines), i+14)):
            out.append("%4d: %s" % (j+1, lines[j]))
open(r"D:\Projects\dsh\mylock\tools\davpath-dump.txt","w",encoding="utf-8").write("\n".join(out))
print("WROTE davpath-dump.txt", len(out), "lines")
