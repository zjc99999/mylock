//! mylock-core —— M2 内核实现（Rust）
//!
//! 逐字节复现 `poc/test-vectors.json`（vault-format-spec.md v1.1.1 §9 唯一契约）。
//! 设计原则：所有密码学运算委托被审计过的 crate（argon2 / aes-kw /
//! chacha20poly1305 / hkdf / hmac / sha2），本 crate 只负责规范冻结的
//! 「字节级拼接」与「解锁顺序」。
//!
//! 注意：XChaCha20-Poly1305 基于 expired IRTF draft（非 IETF 标准）；
//! 本 crate 是内核实现，不是安全认证、不是第三方审计通过、不是产品发布。

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use aes_kw::KwAes256;
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use data_encoding::BASE32_NOPAD;
use hkdf::Hkdf;
use hmac::{Hmac, KeyInit as _, Mac};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// G4 跨平台一致性验证：把 conformance.rs 的私有契约算法提升为公开 API，
/// 供桌面壳 / Android Tauri 命令复用同一份实现（避免两份实现漂移）。
pub mod g4;

// ---------------------------------------------------------------------------
// 冻结常量（vault-format-spec.md §2.1 / PoC 常量）
// ---------------------------------------------------------------------------
pub const MAGIC_VAULT: &[u8] = b"MYVAULT";
pub const MAGIC_OBJ: &[u8] = b"MYOB";
pub const OBJ_VERSION: u8 = 1;
pub const CIPHER_ID: u8 = 1;
pub const CHUNK_SIZE: u32 = 1_048_576; // 1 MiB
pub const CIPHER_NAME: &str = "XCHACHA20-POLY1305";
pub const README: &str = "This is a mylock vault (format v1). All user data is encrypted. Use the recovery CLI to unlock. No account, no cloud, no telemetry.\n";

pub const E_FORMAT: &str = "E_FORMAT";
pub const E_AUTH_FAILED: &str = "E_AUTH_FAILED";
pub const E_DOWNGRADE: &str = "E_DOWNGRADE";
pub const E_MANIFEST: &str = "E_MANIFEST";
pub const E_CORRUPT: &str = "E_CORRUPT";
pub const E_MISSING: &str = "E_MISSING";
pub const E_ORPHAN: &str = "E_ORPHAN"; // 警告，非致命

// ---------------------------------------------------------------------------
// 错误类型
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct CoreError {
    pub code: &'static str,
    pub detail: String,
}

impl CoreError {
    pub fn new(code: &'static str, detail: impl Into<String>) -> Self {
        CoreError { code, detail: detail.into() }
    }
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}
impl std::error::Error for CoreError {}

fn e(code: &'static str, detail: impl Into<String>) -> CoreError {
    CoreError::new(code, detail)
}
fn io(detail: impl std::fmt::Display) -> CoreError {
    CoreError::new(E_FORMAT, format!("io: {}", detail))
}

// ---------------------------------------------------------------------------
// 底层原语（全部委托被审计过的 crate）
// ---------------------------------------------------------------------------
fn u32be(n: u32) -> [u8; 4] {
    n.to_be_bytes()
}

fn sha256(b: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(b);
    h.finalize().to_vec()
}

fn b64e(b: &[u8]) -> String {
    B64.encode(b)
}
pub fn b64d(s: &str) -> Result<Vec<u8>, CoreError> {
    B64.decode(s).map_err(|x| e(E_FORMAT, format!("base64: {}", x)))
}

/// 128-bit（或任意长度）→ RFC4648 base32 大写无填充。
pub fn b32e(b: &[u8]) -> String {
    BASE32_NOPAD.encode(b)
}
pub fn b32d(s: &str) -> Result<Vec<u8>, CoreError> {
    BASE32_NOPAD
        .decode(s.as_bytes())
        .map_err(|x| e(E_FORMAT, format!("base32: {}", x)))
}

/// 仅 HKDF-Expand（RFC 5869 §3.3：输入已是均匀 PRK，跳 Extract）。
fn hkdf_expand(prk: &[u8], info: &[u8]) -> Result<Vec<u8>, CoreError> {
    let hk = Hkdf::<Sha256>::from_prk(prk)
        .map_err(|x| e(E_FORMAT, format!("hkdf from_prk: {}", x)))?;
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .map_err(|x| e(E_FORMAT, format!("hkdf expand: {}", x)))?;
    Ok(okm.to_vec())
}

/// Argon2id(m, t, p, salt, tag=32B, v=0x13, type=ID)，显式 parallelism。
fn argon2id(
    secret: &[u8],
    salt: &[u8],
    m: u32,
    t: u32,
    p: u32,
    out_len: usize,
) -> Result<Vec<u8>, CoreError> {
    let params = Params::new(m, t, p, Some(out_len))
        .map_err(|x| e(E_FORMAT, format!("argon2 params: {}", x)))?;
    let a = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = vec![0u8; out_len];
    a.hash_password_into(secret, salt, &mut out)
        .map_err(|x| e(E_FORMAT, format!("argon2: {}", x)))?;
    Ok(out)
}

fn aes_kw_wrap(kek: &[u8], key: &[u8]) -> Result<Vec<u8>, CoreError> {
    use aes_kw::KeyInit as _;
    let kw = KwAes256::new_from_slice(kek)
        .map_err(|x| e(E_FORMAT, format!("KwAes256 key: {}", x)))?;
    let mut buf = vec![0u8; key.len() + 8];
    let out = kw
        .wrap_key(key, &mut buf)
        .map_err(|x| e(E_AUTH_FAILED, format!("aes-kw wrap: {}", x)))?;
    Ok(out.to_vec())
}

fn aes_kw_unwrap(kek: &[u8], wrapped: &[u8]) -> Result<Vec<u8>, CoreError> {
    use aes_kw::KeyInit as _;
    let kw = KwAes256::new_from_slice(kek)
        .map_err(|x| e(E_FORMAT, format!("KwAes256 key: {}", x)))?;
    if wrapped.len() < 8 {
        return Err(e(E_AUTH_FAILED, "wrapped too short"));
    }
    let mut buf = vec![0u8; wrapped.len() - 8];
    let out = kw
        .unwrap_key(wrapped, &mut buf)
        .map_err(|x| e(E_AUTH_FAILED, format!("aes-kw unwrap: {}", x)))?;
    Ok(out.to_vec())
}

fn xchacha_encrypt(key: &[u8], nonce: &[u8], pt: &[u8], aad: &[u8]) -> Result<Vec<u8>, CoreError> {
    use chacha20poly1305::KeyInit as _;
    let c = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|x| e(E_FORMAT, format!("xchacha key: {}", x)))?;
    let n: XNonce = core::convert::TryFrom::try_from(nonce)
        .map_err(|_| e(E_FORMAT, "xchacha nonce must be 24 bytes"))?;
    c.encrypt(&n, Payload { msg: pt, aad })
        .map_err(|x| e(E_CORRUPT, format!("xchacha encrypt: {}", x)))
}

fn xchacha_decrypt(
    key: &[u8],
    nonce: &[u8],
    combined: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CoreError> {
    use chacha20poly1305::KeyInit as _;
    let c = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|x| e(E_FORMAT, format!("xchacha key: {}", x)))?;
    let n: XNonce = core::convert::TryFrom::try_from(nonce)
        .map_err(|_| e(E_FORMAT, "xchacha nonce must be 24 bytes"))?;
    c.decrypt(&n, Payload { msg: combined, aad })
        .map_err(|x| e(E_CORRUPT, format!("xchacha decrypt: {}", x)))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut m = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    m.update(data);
    m.finalize().into_bytes().to_vec()
}

/// 常量时间比较（长度不等直接 false）。
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut d = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        d |= x ^ y;
    }
    d == 0
}

// ---------------------------------------------------------------------------
// canonical JSON（vault-format-spec.md §3.3 八条，逐字节契约）
// ---------------------------------------------------------------------------
/// 把 serde_json::Value 序列化为规范 JSON（UTF-8）。
/// - 对象键按 Unicode 码点升序（UTF-8 字节序等价）
/// - 字符串转义集合封闭：`\"` / `\\` / 控制字符一律 `\u00xx`（小写）
/// - 非 ASCII 原样 UTF-8；整数十进制；禁浮点；无冗余空白
pub fn canonical(v: &Value) -> Result<Vec<u8>, CoreError> {
    let mut out = Vec::new();
    enc(v, &mut out)?;
    Ok(out)
}

fn enc_str(s: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for ch in s.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            c if (c as u32) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes());
            }
            c => {
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    out.push(b'"');
}

