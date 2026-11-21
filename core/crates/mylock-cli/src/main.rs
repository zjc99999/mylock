//! mylock —— 独立恢复 CLI（§6.2）。
//!
//! 设计约束（来自 vault-format-spec.md v1.1.1 / crypto-review-m1.md）：
//! - 不依赖主 App、不联网、不读任何账号/云配置；与核心 crate 一起开源。
//! - 解锁前阶段只做结构检查 + KDF 硬校验，收敛输出，不泄密、不做 oracle。
//! - `mount` 是唯一依赖 OS 的命令，本内核 CLI 明确拒绝实现（留在桌面层）。
//! - 诚实声明：换口令/换 RK 是「零吊销」；真吊销只有 `rotate-keys` +
//!   物理销毁旧介质。本 CLI 会在相应命令成功后打印该声明。

use std::fs;
use std::path::{Path, PathBuf};

use mylock_core::*;

fn usage() -> &'static str {
    "\
mylock —— 本地优先密文容器的独立恢复 CLI（M2 内核，非产品发布）

用法：
  mylock info <root>
      打印公开头（format / cipher / kdf / key_epoch / jti），不解密、不需凭据。

  mylock verify <root> [--json]
      阶段1：解锁前结构检查 + KDF 硬校验。收敛输出，不区分具体对象。

  mylock create <root> --password <pw> [--file <path> ...]
      用真正的随机密钥/ID/nonce 建一个空容器；打印新 Recovery Key 展示串与 jti。
      --file 可重复，把文件内容作为对象写入（对象名取文件名）。

  mylock export <root> --out <dir> (--password <pw> | --recovery <display>)
      完整解锁 + 阶段2 校验 + 把全部明文对象导出到 <dir>。

  mylock export-object <root> --object <base32-id> --out <file>
                       (--password <pw> | --recovery <display>)
      独立恢复路径：manifest 损坏时凭 object_id 直接解密单个对象，不读 manifest。

  mylock change-password <root> --password <old> --new <new>
      零吊销换主口令（仅重包 password 槽，不重加密数据）。

  mylock rotate-recovery <root> --password <pw>
      零吊销换 Recovery Key（生成新 RK 并重包 recovery 槽，打印新展示串）。

  mylock rotate-keys <old-root> <new-root> --password <pw> --new <newpw>
      真吊销：新 Master/MAC/salt/jti/RK + key_epoch++ + 全量重加密到新容器。
      旧容器须由用户物理销毁；本命令不会删除旧目录。

  mylock mount <root>
      拒绝实现：透明挂载依赖桌面层，独立恢复 CLI 不提供。
      请改用 M3 桌面层的独立服务：
        mylock-serve --root <root> --password <pw> [--port 4918]
      然后用 net use Z: http://127.0.0.1:4918/ 映射为盘符。

凭据：
  --password <pw>      主口令（UTF-8 字节，保留 NULL/非 ASCII）
  --recovery <display> Recovery Key 展示串（E###-XXXX ...），自动校验校验位
  --new <newpw>        新主口令
  --new-recovery <d>   （未用；rotate-recovery 自动生成新 RK）
"
}

fn err(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    std::process::exit(2);
}

/// 解析 `(secret_bytes, slot)`。两种凭据互斥。
fn resolve_secret(a: &Args) -> (Vec<u8>, String) {
    let pw = a.get("password");
    let rk = a.get("recovery");
    match (pw, rk) {
        (Some(p), None) => (p.as_bytes().to_vec(), "password".to_string()),
        (None, Some(d)) => match recovery_display_to_raw(d) {
            Ok((_, raw)) => (raw, "recovery".to_string()),
            Err(x) => err(&format!("recovery display invalid: {}", x)),
        },
        (Some(_), Some(_)) => err("use exactly one of --password / --recovery"),
        (None, None) => err("a credential is required: --password or --recovery"),
    }
}

struct Args {
    pos: Vec<String>,
    flags: std::collections::BTreeMap<String, Vec<String>>,
}
impl Args {
    fn parse() -> Args {
        let mut pos = Vec::new();
        let mut flags: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        let mut it = std::env::args().skip(1);
        while let Some(tok) = it.next() {
            // 布尔 flag 必须先判：否则会被 strip_prefix 当成需要取值的选项而误吞下一个 arg。
            if tok == "--json" {
                flags.entry("json".to_string()).or_default().push(String::new());
            } else if tok == "-h" || tok == "--help" {
                flags.entry("help".to_string()).or_default().push(String::new());
            } else if let Some(k) = tok.strip_prefix("--") {
                let v = it.next().unwrap_or_else(|| err(&format!("--{} needs a value", k)));
                flags.entry(k.to_string()).or_default().push(v);
            } else {
                pos.push(tok);
            }
        }
        Args { pos, flags }
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.flags.get(k).and_then(|v| v.first()).map(|s| s.as_str())
    }
    fn get_all(&self, k: &str) -> Vec<&str> {
        self.flags.get(k).map(|v| v.iter().map(|s| s.as_str()).collect()).unwrap_or_default()
    }
    fn has(&self, k: &str) -> bool {
        self.flags.contains_key(k)
    }
    fn root(&self, idx: usize) -> PathBuf {
        PathBuf::from(self.pos.get(idx).unwrap_or_else(|| err("missing <root> path")))
    }
}

