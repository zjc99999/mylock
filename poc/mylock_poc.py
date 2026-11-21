# -*- coding: utf-8 -*-
"""
mylock M1 PoC — 主链路 + 契约级测试向量（一次性验证脚手架，非产品代码）。

目的（用户 m00606 授权「做可运行 PoC 验主链路与测试向量」）：
  1. 用「被审计过的库」按 vault-format-spec.md v1.1.1 的冻结参数，逐字节跑通
     建库 → 整盘复制到「移动硬盘」→ 口令解锁 → 两阶段 verify → 明文导出 全链路；
  2. 产出 §9 要求的固定输入/输出测试向量（test-vectors.json），作为未来 Rust
     实现必须逐字节对齐的契约；
  3. 跑 §9 规定的全部负向向量，确认错误码与「永不返回未认证明文」；
  4. 顺带把实现时才会暴露的「字节级拼接约定」用可运行代码钉死并暴露给规范。

依赖（全部为被审计过的库，已由 poc/verify_primitives.py 用权威 KAT 校验 26/26 PASS）：
  - PyNaCl 1.6.2 (libsodium)      : XChaCha20-Poly1305 (draft-xchacha-03 §A.3.1)
  - argon2-cffi 25.1.0 (libargon2): Argon2id（显式 parallelism=4，RFC 9106 §5.3）
  - cryptography 50.0.2          : AES-256-KW (RFC 3394), HKDF-Expand (RFC 5869),
                                   HMAC-SHA256 (RFC 4231), SHA-256
  - stdlib                        : hashlib / hmac / json / base64 / struct / shutil

★ 字节级拼接约定（PoC 固定，规范 §4.2/§5.3/§3.4 的 `‖`/info 尚未逐字节冻结 —— 见 PoC 报告 FINDING-4）：
  - 整数（format / key_epoch / manifest_seq / chunk_index / chunk_count / ciphertext_len）
    一律 u32 big-endian（4 字节）；obj_version / cipher_id 为 u8（1 字节）。
  - jti：uuid-v4 字符串的 ASCII 字节（meta 里就是字符串）。
  - object_id / manifest.id：其 **16 字节原始值**（base32 字符串仅用于文件名 / JSON 展示 / 记录 id）。
  - HKDF-Expand info = 固定 ASCII 标签 + 16 字节原始 id（或 slot 名 ASCII）：
        KEK_password = HKDF-Expand(IKM_pw,  b"wrap/password")
        KEK_recovery = HKDF-Expand(IKM_rk,  b"wrap/recovery")
        vmac_key     = HKDF-Expand(MAC_Key, b"meta-version-mac")
        manifest_key = HKDF-Expand(Master,   b"manifest" + manifest_id_raw)
        object_key   = HKDF-Expand(Master,   b"object"   + object_id_raw)
  - chunk AAD    = b"MYVAULT" + u32(format) + jti.ascii + object_id_raw(16)
                   + u32(chunk_index) + u32(chunk_count) + header_hash(32)
  - manifest AAD = b"MYVAULT" + u32(format) + jti.ascii + manifest_id_raw(16)
                   + u32(manifest_seq)
  - 两个 secret 输入（主口令 / Recovery Key 原始 32B）都走同一组 Argon2id(m,t,p,salt)，
    得到的 IKM 再经 slot 域分离 → 两个 KEK 包裹同一 Master∥MAC_Key。
    （规范 §2/§3.4 的单一 kdf 参数 + slot∈{password,recovery} 的直读；见 FINDING-5。）

状态：一次性验证脚手架；不是产品代码、不是 M2 开发、不是第三方审计。
"""
from __future__ import annotations

import base64
import hashlib
import hmac
import json
import os
import shutil
import struct
import sys
from pathlib import Path

import argon2.low_level as AL
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.hkdf import HKDFExpand
from cryptography.hazmat.primitives.keywrap import aes_key_unwrap, aes_key_wrap
from nacl import bindings as B
from nacl.exceptions import CryptoError

# ---------------------------------------------------------------------------
# 0. 固定输入（§9：固定 nonce 才逐字节一致；正常运行用 CSPRNG 随机 nonce）
# ---------------------------------------------------------------------------
MAGIC_VAULT = b"MYVAULT"          # AAD / 格式标识
MAGIC_OBJ = b"MYOB"               # 对象 header 的 magic(4)
FORMAT = 1
OBJ_VERSION = 1
CIPHER_ID = 1                     # XCHACHA20-POLY1305
CHUNK_SIZE = 1048576              # 1 MiB
KEY_EPOCH = 0
MANIFEST_SEQ = 1
BASE_VERSION = 0

PASSPHRASE = "Myl0ck-保险柜-🔐\x00-v1.1.1"       # 含非 ASCII + NULL 字节
PW_BYTES = PASSPHRASE.encode("utf-8")
SALT = bytes.fromhex("00112233445566778899aabbccddeeff")            # 16 B
MASTER = bytes(range(32))                    # 32 B 000102...1f
MAC_KEY = bytes(range(32, 64))               # 32 B 202122...3f
RK_RAW = bytes(range(64, 96))                # Recovery Key 原始 32 B 404142...5f
JTI = "11111111-2222-4333-8444-555555555555"  # 固定的 uuid-v4 形
DEVICE_ID = bytes(range(72, 88))             # 16 B 设备本地身份（记录里是溯源标签快照）