fn enc(v: &Value, out: &mut Vec<u8>) -> Result<(), CoreError> {
    match v {
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Null => out.extend_from_slice(b"null"),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.extend_from_slice(i.to_string().as_bytes());
            } else if let Some(u) = n.as_u64() {
                out.extend_from_slice(u.to_string().as_bytes());
            } else {
                return Err(e(E_FORMAT, "canonical: float/exponent not allowed"));
            }
        }
        Value::String(s) => enc_str(s, out),
        Value::Array(a) => {
            out.push(b'[');
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                enc(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(m) => {
            out.push(b'{');
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                enc_str(k, out);
                out.push(b':');
                enc(&m[*k], out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 字节级拼接（§2.1 冻结）
// ---------------------------------------------------------------------------
pub fn aad_chunk(
    jti: &str,
    format: u32,
    oid_raw: &[u8],
    chunk_index: u32,
    chunk_count: u32,
    header_hash: &[u8],
) -> Vec<u8> {
    let mut a = Vec::new();
    a.extend_from_slice(MAGIC_VAULT);
    a.extend_from_slice(&u32be(format));
    a.extend_from_slice(jti.as_bytes());
    a.extend_from_slice(oid_raw);
    a.extend_from_slice(&u32be(chunk_index));
    a.extend_from_slice(&u32be(chunk_count));
    a.extend_from_slice(header_hash);
    a
}

pub fn aad_manifest(jti: &str, format: u32, manifest_id_raw: &[u8], seq: u32) -> Vec<u8> {
    let mut a = Vec::new();
    a.extend_from_slice(MAGIC_VAULT);
    a.extend_from_slice(&u32be(format));
    a.extend_from_slice(jti.as_bytes());
    a.extend_from_slice(manifest_id_raw);
    a.extend_from_slice(&u32be(seq));
    a
}

/// 对象 header 固定 32 字节。
pub fn object_header(oid_raw: &[u8], chunk_count: u32) -> [u8; 32] {
    let mut h = [0u8; 32];
    h[0..4].copy_from_slice(MAGIC_OBJ);
    h[4] = OBJ_VERSION;
    h[5..21].copy_from_slice(oid_raw);
    h[21..25].copy_from_slice(&CHUNK_SIZE.to_be_bytes());
    h[25..29].copy_from_slice(&chunk_count.to_be_bytes());
    h[29] = CIPHER_ID;
    // h[30..32] = 0 (reserved)
    h
}

/// Recovery Key 展示串（§2 L107）：base32(52) + 4 校验位 + key_epoch 前缀，13+1 组。
pub fn rk_display(rk_raw: &[u8], epoch: u32) -> String {
    let b = b32e(rk_raw);
    let chk = hex::encode(&sha256(rk_raw)[..2]).to_uppercase();
    let body = format!("{}{}", b, chk);
    let mut groups = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let end = (i + 4).min(bytes.len());
        groups.push(String::from_utf8_lossy(&bytes[i..end]).to_string());
        i = end;
    }
    format!("E{:03}-{}", epoch, groups.join(" "))
}

/// 把展示串还原为 RK 原始 32 字节（含校验位验证）。
pub fn recovery_display_to_raw(s: &str) -> Result<(u32, Vec<u8>), CoreError> {
    let (epoch, body_spaced) = match s.split_once('-') {
        Some((p, b)) if p.starts_with('E') && p.len() == 4 => (
            p[1..]
                .parse::<u32>()
                .map_err(|_| e(E_FORMAT, "bad epoch in recovery display"))?,
            b,
        ),
        _ => return Err(e(E_FORMAT, "recovery display must start with E###-")),
    };
    let body: String = body_spaced.chars().filter(|c| *c != ' ').collect();
    if body.len() < 56 {
        return Err(e(E_FORMAT, "recovery display too short"));
    }
    let b32part = &body[..52];
    let chk = body[52..].to_uppercase();
    let raw = b32d(b32part)?;
    let expect = hex::encode(&sha256(&raw)[..2]).to_uppercase();
    if !ct_eq(chk.as_bytes(), expect.as_bytes()) {
        return Err(e(E_AUTH_FAILED, "recovery display checksum mismatch"));
    }
    Ok((epoch, raw))
}

// ---------------------------------------------------------------------------
// 通用建库数据结构
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct KdfParams {
    pub m: u32,
    pub t: u32,
    pub p: u32,
    pub tag_len: usize,
    pub salt_len: usize,
}

#[derive(Debug, Clone)]
pub struct VaultParams {
    pub master: Vec<u8>,
    pub mac_key: Vec<u8>,
    pub salt: Vec<u8>,
    pub jti: String,
    pub key_epoch: u32,
    pub format: u32,
    pub kdf: KdfParams,
    pub manifest_id_raw: Vec<u8>,
    pub manifest_seq: u32,
    pub base_version: u32,
    pub device_id: Vec<u8>,
    pub password: Vec<u8>,
    pub rk_raw: Vec<u8>,
    pub manifest_nonce: Vec<u8>,
    pub cipher: String,
}

#[derive(Debug, Clone)]
pub struct ObjectSpec {
    pub oid_raw: Vec<u8>,
    pub name: String,
    pub kind: String,
    pub parent_id_raw: Vec<u8>,
    pub plaintext: Vec<u8>,
    /// 每 chunk 一个 24B CSPRNG nonce（§4.2：分块 1 MiB，每 chunk 独立随机 nonce）。
    /// 长度必须 = chunk_count；长度不足由 `random_object_nonces` 生成。
    pub nonces: Vec<Vec<u8>>,
    pub updated_seq: u64,
    pub content_version: u64,
}

/// 按 §4.2 把明文切成 1 MiB 块，返回每块一个随机 24B nonce。
/// 空明文仍产生 1 个（带 tag 的）空 chunk（N4）。
pub fn random_object_nonces(plaintext_len: usize) -> Result<Vec<Vec<u8>>, CoreError> {
    let n = chunk_count_for(plaintext_len);
    (0..n).map(|_| random_bytes(24)).collect()
}

/// 明文长度 → chunk 数（至少 1）。供写入与 nonce 生成共用，避免两处口径漂移。
pub fn chunk_count_for(plaintext_len: usize) -> usize {
    if plaintext_len == 0 {
        1
    } else {
        (plaintext_len + CHUNK_SIZE as usize - 1) / CHUNK_SIZE as usize
    }
}

#[derive(Debug, Clone)]
pub struct CreatedObject {
    pub oid_raw: Vec<u8>,
    pub oid_b32: String,
    pub object_key: Vec<u8>,
    pub header: Vec<u8>,
    pub header_hash: Vec<u8>,
    pub aad: Vec<u8>,
    pub nonce: Vec<u8>,
    pub combined: Vec<u8>,
    pub file_bytes: Vec<u8>,
    pub record: Value,
}

#[derive(Debug, Clone)]
pub struct CreatedVault {
    pub ikm_password: Vec<u8>,
    pub ikm_recovery: Vec<u8>,
    pub kek_password: Vec<u8>,
    pub kek_recovery: Vec<u8>,
    pub wrapped_password: Vec<u8>,
    pub wrapped_recovery: Vec<u8>,
    pub vmac_key: Vec<u8>,
    pub mac_input: Vec<u8>,
    pub version_mac: Vec<u8>,
    pub meta_core: Value,
    pub meta_full_bytes: Vec<u8>,
    pub manifest_key: Vec<u8>,
    pub manifest_plaintext: Vec<u8>,
    pub manifest_aad: Vec<u8>,
    pub manifest_nonce: Vec<u8>,
    pub manifest_combined: Vec<u8>,
    pub manifest_file: Vec<u8>,
    pub objects: Vec<CreatedObject>,
}

/// 在内存中构建完整 vault（不写磁盘），通用路径。
pub fn build_vault(p: &VaultParams, objects: &[ObjectSpec]) -> Result<CreatedVault, CoreError> {
    let ikm_pw = Zeroizing::new(argon2id(
        &p.password, &p.salt, p.kdf.m, p.kdf.t, p.kdf.p, p.kdf.tag_len,
    )?);
    let ikm_rk = Zeroizing::new(argon2id(
        &p.rk_raw, &p.salt, p.kdf.m, p.kdf.t, p.kdf.p, p.kdf.tag_len,
    )?);
    let kek_pw = Zeroizing::new(hkdf_expand(ikm_pw.as_slice(), b"wrap/password")?);
    let kek_rk = Zeroizing::new(hkdf_expand(ikm_rk.as_slice(), b"wrap/recovery")?);

    let blob = [p.master.as_slice(), p.mac_key.as_slice()].concat(); // 64B = 8 块
    let wrapped_pw = aes_kw_wrap(kek_pw.as_slice(), &blob)?;
    let wrapped_rk = aes_kw_wrap(kek_rk.as_slice(), &blob)?;

    let vmac_key = hkdf_expand(&p.mac_key, b"meta-version-mac")?;
    let meta_core = json!({
        "magic": "MYVAULT",
        "format": p.format,
        "jti": p.jti,
        "key_epoch": p.key_epoch,
        "cipher": p.cipher,
        "kdf": {
            "alg": "argon2id",
            "m": p.kdf.m,
            "t": p.kdf.t,
            "p": p.kdf.p,
            "salt": b64e(&p.salt),
        },
        "wrapped": [
            { "slot": "password", "alg": "RFC3394-AES-KW", "blob": b64e(&wrapped_pw) },
            { "slot": "recovery", "alg": "RFC3394-AES-KW", "blob": b64e(&wrapped_rk) },
        ],
        "manifest_ref": { "id": b32e(&p.manifest_id_raw), "seq": p.manifest_seq },
        "gc_pending": false,
    });
    let mac_input = canonical(&meta_core)?;
    let version_mac = hmac_sha256(&vmac_key, &mac_input);
    let mut mf = meta_core.as_object().expect("meta_core object").clone();
    mf.insert("versionMac".into(), Value::String(b64e(&version_mac)));
    let meta_full_bytes = canonical(&Value::Object(mf))?;

    let mut created_objects = Vec::new();
    for o in objects {
        let object_key =
            hkdf_expand(&p.master, &[b"object".as_slice(), o.oid_raw.as_slice()].concat())?;
        let count = chunk_count_for(o.plaintext.len()) as u32;
        if o.nonces.len() != count as usize {
            return Err(e(E_FORMAT, format!("nonce count {} != chunk count {}", o.nonces.len(), count)));
        }
        let header = object_header(&o.oid_raw, count);
        let header_hash = sha256(&header);
        let mut file_bytes = Vec::new();
        file_bytes.extend_from_slice(&header);
        let mut combined_all = Vec::new();
        let mut first_aad = Vec::new();
        for (i, nonce) in o.nonces.iter().enumerate() {
            let start = i * CHUNK_SIZE as usize;
            let end = ((i + 1) * CHUNK_SIZE as usize).min(o.plaintext.len());
            let piece = &o.plaintext[start..end];
            let aad = aad_chunk(
                &p.jti, p.format, &o.oid_raw, i as u32, count, &header_hash,
            );
            if i == 0 {
                first_aad = aad.clone();
            }
            let combined = xchacha_encrypt(&object_key, nonce, piece, &aad)?;
            let ct = &combined[..combined.len() - 16];
            let tag = &combined[combined.len() - 16..];
            file_bytes.extend_from_slice(nonce);
            file_bytes.extend_from_slice(&(ct.len() as u32).to_be_bytes());
            file_bytes.extend_from_slice(ct);
            file_bytes.extend_from_slice(tag);
            combined_all.extend_from_slice(&combined);
        }
        let oid_b32 = b32e(&o.oid_raw);
        let record = json!({
            "id": oid_b32,
            "parent_id": b32e(&o.parent_id_raw),
            "name": o.name,
            "kind": o.kind,
            "size": o.plaintext.len(),
            "chunk_count": count,
            "base_version": p.base_version,
            "device_id": b64e(&p.device_id),
            "updated_seq": o.updated_seq,
            "content_version": o.content_version,
            "manifest_seq": p.manifest_seq,
        });
        created_objects.push(CreatedObject {
            oid_raw: o.oid_raw.clone(),
            oid_b32,
            object_key,
            header: header.to_vec(),
            header_hash,
            aad: first_aad,
            nonce: o.nonces.first().cloned().unwrap_or_default(),
            combined: combined_all,
            file_bytes,
            record,
        });
    }

    let manifest_key = hkdf_expand(
        &p.master,
        &[b"manifest".as_slice(), p.manifest_id_raw.as_slice()].concat(),
    )?;
    let records: Vec<Value> = created_objects.iter().map(|o| o.record.clone()).collect();
    let manifest_plaintext = canonical(&Value::Array(records))?;
    let manifest_aad = aad_manifest(&p.jti, p.format, &p.manifest_id_raw, p.manifest_seq);
    let manifest_combined =
        xchacha_encrypt(&manifest_key, &p.manifest_nonce, &manifest_plaintext, &manifest_aad)?;
    let manifest_file = [p.manifest_nonce.as_slice(), manifest_combined.as_slice()].concat();

    Ok(CreatedVault {
        ikm_password: ikm_pw.as_slice().to_vec(),
        ikm_recovery: ikm_rk.as_slice().to_vec(),
        kek_password: kek_pw.as_slice().to_vec(),
        kek_recovery: kek_rk.as_slice().to_vec(),
        wrapped_password: wrapped_pw,
        wrapped_recovery: wrapped_rk,
        vmac_key,
        mac_input,
        version_mac,
        meta_core,
        meta_full_bytes,
        manifest_key,
        manifest_plaintext,
        manifest_aad,
        manifest_nonce: p.manifest_nonce.clone(),
        manifest_combined,
        manifest_file,
        objects: created_objects,
    })
}

/// 把内存 vault 写成静态密文容器（§6.8「不依赖 OS 的保险柜」）。
pub fn write_vault(root: &Path, c: &CreatedVault) -> Result<(), CoreError> {
    if root.exists() {
        fs::remove_dir_all(root).map_err(io)?;
    }
    fs::create_dir_all(root.join("d")).map_err(io)?;
    fs::write(root.join("vault.meta"), &c.meta_full_bytes).map_err(io)?;
    fs::write(root.join("vault.meta.bak"), &c.meta_full_bytes).map_err(io)?;
    for o in &c.objects {
        let shard = root.join("d").join(&o.oid_b32[..2]);
        fs::create_dir_all(&shard).map_err(io)?;
        fs::write(shard.join(&o.oid_b32), &o.file_bytes).map_err(io)?;
    }
    fs::write(root.join("manifest"), &c.manifest_file).map_err(io)?;
    fs::write(root.join("README.txt"), README).map_err(io)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 测试向量复现（M2 验收用；固定输入来自 poc/test-vectors.json）
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct FixedInputs {
    pub passphrase_utf8: Vec<u8>,
    pub salt: Vec<u8>,
    pub master: Vec<u8>,
    pub mac_key: Vec<u8>,
    pub rk_raw: Vec<u8>,
    pub jti: String,
    pub key_epoch: u32,
    pub format: u32,
    pub manifest_id_b32: String,
    pub manifest_seq: u32,
    pub base_version: u32,
    pub device_id: Vec<u8>,
    pub object_id_a_b32: String,
    pub object_id_b_b32: String,
    pub nonce_a: Vec<u8>,
    pub nonce_b: Vec<u8>,
    pub nonce_manifest: Vec<u8>,
    pub plaintext_a: Vec<u8>,
    pub plaintext_b: Vec<u8>,
    pub kdf: KdfParams,
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_else(|| panic!("missing string {}", k)).to_string()
}
fn h(v: &Value, k: &str) -> Vec<u8> {
    hex::decode(v[k].as_str().unwrap_or_else(|| panic!("missing hex {}", k)))
        .unwrap_or_else(|x| panic!("bad hex {}: {}", k, x))
}
fn n(v: &Value, k: &str) -> u64 {
    v[k].as_u64().unwrap_or_else(|| panic!("missing number {}", k))
}

impl FixedInputs {
    pub fn from_json(v: &Value) -> FixedInputs {
        let kdf = &v["kdf"];
        FixedInputs {
            passphrase_utf8: h(v, "passphrase_utf8_hex"),
            salt: h(v, "salt_hex"),
            master: h(v, "master_hex"),
            mac_key: h(v, "mac_key_hex"),
            rk_raw: h(v, "recovery_key_raw_hex"),
            jti: s(v, "jti"),
            key_epoch: n(v, "key_epoch") as u32,
            format: n(v, "format") as u32,
            manifest_id_b32: s(v, "manifest_id_b32"),
            manifest_seq: n(v, "manifest_seq") as u32,
            base_version: n(v, "base_version") as u32,
            device_id: h(v, "device_id_hex"),
            object_id_a_b32: s(v, "object_id_A_b32"),
            object_id_b_b32: s(v, "object_id_B_b32"),
            nonce_a: h(v, "nonce_A_hex"),
            nonce_b: h(v, "nonce_B_hex"),
            nonce_manifest: h(v, "nonce_manifest_hex"),
            plaintext_a: h(v, "plaintext_A_hex"),
            plaintext_b: h(v, "plaintext_B_hex(empty)"),
            kdf: KdfParams {
                m: n(kdf, "m") as u32,
                t: n(kdf, "t") as u32,
                p: n(kdf, "p") as u32,
                tag_len: n(kdf, "tag_len") as usize,
                salt_len: n(kdf, "salt_len") as usize,
            },
        }
    }

    pub fn to_build(&self) -> (VaultParams, Vec<ObjectSpec>) {
        let params = VaultParams {
            master: self.master.clone(),
            mac_key: self.mac_key.clone(),
            salt: self.salt.clone(),
            jti: self.jti.clone(),
            key_epoch: self.key_epoch,
            format: self.format,
            kdf: self.kdf.clone(),
            manifest_id_raw: b32d(&self.manifest_id_b32).expect("manifest id b32"),
            manifest_seq: self.manifest_seq,
            base_version: self.base_version,
            device_id: self.device_id.clone(),
            password: self.passphrase_utf8.clone(),
            rk_raw: self.rk_raw.clone(),
            manifest_nonce: self.nonce_manifest.clone(),
            cipher: CIPHER_NAME.to_string(),
        };
        let root = vec![0u8; 16];
        let objects = vec![
            ObjectSpec {
                oid_raw: b32d(&self.object_id_a_b32).expect("oid A b32"),
                name: "hello.txt".into(),
                kind: "file".into(),
                parent_id_raw: root.clone(),
                plaintext: self.plaintext_a.clone(),
                nonces: vec![self.nonce_a.clone()],
                updated_seq: 1,
                content_version: 1,
            },
            ObjectSpec {
                oid_raw: b32d(&self.object_id_b_b32).expect("oid B b32"),
                name: "empty.bin".into(),
                kind: "file".into(),
                parent_id_raw: root,
                plaintext: self.plaintext_b.clone(),
                nonces: vec![self.nonce_b.clone()],
                updated_seq: 1,
                content_version: 1,
            },
        ];
        (params, objects)
    }
}

fn derive_map(fi: &FixedInputs, c: &CreatedVault) -> BTreeMap<String, String> {
    let mut o = BTreeMap::new();
    o.insert("IKM_password".into(), hex::encode(&c.ikm_password));
    o.insert("IKM_recovery".into(), hex::encode(&c.ikm_recovery));
    o.insert("KEK_password".into(), hex::encode(&c.kek_password));
    o.insert("KEK_recovery".into(), hex::encode(&c.kek_recovery));
    o.insert("wrapped.password.blob".into(), b64e(&c.wrapped_password));
    o.insert("wrapped.recovery.blob".into(), b64e(&c.wrapped_recovery));
    o.insert("versionMac".into(), b64e(&c.version_mac));
    o.insert("versionMac.input_hex".into(), hex::encode(&c.mac_input));
    o.insert("versionMac.key_hex".into(), hex::encode(&c.vmac_key));
    o.insert("master_hex".into(), hex::encode(&fi.master));
    o.insert("mac_key_hex".into(), hex::encode(&fi.mac_key));
    o.insert("manifest_key_hex".into(), hex::encode(&c.manifest_key));
    o.insert("object_key_A_hex".into(), hex::encode(&c.objects[0].object_key));
    o.insert("object_key_B_hex".into(), hex::encode(&c.objects[1].object_key));
    o.insert("object_A_file_hex".into(), hex::encode(&c.objects[0].file_bytes));
    o.insert("object_B_file_hex(empty)".into(), hex::encode(&c.objects[1].file_bytes));
    o.insert("object_A_combined_hex".into(), hex::encode(&c.objects[0].combined));
    o.insert("object_B_combined_hex(empty)".into(), hex::encode(&c.objects[1].combined));
    o.insert("manifest_file_hex".into(), hex::encode(&c.manifest_file));
    o.insert("manifest_combined_hex".into(), hex::encode(&c.manifest_combined));
    o.insert(
        "manifest_plaintext_canonical_hex".into(),
        hex::encode(&c.manifest_plaintext),
    );
    o.insert("manifest_aad_hex".into(), hex::encode(&c.manifest_aad));
    o.insert("chunk_aad_A_hex".into(), hex::encode(&c.objects[0].aad));
    o.insert("chunk_aad_B_hex".into(), hex::encode(&c.objects[1].aad));
    o.insert("meta_canonical_hex".into(), hex::encode(&c.meta_full_bytes));
    o.insert("object_id_A_b32".into(), fi.object_id_a_b32.clone());
    o.insert("object_id_B_b32".into(), fi.object_id_b_b32.clone());
    o.insert("manifest_id_b32".into(), fi.manifest_id_b32.clone());
    o.insert("recovery_key_raw_hex".into(), hex::encode(&fi.rk_raw));
    o.insert("recovery_key_display".into(), rk_display(&fi.rk_raw, fi.key_epoch));
    o
}

/// 纯内存复现：给定固定输入，产出 test-vectors.json 的 30 个 derived_outputs。
pub fn derive_outputs(fi: &FixedInputs) -> Result<BTreeMap<String, String>, CoreError> {
    let (params, objects) = fi.to_build();
    let c = build_vault(&params, &objects)?;
    Ok(derive_map(fi, &c))
}

/// 用固定输入在磁盘上建库，并返回 derived_outputs（M2 CLI / 验收用）。
pub fn create_vault(root: &Path, fi: &FixedInputs) -> Result<BTreeMap<String, String>, CoreError> {
    let (params, objects) = fi.to_build();
    let c = build_vault(&params, &objects)?;
    write_vault(root, &c)?;
    Ok(derive_map(fi, &c))
}

// ---------------------------------------------------------------------------
// 解锁 / 两阶段 verify / 明文重建（严格按 §3.4 顺序）
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Phase1Result {
    pub result: String,
    pub objects: u64,
    pub bytes: u64,
}

pub struct ParsedChunk {
    pub nonce: Vec<u8>,
    pub ct: Vec<u8>,
    pub tag: Vec<u8>,
    pub idx: u32,
}
pub struct ParsedObject {
    pub oid: Vec<u8>,
    pub chunk_count: u32,
    pub header_hash: Vec<u8>,
    pub chunks: Vec<ParsedChunk>,
}

pub fn parse_object_file(data: &[u8]) -> Result<ParsedObject, CoreError> {
    if data.len() < 32 {
        return Err(e(E_CORRUPT, "object shorter than header"));
    }
    let header = &data[..32];
    let magic = &header[0..4];
    let obj_v = header[4];
    let oid = header[5..21].to_vec();
    let chunk_count = u32::from_be_bytes(header[25..29].try_into().unwrap());
    let cipher_id = header[29];
    if magic != MAGIC_OBJ || obj_v != OBJ_VERSION || cipher_id != CIPHER_ID {
        return Err(e(E_FORMAT, "object header magic/version/cipher"));
    }
    if chunk_count < 1 {
        return Err(e(E_CORRUPT, "chunk_count=0 forgery"));
    }
    let header_hash = sha256(header);
    let mut off = 32usize;
    let mut chunks = Vec::new();
    for i in 0..chunk_count {
        if off + 24 + 4 > data.len() {
            return Err(e(E_CORRUPT, "truncated chunk header"));
        }
        let nonce = data[off..off + 24].to_vec();
        off += 24;
        let ct_len = u32::from_be_bytes(data[off..off + 4].try_into().unwrap()) as u64;
        off += 4;
        if (off as u64) + ct_len + 16 > data.len() as u64 {
            return Err(e(E_CORRUPT, "truncated chunk body"));
        }
        let ct = data[off..off + ct_len as usize].to_vec();
        off += ct_len as usize;
        let tag = data[off..off + 16].to_vec();
        off += 16;
        chunks.push(ParsedChunk { nonce, ct, tag, idx: i });
    }
    if off != data.len() {
        return Err(e(E_CORRUPT, "trailing bytes in object"));
    }
    Ok(ParsedObject { oid, chunk_count, header_hash, chunks })
}

pub fn load_meta(root: &Path) -> Result<Value, CoreError> {
    let raw = fs::read(root.join("vault.meta")).map_err(io)?;
    let meta: Value = serde_json::from_slice(&raw)
        .map_err(|x| e(E_FORMAT, format!("meta not JSON: {}", x)))?;
    if canonical(&meta)? != raw {
        return Err(e(E_FORMAT, "meta is not canonical JSON (profile violation)"));
    }
    let obj = meta.as_object().ok_or_else(|| e(E_FORMAT, "meta not object"))?;
    let allowed: BTreeSet<&str> = [
        "magic", "format", "jti", "key_epoch", "cipher", "kdf", "wrapped",
        "manifest_ref", "versionMac", "gc_pending",
    ]
    .into_iter()
    .collect();
    let got: BTreeSet<&str> = obj.keys().map(|k| k.as_str()).collect();
    if got != allowed {
        return Err(e(E_FORMAT, "unknown/missing meta field"));
    }
    if obj.get("magic").and_then(Value::as_str) != Some("MYVAULT")
        || obj.get("format").and_then(Value::as_u64) != Some(1)
    {
        return Err(e(E_FORMAT, "magic/format mismatch"));
    }
    Ok(meta)
}

pub fn kdf_hard_check(meta: &Value) -> Result<(u32, u32, u32, Vec<u8>), CoreError> {
    let k = meta
        .get("kdf")
        .and_then(Value::as_object)
        .ok_or_else(|| e(E_FORMAT, "kdf missing"))?;
    let m = k.get("m").and_then(Value::as_u64).ok_or_else(|| e(E_FORMAT, "kdf.m"))?;
    let t = k.get("t").and_then(Value::as_u64).ok_or_else(|| e(E_FORMAT, "kdf.t"))?;
    let p = k.get("p").and_then(Value::as_u64).ok_or_else(|| e(E_FORMAT, "kdf.p"))?;
    if !(19456 <= m && m <= 1048576 && 2 <= t && t <= 10 && 1 <= p && p <= 16) {
        return Err(e(E_FORMAT, "kdf params out of bounds (F14/N3)"));
    }
    let salt_b64 = k
        .get("salt")
        .and_then(Value::as_str)
        .ok_or_else(|| e(E_FORMAT, "kdf.salt"))?;
    let salt = b64d(salt_b64)?;
    Ok((m as u32, t as u32, p as u32, salt))
}

fn list_object_files(root: &Path) -> Vec<(String, std::path::PathBuf)> {
    let mut out = Vec::new();
    let d = root.join("d");
    let mut shards: Vec<_> = match fs::read_dir(&d) {
        Ok(rd) => rd.filter_map(|x| x.ok()).map(|x| x.path()).collect(),
        Err(_) => return out,
    };
    shards.sort();
    for shard in shards {
        if !shard.is_dir() {
            continue;
        }
        let mut files: Vec<_> = match fs::read_dir(&shard) {
            Ok(rd) => rd.filter_map(|x| x.ok()).map(|x| x.path()).collect(),
            Err(_) => continue,
        };
        files.sort();
        for f in files {
            if f.is_file() {
                let name = f.file_name().unwrap_or_default().to_string_lossy().to_string();
                out.push((name, f));
            }
        }
    }
    out
}

pub fn verify_phase1(root: &Path) -> Result<Phase1Result, CoreError> {
    let meta = load_meta(root)?;
    let _ = kdf_hard_check(&meta)?;
    let mut objects = 0u64;
    let mut total_bytes = 0u64;
    let mut anomalies = 0usize;
    for (_, path) in list_object_files(root) {
        objects += 1;
        let data = fs::read(&path).map_err(io)?;
        total_bytes += data.len() as u64;
        if parse_object_file(&data).is_err() {
            anomalies += 1; // 不早停、不区分具体对象
        }
    }
    Ok(Phase1Result {
        result: if anomalies == 0 { "OK".into() } else { "STRUCT_ANOMALY".into() },
        objects,
        bytes: total_bytes,
    })
}

#[derive(Debug, Clone)]
pub struct Unwrapped {
    pub meta: Value,
    pub master: Vec<u8>,
    pub mac_key: Vec<u8>,
}

/// §3.4 步骤 ①–⑧：解包 + versionMac 全头认证，返回 Master/MAC_Key。
pub fn unwrap_master(root: &Path, secret: &[u8], slot: &str) -> Result<Unwrapped, CoreError> {
    let _p1 = verify_phase1(root)?;
    let meta = load_meta(root)?;
    let (m, t, p, salt) = kdf_hard_check(&meta)?;

    let ikm = Zeroizing::new(argon2id(secret, &salt, m, t, p, 32)?);
    let info = format!("wrap/{}", slot);
    let kek = Zeroizing::new(hkdf_expand(ikm.as_slice(), info.as_bytes())?);

    let wrapped = meta
        .get("wrapped")
        .and_then(Value::as_array)
        .ok_or_else(|| e(E_AUTH_FAILED, "wrapped missing"))?;
    let mut blob_b64: Option<&str> = None;
    for w in wrapped {
        if w.get("slot").and_then(Value::as_str) == Some(slot) {
            blob_b64 = w.get("blob").and_then(Value::as_str);
        }
    }
    let blob_b64 = blob_b64.ok_or_else(|| e(E_AUTH_FAILED, "no such slot"))?;
    let wp = b64d(blob_b64)?;
    let blob = Zeroizing::new(aes_kw_unwrap(kek.as_slice(), &wp)?);
    if blob.len() != 64 {
        return Err(e(E_AUTH_FAILED, "unwrapped blob length"));
    }
    let master = blob[..32].to_vec();
    let mac_key = blob[32..].to_vec();

    let vmac_key = hkdf_expand(&mac_key, b"meta-version-mac")?;
    let mut core = meta.as_object().expect("meta object").clone();
    let stored = b64d(
        meta.get("versionMac")
            .and_then(Value::as_str)
            .ok_or_else(|| e(E_DOWNGRADE, "versionMac missing"))?,
    )?;
    core.remove("versionMac");
    let expected = hmac_sha256(&vmac_key, &canonical(&Value::Object(core))?);
    if !ct_eq(&expected, &stored) {
        return Err(e(E_DOWNGRADE, "versionMac mismatch (whole-header auth)"));
    }
    Ok(Unwrapped { meta, master, mac_key })
}

#[derive(Debug, Clone)]
pub struct OpenResult {
    pub phase1: Phase1Result,
    pub records: usize,
    pub records_raw: Vec<Value>,
    /// 按文件名（明文 name）索引的重建明文，供 `export` 使用。
    pub exported: BTreeMap<String, Vec<u8>>,
    /// 按记录 id（base32 object_id 字符串）索引的重建明文，供 `rotate_keys` 全量重加密使用。
    pub exported_by_id: BTreeMap<String, Vec<u8>>,
    pub orphans: Vec<String>,
}

/// §3.4 完整解锁 + 阶段2 验证 + 明文重建。
pub fn full_open(root: &Path, secret: &[u8], slot: &str) -> Result<OpenResult, CoreError> {
    let p1 = verify_phase1(root)?;
    let u = unwrap_master(root, secret, slot)?;
    let meta = &u.meta;

    let jti = meta.get("jti").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "jti"))?;
    let format = meta.get("format").and_then(Value::as_u64).unwrap_or(1) as u32;
    let mref = meta.get("manifest_ref").and_then(Value::as_object).ok_or_else(|| e(E_MANIFEST, "manifest_ref"))?;
    let mid_b32 = mref.get("id").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "manifest_ref.id"))?;
    let seq = mref.get("seq").and_then(Value::as_u64).unwrap_or(1) as u32;
    let mid = b32d(mid_b32)?;

    let manifest_key = hkdf_expand(
        &u.master,
        &[b"manifest".as_slice(), mid.as_slice()].concat(),
    )?;
    let mfile = fs::read(root.join("manifest")).map_err(io)?;
    if mfile.len() < 24 {
        return Err(e(E_MANIFEST, "manifest too short"));
    }
    let (m_nonce, m_comb) = mfile.split_at(24);
    let aad_m = aad_manifest(jti, format, &mid, seq);
    let m_pt = xchacha_decrypt(&manifest_key, m_nonce, m_comb, &aad_m)
        .map_err(|_| e(E_MANIFEST, "manifest AEAD tag failed"))?;
    let records: Vec<Value> = serde_json::from_slice(&m_pt)
        .map_err(|_| e(E_MANIFEST, "manifest not JSON"))?;
    let rec_seqs: BTreeSet<u64> = records
        .iter()
        .filter_map(|r| r.get("manifest_seq").and_then(Value::as_u64))
        .collect();
    if !rec_seqs.is_empty() && !rec_seqs.contains(&(seq as u64)) {
        return Err(e(E_MANIFEST, "manifest.seq != manifest_ref.seq"));
    }

    let mut exported = BTreeMap::new();
    let mut exported_by_id = BTreeMap::new();
    let d = root.join("d");
    let mut on_disk: BTreeSet<String> = BTreeSet::new();
    for (name, _) in list_object_files(root) {
        on_disk.insert(name);
    }
    for rec in &records {
        let b32id = rec.get("id").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "record.id"))?;
        let name = rec.get("name").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "record.name"))?;
        let path = d.join(&b32id[..2]).join(b32id);
        if !path.exists() {
            return Err(e(E_MISSING, "manifest refs missing object"));
        }
        let data = fs::read(&path).map_err(io)?;
        let parsed = parse_object_file(&data)?;
        let oid_raw = b32d(b32id)?;
        let object_key = hkdf_expand(
            &u.master,
            &[b"object".as_slice(), oid_raw.as_slice()].concat(),
        )?;
        let mut pts = Vec::new();
        for ch in &parsed.chunks {
            let aad = aad_chunk(jti, format, &oid_raw, ch.idx, parsed.chunk_count, &parsed.header_hash);
            let combined = [ch.ct.as_slice(), ch.tag.as_slice()].concat();
            let pt = xchacha_decrypt(&object_key, &ch.nonce, &combined, &aad)
                .map_err(|_| e(E_CORRUPT, "chunk AEAD tag/AAD failed"))?;
            pts.push(pt);
        }
        let plain = pts.concat();
        exported.insert(name.to_string(), plain.clone());
        exported_by_id.insert(b32id.to_string(), plain);
    }
    let referenced: BTreeSet<String> = records
        .iter()
        .filter_map(|r| r.get("id").and_then(Value::as_str).map(|x| x.to_string()))
        .collect();
    let orphans: Vec<String> = on_disk.difference(&referenced).cloned().collect();

    Ok(OpenResult {
        phase1: p1,
        records: records.len(),
        records_raw: records,
        exported,
        exported_by_id,
        orphans,
    })
}