fn main() {
    let a = Args::parse();
    if a.pos.is_empty() || a.has("h") || a.has("help") {
        print!("{}", usage());
        return;
    }
    let cmd = a.pos[0].as_str();
    let r = match cmd {
        "info" => cmd_info(&a),
        "verify" => cmd_verify(&a),
        "create" => cmd_create(&a),
        "export" => cmd_export(&a),
        "export-object" => cmd_export_object(&a),
        "change-password" => cmd_change_password(&a),
        "rotate-recovery" => cmd_rotate_recovery(&a),
        "rotate-keys" => cmd_rotate_keys(&a),
        "mount" => cmd_mount(),
        _ => err(&format!("unknown command '{}' (try --help)", cmd)),
    };
    if let Err(x) = r {
        eprintln!("{}", x);
        std::process::exit(1);
    }
}

fn cmd_info(a: &Args) -> Result<(), CoreError> {
    let meta = load_meta(&a.root(1))?;
    let kdf = meta.get("kdf").and_then(serde_json::Value::as_object);
    println!("magic        : {}", meta.get("magic").and_then(|v| v.as_str()).unwrap_or("?"));
    println!("format       : {}", meta.get("format").and_then(|v| v.as_u64()).unwrap_or(0));
    println!("cipher       : {}", meta.get("cipher").and_then(|v| v.as_str()).unwrap_or("?"));
    println!("key_epoch    : {}", meta.get("key_epoch").and_then(|v| v.as_u64()).unwrap_or(0));
    println!("jti          : {}", meta.get("jti").and_then(|v| v.as_str()).unwrap_or("?"));
    if let Some(k) = kdf {
        println!(
            "kdf          : alg={} m={} t={} p={} salt_len={}B",
            k.get("alg").and_then(|v| v.as_str()).unwrap_or("?"),
            k.get("m").and_then(|v| v.as_u64()).unwrap_or(0),
            k.get("t").and_then(|v| v.as_u64()).unwrap_or(0),
            k.get("p").and_then(|v| v.as_u64()).unwrap_or(0),
            b64d(k.get("salt").and_then(|v| v.as_str()).unwrap_or(""))?.len()
        );
    }
    println!("gc_pending   : {}", meta.get("gc_pending").and_then(|v| v.as_bool()).unwrap_or(false));
    println!("(公开头不含用户数据；未解密、未使用凭据)");
    Ok(())
}

fn cmd_verify(a: &Args) -> Result<(), CoreError> {
    let p = verify_phase1(&a.root(1))?;
    if a.has("json") {
        println!(
            "{{\"phase\":\"pre-unlock\",\"result\":\"{}\",\"objects\":{},\"bytes\":{}}}",
            p.result, p.objects, p.bytes
        );
    } else {
        println!("phase        : pre-unlock");
        println!("result       : {}", p.result);
        println!("objects      : {}", p.objects);
        println!("bytes        : {}", p.bytes);
    }
    println!("(阶段1 只检结构与 KDF 参数；verify 绿不能证明是最新版，也不检恶意篡改)");
    Ok(())
}

fn cmd_create(a: &Args) -> Result<(), CoreError> {
    let root = a.root(1);
    let pw = a.get("password").unwrap_or_else(|| err("create needs --password"));
    let rk = random_bytes(32)?;
    let params = random_vault_params(pw.as_bytes(), &rk, 0)?;
    let mut specs = Vec::new();
    let root_id = vec![0u8; 16];
    for f in a.get_all("file") {
        let p = Path::new(f);
        let name = p
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_else(|| err(&format!("bad file name: {}", f)))
            .to_string();
        let data = fs::read(p).map_err(|x| CoreError::new(E_FORMAT, format!("read {}: {}", f, x)))?;
        let nonces = random_object_nonces(data.len())?;
        specs.push(ObjectSpec {
            oid_raw: random_bytes(16)?,
            name,
            kind: "file".into(),
            parent_id_raw: root_id.clone(),
            plaintext: data,
            nonces,
            updated_seq: 1,
            content_version: 1,
        });
    }
    let c = build_vault(&params, &specs)?;
    write_vault(&root, &c)?;
    println!("created      : {}", root.display());
    println!("objects      : {}", specs.len());
    println!("jti          : {}", params.jti);
    println!("key_epoch    : 0");
    println!("recovery_key : {}", rk_display(&rk, 0));
    println!("(请离线抄存 Recovery Key；换口令/换 RK 为零吊销，真吊销用 rotate-keys)");
    Ok(())
}

