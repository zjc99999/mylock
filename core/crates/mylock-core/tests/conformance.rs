//! G4 跨平台一致性门禁（M4 第 1 点：向量归档 + CI 门禁基础设施）。
//!
//! 与 `vectors.rs` 的区别 / 本文件的目的
//! -----------------------------------
//! - `vectors.rs` 从 `poc/test-vectors.json` 读契约，验证「Rust 单端逐字节复现」。
//! - **本文件从 `tests/vectors/test-vectors.json`（归档副本）读契约**，并且额外
//!   计算两个与平台无关的冻结锚：
//!     ① 归档文件原始字节的 SHA-256 = `EXPECTED_SHA256`；
//!     ② 30 个 derived_outputs 的「排序键值拼接」SHA-256 = `G4_FINGERPRINT`。
//!   这两个常量与 `tools/check_vectors.py` / `tests/vectors/SHA256SUMS` 同源冻结。
//!
//! 为什么这能证明「跨平台一致」而不需要 runner 之间通信
//! ----------------------------------------------------
//! G4 要求 Win/mac/Linux/Android/iOS 五端 + 恢复 CLI 对固定向量产出逐字节相同的
//! 密文。每个 CI runner 各自独立地：读同一份归档字节 → 校验 SHA-256 → 用本平台的
//! CPU/OS/编译器跑 `derive_outputs` → 得到 30 个派生值 → 算 G4 指纹 → 与冻结常量比。
//! 只要三个桌面平台的 CI 都绿，就证明「同一字节输入在三套工具链上产出同一字节输出」。
//! 移动端（Android/iOS）暂无实现，**该端门禁不得伪造绿灯**，见
//! `tests/vectors/CONTRACT.md` 的激活条件表。
//!
//! 诚实边界：本文件是「固定输入向量」的一致性门禁，**不要求运行时确定性密文**
//! （正常运行用 CSPRNG 24B nonce；G4 只约束固定向量，见 crypto-review-m1.md §9【9】）。

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use mylock_core::{derive_outputs, FixedInputs};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// 归档文件原始字节的冻结 SHA-256（13499 B，CRLF×190，`.gitattributes` 标 `-text`）。
/// 与 `tests/vectors/SHA256SUMS`、`tools/check_vectors.py:VECTOR_SHA256` 一致。
const EXPECTED_SHA256: &str =
    "07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939";

/// 30 个 derived_outputs 按 key 排序后 `k=v\n` 拼接（UTF-8，8254 B）的 SHA-256。
/// 由 `tools/compute_fingerprint.py` 从契约算出并冻结；三平台 CI 各自独立复算比对。
const G4_FINGERPRINT: &str =
    "d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa";

fn repo_file(rel: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR = core/crates/mylock-core ; 上溯 3 层到仓库根
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(rel)
}

fn archive_bytes() -> Vec<u8> {
    let p = repo_file("tests/vectors/test-vectors.json");
    fs::read(&p).unwrap_or_else(|e| panic!("cannot read {}: {}", p.display(), e))
}

fn contract() -> Value {
    let raw = archive_bytes();
    serde_json::from_slice(&raw).expect("archived test-vectors.json is valid JSON")
}

fn sha256_hex(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}

/// 与 `tools/compute_fingerprint.py` 完全一致的指纹算法：
/// 按 key 排序（BTreeMap 天然有序），逐项拼 `k=v\n`，对 UTF-8 字节取 SHA-256。
fn fingerprint(m: &BTreeMap<String, String>) -> String {
    let mut s = String::new();
    for (k, v) in m {
        s.push_str(k);
        s.push('=');
        s.push_str(v);
        s.push('\n');
    }
    sha256_hex(s.as_bytes())
}

fn contract_outputs(v: &Value) -> BTreeMap<String, String> {
    v["derived_outputs"]
        .as_object()
        .expect("derived_outputs object")
        .iter()
        .map(|(k, val)| {
            (
                k.clone(),
                val.as_str()
                    .unwrap_or_else(|| panic!("non-string derived_outputs[{}]", k))
                    .to_string(),
            )
        })
        .collect()
}

/// 锚 ①：归档字节必须与冻结 SHA-256 一致（顺带捕获 EOL 归一化 / 篡改 / 截断）。
#[test]
fn archive_sha256_anchor_matches() {
    let raw = archive_bytes();
    assert_eq!(
        raw.len(),
        13499,
        "archive byte length changed (EOL normalization or corruption?)"
    );
    assert_eq!(
        sha256_hex(&raw),
        EXPECTED_SHA256,
        "archived vector bytes drifted; update ALL anchors with a written reason"
    );
    // 两份副本必须字节相同，防止「只改一份」造成分叉。
    let poc = fs::read(repo_file("poc/test-vectors.json"))
        .expect("poc/test-vectors.json must exist");
    assert_eq!(poc, raw, "poc/ and tests/vectors/ copies diverged");
}

/// 锚 ②（契约侧）：归档里 30 个派生值的指纹 = 冻结常量。
#[test]
fn contract_g4_fingerprint_matches() {
    let v = contract();
    let want = contract_outputs(&v);
    assert_eq!(want.len(), 30, "contract must have 30 derived_outputs");
    assert_eq!(
        fingerprint(&want),
        G4_FINGERPRINT,
        "contract derived_outputs changed; a format bump needs new anchors + review"
    );
}

/// 锚 ②（实现侧，真正的跨平台证据）：本平台独立跑 `derive_outputs`，
/// 30 个值逐字节等于契约，且它们的指纹也等于同一个冻结常量。
#[test]
fn this_platform_reproduces_g4_fingerprint() {
    let v = contract();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let got = derive_outputs(&fi).expect("derive_outputs on this platform");
    let want = contract_outputs(&v);

    assert_eq!(got.len(), want.len(), "derived_outputs key count mismatch");
    for (k, w) in &want {
        match got.get(k) {
            None => panic!("this platform did NOT produce derived_outputs[{}]", k),
            Some(g) => assert_eq!(g, w, "derived_outputs[{}] mismatch on this platform", k),
        }
    }
    assert_eq!(
        fingerprint(&got),
        G4_FINGERPRINT,
        "G4 fingerprint mismatch: this platform's derived bytes are not the frozen ones"
    );
}

/// 负向向量契约（13 条）与关键硬约束仍在（防止归档被「瘦身」后仍过正向门禁）。
#[test]
fn contract_negative_and_kdf_invariants_intact() {
    let v = contract();
    let neg = v["negative_vectors"].as_array().expect("negative_vectors array");
    assert_eq!(neg.len(), 13, "contract must carry 13 negative vectors");
    assert!(
        neg.iter().all(|x| x["pass"].as_bool() == Some(true)),
        "a negative_vectors entry is not marked pass:true"
    );
    let fixed = &v["fixed_inputs"];
    assert_eq!(
        fixed["kdf"]["p"].as_u64(),
        Some(4),
        "parallelism=4 is a hard constraint (libsodium crypto_pwhash p=1 cannot reproduce)"
    );
    assert_eq!(
        fixed["format"].as_u64(),
        Some(1),
        "a format bump requires new vectors + anchors + review"
    );
    assert_eq!(
        v["summary"]["negative_vectors_matched"].as_str(),
        Some("13/13"),
        "summary.negative_vectors_matched must stay 13/13"
    );
}