/// 独立恢复路径（§6.2 `export --object`）：manifest 损坏时凭 object_id 直接解密
/// 单个对象，**不读 manifest**。解包与全头 versionMac 认证仍按 §3.4 完整执行。
pub fn export_object_by_id(
    root: &Path,
    secret: &[u8],
    slot: &str,
    id_b32: &str,
) -> Result<Vec<u8>, CoreError> {
    let u = unwrap_master(root, secret, slot)?;
    let meta = &u.meta;
    let jti = meta.get("jti").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "jti"))?;
    let format = meta.get("format").and_then(Value::as_u64).unwrap_or(1) as u32;
    let oid_raw = b32d(id_b32)?;
    let path = root.join("d").join(&id_b32[..2]).join(id_b32);
    if !path.exists() {
        return Err(e(E_MISSING, "object not found"));
    }
    let data = fs::read(&path).map_err(io)?;
    let parsed = parse_object_file(&data)?;
    let object_key = hkdf_expand(
        &u.master,
        &[b"object".as_slice(), oid_raw.as_slice()].concat(),
    )?;
    let mut out = Vec::new();
    for ch in &parsed.chunks {
        let aad = aad_chunk(jti, format, &oid_raw, ch.idx, parsed.chunk_count, &parsed.header_hash);
        let combined = [ch.ct.as_slice(), ch.tag.as_slice()].concat();
        let pt = xchacha_decrypt(&object_key, &ch.nonce, &combined, &aad)
            .map_err(|_| e(E_CORRUPT, "chunk AEAD tag/AAD failed"))?;
        out.extend_from_slice(&pt);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// 凭据轮换（零吊销 / 真吊销）与随机建库
// ---------------------------------------------------------------------------
/// CSPRNG 随机字节（getrandom 0.4，Windows 后端由 cfg 自动选择）。
pub fn random_bytes(n: usize) -> Result<Vec<u8>, CoreError> {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).map_err(|x| e(E_FORMAT, format!("csp rng: {}", x)))?;
    Ok(b)
}