OBJECT_ID_A = bytes(range(16))               # 普通文件对象 16 B
OBJECT_ID_B = bytes(range(16, 32))           # 空文件对象 16 B
MANIFEST_ID = bytes(range(48, 64))           # manifest.id 原始 16 B
ROOT_ID = bytes(16)                          # 根 parent_id 原始 16 B

NONCE_A = bytes(range(24))                   # 对象 A 的 chunk[0] 固定 24 B nonce
NONCE_B = bytes(range(24, 48))               # 空对象 B 的 chunk[0] 固定 24 B nonce
NONCE_MANIFEST = bytes(range(48, 72))        # manifest 固定 24 B nonce

PLAINTEXT_A = b"Hello, mylock \xe4\xbf\x9d\xe9\x99\xa9\xe6\x9f\x9c!\nsecond\x01line\n"
PLAINTEXT_B = b""                            # 0 字节空对象向量（F15/N4）

# KDF 冻结参数（§2 / §3.5）
KDF_M, KDF_T, KDF_P = 65536, 3, 4
KDF_TAGLEN = 32


def b32(raw16: bytes) -> str:
    """128-bit → RFC4648 base32 大写无填充，26 字符。"""
    return base64.b32encode(raw16).decode("ascii").rstrip("=")


OBJECT_ID_A_B32 = b32(OBJECT_ID_A)
OBJECT_ID_B_B32 = b32(OBJECT_ID_B)
MANIFEST_ID_B32 = b32(MANIFEST_ID)
ROOT_ID_B32 = b32(ROOT_ID)

# ---------------------------------------------------------------------------
# 1. 原语封装（全部委托被审计过的库）
# ---------------------------------------------------------------------------
def u32(n: int) -> bytes:
    return struct.pack(">I", n)


def sha256(b: bytes) -> bytes:
    return hashlib.sha256(b).digest()


def hkdf_expand(prk: bytes, info: bytes, length: int = 32) -> bytes:
    """仅 HKDF-Expand（RFC 5869 §3.3：prk 已均匀，跳 Extract）。"""
    return HKDFExpand(algorithm=hashes.SHA256(), length=length, info=info).derive(prk)


def argon2id(secret: bytes) -> bytes:
    """Argon2id(m=65536 KiB, t=3, p=4, salt=16B, tag=32B, v=0x13, type=ID)。"""
    return AL.hash_secret_raw(
        secret=secret, salt=SALT,
        time_cost=KDF_T, memory_cost=KDF_M, parallelism=KDF_P,
        hash_len=KDF_TAGLEN, type=AL.Type.ID, version=0x13,
    )


def xchacha_encrypt(key: bytes, nonce: bytes, pt: bytes, aad: bytes) -> bytes:
    """返回 libsodium 的 combined = ciphertext || tag(16)。"""
    return B.crypto_aead_xchacha20poly1305_ietf_encrypt(pt, aad, nonce, key)


def xchacha_decrypt(key: bytes, nonce: bytes, combined: bytes, aad: bytes) -> bytes:
    return B.crypto_aead_xchacha20poly1305_ietf_decrypt(combined, aad, nonce, key)


def canonical(obj) -> bytes:
    """§3.3 canonical JSON profile 八条（v1.1.1 规则⑤勘误后的实现）。"""
    out = []
    def enc(o):
        if o is True:  out.append("true")
        elif o is False: out.append("false")
        elif o is None: out.append("null")
        elif isinstance(o, int): out.append(str(o))
        elif isinstance(o, str):
            out.append('"')
            for ch in o:
                c = ord(ch)
                if ch == '"': out.append('\\"')
                elif ch == "\\": out.append("\\\\")
                elif c < 0x20: out.append("\\u%04x" % c)   # 控制字符一律 \u00xx 小写
                else: out.append(ch)                          # 非 ASCII 原样输出
            out.append('"')
        elif isinstance(o, list):
            out.append("[")
            for i, v in enumerate(o):
                if i: out.append(",")
                enc(v)
            out.append("]")
        elif isinstance(o, dict):
            out.append("{")
            for i, k in enumerate(sorted(o.keys())):           # Unicode 码点升序
                if i: out.append(",")
                enc(k); out.append(":"); enc(o[k])
            out.append("}")
        else:
            raise TypeError("canonical: unsupported %r" % type(o))
    enc(obj)
    return "".join(out).encode("utf-8")


# ---------------------------------------------------------------------------
# 2. 派生 / 包裹 / 认证
# ---------------------------------------------------------------------------
def build_aad_chunk(object_id_raw: bytes, chunk_index: int, chunk_count: int,
                    header_hash: bytes) -> bytes:
    return (MAGIC_VAULT + u32(FORMAT) + JTI.encode("ascii") + object_id_raw
            + u32(chunk_index) + u32(chunk_count) + header_hash)


