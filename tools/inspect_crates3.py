# -*- coding: utf-8 -*-
"""Final focused API checks before writing Rust."""
import io, re, tarfile, urllib.request
OUT=[]
def p(s=""): OUT.append(str(s))
def files(name, ver):
    url=f"https://static.crates.io/crates/{name}/{name}-{ver}.crate"
    with urllib.request.urlopen(url, timeout=60) as r: data=r.read()
    tf=tarfile.open(fileobj=io.BytesIO(data), mode="r:gz")
    return {m.name.split("/",1)[-1]: tf.extractfile(m).read().decode("utf-8","replace")
            for m in tf.getmembers() if m.isfile() and m.name.endswith(".rs")}

def show(name, ver, needles, before=2, after=10):
    p("="*70); p(f"### {name} {ver}")
    fs=files(name,ver)
    for needle in needles:
        p(f"-- needle: {needle}")
        n=0
        for fn,body in fs.items():
            for m in re.finditer(needle, body):
                ln=body.count("\n",0,m.start())+1
                p("\n".join(body.splitlines()[max(0,ln-before):ln+after]))
                p("   ...")
                n+=1
                if n>=3: break
            if n>=3: break
        if n==0: p("  (NO HIT)")

show("chacha20poly1305","0.11.0",[
    r"pub type XNonce",
    r"type XNonce|struct XNonce",
    r"pub type Nonce",
])
show("hkdf","0.13.0",[
    r"pub struct Hkdf\b",
    r"type Hkdf|Default for|I = ",
    r"pub type Hkdf$",
    r"impl<H> Hkdf<H",
])
show("argon2","0.6.0",[
    r"pub fn new\(algorithm",
    r"impl Argon2<'|impl Default for Argon2",
    r"pub struct Argon2<'",
])
show("aes-kw","0.3.1",[
    r"pub fn wrap_key<'a>",
    r"pub fn unwrap_key<'a>",
    r"pub fn new_from_slice",
])
show("aead","0.6.0",[
    r"fn encrypt\(&self, nonce",
    r"fn decrypt\(&self, nonce",
])
open("D:/Projects/dsh/mylock/tools/crate-api-dump2.txt","w",encoding="utf-8").write("\n".join(OUT))
print("wrote", len(OUT), "lines")