fn default_kdf() -> KdfParams {
    KdfParams { m: 65536, t: 3, p: 4, tag_len: 32, salt_len: 16 }
}

fn random_uuid_v4() -> Result<String, CoreError> {
    let mut b = random_bytes(16)?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    Ok(format!(
        "{}-{}-{}-{}-{}",
        hex::encode(&b[0..4]),
        hex::encode(&b[4..6]),
        hex::encode(&b[6..8]),
        hex::encode(&b[8..10]),
        hex::encode(&b[10..16]),
    ))
}

/// 用真正的随机密钥 / ID / nonce 构造通用 `VaultParams`（产品建库路径）。
pub fn random_vault_params(
    password: &[u8],
    rk_raw: &[u8],
    key_epoch: u32,
) -> Result<VaultParams, CoreError> {
    Ok(VaultParams {
        master: random_bytes(32)?,
        mac_key: random_bytes(32)?,
        salt: random_bytes(16)?,
        jti: random_uuid_v4()?,
        key_epoch,
        format: 1,
        kdf: default_kdf(),
        manifest_id_raw: random_bytes(16)?,
        manifest_seq: 1,
        base_version: 0,
        device_id: random_bytes(16)?,
        password: password.to_vec(),
        rk_raw: rk_raw.to_vec(),
        manifest_nonce: random_bytes(24)?,
        cipher: CIPHER_NAME.to_string(),
    })
}

