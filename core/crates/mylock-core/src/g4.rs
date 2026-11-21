//! G4 跨平台一致性验证的公开实现。
//!
//! 把 `tests/conformance.rs` 里私有的契约算法（contract_outputs / fingerprint /
//! sha256_hex）提升为 crate 的公开 API，供桌面壳与 Android Tauri 命令复用
//! 同一份实现，避免「壳层自己再算一遍」造成两份实现漂移。
//!
//! 两个不同的冻结锚，勿混淆：
//! - [`EXPECTED_CONTRACT_SHA256`]：归档文件原始字节（13499 B，CRLF×190）的 SHA-256。
//! - [`G4_FINGERPRINT`]：30 个 derived_outputs 按 key 排序后 `k=v\n` 拼接的 SHA-256。
//!
//! 硬约束：`fixed_inputs.kdf.p` 必须为 4。降 p=1 会改变 IKM/KEK，导致 30/30
//! 与指纹全部失败。p 是纯算法参数，无平台/feature 限制，绝不可为「跑通」而降。

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    create_vault, derive_outputs, run_negative_vectors_in, CoreError, FixedInputs,
};

/// 30 项按 key 排序后 `k=v\n` 拼接（UTF-8，8254 B）的 SHA-256 冻结常量。
pub const G4_FINGERPRINT: &str =
    "d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa";

/// tests/vectors/test-vectors.json 原始字节的冻结 SHA-256（13499 B）。
pub const EXPECTED_CONTRACT_SHA256: &str =
    "07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939";

/// 契约要求的最小派生项数量。
pub const EXPECTED_DERIVED_COUNT: usize = 30;
/// 契约要求的负向向量数量。
pub const EXPECTED_NEGATIVE_COUNT: usize = 13;
/// 冻结的 Argon2id 并行度（p=4）。
pub const EXPECTED_ARGON2_P: u32 = 4;

fn sha256_hex(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}

/// 与 `tools/compute_fingerprint.py` 完全一致的指纹算法：
/// 按 key 排序（BTreeMap 天然有序），逐项拼 `k=v\n`，对 UTF-8 字节取 SHA-256。
pub fn g4_fingerprint(m: &BTreeMap<String, String>) -> String {
    let mut s = String::new();
    for (k, v) in m {
        s.push_str(k);
        s.push('=');
        s.push_str(v);
        s.push('\n');
    }
    sha256_hex(s.as_bytes())
}

/// 从契约 JSON 取出 30 个 derived_outputs（不 panic，跨 FFI 安全）。
pub fn g4_contract_outputs(v: &Value) -> Result<BTreeMap<String, String>, CoreError> {
    let obj = v
        .get("derived_outputs")
        .and_then(Value::as_object)
        .ok_or_else(|| CoreError::new(crate::E_FORMAT, "derived_outputs must be an object"))?;
    let mut out = BTreeMap::new();
    for (k, val) in obj {
        let s = val
            .as_str()
            .ok_or_else(|| CoreError::new(crate::E_FORMAT, format!("derived_outputs[{}] not string", k)))?;
        out.insert(k.clone(), s.to_string());
    }
    Ok(out)
}

/// 契约侧指纹：直接从归档 JSON 里的 derived_outputs 计算（实现侧无关）。
pub fn g4_contract_fingerprint(v: &Value) -> Result<String, CoreError> {
    Ok(g4_fingerprint(&g4_contract_outputs(v)?))
}

/// 单个负向向量的比对明细。
#[derive(Debug, Clone, Serialize)]
pub struct G4NegativeDetail {
    pub id: String,
    /// 本平台实际得到的代码（或 "NO_ERROR"）。
    pub got: String,
    /// 契约允许的期望代码列表。
    pub expected: Vec<String>,
    /// got 是否命中 expected。
    pub pass: bool,
}