def build_aad_manifest(manifest_seq: int) -> bytes:
    return (MAGIC_VAULT + u32(FORMAT) + JTI.encode("ascii") + MANIFEST_ID
            + u32(manifest_seq))


def build_object_header(object_id_raw: bytes, chunk_count: int) -> bytes:
    """header 固定 32 字节：magic(4)|obj_version(1)|object_id(16)|chunk_size(4)|
       chunk_count(4)|cipher_id(1)|reserved(2)。"""
    return (MAGIC_OBJ + bytes([OBJ_VERSION]) + object_id_raw
            + u32(CHUNK_SIZE) + u32(chunk_count) + bytes([CIPHER_ID])
            + b"\x00\x00")


def encrypt_object(object_id_raw: bytes, plaintext: bytes, nonce: bytes):
    """返回 (object_file_bytes, object_key_hex, combined_hex)。"""
    object_key = hkdf_expand(MASTER, b"object" + object_id_raw)
    header = build_object_header(object_id_raw, 1)   # 测试向量均单 chunk（空对象也 1 chunk）
    header_hash = sha256(header)
    aad = build_aad_chunk(object_id_raw, 0, 1, header_hash)
    combined = xchacha_encrypt(object_key, nonce, plaintext, aad)
    ct, tag = combined[:-16], combined[-16:]
    chunk = nonce + u32(len(ct)) + ct + tag
    return header + chunk, object_key, combined


def manifest_plaintext():
    """§5.3 canonical JSON 记录数组（device_id = 溯源标签快照，base64）。"""
    dev = base64.b64encode(DEVICE_ID).decode("ascii")
    return [
        {"id": OBJECT_ID_A_B32, "parent_id": ROOT_ID_B32, "name": "hello.txt",
         "kind": "file", "size": len(PLAINTEXT_A), "chunk_count": 1,
         "base_version": BASE_VERSION, "device_id": dev, "updated_seq": 1,
         "content_version": 1, "manifest_seq": MANIFEST_SEQ},
        {"id": OBJECT_ID_B_B32, "parent_id": ROOT_ID_B32, "name": "empty.bin",
         "kind": "file", "size": 0, "chunk_count": 1,
         "base_version": BASE_VERSION, "device_id": dev, "updated_seq": 1,
         "content_version": 1, "manifest_seq": MANIFEST_SEQ},
    ]