/// 零吊销换凭据（N2/F17）：用 `auth_slot` 的当前凭据解包，仅重包 `target_slot`
/// 并重算 versionMac，全库数据不重加密。
/// 诚实声明：持有旧 `vault.meta` + 旧凭据者仍可解当前数据；真吊销须 `rotate_keys`。
pub fn change_credential(
    root: &Path,
    current_secret: &[u8],
    auth_slot: &str,
    target_slot: &str,
    new_secret: &[u8],
) -> Result<(), CoreError> {
    let u = unwrap_master(root, current_secret, auth_slot)?;
    let (m, t, p, salt) = kdf_hard_check(&u.meta)?;
    let kdf = KdfParams { m, t, p, tag_len: 32, salt_len: salt.len() };
    rewrap_and_sign(root, &u.master, &u.mac_key, target_slot, new_secret, &kdf, &salt)
}

/// 真吊销（N2/F17）：新 Master / MAC / salt / jti / RK，`key_epoch++`，全量重加密到新 root。
/// 旧介质必须由用户物理销毁；本函数不删除旧 root。
/// 返回 `(新 Recovery Key 展示串, 新 jti)`。
pub fn rotate_keys(
    old_root: &Path,
    new_root: &Path,
    old_secret: &[u8],
    old_slot: &str,
    new_password: &[u8],
) -> Result<(String, String), CoreError> {
    let opened = full_open(old_root, old_secret, old_slot)?;
    let old_meta = load_meta(old_root)?;
    let old_epoch = old_meta.get("key_epoch").and_then(Value::as_u64).unwrap_or(0) as u32;
    let new_epoch = old_epoch + 1;
    let new_rk = random_bytes(32)?;
    let params = random_vault_params(new_password, &new_rk, new_epoch)?;
    let mut specs = Vec::new();
    for rec in &opened.records_raw {
        let id = rec.get("id").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "record.id"))?;
        let name = rec.get("name").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "record.name"))?;
        let kind = rec.get("kind").and_then(Value::as_str).unwrap_or("file").to_string();
        let parent = rec.get("parent_id").and_then(Value::as_str).ok_or_else(|| e(E_MANIFEST, "record.parent_id"))?;
        let us = rec.get("updated_seq").and_then(Value::as_u64).unwrap_or(1);
        let cv = rec.get("content_version").and_then(Value::as_u64).unwrap_or(1);
        let pt = opened.exported_by_id.get(id).cloned().unwrap_or_default();
        let nonces = random_object_nonces(pt.len())?;
        specs.push(ObjectSpec {
            oid_raw: random_bytes(16)?,
            name: name.to_string(),
            kind,
            parent_id_raw: b32d(parent)?,
            plaintext: pt,
            nonces,
            updated_seq: us,
            content_version: cv,
        });
    }
    let c = build_vault(&params, &specs)?;
    write_vault(new_root, &c)?;
    Ok((rk_display(&new_rk, new_epoch), params.jti.clone()))
}

