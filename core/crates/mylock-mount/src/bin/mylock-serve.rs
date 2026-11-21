//! `mylock-serve` —— M3 桌面层的独立服务二进制。
//! 把 mylock-core 的密文 vault 暴露为 loopback-only WebDAV 透明文件夹。
//!
//! 它刻意不并入 `mylock`（独立恢复 CLI）：恢复 CLI 保持零网络、零 tokio、零 dav-server。

use std::path::PathBuf;

use mylock_core::{random_bytes, recovery_display_to_raw, CoreError};
use mylock_mount::serve;

fn usage() -> &'static str {
    "\
mylock-serve — M3 桌面层：loopback-only WebDAV 透明加密文件夹

用法:
  mylock-serve --root <vault目录> (--password <口令> | --recovery <恢复码>) [--port 4918]

选项:
  --root       vault 目录（含 vault.meta / manifest / d/）
  --password   主口令（UTF-8 字节直接作为 password 槽凭据）
  --recovery   Recovery Key 展示码（如 E000-XXXX-...），走 recovery 槽
  --port       监听端口，默认 4918（只绑定 127.0.0.1）
  -h, --help   显示本帮助

安全边界（务必阅读）:
  * 仅监听 127.0.0.1，但 WebDAV loopback 没有 per-request 认证，
    任何能连到该端口的本机进程都能读写已解锁的挂载点。
  * 解锁态只能缓解、不能防止合法采集者；拒绝在云同步根目录内挂载。
  * 这不是 WinFsp/FUSE 驱动级挂载；Windows 请用 `net use` 映射。
"
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), CoreError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut root: Option<PathBuf> = None;
    let mut password: Option<String> = None;
    let mut recovery: Option<String> = None;
    let mut port: u16 = 4918;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print!("{}", usage());
                return Ok(());
            }
            "--root" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| CoreError::new("E_ARGS", "--root 缺少取值"))?;
                root = Some(PathBuf::from(v));
                i += 2;
            }
            "--password" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| CoreError::new("E_ARGS", "--password 缺少取值"))?;
                password = Some(v.clone());
                i += 2;
            }
            "--recovery" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| CoreError::new("E_ARGS", "--recovery 缺少取值"))?;
                recovery = Some(v.clone());
                i += 2;
            }
            "--port" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| CoreError::new("E_ARGS", "--port 缺少取值"))?;
                port = v
                    .parse::<u16>()
                    .map_err(|_| CoreError::new("E_ARGS", format!("非法端口: {}", v)))?;
                i += 2;
            }
            other => {
                return Err(CoreError::new(
                    "E_ARGS",
                    format!("未知参数 {}（用 --help 查看用法）", other),
                ))
            }
        }
    }

    let root = root.ok_or_else(|| CoreError::new("E_ARGS", "必须提供 --root"))?;
    let (secret, slot) = match (password, recovery) {
        (Some(p), None) => (p.into_bytes(), "password".to_string()),
        (None, Some(r)) => {
            let (_, raw) = recovery_display_to_raw(&r)?;
            (raw, "recovery".to_string())
        }
        (Some(_), Some(_)) => {
            return Err(CoreError::new(
                "E_ARGS",
                "--password 与 --recovery 只能提供其中一个",
            ))
        }
        (None, None) => {
            return Err(CoreError::new(
                "E_ARGS",
                "必须提供 --password 或 --recovery 之一",
            ))
        }
    };
    let device_id = random_bytes(16)?;

    serve(root, secret, slot, port, device_id).await
}
