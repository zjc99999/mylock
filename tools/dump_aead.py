import io, re, tarfile, urllib.request
url="https://static.crates.io/crates/aead/aead-0.6.0.crate"
with urllib.request.urlopen(url, timeout=60) as r: data=r.read()
tf=tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
for m in tf.getmembers():
    if m.isfile() and m.name.endswith("src/lib.rs"):
        b=tf.extractfile(m).read().decode()
        lines=b.splitlines()
        for i,l in enumerate(lines):
            if re.search(r"fn encrypt|fn decrypt|pub trait Aead|type NonceSize|generate_nonce|fn new_from_slice|impl.*Aead for", l):
                print(f"--- L{i+1} ---")
                print("\n".join(lines[max(0,i-1):i+14]))
print("=== chacha20poly1305 lib.rs around ChaChaPoly1305 impl ===")
url2="https://static.crates.io/crates/chacha20poly1305/chacha20poly1305-0.11.0.crate"
with urllib.request.urlopen(url2, timeout=60) as r: d2=r.read()
tf2=tarfile.open(fileobj=io.BytesIO(d2), mode="r:gz")
for m in tf2.getmembers():
    if m.isfile() and m.name.endswith("src/lib.rs"):
        b=tf2.extractfile(m).read().decode(); lines=b.splitlines()
        for i,l in enumerate(lines):
            if re.search(r"impl.*Aead|fn encrypt|fn decrypt|type Ciphertext|new_from_slice|KeyInit", l):
                print(f"--- L{i+1} ---")
                print("\n".join(lines[max(0,i-1):i+12]))