/// G4 验证报告（Serialize，可直接作为 Tauri 命令返回值）。
#[derive(Debug, Clone, Serialize)]
pub struct G4Report {
    /// 归档原始字节 SHA-256 是否与冻结锚一致（捕获 EOL 归一化/篡改/截断）。
    pub contract_sha256_ok: bool,
    /// 实际算出的归档字节 SHA-256（hex）。
    pub contract_sha256: String,
    /// 契约里 derived_outputs 的数量（应为 30）。
    pub derived_checked: usize,
    /// 本平台复现且与契约逐字节一致的派生值数量。
    pub derived_match: usize,
    /// 不一致或缺失的 key 列表（为空表示 30/30 全中）。
    pub derived_mismatches: Vec<String>,
    /// 本平台 30 项派生值算出的 G4 指纹。
    pub fingerprint: String,
    /// 指纹是否等于冻结常量 G4_FINGERPRINT。
    pub fingerprint_ok: bool,
    /// 固定输入里实际使用的 Argon2id 并行度 p。
    pub argon2_p: u32,
    /// p 是否等于冻结的 4。
    pub argon2_ok: bool,
    /// 是否执行了 13 项负向向量（需要传入工作目录）。
    pub negative_run: bool,
    /// 执行的负向向量数量。
    pub negative_total: usize,
    /// 按契约 expect 比对后通过的负向向量数量。
    pub negative_passed: usize,
    /// 负向向量逐项明细。
    pub negative_details: Vec<G4NegativeDetail>,
    /// 总判定：派生 30/30 + 指纹 + 归档字节 + p=4 +（若运行）负向 13/13。
    pub ok: bool,
}

