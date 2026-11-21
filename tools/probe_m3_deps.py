"""一次性探针：M3 依赖钉死版本 + dav-server 0.11.0 的 feature gate / fakels / memfs 树 API。非产品代码。"""
import io, json, re, tarfile, urllib.request

OUT = r"D:\Projects\dsh\mylock\tools\m3-deps-dump.txt"
lines = []

# 1) crates.io max_stable_version
crates = ["hyper", "hyper-util", "tokio", "bytes", "http", "http-body-util",
          "futures-util", "serde", "serde_json", "url", "tauri", "tauri-build"]
lines.append("========== crates.io max_stable_version (2026-11-14) ==========")
for c in crates:
    try:
        d = json.loads(urllib.request.urlopen(
            f"https://crates.io/api/v1/crates/{c}", timeout=30).read().decode())
        lines.append(f"{c:20s} {d['crate']['max_stable_version']:12s} license={d['crate'].get('license')}")
    except Exception as ex:
        lines.append(f"{c:20s} ERROR {ex}")

# 2) dav-server lib.rs module gates + fakels
raw = urllib.request.urlopen(
    "https://static.crates.io/crates/dav-server/dav-server-0.11.0.crate", timeout=60).read()
tf = tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz")
lib = tf.extractfile("dav-server-0.11.0/src/lib.rs").read().decode("utf-8", "replace")
lines.append("\n========== dav-server 0.11.0 lib.rs pub mod / cfg gates ==========")
for m in re.finditer(r"(?m)^.*(?:pub mod|cfg\(feature).*$", lib):
    t = m.group(0).strip()
    if len(t) < 220:
        lines.append(t)

lines.append("\n========== fakels.rs (first 45 lines) ==========")
fak = tf.extractfile("dav-server-0.11.0/src/fakels.rs").read().decode("utf-8", "replace")
lines.append("\n".join(fak.splitlines()[:45]))

lines.append("\n========== memfs.rs DavFile / DavMetaData / DavDirEntry impls (first 30 lines each) ==========")
memfs = tf.extractfile("dav-server-0.11.0/src/memfs.rs").read().decode("utf-8", "replace")
for kw in ["impl DavFile for", "impl DavMetaData for", "impl DavDirEntry for", "impl DavFileSystem for", "struct MemFs", "enum MemFsNode", "struct MemFsFile"]:
    i = memfs.find(kw)
    if i >= 0:
        lines.append("---- " + kw + " ----")
        lines.append("\n".join(memfs.splitlines()[memfs[:i].count("\n"):memfs[:i].count("\n") + 26]))

lines.append("\n========== fs.rs DavFileSystem trait full (up to GuardedFileSystem) ==========")
fs = tf.extractfile("dav-server-0.11.0/src/fs.rs").read().decode("utf-8", "replace")
i = fs.find("pub trait DavFileSystem")
j = fs.find("pub trait GuardedFileSystem")
lines.append(fs[i:j][:9000])

with open(OUT, "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
print("written", OUT)
