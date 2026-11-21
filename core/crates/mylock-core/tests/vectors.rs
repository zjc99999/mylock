//! M2 验收测试：Rust 端逐字节复现 `poc/test-vectors.json`。
//!
//! 这是本项目第一验收门槛：30 个 derived_outputs 逐字节一致 + 主链路
//! （建库→整盘复制→解锁→verify→export）+ 13/13 负向向量 + 恢复槽端到端。
//! 契约文件是唯一真源；本测试不得硬编码任何派生值，全部从 JSON 读取比对。

use std::fs;
use std::path::PathBuf;

use mylock_core::*;
use serde_json::Value;

fn vectors_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = core/crates/mylock-core ; ../../../poc/test-vectors.json
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../poc/test-vectors.json")
        .canonicalize()
        .expect("poc/test-vectors.json must exist")
}

fn load() -> Value {
    let raw = fs::read(vectors_path()).expect("read test-vectors.json");
    serde_json::from_slice(&raw).expect("test-vectors.json is valid JSON")
}

fn temp_dir(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("mylock-m2-test-{}-{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

/// 30 个 derived_outputs 逐字节一致（纯内存复现，不写磁盘）。
#[test]
fn derived_outputs_match_byte_for_byte() {
    let v = load();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let got = derive_outputs(&fi).expect("derive_outputs");
    let want = v["derived_outputs"].as_object().expect("derived_outputs object");

    assert_eq!(got.len(), want.len(), "derived_outputs key count mismatch");
    for (k, w) in want {
        let w = w.as_str().unwrap_or_else(|| panic!("non-string value for {}", k));
        match got.get(k) {
            None => panic!("Rust did NOT produce derived_outputs[{}]", k),
            Some(g) => assert_eq!(g, w, "derived_outputs[{}] mismatch", k),
        }
    }
}

/// canonical JSON profile 逐字节一致（含 versionMac.input_hex / meta_canonical_hex /
/// manifest_plaintext_canonical_hex 三个含控制字符与非 ASCII 的 canonical 串）。
#[test]
fn canonical_profile_matches() {
    let v = load();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let (params, objects) = fi.to_build();
    let c = build_vault(&params, &objects).expect("build_vault");
    let want = &v["derived_outputs"];
    assert_eq!(
        hex::encode(&c.mac_input),
        want["versionMac.input_hex"].as_str().unwrap(),
        "canonical(meta_core)"
    );
    assert_eq!(
        hex::encode(&c.meta_full_bytes),
        want["meta_canonical_hex"].as_str().unwrap(),
        "canonical(meta_full)"
    );
    assert_eq!(
        hex::encode(&c.manifest_plaintext),
        want["manifest_plaintext_canonical_hex"].as_str().unwrap(),
        "canonical(records[])"
    );
}

/// 主链路：建库 → 整盘复制（模拟移动硬盘）→ 解锁 → verify phase1/2 → export。
#[test]
fn full_pipeline_and_positive_vectors() {
    let v = load();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let base = temp_dir("pipeline");
    let src = base.join("vault");
    let usb = base.join("usb_copy");
    create_vault(&src, &fi).expect("create_vault");
    // 整盘复制：普通文件目录复制，零 OS 特性依赖（§6.8）
    copy_tree(&src, &usb);

    // phase1（解锁前，不输口令）
    let p1 = verify_phase1(&usb).expect("verify_phase1");
    assert_eq!(p1.result, "OK");
    assert_eq!(p1.objects, v["verify_phase1_json"]["objects"].as_u64().unwrap());
    assert_eq!(p1.bytes, v["verify_phase1_json"]["bytes"].as_u64().unwrap());

    // 主口令解锁 + phase2 + 明文重建
    let opened = full_open(&usb, &fi.passphrase_utf8, "password").expect("full_open password");
    assert_eq!(opened.records as u64, v["verify_phase2"]["records"].as_u64().unwrap());
    assert!(opened.orphans.is_empty(), "no orphans expected");
    assert_eq!(
        hex::encode(&opened.exported["hello.txt"]),
        v["export"]["hello.txt"].as_str().unwrap(),
        "hello.txt"
    );
    assert_eq!(
        hex::encode(&opened.exported["empty.bin"]),
        v["export"]["empty.bin"].as_str().unwrap(),
        "empty.bin"
    );

    // 恢复槽端到端（F13 slot 域分离真正跑通）
    let rk = full_open(&usb, &fi.rk_raw, "recovery").expect("full_open recovery");
    assert_eq!(rk.exported["hello.txt"], opened.exported["hello.txt"]);
    assert_eq!(rk.exported["empty.bin"], opened.exported["empty.bin"]);

    // RK 喂给 password 槽 → E_AUTH_FAILED（域分离生效）
    let err = full_open(&usb, &fi.rk_raw, "password").unwrap_err();
    assert_eq!(err.code, E_AUTH_FAILED, "RK into password slot must fail");

    // 独立恢复路径（manifest 损坏时凭 object_id 兜底）
    let by_id = export_object_by_id(&usb, &fi.passphrase_utf8, "password", &fi.object_id_a_b32)
        .expect("export_object_by_id");
    assert_eq!(by_id, fi.plaintext_a);

    let _ = fs::remove_dir_all(&base);
}

/// 13/13 负向向量：Rust 端复现 PoC 的 mutate + expect。
#[test]
fn negative_vectors_13_of_13() {
    let v = load();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let base = temp_dir("neg");
    let vault = base.join("vault");
    create_vault(&vault, &fi).expect("create_vault");

    let results = run_negative_vectors(&vault, &fi).expect("run_negative_vectors");
    assert_eq!(results.len(), 13, "must run 13 negative vectors");

    // expect 取自契约（唯一真源），而非 Rust 代码里的硬编码
    let expects = v["negative_vectors"].as_array().expect("negative_vectors array");
    for r in &results {
        let e = expects
            .iter()
            .find(|x| x["id"].as_str() == Some(r.id.as_str()))
            .unwrap_or_else(|| panic!("no contract entry for {}", r.id));
        let expected_codes: Vec<&str> = match &e["expect"] {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().map(|x| x.as_str().unwrap()).collect(),
            _ => panic!("bad expect for {}", r.id),
        };
        assert!(
            expected_codes.contains(&r.got.as_str()),
            "negative[{}]: got {}, expected {:?}",
            r.id,
            r.got,
            expected_codes
        );
        assert!(r.pass, "negative[{}] marked not-pass", r.id);
    }
    let _ = fs::remove_dir_all(&base);
}

/// Recovery Key 展示串 ↔ 原始 32 字节往返（含校验位 + key_epoch）。
#[test]
fn recovery_display_roundtrip() {
    let v = load();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let display = v["fixed_inputs"]["recovery_key_display"].as_str().unwrap();
    assert_eq!(rk_display(&fi.rk_raw, fi.key_epoch), display);
    let (epoch, raw) = recovery_display_to_raw(display).expect("decode display");
    assert_eq!(epoch, fi.key_epoch);
    assert_eq!(raw, fi.rk_raw);
    // 校验位篡改必须被拒
    let mut bad = display.to_string();
    let last = bad.pop().unwrap();
    bad.push(if last != 'Z' { 'Z' } else { 'Y' });
    assert!(recovery_display_to_raw(&bad).is_err(), "checksum must reject tampering");
}

/// 零吊销换口令：旧口令失效、新口令可解、数据不重加密（对象文件逐字节不变）。
#[test]
fn change_credential_is_zero_revocation() {
    let v = load();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let base = temp_dir("chpw");
    let vault = base.join("vault");
    create_vault(&vault, &fi).expect("create_vault");
    let obj_path = vault
        .join("d")
        .join(&fi.object_id_a_b32[..2])
        .join(&fi.object_id_a_b32);
    let before = fs::read(&obj_path).unwrap();

    let new_pw = b"a-brand-new-passphrase\0";
    change_credential(&vault, &fi.passphrase_utf8, "password", "password", new_pw)
        .expect("change_credential");

    // 数据未重加密
    assert_eq!(fs::read(&obj_path).unwrap(), before, "objects must NOT be re-encrypted");
    // 新口令可解
    let opened = full_open(&vault, new_pw, "password").expect("open with new password");
    assert_eq!(opened.exported["hello.txt"], fi.plaintext_a);
    // 旧口令失败
    let err = full_open(&vault, &fi.passphrase_utf8, "password").unwrap_err();
    assert_eq!(err.code, E_AUTH_FAILED);
    // 诚实声明：恢复槽仍能用旧 RK 解开同一数据（零吊销）
    let rk = full_open(&vault, &fi.rk_raw, "recovery").expect("recovery still works");
    assert_eq!(rk.exported["hello.txt"], fi.plaintext_a);

    let _ = fs::remove_dir_all(&base);
}

/// 真吊销 rotate-keys：key_epoch++、新 jti、旧口令打不开新库、新口令 + 新 RK 能打开。
#[test]
fn rotate_keys_is_true_revocation() {
    let v = load();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let base = temp_dir("rotate");
    let old = base.join("old");
    let new = base.join("new");
    create_vault(&old, &fi).expect("create_vault");

    let old_meta: Value =
        serde_json::from_slice(&fs::read(old.join("vault.meta")).unwrap()).unwrap();
    let new_pw = b"rotated-master-password\0";
    let (rk_display_str, _new_jti) =
        rotate_keys(&old, &new, &fi.passphrase_utf8, "password", new_pw)
            .expect("rotate_keys");

    let new_meta: Value =
        serde_json::from_slice(&fs::read(new.join("vault.meta")).unwrap()).unwrap();
    assert_eq!(
        new_meta["key_epoch"].as_u64().unwrap(),
        old_meta["key_epoch"].as_u64().unwrap() + 1,
        "key_epoch must increment"
    );
    assert_ne!(
        new_meta["jti"].as_str().unwrap(),
        old_meta["jti"].as_str().unwrap(),
        "jti must rotate"
    );

    // 旧口令打不开新库
    let err = full_open(&new, &fi.passphrase_utf8, "password").unwrap_err();
    assert_eq!(err.code, E_AUTH_FAILED, "old password must be revoked on new vault");

    // 新口令可解，数据完好
    let opened = full_open(&new, new_pw, "password").expect("open rotated vault");
    assert_eq!(opened.exported["hello.txt"], fi.plaintext_a);
    assert_eq!(opened.exported["empty.bin"], fi.plaintext_b);

    // 新 Recovery Key 展示串可还原并用 recovery 槽解锁
    let (epoch, rk_raw) = recovery_display_to_raw(&rk_display_str).expect("new rk display");
    assert_eq!(epoch, old_meta["key_epoch"].as_u64().unwrap() as u32 + 1);
    let by_rk = full_open(&new, &rk_raw, "recovery").expect("open rotated via recovery");
    assert_eq!(by_rk.exported["hello.txt"], fi.plaintext_a);

    let _ = fs::remove_dir_all(&base);
}

fn copy_tree(src: &std::path::Path, dst: &std::path::Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let d = dst.join(entry.file_name());
        if p.is_dir() {
            copy_tree(&p, &d);
        } else {
            fs::copy(&p, &d).unwrap();
        }
    }
}

/// 确定性模糊测试骨架（vault-format-spec.md §9「模糊测试永不返回未认证明文」）。
/// 不引入外部 fuzz crate：用固定种子的 xorshift 覆盖 vault.meta / manifest /
/// 对象文件的随机单字节翻转与随机截断，断言 ①永不 panic ②Either 报错（且是
/// 已知错误码）或返回与原始明文逐字节一致的内容——绝不返回未认证明文。
///
/// 用「低但仍被 kdf_hard_check 接受」的 KDF 参数建库：m=19456,t=2,p=1 恰是
/// 规范 §3.5 的下界，保证 full_open 走真实解锁路径但每轮 Argon2id 成本最低，
/// 使 200 轮模糊测试保持在秒级（用全量 m=65536,t=3,p=4 会慢 ~20 倍）。
#[test]
fn fuzz_never_returns_unauthenticated_plaintext() {
    let base = temp_dir("fuzz");
    let src = base.join("src");
    let pw = b"fuzz-password".to_vec();
    let rk = vec![0x5au8; 32];
    let mut params = random_vault_params(&pw, &rk, 0).expect("random_vault_params");
    params.kdf = KdfParams { m: 19456, t: 2, p: 1, tag_len: 32, salt_len: 16 };
    let plain_a = b"fuzz-plaintext-\xe4\xbf\x9d\xe9\x99\xa9\xe6\x9f\x9c-A".to_vec();
    let plain_b: Vec<u8> = Vec::new();
    let oid_a = vec![0x11u8; 16];
    let oid_b = vec![0x22u8; 16];
    let specs = vec![
        ObjectSpec {
            oid_raw: oid_a.clone(), name: "hello.txt".into(), kind: "file".into(),
            parent_id_raw: vec![0u8; 16], plaintext: plain_a.clone(),
            nonces: random_object_nonces(plain_a.len()).unwrap(),
            updated_seq: 1, content_version: 1,
        },
        ObjectSpec {
            oid_raw: oid_b.clone(), name: "empty.bin".into(), kind: "file".into(),
            parent_id_raw: vec![0u8; 16], plaintext: plain_b.clone(),
            nonces: random_object_nonces(plain_b.len()).unwrap(),
            updated_seq: 1, content_version: 1,
        },
    ];
    let c = build_vault(&params, &specs).expect("build_vault");
    write_vault(&src, &c).expect("write_vault");
    // 先确认未篡改时能正常解锁（否则后面的断言没有基线）
    assert_eq!(
        full_open(&src, &pw, "password").expect("baseline open").exported["hello.txt"],
        plain_a
    );

    // 收集所有可变文件（相对路径）
    fn collect(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        for e in fs::read_dir(dir).unwrap() {
            let e = e.unwrap();
            let p = e.path();
            if p.is_dir() {
                collect(root, &p, out);
            } else if p.file_name().and_then(|s| s.to_str()) != Some("README.txt") {
                out.push(p.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    let mut targets = Vec::new();
    collect(&src, &src, &mut targets);
    assert!(targets.iter().any(|t| t.ends_with("vault.meta")));
    assert!(targets.iter().any(|t| t.ends_with("manifest")));

    // xorshift64*，固定种子 → 可复现
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let known = [
        E_FORMAT, E_AUTH_FAILED, E_DOWNGRADE, E_MANIFEST, E_CORRUPT, E_MISSING, E_ORPHAN,
    ];

    let work = base.join("work");
    for round in 0..200u32 {
        let _ = fs::remove_dir_all(&work);
        copy_tree(&src, &work);
        let rel = &targets[(next() as usize) % targets.len()];
        let path = work.join(rel);
        let mut data = fs::read(&path).unwrap_or_default();
        if data.is_empty() {
            continue;
        }
        // 一半概率翻转一个随机字节，一半概率截断到随机长度
        if next() & 1 == 0 {
            let i = (next() as usize) % data.len();
            data[i] ^= 1 << (next() % 8) as u8;
            fs::write(&path, &data).unwrap();
        } else {
            let keep = (next() as usize) % data.len();
            data.truncate(keep);
            fs::write(&path, &data).unwrap();
        }

        let pw2 = pw.clone();
        let work2 = work.clone();
        let res = std::panic::catch_unwind(move || full_open(&work2, &pw2, "password"));
        match res {
            Err(_) => panic!("round {} panicked on mutated {:?}", round, rel),
            Ok(Err(e)) => assert!(
                known.contains(&e.code),
                "round {} unexpected error code {} on {:?}",
                round, e.code, rel
            ),
            Ok(Ok(opened)) => {
                // 若有幸仍解锁成功，明文必须与原始逐字节一致（不得部分解密/串改）
                assert_eq!(
                    opened.exported.get("hello.txt"),
                    Some(&plain_a),
                    "round {} returned altered plaintext on {:?}",
                    round, rel
                );
            }
        }
    }
    let _ = fs::remove_dir_all(&base);
}
