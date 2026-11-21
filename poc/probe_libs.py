import sys, importlib.util as u
print("exe", sys.executable)
print("ver", sys.version.split()[0])
for m in ["cryptography", "argon2", "nacl", "Crypto", "Cryptodome", "cffi", "hkdf"]:
    s = u.find_spec(m)
    if s:
        try:
            mod = __import__(m)
            v = getattr(mod, "__version__", "?")
        except Exception as e:
            v = "import-error:" + repr(e)
        print(m, "OK", v)
    else:
        print(m, "-")

# deeper capability probe
try:
    from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305, AESGCM
    print("cryptography: ChaCha20Poly1305 OK, AESGCM OK")
    try:
        from cryptography.hazmat.primitives.ciphers.aead import XChaCha20Poly1305  # type: ignore
        print("cryptography: XChaCha20Poly1305 OK")
    except Exception as e:
        print("cryptography: XChaCha20Poly1305 MISSING", type(e).__name__)
    from cryptography.hazmat.primitives.kdf.hkdf import HKDF, HKDFExpand
    print("cryptography: HKDF/HKDFExpand OK")
    from cryptography.hazmat.primitives import keywrap
    print("cryptography: keywrap OK", [n for n in dir(keywrap) if "wrap" in n])
    from cryptography.hazmat.primitives.kdf.scrypt import Scrypt
    print("cryptography: Scrypt OK")
    try:
        from cryptography.hazmat.primitives.kdf.argon2 import Argon2id  # type: ignore
        print("cryptography: Argon2id OK")
    except Exception as e:
        print("cryptography: Argon2id MISSING", type(e).__name__)
except Exception as e:
    print("cryptography probe error", repr(e))

try:
    import nacl
    from nacl.bindings import crypto_aead_xchacha20poly1305_ietf_encrypt
    print("pynacl: xchacha20poly1305 ietf encrypt OK")
    from nacl.pwhash import argon2id
    print("pynacl: argon2id OK")
except Exception as e:
    print("pynacl probe error", repr(e))

try:
    import argon2
    print("argon2-cffi OK", argon2.__version__)
except Exception as e:
    print("argon2-cffi probe error", repr(e))
