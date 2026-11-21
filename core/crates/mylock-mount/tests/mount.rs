//! 桌面层测试：VFS 写回、CRUD、云同步根拒绝、WebDAV HTTP 冒烟 + 增量写回。
//!
//! 诚实边界（与 m3-architecture.md / m3-report.md / m4-step2-report.md 一致）：
//! - 这里验证的是 loopback WebDAV 会话内的「内存明文树 → 加密落盘」链路，
//!   不是 WinFsp/FUSE 驱动级挂载，也不是产品发布。
//! - M4 第 2 点起写回为增量：内容未变节点复用原密文对象文件、只重写 manifest；
//!   仍是每次写回重建整份 manifest（不是增量日志），属已知边界。
//! - WebDAV loopback 无 per-request 认证；解锁态只缓解不消除采集风险。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use dav_server::davpath::DavPath;
use dav_server::fs::{DavFileSystem, OpenOptions, ReadDirMeta};
use futures_util::StreamExt;
use mylock_core::{create_vault, full_open, unwrap_master, FixedInputs};
use mylock_mount::{build_handler, is_sync_root, VaultFs};

// ---------------------------------------------------------------------------
// 测试夹具：用 M1 PoC 的固定向量建一个确定性的 vault（hello.txt / empty.bin）
// ---------------------------------------------------------------------------

fn vectors() -> serde_json::Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../poc/test-vectors.json")
        .canonicalize()
        .expect("test-vectors.json must exist");
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

struct Fixture {
    root: PathBuf,
    fi: FixedInputs,
}

fn fixture(tag: &str) -> Fixture {
    let v = vectors();
    let fi = FixedInputs::from_json(&v["fixed_inputs"]);
    let base = std::env::temp_dir().join(format!("mylock-m3-{}-{}", tag, std::process::id()));
    let root = base.join("vault");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&root).unwrap();
    create_vault(&root, &fi).unwrap();
    Fixture { root, fi }
}

