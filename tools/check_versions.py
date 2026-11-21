import json, urllib.request

crates = ["argon2","aead","chacha20poly1305","aes-kw","aes","hkdf","hmac","sha2",
          "subtle","zeroize","base64","data-encoding","serde","serde_json","hex",
          "getrandom","uuid","clap","digest","cipher","generic-array"]
out = []
for c in crates:
    try:
        req = urllib.request.Request("https://crates.io/api/v1/crates/" + c,
                                     headers={"User-Agent": "mylock-m2-check/1.0"})
        with urllib.request.urlopen(req, timeout=30) as r:
            j = json.load(r)
        v = j["crate"].get("max_stable_version") or j["crate"].get("newest_version")
        out.append("%-20s %s" % (c, v))
    except Exception as e:
        out.append("%-20s ERROR %s" % (c, e))
print("\n".join(out))

print("\n=== deps of key crates (latest) ===")
for c, ver in [("hkdf","0.13.0"),("aes-kw","0.3.1"),("chacha20poly1305","0.11.0"),
               ("argon2","0.6.0"),("hmac","0.13.0"),("sha2","0.11.1"),("aead","0.6.0")]:
    try:
        url = "https://crates.io/api/v1/crates/%s/%s/dependencies" % (c, ver)
        req = urllib.request.Request(url, headers={"User-Agent":"mylock-m2-check/1.0"})
        with urllib.request.urlopen(req, timeout=30) as r:
            j = json.load(r)
        deps = ["%s %s%s" % (d["crate_id"], d["req"],
                             " (optional)" if d.get("optional") else "")
                for d in j["dependencies"] if d.get("kind")=="normal"]
        print("%s %s:" % (c, ver))
        for d in deps: print("   ", d)
    except Exception as e:
        print("%s %s: ERROR %s" % (c, ver, e))