# ---------------------------------------------------------------------------
# 3. 建库（写静态密文容器 —— §6.8「不依赖 OS 的保险柜」）
# ---------------------------------------------------------------------------
def create_vault(root: Path) -> dict:
    if root.exists():
        shutil.rmtree(root)
    (root / "d").mkdir(parents=True)

    # --- 两个 secret → 两个 IKM → 两个 KEK（slot 域分离） ---
    IKM_pw = argon2id(PW_BYTES)
    IKM_rk = argon2id(RK_RAW)
    KEK_pw = hkdf_expand(IKM_pw, b"wrap/password")
    KEK_rk = hkdf_expand(IKM_rk, b"wrap/recovery")

    blob = MASTER + MAC_KEY                     # 64 B = 8 个 64-bit 块
    wp = aes_key_wrap(KEK_pw, blob)            # → 72 B
    wr = aes_key_wrap(KEK_rk, blob)            # → 72 B

    # --- versionMac（全头认证，解锁后比对）---
    vmac_key = hkdf_expand(MAC_KEY, b"meta-version-mac")
    meta_core = {
        "magic": "MYVAULT",
        "format": FORMAT,
        "jti": JTI,
        "key_epoch": KEY_EPOCH,
        "cipher": "XCHACHA20-POLY1305",
        "kdf": {"alg": "argon2id", "m": KDF_M, "t": KDF_T, "p": KDF_P,
                "salt": base64.b64encode(SALT).decode("ascii")},
        "wrapped": [
            {"slot": "password", "alg": "RFC3394-AES-KW",
             "blob": base64.b64encode(wp).decode("ascii")},
            {"slot": "recovery", "alg": "RFC3394-AES-KW",
             "blob": base64.b64encode(wr).decode("ascii")},
        ],
        "manifest_ref": {"id": MANIFEST_ID_B32, "seq": MANIFEST_SEQ},
        "gc_pending": False,
    }
    mac_input = canonical(meta_core)
    versionMac = hmac.new(vmac_key, mac_input, hashlib.sha256).digest()
    meta_full = dict(meta_core)
    meta_full["versionMac"] = base64.b64encode(versionMac).decode("ascii")
    meta_bytes = canonical(meta_full)
    (root / "vault.meta").write_bytes(meta_bytes)
    (root / "vault.meta.bak").write_bytes(meta_bytes)   # PoC：与 meta 同构，自带同一 versionMac

    # --- 加密对象 ---
    objA, keyA, combA = encrypt_object(OBJECT_ID_A, PLAINTEXT_A, NONCE_A)
    objB, keyB, combB = encrypt_object(OBJECT_ID_B, PLAINTEXT_B, NONCE_B)
    for b32id, data in ((OBJECT_ID_A_B32, objA), (OBJECT_ID_B_B32, objB)):
        shard = root / "d" / b32id[:2]
        shard.mkdir(exist_ok=True)
        (shard / b32id).write_bytes(data)

    # --- 加密 manifest ---
    manifest_key = hkdf_expand(MASTER, b"manifest" + MANIFEST_ID)
    pt = canonical(manifest_plaintext())
    aad_m = build_aad_manifest(MANIFEST_SEQ)
    comb_m = xchacha_encrypt(manifest_key, NONCE_MANIFEST, pt, aad_m)
    (root / "manifest").write_bytes(NONCE_MANIFEST + comb_m)

    (root / "README.txt").write_text(
        "This is a mylock vault (format v1). All user data is encrypted. "
        "Use the recovery CLI to unlock. No account, no cloud, no telemetry.\n",
        encoding="utf-8")

    return {
        "IKM_password": IKM_pw.hex(),
        "IKM_recovery": IKM_rk.hex(),
        "KEK_password": KEK_pw.hex(),
        "KEK_recovery": KEK_rk.hex(),
        "wrapped.password.blob": base64.b64encode(wp).decode("ascii"),
        "wrapped.recovery.blob": base64.b64encode(wr).decode("ascii"),
        "versionMac": base64.b64encode(versionMac).decode("ascii"),
        "versionMac.input_hex": mac_input.hex(),
        "versionMac.key_hex": vmac_key.hex(),
        "master_hex": MASTER.hex(),
        "mac_key_hex": MAC_KEY.hex(),
        "manifest_key_hex": manifest_key.hex(),
        "object_key_A_hex": keyA.hex(),
        "object_key_B_hex": keyB.hex(),
        "object_A_file_hex": objA.hex(),
        "object_B_file_hex(empty)": objB.hex(),
        "object_A_combined_hex": combA.hex(),
        "object_B_combined_hex(empty)": combB.hex(),
        "manifest_file_hex": (NONCE_MANIFEST + comb_m).hex(),
        "manifest_combined_hex": comb_m.hex(),
        "manifest_plaintext_canonical_hex": pt.hex(),
        "manifest_aad_hex": aad_m.hex(),
        "chunk_aad_A_hex": build_aad_chunk(OBJECT_ID_A, 0, 1, sha256(build_object_header(OBJECT_ID_A, 1))).hex(),
        "chunk_aad_B_hex": build_aad_chunk(OBJECT_ID_B, 0, 1, sha256(build_object_header(OBJECT_ID_B, 1))).hex(),
        "meta_canonical_hex": meta_bytes.hex(),
        "object_id_A_b32": OBJECT_ID_A_B32,
        "object_id_B_b32": OBJECT_ID_B_B32,
        "manifest_id_b32": MANIFEST_ID_B32,
        "recovery_key_raw_hex": RK_RAW.hex(),
        "recovery_key_display": _rk_display(RK_RAW, KEY_EPOCH),
    }


def _rk_display(rk_raw: bytes, epoch: int) -> str:
    """§2 L107：base32(52) + 4 校验位 + key_epoch 前缀，13×4 分组（PoC 固定编码）。"""
    b = base64.b32encode(rk_raw).decode("ascii").rstrip("=")   # 52 字符
    chk = sha256(rk_raw)[:2].hex()[:4].upper()                  # 4 校验位（PoC 固定：SHA-256 前 2 字节 hex）
    body = b + chk
    groups = " ".join(body[i:i + 4] for i in range(0, len(body), 4))
    return "E%03d-%s" % (epoch, groups)


# ---------------------------------------------------------------------------
# 4. 解锁 / 两阶段 verify / export（严格按 §3.4 顺序）
# ---------------------------------------------------------------------------
class OpenError(Exception):
    def __init__(self, code: str, detail: str = ""):
        super().__init__(code)
        self.code, self.detail = code, detail


def kdf_hard_check(meta: dict):
    """§3.5：读 meta 后、跑 Argon2id 之前；越界 E_FORMAT，绝不分配 KDF 内存。"""
    k = meta.get("kdf", {})
    m, t, p = k.get("m"), k.get("t"), k.get("p")
    if not (isinstance(m, int) and isinstance(t, int) and isinstance(p, int)):
        raise OpenError("E_FORMAT", "kdf params missing/non-int")
    if not (19456 <= m <= 1048576 and 2 <= t <= 10 and 1 <= p <= 16):
        raise OpenError("E_FORMAT", "kdf params out of bounds (F14/N3)")


def load_meta(root: Path) -> dict:
    """严格 canonical JSON 校验（§3.3 规则 8 + 逐字节契约）。"""
    raw = (root / "vault.meta").read_bytes()
    try:
        meta = json.loads(raw.decode("utf-8"))
    except Exception as e:                       # noqa: BLE001
        raise OpenError("E_FORMAT", "meta not JSON: %r" % e)
    # 禁止非规范序列化（键序/空白/转义/重复键）
    try:
        re = canonical(meta)
    except Exception as e:                       # noqa: BLE001
        raise OpenError("E_FORMAT", "canonical serialization failed: %r" % e)
    if re != raw:
        raise OpenError("E_FORMAT", "meta is not canonical JSON (profile violation)")
    allowed = {"magic", "format", "jti", "key_epoch", "cipher", "kdf",
               "wrapped", "manifest_ref", "versionMac", "gc_pending"}
    if set(meta.keys()) != allowed:
        raise OpenError("E_FORMAT", "unknown/missing meta field")
    if meta.get("magic") != "MYVAULT" or meta.get("format") != FORMAT:
        raise OpenError("E_FORMAT", "magic/format mismatch")
    return meta


