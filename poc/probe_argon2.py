"""Probe: confirm argon2-cffi low-level RAW output API + signature (for PoC KDF)."""
import json
import argon2.low_level as ll

out = {"version": getattr(__import__("argon2"), "__version__", "?"), "has_hash_secret_raw": hasattr(ll, "hash_secret_raw")}

# Candidate raw helper — this is exactly what the PoC will use.
pw = b"Myl0ck-\xe5\x8f\xa3\xe4\xbb\xa4-\xf0\x9f\x94\x90\x00-end"  # fixed, includes NULL byte
salt = bytes.fromhex("00112233445566778899aabbccddeeff")
try:
    v = ll.hash_secret_raw(
        secret=pw, salt=salt,
        time_cost=3, memory_cost=65536, parallelism=4,
        hash_len=32, type=ll.Type.ID,
    )
    out["hash_secret_raw"] = {"ok": True, "len": len(v), "hex": v.hex()}
except Exception as e:  # noqa: BLE001
    out["hash_secret_raw"] = {"ok": False, "err": repr(e)}

# Cross-check with p=1 must differ; libsodium p=1 must equal libargon2 p=1.
try:
    v1 = ll.hash_secret_raw(secret=pw, salt=salt, time_cost=3, memory_cost=65536,
                            parallelism=1, hash_len=32, type=ll.Type.ID)
    out["p1_hex"] = v1.hex()
    out["p4_ne_p1"] = (v != v1)
except Exception as e:  # noqa: BLE001
    out["p1"] = repr(e)

print(json.dumps(out, ensure_ascii=False, indent=2))