/// 用已知 master/mac_key 重包某 slot 并重算 versionMac（原子写 meta + bak）。
pub fn rewrap_and_sign(
    root: &Path,
    master: &[u8],
    mac_key: &[u8],
    slot: &str,
    new_secret: &[u8],
    kdf: &KdfParams,
    salt: &[u8],
) -> Result<(), CoreError> {
    let ikm = Zeroizing::new(argon2id(new_secret, salt, kdf.m, kdf.t, kdf.p, kdf.tag_len)?);
    let info = format!("wrap/{}", slot);
    let kek = Zeroizing::new(hkdf_expand(ikm.as_slice(), info.as_bytes())?);
    let blob = [master, mac_key].concat();
    let wrapped = aes_kw_wrap(kek.as_slice(), &blob)?;

    let meta = load_meta(root)?;
    let mut obj = meta.as_object().expect("meta object").clone();
    if let Some(arr) = obj.get_mut("wrapped").and_then(Value::as_array_mut) {
        for w in arr.iter_mut() {
            if w.get("slot").and_then(Value::as_str) == Some(slot) {
                if let Some(m) = w.as_object_mut() {
                    m.insert("blob".into(), Value::String(b64e(&wrapped)));
                }
            }
        }
    }
    obj.remove("versionMac");
    let mac_input = canonical(&Value::Object(obj.clone()))?;
    let vmac_key = hkdf_expand(mac_key, b"meta-version-mac")?;
    let version_mac = hmac_sha256(&vmac_key, &mac_input);
    obj.insert("versionMac".into(), Value::String(b64e(&version_mac)));
    let bytes = canonical(&Value::Object(obj))?;

    let tmp = root.join("vault.meta.tmp");
    fs::write(&tmp, &bytes).map_err(io)?;
    fs::write(root.join("vault.meta.bak"), &bytes).map_err(io)?;
    fs::rename(&tmp, root.join("vault.meta")).map_err(io)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 负向向量（§9）—— Rust 端复现 PoC 的 13 个 mutate + expect
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct NegResult {
    pub id: String,
    pub got: String,
    pub pass: bool,
}

fn clone_tree(src: &Path, dst: &Path) -> Result<(), CoreError> {
    fs::create_dir_all(dst).map_err(io)?;
    for entry in fs::read_dir(src).map_err(io)? {
        let entry = entry.map_err(io)?;
        let p = entry.path();
        let d = dst.join(entry.file_name());
        if p.is_dir() {
            clone_tree(&p, &d)?;
        } else {
            fs::copy(&p, &d).map_err(io)?;
        }
    }
    Ok(())
}

fn mutate_meta(root: &Path, f: impl Fn(&mut Map<String, Value>)) {
    let p = root.join("vault.meta");
    let raw = fs::read(&p).unwrap();
    let v: Value = serde_json::from_slice(&raw).unwrap();
    let mut obj = v.as_object().unwrap().clone();
    f(&mut obj);
    fs::write(&p, canonical(&Value::Object(obj)).unwrap()).unwrap();
}

fn mutate_bytes(path: &Path, f: impl Fn(&mut Vec<u8>)) {
    let mut d = fs::read(path).unwrap();
    f(&mut d);
    fs::write(path, &d).unwrap();
}

pub fn run_negative_vectors(base: &Path, fi: &FixedInputs) -> Result<Vec<NegResult>, CoreError> {
    let tmp = std::env::temp_dir().join(format!("mylock-m2-neg-{}", std::process::id()));
    run_negative_vectors_in(base, fi, &tmp)
}

/// 与 [`run_negative_vectors`] 相同，但由调用方指定临时工作目录 `tmp`。
///
/// Android 上 `std::env::temp_dir()` 指向的 /data/local/tmp 可能不可写或
/// noexec，导致 13 项负向向量无法 clone_tree；上层可用应用私有 cache 目录
/// （如 Tauri `app.path().app_cache_dir()`）调用本函数。
pub fn run_negative_vectors_in(
    base: &Path,
    fi: &FixedInputs,
    tmp: &Path,
) -> Result<Vec<NegResult>, CoreError> {
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).map_err(io)?;
    let pw = fi.passphrase_utf8.as_slice();

    fn one(
        base: &Path,
        tmp: &Path,
        tag: &str,
        pw: &[u8],
        expect: &[&str],
        mutate: impl Fn(&Path),
    ) -> NegResult {
        let dst = tmp.join(tag);
        let _ = fs::remove_dir_all(&dst);
        clone_tree(base, &dst).unwrap();
        mutate(&dst);
        let got = match full_open(&dst, pw, "password") {
            Ok(_) => "NO_ERROR".to_string(),
            Err(err) => err.code.to_string(),
        };
        NegResult {
            id: tag.to_string(),
            pass: expect.iter().any(|x| *x == got),
            got,
        }
    }

    let id_a = fi.object_id_a_b32.clone();
    let id_b = fi.object_id_b_b32.clone();
    let mut out = Vec::new();

    out.push(one(base, &tmp, "meta_1bit", pw, &["E_DOWNGRADE"], |r| {
        mutate_meta(r, |m| {
            let mut j = m.get("jti").and_then(Value::as_str).unwrap().to_string();
            let last = j.pop().unwrap();
            j.push(if last != 'Z' { 'Z' } else { 'Y' });
            m.insert("jti".into(), Value::String(j));
        });
    }));

    out.push(one(base, &tmp, "downgrade_format", pw, &["E_FORMAT", "E_DOWNGRADE"], |r| {
        mutate_meta(r, |m| {
            let f = m.get("format").and_then(Value::as_u64).unwrap() + 1;
            m.insert("format".into(), Value::from(f));
        });
    }));

    out.push(one(base, &tmp, "key_epoch_change", pw, &["E_DOWNGRADE"], |r| {
        mutate_meta(r, |m| {
            m.insert("key_epoch".into(), Value::from(99u64));
        });
    }));

    out.push(one(base, &tmp, "wrapped_blob_1bit", pw, &["E_AUTH_FAILED"], |r| {
        mutate_meta(r, |m| {
            let arr = m.get_mut("wrapped").and_then(Value::as_array_mut).unwrap();
            let s = arr[0].get("blob").and_then(Value::as_str).unwrap().to_string();
            let c = if s.starts_with('A') { "B" } else { "A" };
            let ns = format!("{}{}", c, &s[1..]);
            arr[0].as_object_mut().unwrap().insert("blob".into(), Value::String(ns));
        });
    }));

    out.push(one(base, &tmp, "slot_swap", pw, &["E_AUTH_FAILED"], |r| {
        mutate_meta(r, |m| {
            let arr = m.get_mut("wrapped").and_then(Value::as_array_mut).unwrap();
            let a = arr[0].get("blob").and_then(Value::as_str).unwrap().to_string();
            let b = arr[1].get("blob").and_then(Value::as_str).unwrap().to_string();
            arr[0].as_object_mut().unwrap().insert("blob".into(), Value::String(b));
            arr[1].as_object_mut().unwrap().insert("blob".into(), Value::String(a));
        });
    }));

    out.push(one(base, &tmp, "kdf_oob_m4tib", pw, &["E_FORMAT"], |r| {
        mutate_meta(r, |m| {
            let k = m.get_mut("kdf").and_then(Value::as_object_mut).unwrap();
            k.insert("m".into(), Value::from(4u64 * 1024 * 1024 * 1024));
        });
    }));

    {
        let p = format!("d/{}/{}", &id_a[..2], id_a);
        out.push(one(base, &tmp, "chunk_1bit", pw, &["E_CORRUPT"], |r| {
            let fp = r.join(&p);
            mutate_bytes(&fp, |d| d[32 + 24 + 4] ^= 0x01);
        }));
    }
    {
        let p = format!("d/{}/{}", &id_b[..2], id_b);
        out.push(one(base, &tmp, "delete_object", pw, &["E_MISSING"], |r| {
            fs::remove_file(r.join(&p)).unwrap();
        }));
        out.push(one(base, &tmp, "truncate_object", pw, &["E_CORRUPT", "STRUCT_ANOMALY"], |r| {
            // 用 A 对象做截断（与 PoC 一致）
            let pa = format!("d/{}/{}", &id_a[..2], id_a);
            let fp = r.join(&pa);
            let mut d = fs::read(&fp).unwrap();
            d.truncate(d.len() - 3);
            fs::write(&fp, &d).unwrap();
        }));
        out.push(one(base, &tmp, "empty_chunkcount0", pw, &["E_CORRUPT"], |r| {
            let fp = r.join(&p);
            mutate_bytes(&fp, |d| d[25..29].copy_from_slice(&0u32.to_be_bytes()));
        }));
    }

    {
        let pa = format!("d/{}/{}", &id_a[..2], id_a);
        out.push(one(base, &tmp, "orphan_object", pw, &["NO_ERROR"], |r| {
            let dir = r.join("d").join("ZZ");
            fs::create_dir_all(&dir).unwrap();
            fs::copy(r.join(&pa), dir.join("ZZAAAAAAAAAAAAAAAAAAAAAAAAAA")).unwrap();
        }));
    }

    out.push(one(base, &tmp, "manifest_1bit", pw, &["E_MANIFEST"], |r| {
        let fp = r.join("manifest");
        mutate_bytes(&fp, |d| d[24] ^= 0x01);
    }));

    out.push(one(
        base,
        &tmp,
        "wrong_password",
        "definitely-wrong-\u{0}".as_bytes(),
        &["E_AUTH_FAILED"],
        |_| {},
    ));

    let _ = fs::remove_dir_all(&tmp);
    Ok(out)
}