fn cmd_export(a: &Args) -> Result<(), CoreError> {
    let root = a.root(1);
    let out = PathBuf::from(a.get("out").unwrap_or_else(|| err("export needs --out <dir>")));
    let (secret, slot) = resolve_secret(a);
    let opened = full_open(&root, &secret, &slot)?;
    fs::create_dir_all(&out)
        .map_err(|x| CoreError::new(E_FORMAT, format!("mkdir {}: {}", out.display(), x)))?;
    for (name, data) in &opened.exported {
        let dest = out.join(name);
        fs::write(&dest, data)
            .map_err(|x| CoreError::new(E_FORMAT, format!("write {}: {}", dest.display(), x)))?;
        println!("exported     : {} ({}B)", dest.display(), data.len());
    }
    if !opened.orphans.is_empty() {
        println!(
            "warning      : {} orphan object(s) not referenced by manifest",
            opened.orphans.len()
        );
    }
    println!("records      : {}", opened.records);
    println!("(明文导出后其生命周期脱离本产品保护；请自行确保落盘位置安全)");
    Ok(())
}

fn cmd_export_object(a: &Args) -> Result<(), CoreError> {
    let root = a.root(1);
    let id = a.get("object").unwrap_or_else(|| err("export-object needs --object <base32-id>"));
    let out = PathBuf::from(a.get("out").unwrap_or_else(|| err("export-object needs --out <file>")));
    let (secret, slot) = resolve_secret(a);
    let data = export_object_by_id(&root, &secret, &slot, id)?;
    fs::write(&out, &data)
        .map_err(|x| CoreError::new(E_FORMAT, format!("write {}: {}", out.display(), x)))?;
    println!("exported     : {} ({}B)", out.display(), data.len());
    println!("(独立恢复路径：未读取 manifest；仍完成全头 versionMac 认证)");
    Ok(())
}

fn cmd_change_password(a: &Args) -> Result<(), CoreError> {
    let root = a.root(1);
    let old = a.get("password").unwrap_or_else(|| err("change-password needs --password <old>"));
    let new = a.get("new").unwrap_or_else(|| err("change-password needs --new <new>"));
    change_credential(&root, old.as_bytes(), "password", "password", new.as_bytes())?;
    println!("changed      : password slot re-wrapped");
    println!("ZERO-REVOCATION: 持有旧 vault.meta + 旧口令者仍可解当前数据；");
    println!("                 旧 Recovery Key 也仍有效。真吊销须 rotate-keys + 物理销毁旧介质。");
    Ok(())
}

fn cmd_rotate_recovery(a: &Args) -> Result<(), CoreError> {
    let root = a.root(1);
    let pw = a.get("password").unwrap_or_else(|| err("rotate-recovery needs --password"));
    let meta = load_meta(&root)?;
    let (m, t, p, salt) = kdf_hard_check(&meta)?;
    let u = unwrap_master(&root, pw.as_bytes(), "password")?;
    let kdf = KdfParams { m, t, p, tag_len: 32, salt_len: salt.len() };
    let new_rk = random_bytes(32)?;
    let epoch = meta.get("key_epoch").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
    rewrap_and_sign(&root, &u.master, &u.mac_key, "recovery", &new_rk, &kdf, &salt)?;
    println!("changed      : recovery slot re-wrapped");
    println!("recovery_key : {}", rk_display(&new_rk, epoch));
    println!("ZERO-REVOCATION: 旧 Recovery Key 失效，但持有旧 vault.meta + 旧口令者仍可解当前数据。");
    println!("                 真吊销须 rotate-keys + 物理销毁旧介质。");
    Ok(())
}

fn cmd_rotate_keys(a: &Args) -> Result<(), CoreError> {
    let old = a.root(1);
    let new = a.root(2);
    let pw = a.get("password").unwrap_or_else(|| err("rotate-keys needs --password"));
    let newpw = a.get("new").unwrap_or_else(|| err("rotate-keys needs --new <newpw>"));
    let (rk_display_str, jti) = rotate_keys(&old, &new, pw.as_bytes(), "password", newpw.as_bytes())?;
    println!("rotated      : {}", new.display());
    println!("jti          : {}", jti);
    println!("recovery_key : {}", rk_display_str);
    println!("TRUE REVOCATION: 旧口令 / 旧 RK / 旧 Master 均打不开新容器。");
    println!("ACTION REQUIRED: 物理销毁旧介质 {} 及其所有副本（含云盘/备份）。", old.display());
    println!("                 本命令没有、也不会替你删除旧目录。");
    Ok(())
}

fn cmd_mount() -> Result<(), CoreError> {
    eprintln!("mount 未实现：透明挂载依赖桌面层，独立恢复 CLI 不提供。");
    eprintln!("请使用 export 取明文，或改用 M3 桌面层的独立服务 mylock-serve：");
    eprintln!("    mylock-serve --root <root> --password <pw> [--port 4918]");
    eprintln!("    net use Z: http://127.0.0.1:4918/");
    eprintln!("注意：WebDAV loopback 无 per-request 认证；拒绝在云同步根目录内挂载。");
    Err(CoreError::new(E_FORMAT, "mount is not implemented in the core CLI"))
}
