import io, re, tarfile, urllib.request
url = "https://static.crates.io/crates/getrandom/getrandom-0.4.3.crate"
with urllib.request.urlopen(url, timeout=60) as r: data = r.read()
tf = tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
out = []
for m in tf.getmembers():
    if not m.isfile(): continue
    if m.name.endswith(("Cargo.toml", "src/lib.rs", "src/backends.rs")):
        out.append("=== " + m.name + " ===")
        b = tf.extractfile(m).read().decode("utf-8", "replace")
        lines = b.splitlines()
        for i, l in enumerate(lines):
            if re.search(r"^\[features\]|^default|sys_rng|wasm_js|linux_android|windows|cfg\(|pub fn fill|fn fill_uninit|mod backends|extern_impl|compile_error|no.*backend|select", l):
                out.append("  L%d: %s" % (i + 1, l))
with open(r"D:\Projects\dsh\mylock\tools\getrandom-dump.txt", "w", encoding="utf-8") as f:
    f.write("\n".join(out))
print("wrote", len(out), "lines")