// ---------------------------------------------------------------------------
// M3 桌面层：把内存明文树重新加密落盘（写回 helper）
// ---------------------------------------------------------------------------

/// 一个明文节点，供 WebDAV 挂载会话把内存树写回 vault。
/// `parent_index` 是 `nodes` 切片内的下标；None 表示根节点（虚拟 ROOT）的子节点。
/// `parent_index` 是 `nodes` 切片下标；`rewrite_vault_from_plaintext` 先为全部节点
/// 预生成 object_id 数组再按下标映射父 id，故切片顺序无关（父可排在任何位置）。
#[derive(Debug, Clone)]
pub struct PlainNode {
    pub name: String,
    pub kind: String, // "file" | "dir"
    pub parent_index: Option<usize>,
    pub plaintext: Vec<u8>,
}

/// 增量写回节点：在 `PlainNode` 基础上多一个「可复用的既有 object_id」。
///
/// - `reuse_oid = Some(oid)`：明文未变，复用 `oid` 及盘上既有密文对象文件，不重加密；
///   若该文件缺失则自动降级为「全新 oid + 重加密」。
/// - `reuse_oid = None`：新建或明文已变，分配全新 128-bit oid + 全新随机 nonce
///   （严格 copy-on-write，旧密文成为孤儿）。
///
/// 注意：`name` / `parent_id` 只存在于 manifest record，不进入对象 AAD
/// （AAD = jti|format|oid|chunk_index|chunk_count|header_hash），
/// 故重命名/移动无需重加密对象文件，仍可安全复用。
#[derive(Debug, Clone)]
pub struct IncNode {
    pub name: String,
    pub kind: String,
    pub parent_index: Option<usize>,
    pub plaintext: Vec<u8>,
    pub reuse_oid: Option<[u8; 16]>,
}

/// 全量写回：把每个节点都视为「明文已变」的增量写回退化情形。
///
/// 保留此入口是为了让 M3 既有调用方/测试零改动；实际实现统一走
/// [`rewrite_vault_incremental`]，只是不给任何节点 `reuse_oid`。
pub fn rewrite_vault_from_plaintext(
    root: &Path,
    u: &Unwrapped,
    device_id: &[u8],
    nodes: &[PlainNode],
) -> Result<Vec<String>, CoreError> {
    let inc: Vec<IncNode> = nodes
        .iter()
        .map(|n| IncNode {
            name: n.name.clone(),
            kind: n.kind.clone(),
            parent_index: n.parent_index,
            plaintext: n.plaintext.clone(),
            reuse_oid: None,
        })
        .collect();
    rewrite_vault_incremental(root, u, device_id, &inc)
}

/// 在**已解锁的挂载会话内**把内存明文树重新加密落盘（增量版，M4 第 2 点）。
///
/// 关键设计（见 m3-architecture.md / m4-step2-report.md）：
/// - `vault.meta` 头（master/mac_key/jti/key_epoch/manifest_ref/salt/wrapped/versionMac）
///   在会话内**保持不变**，故不重算 Argon2id / AES-KW / versionMac。
/// - `reuse_oid = Some(oid)` 且盘上 `d/<b32[:2]>/<b32>` 存在、可解析、头部 oid 与
///   chunk_count 与当前明文一致 → **完全不重写该对象文件**（明文未变，密文可复用）；
///   否则分配全新 128-bit object_id + 全新随机 nonce 重加密（严格 copy-on-write）。
/// - manifest 每次都用全新 nonce 重写（同一 manifest_key、同一 AAD，因 mid/seq 不变）。
///   `name` / `parent_id` 只存在于 manifest record、不进入对象 AAD，故重命名/移动
///   只改 manifest、无需重加密任何对象文件。
/// - 顺序：先写全部新对象 → 原子替换 manifest → 再清理不在记录中的旧孤儿（崩溃安全）。
/// - `u` 必须是经 `unwrap_master` 全头认证过的会话密钥。
/// - 返回与 `nodes` **同序**的最终 object_id 列表（复用的原 id 或新分配的 id），
///   供调用方回写内存状态。
///
/// 前置条件（调用方契约）：`reuse_oid = Some(oid)` 仅当该节点明文与盘上对象
/// 逐字节一致；否则应传 `None`。本函数已用 header oid + chunk_count 校验做
/// 防御性降级，但无法在不解密的前提下校验密文内容。
pub fn rewrite_vault_incremental(
    root: &Path,
    u: &Unwrapped,
    device_id: &[u8],
    nodes: &[IncNode],
) -> Result<Vec<String>, CoreError> {
    let meta = &u.meta;
    let jti = meta
        .get("jti")
        .and_then(Value::as_str)
        .ok_or_else(|| e(E_MANIFEST, "jti"))?
        .to_string();
    let format = meta.get("format").and_then(Value::as_u64).unwrap_or(1) as u32;
    let mref = meta
        .get("manifest_ref")
        .and_then(Value::as_object)
        .ok_or_else(|| e(E_MANIFEST, "manifest_ref"))?;
    let mid_b32 = mref
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| e(E_MANIFEST, "manifest_ref.id"))?;
    let seq = mref.get("seq").and_then(Value::as_u64).unwrap_or(1) as u32;
    let mid = b32d(mid_b32)?;
    let root_id = vec![0u8; 16];

    let d = root.join("d");
    fs::create_dir_all(&d).map_err(io)?;

    // 先为每个节点决定「最终使用的 oid」以及「是否需要重加密」：
    // 全部决定完再写盘/建 record，parent_index 才能按下标映射到父节点最终 oid
    // （无论父是复用还是新分配），故 nodes 切片顺序无关。
    let mut oids: Vec<Vec<u8>> = Vec::with_capacity(nodes.len());
    let mut must_write: Vec<bool> = Vec::with_capacity(nodes.len());
    for n in nodes {
        let reusable = match n.reuse_oid {
            Some(oid) => {
                let b = b32e(&oid);
                let p = d.join(&b[..2]).join(&b);
                match fs::read(&p).ok().and_then(|bytes| parse_object_file(&bytes).ok()) {
                    Some(po) => {
                        po.oid == oid.to_vec()
                            && po.chunk_count as usize == chunk_count_for(n.plaintext.len())
                    }
                    None => false,
                }
            }
            None => false,
        };
        if reusable {
            oids.push(n.reuse_oid.unwrap().to_vec());
            must_write.push(false);
        } else {
            oids.push(random_bytes(16)?);
            must_write.push(true);
        }
    }
    let parent_raw = |i: usize| -> Vec<u8> {
        match nodes[i].parent_index {
            Some(p) if p < oids.len() => oids[p].clone(),
            _ => root_id.clone(),
        }
    };

    let mut records: Vec<Value> = Vec::with_capacity(nodes.len());
    let mut all_ids: Vec<String> = Vec::with_capacity(nodes.len());
    for (i, n) in nodes.iter().enumerate() {
        let oid = &oids[i];
        let oid_b32 = b32e(oid);
        if must_write[i] {
            let object_key =
                hkdf_expand(&u.master, &[b"object".as_slice(), oid.as_slice()].concat())?;
            let nonces = random_object_nonces(n.plaintext.len())?;
            let count = nonces.len() as u32;
            let header = object_header(oid, count);
            let header_hash = sha256(&header);
            let mut file_bytes = Vec::new();
            file_bytes.extend_from_slice(&header);
            for (ci, nonce) in nonces.iter().enumerate() {
                let start = ci * CHUNK_SIZE as usize;
                let end = ((ci + 1) * CHUNK_SIZE as usize).min(n.plaintext.len());
                let piece = &n.plaintext[start..end];
                let aad = aad_chunk(&jti, format, oid, ci as u32, count, &header_hash);
                let combined = xchacha_encrypt(&object_key, nonce, piece, &aad)?;
                let ct = &combined[..combined.len() - 16];
                let tag = &combined[combined.len() - 16..];
                file_bytes.extend_from_slice(nonce);
                file_bytes.extend_from_slice(&(ct.len() as u32).to_be_bytes());
                file_bytes.extend_from_slice(ct);
                file_bytes.extend_from_slice(tag);
            }
            let shard = d.join(&oid_b32[..2]);
            fs::create_dir_all(&shard).map_err(io)?;
            fs::write(shard.join(&oid_b32), &file_bytes).map_err(io)?;
        }
        // 复用节点不写 d/ 文件；record 仍按当前 name/parent/oid 重建。
        let count = chunk_count_for(n.plaintext.len()) as u32;
        all_ids.push(oid_b32.clone());
        records.push(json!({
            "id": oid_b32,
            "parent_id": b32e(&parent_raw(i)),
            "name": n.name,
            "kind": n.kind,
            "size": n.plaintext.len(),
            "chunk_count": count,
            "base_version": 0,
            "device_id": b64e(device_id),
            "updated_seq": 1,
            "content_version": 1,
            "manifest_seq": seq,
        }));
    }

    // manifest（同一 key / 同一 AAD [mid,seq 不变]，仅换 nonce）
    let manifest_key =
        hkdf_expand(&u.master, &[b"manifest".as_slice(), mid.as_slice()].concat())?;
    let manifest_plaintext = canonical(&Value::Array(records))?;
    let manifest_aad = aad_manifest(&jti, format, &mid, seq);
    let manifest_nonce = random_bytes(24)?;
    let manifest_combined = xchacha_encrypt(
        &manifest_key,
        &manifest_nonce,
        &manifest_plaintext,
        &manifest_aad,
    )?;
    let manifest_file = [manifest_nonce.as_slice(), manifest_combined.as_slice()].concat();

    // 原子替换 manifest：tmp + fsync + rename（Windows 下 rename 覆盖已存在文件）。
    // 此前新对象已全部写盘；此刻 manifest 仍指向旧对象，任何时刻崩溃都保持自洽。
    let tmp = root.join("manifest.tmp");
    {
        use std::io::Write as _;
        let mut f = fs::File::create(&tmp).map_err(io)?;
        f.write_all(&manifest_file).map_err(io)?;
        f.sync_all().map_err(io)?;
    }
    let final_manifest = root.join("manifest");
    fs::rename(&tmp, &final_manifest)
        .or_else(|_| {
            let _ = fs::remove_file(&final_manifest);
            fs::rename(&tmp, &final_manifest)
        })
        .map_err(io)?;

    // manifest 已只指向 all_ids —— 此时清理旧孤儿对象才安全（崩溃只留下无害孤儿）。
    // 注意 keep 必须含「复用 + 新分配」的全部 id，否则会误删复用对象。
    let keep: std::collections::BTreeSet<&str> =
        all_ids.iter().map(|s| s.as_str()).collect();
    if let Ok(shards) = fs::read_dir(&d) {
        for shard in shards.flatten() {
            if !shard.path().is_dir() {
                continue;
            }
            if let Ok(files) = fs::read_dir(shard.path()) {
                for file in files.flatten() {
                    let name = file.file_name().to_string_lossy().into_owned();
                    if !keep.contains(name.as_str()) {
                        let _ = fs::remove_file(file.path());
                    }
                }
            }
        }
    }
    // 注意：vault.meta / vault.meta.bak 完全不重写 —— 头与 versionMac 保持字节不变。
    Ok(all_ids)
}