def parse_object_file(data: bytes):
    """解析对象文件；结构异常抛 OpenError(E_CORRUPT/E_FORMAT)。"""
    if len(data) < 32:
        raise OpenError("E_CORRUPT", "object shorter than header")
    header = data[:32]
    magic = header[:4]
    obj_v = header[4]
    oid = header[5:21]
    chunk_size = struct.unpack(">I", header[21:25])[0]
    chunk_count = struct.unpack(">I", header[25:29])[0]
    cipher_id = header[29]
    if magic != MAGIC_OBJ or obj_v != OBJ_VERSION or cipher_id != CIPHER_ID:
        raise OpenError("E_FORMAT", "object header magic/version/cipher")
    if chunk_count < 1:
        raise OpenError("E_CORRUPT", "chunk_count=0 forgery (F15/N4)")
    header_hash = sha256(header)
    off = 32
    chunks = []
    for i in range(chunk_count):
        if off + 24 + 4 > len(data):
            raise OpenError("E_CORRUPT", "truncated chunk header")
        nonce = data[off:off + 24]; off += 24
        ct_len = struct.unpack(">I", data[off:off + 4])[0]; off += 4
        if off + ct_len + 16 > len(data):
            raise OpenError("E_CORRUPT", "truncated chunk body")
        ct = data[off:off + ct_len]; off += ct_len
        tag = data[off:off + 16]; off += 16
        chunks.append((nonce, ct, tag, i))
    if off != len(data):
        raise OpenError("E_CORRUPT", "trailing bytes in object")
    return {"oid": oid, "chunk_count": chunk_count, "header_hash": header_hash,
            "chunks": chunks}


def verify_phase1(root: Path) -> dict:
    """§6.1 阶段1：解锁前，非密码学结构检查；全量扫描、不早停、定长输出。"""
    meta = load_meta(root)                 # 结构/格式（含 canonical 检查）
    kdf_hard_check(meta)                  # KDF 参数硬校验（防 OOM DoS）
    objects, total_bytes = 0, 0
    anomalies = []
    d = root / "d"
    for shard in sorted(d.iterdir()) if d.exists() else []:
        if not shard.is_dir():
            continue
        for f in sorted(shard.iterdir()):
            if not f.is_file():
                continue
            objects += 1
            total_bytes += f.stat().st_size
            try:
                parse_object_file(f.read_bytes())   # 含 chunk_count>=1 / 截断检查
            except OpenError:
                anomalies.append(1)                  # 不早停、不区分具体对象
    result = "OK" if not anomalies else "STRUCT_ANOMALY"
    return {"phase": "pre-unlock", "result": result,
            "objects": objects, "bytes": total_bytes}