fn open_fs(f: &Fixture) -> VaultFs {
    let u = unwrap_master(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    let opened = full_open(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    VaultFs::from_open(f.root.clone(), u, &opened, vec![7u8; 16])
}

/// 列出 d/ 下所有对象文件（b32 object_id → 密文文件逐字节内容）。
/// 增量写回的复用断言靠它验证「未变节点的对象文件字节一字未动」。
fn object_bytes(root: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let d = root.join("d");
    if let Ok(shards) = fs::read_dir(&d) {
        for shard in shards.flatten() {
            if !shard.path().is_dir() {
                continue;
            }
            if let Ok(files) = fs::read_dir(shard.path()) {
                for e in files.flatten() {
                    if let Ok(bytes) = fs::read(e.path()) {
                        out.insert(e.file_name().to_string_lossy().into_owned(), bytes);
                    }
                }
            }
        }
    }
    out
}

fn write_opts() -> OpenOptions {
    OpenOptions {
        read: false,
        write: true,
        append: false,
        truncate: true,
        create: true,
        create_new: false,
        size: None,
        checksum: None,
    }
}

async fn collect_body(body: dav_server::body::Body) -> Vec<u8> {
    let mut body = Box::pin(body);
    let mut out = Vec::new();
    while let Some(chunk) = body.next().await {
        let b = chunk.expect("body stream error");
        out.extend_from_slice(b.as_ref());
    }
    out
}

// ---------------------------------------------------------------------------
// 1. 云同步根拒绝（纯函数 + build_handler 私有 ensure_not_sync_root）
// ---------------------------------------------------------------------------

#[test]
fn sync_root_detection_is_case_insensitive() {
    assert!(is_sync_root(Path::new("C:/Users/x/OneDrive/vault")).is_some());
    assert!(is_sync_root(Path::new("C:/Users/x/onedrive/vault")).is_some());
    assert!(is_sync_root(Path::new("D:/Dropbox/notes/vault")).is_some());
    assert!(is_sync_root(Path::new("C:/Users/x/Google Drive/vault")).is_some());
    assert!(is_sync_root(Path::new("C:/Users/x/坚果云/vault")).is_some());
    assert!(is_sync_root(Path::new("D:/data/plain-vault")).is_none());
}

#[test]
fn build_handler_rejects_cloud_sync_root() {
    let base = std::env::temp_dir()
        .join(format!("mylock-m3-sync-{}-{}", std::process::id(), "a"))
        .join("OneDrive");
    let root = base.join("vault");
    let _ = fs::remove_dir_all(base.parent().unwrap());
    fs::create_dir_all(&root).unwrap();

    // DavHandler 不实现 Debug，故不能用 unwrap_err()；用 match 直接取错误码。
    let code = match build_handler(&root, b"irrelevant", "password", vec![0u8; 16]) {
        Ok(_) => panic!("sync root must be rejected"),
        Err(e) => e.code,
    };
    assert_eq!(code, "E_SYNC_ROOT");
}

// ---------------------------------------------------------------------------
// 2. 写回主链路：新增文件 → 重新 full_open 逐字节一致
//    M4 第 2 点：未变节点复用原对象 id、内容未变的密文文件字节一字不动。
// ---------------------------------------------------------------------------

#[tokio::test]
async fn write_back_persists_new_file_and_keeps_header_unchanged() {
    let f = fixture("writeback");
    let meta_before = fs::read(f.root.join("vault.meta")).unwrap();
    let before = object_bytes(&f.root);
    assert_eq!(before.len(), 2);

    let vfs = open_fs(&f);
    let path = DavPath::new("/new.txt").unwrap();
    let mut file = vfs.open(&path, write_opts()).await.unwrap();
    file.write_bytes(Bytes::from_static(b"brand new secret\n"))
        .await
        .unwrap();
    file.flush().await.unwrap();
    drop(file);

    // 头保持不变：会话内不重算 Argon2id / AES-KW / versionMac。
    let meta_after = fs::read(f.root.join("vault.meta")).unwrap();
    assert_eq!(
        meta_before, meta_after,
        "vault.meta header must remain byte-identical across a mounted session"
    );

    // 增量写回：两个未变对象仍以原 id 存在，且密文文件逐字节未动（真复用，
    // 不是「换了 id 但恰好内容相同」——密文里含全新 nonce，重加密必不同）。
    let after = object_bytes(&f.root);
    assert_eq!(after.len(), 3, "两个老对象 + 一个新对象");
    for (id, bytes) in &before {
        assert_eq!(
            after.get(id),
            Some(bytes),
            "unchanged object {id} must be reused byte-for-byte"
        );
    }
    let fresh: Vec<&String> = after.keys().filter(|k| !before.contains_key(*k)).collect();
    assert_eq!(fresh.len(), 1, "only the new object gets a fresh id");

    // 重新解锁读回：新文件与老文件都逐字节一致。
    let reopened = full_open(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    assert_eq!(
        reopened.exported["new.txt"].as_slice(),
        &b"brand new secret\n"[..]
    );
    assert_eq!(
        reopened.exported["hello.txt"].as_slice(),
        f.fi.plaintext_a.as_slice()
    );
    assert_eq!(reopened.exported["empty.bin"].as_slice(), &b""[..]);
}

// ---------------------------------------------------------------------------
// 2b. 增量写回专项：连续改写单文件只换该文件 oid；rename/删除不误删复用对象
// ---------------------------------------------------------------------------

#[tokio::test]
async fn incremental_writeback_only_rewrites_changed_objects() {
    let f = fixture("inc");
    let before = object_bytes(&f.root);
    assert_eq!(before.len(), 2);

    let vfs = open_fs(&f);

    // 第一次改写 hello.txt：只有它该换新 oid，empty.bin 必须字节不变。
    let hello = DavPath::new("/hello.txt").unwrap();
    let mut file = vfs.open(&hello, write_opts()).await.unwrap();
    file.write_bytes(Bytes::from_static(b"edited once\n"))
        .await
        .unwrap();
    file.flush().await.unwrap();
    drop(file);

    let after1 = object_bytes(&f.root);
    assert_eq!(after1.len(), 2);
    let still: Vec<&String> = after1
        .keys()
        .filter(|k| before.contains_key(*k))
        .collect();
    assert_eq!(still.len(), 1, "exactly one unchanged object is reused");
    // 复用的那一个字节必须与初始完全一致。
    let reused_id = still[0];
    assert_eq!(after1.get(reused_id), before.get(reused_id));

    // 第二次仅重命名 hello.txt → renamed.txt（内容不再改）：
    // name/parent_id 只进 manifest，不进对象 AAD，故对象文件应全部继续复用。
    let renamed = DavPath::new("/renamed.txt").unwrap();
    vfs.rename(&hello, &renamed).await.unwrap();
    let after2 = object_bytes(&f.root);
    assert_eq!(after2, after1, "rename must not re-encrypt any object file");

    // 重新解锁：改名生效、内容正确、empty.bin 仍在。
    let reopened = full_open(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    assert_eq!(reopened.exported["renamed.txt"].as_slice(), b"edited once\n");
    assert_eq!(reopened.exported["empty.bin"].as_slice(), b"");
    assert!(!reopened.exported.contains_key("hello.txt"));

    // 删除一个文件后，另一个未被改动的复用对象仍须保留（清孤儿不误删）。
    vfs.remove_file(&renamed).await.unwrap();
    let after3 = object_bytes(&f.root);
    assert_eq!(after3.len(), 1, "only the untouched empty.bin object remains");
    assert_eq!(after3.get(reused_id), before.get(reused_id));
    let final_open = full_open(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    assert_eq!(final_open.exported["empty.bin"].as_slice(), b"");
}

// ---------------------------------------------------------------------------
// 3. VFS CRUD：建目录 → 目录内写文件 → read_dir → 改名 → 删文件 → 删目录
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vfs_create_dir_write_rename_remove_roundtrip() {
    let f = fixture("crud");
    let vfs = open_fs(&f);

    let docs = DavPath::new("/docs").unwrap();
    vfs.create_dir(&docs).await.unwrap();

    let a = DavPath::new("/docs/a.txt").unwrap();
    let mut file = vfs.open(&a, write_opts()).await.unwrap();
    file.write_bytes(Bytes::from_static(b"inside folder"))
        .await
        .unwrap();
    file.flush().await.unwrap();
    drop(file);

    let entries = vfs
        .read_dir(&docs, ReadDirMeta::Data)
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].as_ref().unwrap().name(), b"a.txt".to_vec());

    let b = DavPath::new("/docs/b.txt").unwrap();
    vfs.rename(&a, &b).await.unwrap();
    vfs.remove_file(&b).await.unwrap();
    vfs.remove_dir(&docs).await.unwrap();

    let reopened = full_open(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    assert_eq!(reopened.exported.len(), 2);
    assert!(reopened.exported.contains_key("hello.txt"));
    assert!(reopened.exported.contains_key("empty.bin"));
}

// ---------------------------------------------------------------------------
// 4. 非 ASCII 文件名：DavPath 逐段百分号解码后不自行解码（b73 A 级事实）
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unicode_filename_roundtrip() {
    let f = fixture("unicode");
    let vfs = open_fs(&f);
    // 真实 WebDAV 客户端把 UTF-8 百分号编码进 URI；DavPath::new 会逐段解码。
    let p = DavPath::new("/%E4%BF%9D%E9%99%A9%E6%9F%9C%F0%9F%94%90.txt").unwrap();
    let mut file = vfs.open(&p, write_opts()).await.unwrap();
    file.write_bytes(Bytes::from_static(b"unicode ok"))
        .await
        .unwrap();
    file.flush().await.unwrap();
    drop(file);

    let reopened = full_open(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    assert_eq!(
        reopened.exported["保险柜🔐.txt"].as_slice(),
        &b"unicode ok"[..]
    );
}

// ---------------------------------------------------------------------------
// 5. WebDAV HTTP 层冒烟：GET 既有文件 → PUT 新文件 → GET 回读 → full_open 验证落盘
// ---------------------------------------------------------------------------

#[tokio::test]
async fn http_put_get_roundtrip() {
    let f = fixture("http");
    let handler = build_handler(&f.root, &f.fi.passphrase_utf8, "password", vec![9u8; 16]).unwrap();

    // GET 既有文件（读路径 + manifest 重建）
    let get = http::Request::builder()
        .method("GET")
        .uri("/hello.txt")
        .body(dav_server::body::Body::empty())
        .unwrap();
    let resp = handler.handle(get).await;
    assert!(resp.status().is_success(), "GET failed: {:?}", resp.status());
    assert_eq!(collect_body(resp.into_body()).await, f.fi.plaintext_a);

    // PUT 新文件（dav-server handle_put 用 create+truncate 走 open，再 write_buf）
    let payload = "written over webdav";
    let put = http::Request::builder()
        .method("PUT")
        .uri("/webdav.txt")
        .header("content-length", payload.len())
        .body(dav_server::body::Body::from(payload))
        .unwrap();
    let resp = handler.handle(put).await;
    assert!(resp.status().is_success(), "PUT failed: {:?}", resp.status());

    // GET 回读
    let get2 = http::Request::builder()
        .method("GET")
        .uri("/webdav.txt")
        .body(dav_server::body::Body::empty())
        .unwrap();
    let resp = handler.handle(get2).await;
    assert!(resp.status().is_success(), "GET2 failed: {:?}", resp.status());
    assert_eq!(collect_body(resp.into_body()).await.as_slice(), payload.as_bytes());

    // 验证真的写进了密文容器（不是只停留在内存树）
    let reopened = full_open(&f.root, &f.fi.passphrase_utf8, "password").unwrap();
    assert_eq!(
        reopened.exported["webdav.txt"].as_slice(),
        payload.as_bytes()
    );
    assert_eq!(
        reopened.exported["hello.txt"].as_slice(),
        f.fi.plaintext_a.as_slice()
    );
}