// ---------------------------------------------------------------------------
// 单元测试：把每个被审计过的 Rust 原语直接锚定到权威已发布 KAT
// （不只是通过 PoC 间接对齐；这些字面值来自 RFC / IRTF draft 一手页）
// ---------------------------------------------------------------------------
#[cfg(test)]
mod kat_tests {
    use super::*;

    /// RFC 3394 §4.6：256-bit KEK 包 256-bit 数据 → 40B 密文。
    #[test]
    fn rfc3394_aes_kw_256_kek_kat() {
        let kek = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
        let key = hex::decode("00112233445566778899aabbccddeeff000102030405060708090a0b0c0d0e0f").unwrap();
        let expected = "28c9f404c4b810f4cbccb35cfb87f8263f5786e2d80ed326cbc7f0e71a99f43bfb988b9b7a02dd21";
        assert_eq!(hex::encode(aes_kw_wrap(&kek, &key).unwrap()), expected);
        assert_eq!(aes_kw_unwrap(&kek, &hex::decode(expected).unwrap()).unwrap(), key);
        // 本产品实况：64B(8 块) → 72B(n+1 块)
        let blob64 = [0u8; 64];
        assert_eq!(aes_kw_wrap(&kek, &blob64).unwrap().len(), 72);
    }

    /// draft-irtf-cfrg-xchacha-03 §A.3.1：XChaCha20-Poly1305 AEAD 向量。
    #[test]
    fn xchacha_draft_a31_kat() {
        let key = hex::decode("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f").unwrap();
        let nonce = hex::decode("404142434445464748494a4b4c4d4e4f5051525354555657").unwrap();
        let aad = hex::decode("50515253c0c1c2c3c4c5c6c7").unwrap();
        let pt = hex::decode("4c616469657320616e642047656e746c656d656e206f662074686520636c617373206f66202739393a204966204920636f756c64206f6666657220796f75206f6e6c79206f6e652074697020666f7220746865206675747572652c2073756e73637265656e20776f756c642062652069742e").unwrap();
        let ct = hex::decode("bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b4522f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff921f9664c97637da9768812f615c68b13b52e").unwrap();
        let tag = hex::decode("c0875924c1c7987947deafd8780acf49").unwrap();
        let combined = [ct.as_slice(), tag.as_slice()].concat();
        assert_eq!(xchacha_encrypt(&key, &nonce, &pt, &aad).unwrap(), combined);
        assert_eq!(xchacha_decrypt(&key, &nonce, &combined, &aad).unwrap(), pt);
        // AAD 篡改必须失败；空明文仍产 16B tag（坐实 N4）
        assert!(xchacha_decrypt(&key, &nonce, &combined, b"bad-aad").is_err());
        assert_eq!(xchacha_encrypt(&key, &nonce, b"", &aad).unwrap().len(), 16);
    }

    /// RFC 5869 Appendix A.1：HKDF-SHA256（我们只用 Expand 分支 + 均匀 PRK）。
    #[test]
    fn rfc5869_hkdf_sha256_kat() {
        let ikm = vec![0x0bu8; 22];
        let salt = hex::decode("000102030405060708090a0b0c").unwrap();
        let info = hex::decode("f0f1f2f3f4f5f6f7f8f9").unwrap();
        // PRK = HMAC-SHA256(salt, ikm)
        let prk = hmac_sha256(&salt, &ikm);
        assert_eq!(hex::encode(&prk), "077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5");
        // 仅 Expand（from_prk 跳过 Extract）→ 与 RFC A.1 OKM 前 32B 逐字节一致
        let okm = hkdf_expand(&prk, &info).unwrap();
        let expect = "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865";
        assert_eq!(hex::encode(&okm), &expect[..64]);
    }

    /// RFC 4231 §4.2 Test Case 1：HMAC-SHA256("Hi There")。
    #[test]
    fn rfc4231_hmac_sha256_kat() {
        let key = vec![0x0bu8; 20];
        let data = b"Hi There";
        assert_eq!(
            hex::encode(hmac_sha256(&key, data)),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    /// Argon2id 冻结参数确定性（RFC 9106 §5.3 精确 KAT 含可选 Secret+AD，
    /// 而 argon2 crate 高层 API 不暴露 AD；这里锚定「同参同输入同输出」与
    /// 「p=4 ≠ p=1」，后者正是 crypto-review §12.4 的技术栈硬约束）。
    #[test]
    fn argon2id_params_and_parallelism_are_binding() {
        let pw = vec![0x01u8; 32];
        let salt = vec![0x02u8; 16];
        let o4 = argon2id(&pw, &salt, 32, 3, 4, 32).unwrap();
        assert_eq!(hex::encode(&o4), "03aab965c12001c9d7d0d2de33192c0494b684bb148196d73c1df1acaf6d0c2e");
        let o1 = argon2id(&pw, &salt, 32, 3, 1, 32).unwrap();
        assert_ne!(o4, o1, "parallelism=4 与 =1 必须不同；libsodium crypto_pwhash (p=1) 不可替代");
    }

    /// canonical JSON §3.3 八条：键排序、控制字符一律 \u00xx（含 \n）、非 ASCII 原样。
    #[test]
    fn canonical_json_profile_kat() {
        let v: Value = serde_json::from_str(r#"{"z":1,"a":"line\nquote\"","n":"保险柜","b":true,"f":null}"#).unwrap();
        let s = String::from_utf8(canonical(&v).unwrap()).unwrap();
        assert_eq!(s, "{\"a\":\"line\\u000aquote\\\"\",\"b\":true,\"f\":null,\"n\":\"保险柜\",\"z\":1}");
        // 禁浮点
        assert!(canonical(&serde_json::json!({"x": 1.5})).is_err());
    }

    /// 多分块（>1 MiB）写入→整盘复制→解锁→明文逐字节一致（§4.2 1 MiB 分块）。
    #[test]
    fn multi_chunk_roundtrip() {
        let dir = std::env::temp_dir().join(format!("mylock_mc_{}", std::process::id()));
        let root = dir.join("vault");
        let copy = dir.join("copy");
        let _ = fs::remove_dir_all(&dir);
        let pw = b"multi-chunk-\xe4\xbf\x9d\xe9\x99\xa9\xe6\x9f\x9c".to_vec();
        let rk = vec![0x42u8; 32];
        let params = random_vault_params(&pw, &rk, 0).unwrap();
        // 2.5 MiB → 3 个 chunk（1MiB + 1MiB + 0.5MiB）
        let mut big = Vec::new();
        for i in 0..(2 * 1024 * 1024 + 512 * 1024) {
            big.push((i % 251) as u8);
        }
        let nonces = random_object_nonces(big.len()).unwrap();
        assert_eq!(nonces.len(), 3);
        let oid = vec![0x77u8; 16];
        let specs = vec![ObjectSpec {
            oid_raw: oid.clone(), name: "big.bin".into(), kind: "file".into(),
            parent_id_raw: vec![0u8; 16], plaintext: big.clone(), nonces,
            updated_seq: 1, content_version: 1,
        }];
        let c = build_vault(&params, &specs).unwrap();
        write_vault(&root, &c).unwrap();
        // 整盘复制（§6.8 载体无关）
        clone_tree(&root, &copy).unwrap();
        let opened = full_open(&copy, &pw, "password").unwrap();
        assert_eq!(opened.exported.get("big.bin").unwrap(), &big);
        assert_eq!(opened.exported_by_id.get(&b32e(&oid)).unwrap(), &big);
        // 独立恢复路径（不读 manifest）也得同样明文
        assert_eq!(export_object_by_id(&copy, &pw, "password", &b32e(&oid)).unwrap(), big);
        let _ = fs::remove_dir_all(&dir);
    }
}