def full_open(root: Path, secret: bytes, slot: str = "password") -> dict:
    """按 §3.4 冻结顺序完整解锁 + 阶段2 验证 + export；返回结构化结果或抛 OpenError。

    secret = 主口令的 UTF-8 字节，或 Recovery Key 的原始 32 B；slot ∈ {password, recovery}。
    §6.2 CLI 一次只接一个凭据（--password | --recovery），故这里单槽打开。
    """
    if isinstance(secret, str):
        secret = secret.encode("utf-8")                     # 容错：CLI 传口令字符串 / RK 原始 32B
    p1 = verify_phase1(root)
    meta = load_meta(root)
    kdf_hard_check(meta)                                   # ②（再来一次，确保顺序）

    IKM = argon2id(secret)                                 # ③
    KEK = hkdf_expand(IKM, ("wrap/" + slot).encode("ascii"))   # ④ slot 域分离
    blob_b64 = None
    for w in meta["wrapped"]:
        if w.get("slot") == slot:
            blob_b64 = w["blob"]
    if blob_b64 is None:
        raise OpenError("E_AUTH_FAILED", "no %s slot" % slot)
    try:
        blob = aes_key_unwrap(KEK, base64.b64decode(blob_b64))   # ⑤ 首道 2^-64 过滤
    except Exception:                                    # noqa: BLE001
        raise OpenError("E_AUTH_FAILED", "unwrap failed (unified, F16)")
    Master, MAC_Key = blob[:32], blob[32:]

    vmac_key = hkdf_expand(MAC_Key, b"meta-version-mac")  # ⑥
    core = {k: v for k, v in meta.items() if k != "versionMac"}
    expected = hmac.new(vmac_key, canonical(core), hashlib.sha256).digest()  # ⑦
    if not hmac.compare_digest(expected, base64.b64decode(meta["versionMac"])):
        raise OpenError("E_DOWNGRADE", "versionMac mismatch (whole-header auth)")

    # ⑨ 解密 manifest
    manifest_key = hkdf_expand(Master, b"manifest" + MANIFEST_ID)
    mfile = (root / "manifest").read_bytes()
    m_nonce, m_comb = mfile[:24], mfile[24:]
    aad_m = build_aad_manifest(meta["manifest_ref"]["seq"])
    try:
        m_pt = xchacha_decrypt(manifest_key, m_nonce, m_comb, aad_m)
    except CryptoError:
        raise OpenError("E_MANIFEST", "manifest AEAD tag failed")
    records = json.loads(m_pt.decode("utf-8"))
    # meta ↔ manifest 双向绑定
    rec_seq = {r["manifest_seq"] for r in records}
    if rec_seq and meta["manifest_ref"]["seq"] not in rec_seq:
        raise OpenError("E_MANIFEST", "manifest.seq != manifest_ref.seq")

    # 逐对象校验 + 明文重建
    exported = {}
    on_disk = set()
    d = root / "d"
    for shard in d.iterdir():
        if shard.is_dir():
            for f in shard.iterdir():
                on_disk.add(f.name)
    for rec in records:
        b32id = rec["id"]
        path = d / b32id[:2] / b32id
        if not path.exists():
            raise OpenError("E_MISSING", "manifest refs missing object")
        parsed = parse_object_file(path.read_bytes())
        oid_raw = _b32decode16(b32id)                # 由 b32 反解 16B 原始 id
        object_key = hkdf_expand(Master, b"object" + oid_raw)
        pts = []
        for nonce, ct, tag, idx in parsed["chunks"]:
            aad = build_aad_chunk(oid_raw, idx, parsed["chunk_count"], parsed["header_hash"])
            try:
                pts.append(xchacha_decrypt(object_key, nonce, ct + tag, aad))
            except CryptoError:
                raise OpenError("E_CORRUPT", "chunk AEAD tag/AAD failed")
        exported[rec["name"]] = b"".join(pts)
    orphans = sorted(on_disk - {r["id"] for r in records})
    return {"ok": True, "phase1": p1, "error": None,
            "records": len(records), "exported": exported,
            "orphans": orphans,
            "E_ORPHAN": "WARN" if orphans else None}


def _b32decode16(s: str) -> bytes:
    return base64.b32decode(s + "=" * ((8 - len(s) % 8) % 8))


# ---------------------------------------------------------------------------
# 5. 负向向量（§9 L407–415）
# ---------------------------------------------------------------------------
def clone(root: Path, tag: str) -> Path:
    dst = root.parent / ("_neg_" + tag)
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(root, dst)
    return dst


def flip_bit_in_meta(root: Path, field: str) -> None:
    p = root / "vault.meta"
    meta = json.loads(p.read_bytes().decode("utf-8"))
    v = meta[field]
    # 改一个字符（仍保持合法 canonical JSON），改的是「值」而非格式 → versionMac 失败
    meta[field] = (v[:-1] + ("Z" if v[-1] != "Z" else "Y")) if isinstance(v, str) else v + 1
    p.write_bytes(canonical(meta))