/// 运行完整 G4 验证。
///
/// - `contract_bytes`：`tests/vectors/test-vectors.json` 的**原始字节**
///   （务必用 `include_bytes!` 或 fs::read，勿反序列化再序列化，否则 CRLF 丢失）。
/// - `work_dir`：可选的、可写的工作目录；提供时额外执行 13 项负向向量。
///   Android 应传应用私有 cache 目录（`/data/data/<pkg>/cache`），
///   因为 `std::env::temp_dir()` 在 Android 上未必可写。
///
/// 注意：`FixedInputs::from_json` 是 panic 式取值；本函数先用字节锚校验
/// 归档完整性，只有冻结的原始字节才继续解析，因此不会因外部篡改而 panic。
pub fn g4_verify(contract_bytes: &[u8], work_dir: Option<&Path>) -> Result<G4Report, CoreError> {
    let actual_sha = sha256_hex(contract_bytes);
    let sha_ok = actual_sha == EXPECTED_CONTRACT_SHA256;
    if !sha_ok {
        return Err(CoreError::new(
            crate::E_FORMAT,
            format!(
                "G4 contract bytes drifted: sha256={}, expected={}",
                actual_sha, EXPECTED_CONTRACT_SHA256
            ),
        ));
    }

    let v: Value = serde_json::from_slice(contract_bytes)
        .map_err(|e| CoreError::new(crate::E_FORMAT, format!("bad G4 contract JSON: {}", e)))?;

    // 从冻结契约解析固定输入（panic 式取值，但字节锚已保证是冻结副本）。
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let got = derive_outputs(&fi)?;
    let want = g4_contract_outputs(&v)?;

    let mut mismatches = Vec::new();
    let mut derived_match = 0usize;
    for (k, expected) in &want {
        match got.get(k) {
            Some(actual) if actual == expected => derived_match += 1,
            _ => mismatches.push(k.clone()),
        }
    }
    // 多出的 key 也算不一致（契约必须恰好 30 项）。
    for k in got.keys() {
        if !want.contains_key(k) {
            mismatches.push(k.clone());
        }
    }
    mismatches.sort();
    mismatches.dedup();

    let fingerprint = g4_fingerprint(&got);
    let fingerprint_ok = fingerprint == G4_FINGERPRINT;
    let argon2_p = fi.kdf.p;
    let argon2_ok = argon2_p == EXPECTED_ARGON2_P;

    let mut negative_run = false;
    let mut negative_total = 0usize;
    let mut negative_passed = 0usize;
    let mut negative_details: Vec<G4NegativeDetail> = Vec::new();

    if let Some(wd) = work_dir {
        let vault_dir = wd.join("g4-vault");
        let neg_dir = wd.join("g4-neg");
        let _ = std::fs::remove_dir_all(&vault_dir);
        let _ = std::fs::remove_dir_all(&neg_dir);
        create_vault(&vault_dir, &fi)?;
        let results = run_negative_vectors_in(&vault_dir, &fi, &neg_dir)?;
        let expects = v
            .get("negative_vectors")
            .and_then(Value::as_array)
            .ok_or_else(|| CoreError::new(crate::E_FORMAT, "negative_vectors must be an array"))?;

        negative_run = true;
        negative_total = results.len();
        for r in &results {
            let e = expects
                .iter()
                .find(|x| x["id"].as_str() == Some(r.id.as_str()))
                .ok_or_else(|| CoreError::new(crate::E_FORMAT, format!("no contract entry for {}", r.id)))?;
            let expected_codes: Vec<String> = match &e["expect"] {
                Value::String(s) => vec![s.clone()],
                Value::Array(a) => a
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect(),
                _ => {
                    return Err(CoreError::new(
                        crate::E_FORMAT,
                        format!("bad expect for {}", r.id),
                    ))
                }
            };
            let pass = expected_codes.iter().any(|x| x == &r.got);
            if pass {
                negative_passed += 1;
            }
            negative_details.push(G4NegativeDetail {
                id: r.id.clone(),
                got: r.got.clone(),
                expected: expected_codes,
                pass,
            });
        }
        let _ = std::fs::remove_dir_all(&vault_dir);
        let _ = std::fs::remove_dir_all(&neg_dir);
    }

    let derived_ok = derived_match == EXPECTED_DERIVED_COUNT && mismatches.is_empty();
    let negative_ok = !negative_run
        || (negative_total == EXPECTED_NEGATIVE_COUNT && negative_passed == EXPECTED_NEGATIVE_COUNT);

    Ok(G4Report {
        contract_sha256_ok: sha_ok,
        contract_sha256: actual_sha,
        derived_checked: want.len(),
        derived_match,
        derived_mismatches: mismatches,
        fingerprint,
        fingerprint_ok,
        argon2_p,
        argon2_ok,
        negative_run,
        negative_total,
        negative_passed,
        negative_details,
        ok: derived_ok && fingerprint_ok && sha_ok && argon2_ok && negative_ok,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../tests/vectors/test-vectors.json"
    ));

    #[test]
    fn g4_fingerprint_matches_frozen_constant() {
        let v: Value = serde_json::from_slice(CONTRACT).unwrap();
        let outputs = g4_contract_outputs(&v).unwrap();
        assert_eq!(outputs.len(), EXPECTED_DERIVED_COUNT);
        assert_eq!(g4_fingerprint(&outputs), G4_FINGERPRINT);
    }

    #[test]
    fn g4_verify_derived_only_is_green() {
        let report = g4_verify(CONTRACT, None).unwrap();
        assert!(report.contract_sha256_ok, "byte anchor must match");
        assert_eq!(report.derived_match, EXPECTED_DERIVED_COUNT);
        assert!(report.derived_mismatches.is_empty());
        assert!(report.fingerprint_ok);
        assert!(report.argon2_ok, "p must be 4");
        assert!(!report.negative_run);
        assert!(report.ok, "derived-only G4 must be green");
    }

    #[test]
    fn g4_verify_rejects_tampered_contract() {
        let mut bad = CONTRACT.to_vec();
        if let Some(i) = bad.iter().position(|b| *b == b'{') {
            bad[i] = b'X';
        }
        assert!(g4_verify(&bad, None).is_err(), "byte anchor must reject drift");
    }

    /// Validates the exact full path the Android `g4_self_check(run_negative=true)`
    /// command executes: derived 30/30 + fingerprint + byte anchor + p=4 + 13 negatives,
    /// with an explicit writable work dir (as the phone passes its app cache dir).
    #[test]
    fn g4_verify_full_with_workdir_is_green() {
        let work = std::env::temp_dir().join(format!(
            "mylock-g4-full-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&work);
        let report = g4_verify(CONTRACT, Some(&work)).unwrap();
        assert!(report.contract_sha256_ok, "byte anchor must match");
        assert_eq!(report.derived_checked, EXPECTED_DERIVED_COUNT);
        assert_eq!(report.derived_match, EXPECTED_DERIVED_COUNT);
        assert!(report.derived_mismatches.is_empty());
        assert!(report.fingerprint_ok, "fingerprint must match frozen constant");
        assert!(report.argon2_ok, "p must be 4");
        assert!(report.negative_run, "negatives must have run with a work dir");
        assert_eq!(report.negative_total, EXPECTED_NEGATIVE_COUNT);
        assert_eq!(report.negative_passed, EXPECTED_NEGATIVE_COUNT);
        for d in &report.negative_details {
            assert!(d.pass, "negative {} failed: got={}", d.id, d.got);
        }
        assert!(report.ok, "full G4 (derived + negative) must be green");
        let _ = std::fs::remove_dir_all(&work);
    }
}