def negative_vectors(base: Path, password: str) -> list:
    results = []
    def run(tag, mutate, expect, pw=password):
        r = clone(base, tag)
        try:
            mutate(r)
            try:
                full_open(r, pw)
                got = "NO_ERROR"
            except OpenError as e:
                got = e.code
            except Exception as e:                 # noqa: BLE001
                got = "EXC:" + type(e).__name__
        finally:
            pass
        results.append({"id": tag, "expect": expect, "got": got,
                        "pass": (got in expect if isinstance(expect, list) else got == expect)})
        shutil.rmtree(r, ignore_errors=True)

    run("meta_1bit", lambda r: flip_bit_in_meta(r, "jti"), "E_DOWNGRADE")
    run("downgrade_format", lambda r: flip_bit_in_meta(r, "format"), ["E_FORMAT", "E_DOWNGRADE"])
    run("key_epoch_change", lambda r: _set_meta(r, "key_epoch", 99), "E_DOWNGRADE")

    def blob_flip(r):
        p = r / "vault.meta"
        meta = json.loads(p.read_bytes().decode("utf-8"))
        s = meta["wrapped"][0]["blob"]
        meta["wrapped"][0]["blob"] = ("A" if s[0] != "A" else "B") + s[1:]
        p.write_bytes(canonical(meta))
    run("wrapped_blob_1bit", blob_flip, "E_AUTH_FAILED")

    def slot_swap(r):
        p = r / "vault.meta"
        meta = json.loads(p.read_bytes().decode("utf-8"))
        a, b = meta["wrapped"][0]["blob"], meta["wrapped"][1]["blob"]
        meta["wrapped"][0]["blob"], meta["wrapped"][1]["blob"] = b, a
        p.write_bytes(canonical(meta))
    run("slot_swap", slot_swap, "E_AUTH_FAILED")

    # KDF 越界：必须在阶段1（跑 Argon2id 之前）就 E_FORMAT，绝不分配 4TiB 内存
    run("kdf_oob_m4tib", lambda r: _set_meta(r, "kdf", dict(
        json.loads((r / "vault.meta").read_bytes().decode("utf-8"))["kdf"], m=4 * 1024 * 1024 * 1024)),
        "E_FORMAT")

    def chunk_flip(r):
        p = r / "d" / OBJECT_ID_A_B32[:2] / OBJECT_ID_A_B32
        data = bytearray(p.read_bytes())
        data[32 + 24 + 4] ^= 0x01           # 翻转第一个 chunk 的 ciphertext 1 bit
        p.write_bytes(bytes(data))
    run("chunk_1bit", chunk_flip, "E_CORRUPT")

    def delete_obj(r):
        (r / "d" / OBJECT_ID_B_B32[:2] / OBJECT_ID_B_B32).unlink()
    run("delete_object", delete_obj, "E_MISSING")

    def truncate_obj(r):
        p = r / "d" / OBJECT_ID_A_B32[:2] / OBJECT_ID_A_B32
        d = p.read_bytes()
        p.write_bytes(d[:-3])                # 截断 → 结构检查即报
    run("truncate_object", truncate_obj, ["E_CORRUPT", "STRUCT_ANOMALY"])

    def orphan_obj(r):
        p = r / "d" / "ZZ" / "ZZAAAAAAAAAAAAAAAAAAAAAAAAAA"
        p.parent.mkdir(exist_ok=True)
        # 复制一个合法对象但文件名换成未被引用的 id → 解锁后 E_ORPHAN 警告
        src = r / "d" / OBJECT_ID_A_B32[:2] / OBJECT_ID_A_B32
        shutil.copy(src, p)
    run("orphan_object", orphan_obj, "NO_ERROR")   # 孤儿是警告非致命；full_open 不抛错

    def empty_chunkcount0(r):
        p = r / "d" / OBJECT_ID_B_B32[:2] / OBJECT_ID_B_B32
        d = bytearray(p.read_bytes())
        # header chunk_count 在偏移 25..29，改成 0
        d[25:29] = (0).to_bytes(4, "big")
        p.write_bytes(bytes(d))
    run("empty_chunkcount0", empty_chunkcount0, "E_CORRUPT")

    def manifest_flip(r):
        p = r / "manifest"
        d = bytearray(p.read_bytes())
        d[24] ^= 0x01                       # manifest AEAD 密文 1 bit
        p.write_bytes(bytes(d))
    run("manifest_1bit", manifest_flip, "E_MANIFEST")

    # 弱口令：错误的口令 → 统一 E_AUTH_FAILED（不区分「口令错」与「blob 篡改」）
    run("wrong_password", lambda r: None, "E_AUTH_FAILED", pw="definitely-wrong-\x00")
    return results


def _set_meta(root: Path, field: str, value) -> None:
    p = root / "vault.meta"
    meta = json.loads(p.read_bytes().decode("utf-8"))
    meta[field] = value
    p.write_bytes(canonical(meta))


# ---------------------------------------------------------------------------
# 6. main
# ---------------------------------------------------------------------------
def main() -> int:
    here = Path(__file__).parent
    base = here / "vault_poc"
    usb = here / "usb_copy"          # 模拟「拷贝到移动硬盘」（§6.8）
    out = here / "export_out"

    print("=" * 72)
    print("mylock M1 PoC — 主链路 + 契约级测试向量")
    print("python:", sys.version.split()[0])
    print("=" * 72)

    # (1) 建库
    derived = create_vault(base)
    print("[1] create_vault OK  (objects=2, one of them 0-byte empty)")

    # (2) 整盘复制 → 移动硬盘；从副本解锁（载体无关）
    if usb.exists():
        shutil.rmtree(usb)
    shutil.copytree(base, usb)
    print("[2] whole-tree copy -> 'usb_copy' OK (carrier-independent)")

    # (3) 阶段1 verify（不输口令）
    p1 = verify_phase1(usb)
    print("[3] verify phase1:", json.dumps(p1, ensure_ascii=False))

    # (4) 阶段2：口令解锁 + 完整验证 + export
    res = full_open(usb, PASSPHRASE)
    print("[4] unlock + verify phase2 OK  records=%d orphans=%d" %
          (res["records"], len(res["orphans"])))
    assert res["exported"]["hello.txt"] == PLAINTEXT_A, "export A mismatch"
    assert res["exported"]["empty.bin"] == PLAINTEXT_B, "export empty mismatch"
    if out.exists():
        shutil.rmtree(out)
    out.mkdir()
    for name, data in res["exported"].items():
        (out / name).write_bytes(data)
    print("    export rebuild OK: hello.txt=%dB empty.bin=%dB" %
          (len(res["exported"]["hello.txt"]), len(res["exported"]["empty.bin"])))

    # (4b) 恢复槽端到端验证（忘记口令时唯一兜底；实参 = Recovery Key 原始 32 B）
    res_rk = full_open(usb, RK_RAW, slot="recovery")
    assert res_rk["exported"]["hello.txt"] == PLAINTEXT_A, "recovery export A mismatch"
    assert res_rk["exported"]["empty.bin"] == PLAINTEXT_B, "recovery export empty mismatch"
    print("[4b] recovery-slot end-to-end OK (F13 slot domain separation exercised)")
    # 反向：把 RK 当口令喂给 password 槽必须统一失败（域分离生效）
    try:
        full_open(usb, RK_RAW, slot="password")
        rk_as_pw = "NO_ERROR"
    except OpenError as e:
        rk_as_pw = e.code
    print("[4c] RK offered to password slot ->", rk_as_pw,
          "(expect E_AUTH_FAILED / E_DOWNGRADE)")

    # (5) 负向向量
    neg = negative_vectors(usb, PASSPHRASE)
    npass = sum(1 for n in neg if n["pass"])
    print("[6] negative vectors: %d/%d expected-code matched" % (npass, len(neg)))
    for n in neg:
        print("    %-22s expect=%-22s got=%-22s %s" %
              (n["id"], str(n["expect"]), n["got"], "OK" if n["pass"] else "**CHECK**"))

    # (6) 落盘 test-vectors.json（§9 契约）
    vectors = {
        "_meta": {
            "doc": "vault-format-spec.md v1.1.1 §9 固定测试向量；mylock_poc.py 用被审计库产出",
            "status": "一次性验证脚手架产物，非产品代码；固定 nonce 才逐字节一致",
            "primitives": {
                "XChaCha20-Poly1305": "PyNaCl/libsodium (draft-xchacha-03 §A.3.1 KAT 已过)",
                "Argon2id": "argon2-cffi/libargon2 (RFC 9106 §5.3 KAT 已过, parallelism=4)",
                "AES-256-KW": "cryptography (RFC 3394 §4.6 KAT 已过)",
                "HKDF-Expand": "cryptography (RFC 5869 A.1 KAT 已过)",
                "HMAC-SHA256": "cryptography (RFC 4231 TC1 KAT 已过)",
            },
            "byte_conventions": {
                "integers": "u32 big-endian",
                "jti": "uuid-v4 string ASCII",
                "object_id/manifest.id": "16 raw bytes in HKDF info & AAD; base32 string on disk/JSON",
                "HKDF info": "ASCII label + 16B raw id (or slot ASCII)",
                "chunk AAD": "b'MYVAULT'+u32(format)+jti.ascii+oid_raw+u32(idx)+u32(count)+sha256(header)",
                "manifest AAD": "b'MYVAULT'+u32(format)+jti.ascii+manifest_id_raw+u32(seq)",
                "two secrets": "passphrase & raw RK both -> Argon2id(m,t,p,salt) -> slot-separated KEK",
            },
        },
        "fixed_inputs": {
            "passphrase": PASSPHRASE,
            "passphrase_utf8_hex": PW_BYTES.hex(),
            "salt_hex": SALT.hex(),
            "master_hex": MASTER.hex(),
            "mac_key_hex": MAC_KEY.hex(),
            "recovery_key_raw_hex": RK_RAW.hex(),
            "recovery_key_display": derived["recovery_key_display"],
            "jti": JTI,
            "key_epoch": KEY_EPOCH,
            "format": FORMAT,
            "manifest_id_b32": MANIFEST_ID_B32,
            "manifest_seq": MANIFEST_SEQ,
            "base_version": BASE_VERSION,
            "device_id_hex": DEVICE_ID.hex(),
            "object_id_A_b32": OBJECT_ID_A_B32,
            "object_id_B_b32": OBJECT_ID_B_B32,
            "nonce_A_hex": NONCE_A.hex(),
            "nonce_B_hex": NONCE_B.hex(),
            "nonce_manifest_hex": NONCE_MANIFEST.hex(),
            "plaintext_A_hex": PLAINTEXT_A.hex(),
            "plaintext_B_hex(empty)": PLAINTEXT_B.hex(),
            "kdf": {"alg": "argon2id", "m": KDF_M, "t": KDF_T, "p": KDF_P,
                    "tag_len": KDF_TAGLEN, "salt_len": len(SALT)},
        },
        "derived_outputs": derived,
        "verify_phase1_json": p1,
        "verify_phase2": {"ok": res["ok"], "records": res["records"],
                          "orphans": res["orphans"], "E_ORPHAN": res["E_ORPHAN"]},
        "export": {k: v.hex() for k, v in res["exported"].items()},
        "negative_vectors": neg,
        "summary": {
            "positive_pipeline": "PASS (create -> usb copy -> unlock -> verify -> export)",
            "negative_vectors_matched": "%d/%d" % (npass, len(neg)),
        },
    }
    vf = here / "test-vectors.json"
    vf.write_text(json.dumps(vectors, ensure_ascii=False, indent=2), encoding="utf-8")
    print("[7] wrote", vf.name, "(%d bytes)" % vf.stat().st_size)

    ok = (res["ok"] and npass == len(neg))
    print("=" * 72)
    print("RESULT:", "ALL PASS" if ok else "ATTENTION — see **CHECK** lines above")
    print("=" * 72)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
