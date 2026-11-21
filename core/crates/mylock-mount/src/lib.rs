//! mylock-mount —— M3 桌面层：把 `mylock-core` 的密文 vault 暴露成「透明加密文件夹」。
//!
//! # 这一层是什么
//! 一个 **loopback-only WebDAV 服务**。OS 原生 `net use`（Windows）/ `mount_webdav`（macOS）
//! 把 `http://127.0.0.1:<port>/` 映射成盘符/目录，用户看到的目录树是明文，
//! 落盘的始终是 `vault.meta` + `manifest` + `d/` 下逐文件独立加密的密文对象。
//!
//! # 明确不是什么（诚实边界，见 m3-architecture.md / threat-model.md §3.1）
//! - **不是 WinFsp/FUSE 驱动级挂载**：不需要装驱动，也不是内核态文件系统。
//! - **没有 per-request 认证**：只监听 127.0.0.1，但任何能连到该端口的本机进程
//!   都能读写已解锁的挂载点。这是 WebDAV loopback 方案的固有代价。
//! - **解锁态只缓解、不防止**合法采集者；锁定后库外已产生的持久副本不保证消失。
//! - **拒绝在云同步根目录内挂载**（OneDrive/Dropbox/Google Drive/iCloud Drive/坚果云…）。
//! - 内核/桌面壳 ≠ 产品发布 ≠ 五端一致性验收 ≠ 第三方安全审计。
//!
//! # 写回策略（M4 第 2 点：增量写回）
//! 挂载会话内 `vault.meta` 头保持不变（不重算 Argon2id / AES-KW / versionMac）。
//! 每次写操作调用 `mylock_core::rewrite_vault_incremental`：
//! - 内容未变的节点复用原 object_id 与原密文对象文件（不重加密）；
//! - 内容已变/新建的节点才分配全新 128-bit object_id + 全新随机 nonce
//!   （严格 copy-on-write）；
//! - `name` / `parent_id` 只存在于 manifest record、不进对象 AAD，
//!   故重命名/移动只改 manifest、无需重加密任何对象文件；
//! - manifest 每次全新 nonce 重写，「先写新对象 → 原子替换 manifest →
//!   再清理不在记录中的旧孤儿」保证崩溃安全。
//! 相比 M3 的全量重加密，写单文件不再拖累整个库；仍是「每次写回重建整份
//! manifest」，不是增量日志，属已知边界（见 m4-step2-report.md）。

use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::io::SeekFrom;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use bytes::{Buf, Bytes};
use dav_server::davpath::DavPath;
use dav_server::fakels::FakeLs;
use dav_server::fs::{
    DavDirEntry, DavFile, DavFileSystem, DavMetaData, FsError, FsFuture, FsResult, FsStream,
    GuardedFileSystem, OpenOptions, ReadDirMeta,
};
use dav_server::{DavConfig, DavHandler, DavMethodSet};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use mylock_core::{
    b32d, b32e, full_open, unwrap_master, CoreError, IncNode, OpenResult, Unwrapped,
};
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// 云同步根目录拒绝
// ---------------------------------------------------------------------------

/// 命中云同步目录标记时返回人类可读原因。大小写不敏感，逐路径组件匹配。
pub fn is_sync_root(path: &Path) -> Option<String> {
    const MARKERS: &[&str] = &[
        "onedrive",
        "dropbox",
        "google drive",
        "googledrive",
        "iclouddrive",
        "icloud drive",
        "坚果云",
        "nutstore",
        "jianguoyun",
        "sharepoint",
        "pcloud",
        "mega.nz",
        "box sync",
        "seafile",
        "百度网盘",
        "阿里云盘",
        "同步空间",
    ];
    for comp in path.components() {
        let s = comp.as_os_str().to_string_lossy().to_lowercase();
        for m in MARKERS {
            if s.contains(m) {
                return Some(format!("路径组件命中云同步目录标记「{}」：{}", m, s));
            }
        }
    }
    None
}

fn ensure_not_sync_root(path: &Path) -> Result<(), CoreError> {
    match is_sync_root(path) {
        Some(reason) => Err(CoreError::new(
            "E_SYNC_ROOT",
            format!(
                "拒绝在云同步目录内挂载 vault：{}。请把 vault 放到本地磁盘或离线移动介质；\
                 若要备份，请同步整个密文容器（vault.meta + manifest + d/），\
                 而不是在同步根内直接挂载。",
                reason
            ),
        )),
        None => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// 内存明文树
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Node {
    parent: u64, // 0 = 虚拟根 ROOT，不入库
    name: String,
    is_dir: bool,
    data: Vec<u8>,
    modified: SystemTime,
    /// 盘上对象 id（16 raw bytes）。None = 本次会话新建、尚无盘上对象。
    /// 只有明文未变的节点才把它作为 `reuse_oid` 传给增量写回 helper。
    oid: Option<Vec<u8>>,
    /// 明文自上次写回后是否被改动。true → 分配全新 oid + 全新 nonce 重加密。
    /// 重命名/移动只改 `name`/`parent`，它们只进 manifest、不进对象 AAD，
    /// 故只需重写 manifest，不需要把节点标 dirty 重加密对象文件。
    dirty: bool,
}

#[derive(Debug)]
struct Inner {
    nodes: BTreeMap<u64, Node>, // key = 稳定逻辑 id，与盘上 object_id 解耦
    next_id: u64,
}

/// `DavFileSystem` 实现。`Clone + Send + Sync` 使其命中 dav-server 的
/// `impl<Fs: DavFileSystem + Clone + Send + Sync> GuardedFileSystem<()> for Fs` blanket impl。
#[derive(Debug, Clone)]
pub struct VaultFs {
    inner: Arc<Mutex<Inner>>,
    root: PathBuf,
    unwrapped: Unwrapped, // 会话密钥，写回时用；vault.meta 头在会话内不变
    device_id: Vec<u8>,
}

impl VaultFs {
    /// 由一次 `unwrap_master`（拿会话密钥）+ 一次 `full_open`（拿明文树）构造。
    /// 这里刻意接受两次 Argon2id 的一次性启动开销，换取 core 零改动。
    pub fn from_open(
        root: PathBuf,
        u: Unwrapped,
        opened: &OpenResult,
        device_id: Vec<u8>,
    ) -> Self {
        let root_b32 = b32e(&[0u8; 16]); // "AAAA...A"（虚拟根 id，不入库）
        let mut id_of: HashMap<String, u64> = HashMap::new();
        let mut next_id = 1u64;

        // 第一遍：为每个记录分配稳定逻辑 id
        for rec in &opened.records_raw {
            let bid = rec["id"].as_str().unwrap_or("").to_string();
            if !bid.is_empty() && !id_of.contains_key(&bid) {
                id_of.insert(bid, next_id);
                next_id += 1;
            }
        }
        // 第二遍：按 parent_id 建层级，明文来自 exported_by_id（不是按 name 平铺）
        let mut nodes: BTreeMap<u64, Node> = BTreeMap::new();
        for rec in &opened.records_raw {
            let bid = rec["id"].as_str().unwrap_or("");
            let id = match id_of.get(bid) {
                Some(v) => *v,
                None => continue,
            };
            let parent_b = rec["parent_id"].as_str().unwrap_or(&root_b32);
            let parent = if parent_b.is_empty() || parent_b == root_b32 {
                0
            } else {
                // 父记录缺失（孤儿）：挂到根，避免静默丢数据
                *id_of.get(parent_b).unwrap_or(&0)
            };
            nodes.insert(
                id,
                Node {
                    parent,
                    name: rec["name"].as_str().unwrap_or("unnamed").to_string(),
                    is_dir: rec["kind"].as_str() == Some("dir"),
                    data: opened.exported_by_id.get(bid).cloned().unwrap_or_default(),
                    modified: SystemTime::now(),
                    // 保留盘上 object_id，供增量写回复用；解析失败则降级为
                    // 新建节点（下次写回会分配全新 oid 重加密，安全但不省 IO）。
                    oid: b32d(bid).ok(),
                    dirty: false,
                },
            );
        }
        VaultFs {
            inner: Arc::new(Mutex::new(Inner { nodes, next_id })),
            root,
            unwrapped: u,
            device_id,
        }
    }

    /// 把当前内存明文树写回磁盘（M4 第 2 点：增量）。
    ///
    /// 持有内存锁跨越「快照 → 增量写回 → oid 回写」全过程，使并发写原子化：
    /// `rewrite_vault_incremental` 只读写磁盘上的 `d/` 与 `manifest`、不回调本结构，
    /// 不会与这把锁互等；且这是同步函数、没有 `await`，持锁做文件 IO 无
    /// `Future: Send` 问题。相比 M3「先出锁再写回」，这样还能保证「把新 oid
    /// 写回节点并清 dirty」不会被另一路并发写穿插，避免丢失改动。
    ///
    /// 内容未变的节点以 `reuse_oid` 传给 helper 复用其密文对象文件；内容已变
    /// （`dirty=true`）或新建（`oid=None`）的节点才分配全新 oid + 全新 nonce。
    /// 重命名/移动只改 `name`/`parent`，二者仅在 manifest、不进对象 AAD，故无需
    /// 标 dirty，靠每次必写的 manifest 反映即可。
    pub fn persist(&self) -> FsResult<()> {
        let mut inner = self.inner.lock().map_err(|_| FsError::GeneralFailure)?;

        let mut ids: Vec<u64> = Vec::with_capacity(inner.nodes.len());
        let mut idx: BTreeMap<u64, usize> = BTreeMap::new();
        let mut inc: Vec<IncNode> = Vec::with_capacity(inner.nodes.len());
        for (id, n) in &inner.nodes {
            let pos = inc.len();
            idx.insert(*id, pos);
            ids.push(*id);
            // 只有「非脏 + 持有有效盘上 oid」的节点才可复用既有密文对象。
            let reuse_oid = if n.dirty {
                None
            } else {
                n.oid.as_ref().and_then(|v| {
                    if v.len() == 16 {
                        let mut a = [0u8; 16];
                        a.copy_from_slice(v);
                        Some(a)
                    } else {
                        None
                    }
                })
            };
            inc.push(IncNode {
                name: n.name.clone(),
                kind: if n.is_dir { "dir" } else { "file" }.to_string(),
                parent_index: None,
                plaintext: n.data.clone(),
                reuse_oid,
            });
        }
        for (id, n) in &inner.nodes {
            let pi = if n.parent == 0 {
                None
            } else {
                idx.get(&n.parent).copied()
            };
            if let Some(&pos) = idx.get(id) {
                inc[pos].parent_index = pi;
            }
        }

        let all_ids = mylock_core::rewrite_vault_incremental(
            &self.root,
            &self.unwrapped,
            &self.device_id,
            &inc,
        )
        .map_err(|e| {
            eprintln!("[mylock-mount] 写回失败: {}", e);
            FsError::GeneralFailure
        })?;

        // 把每个节点最终采用的 oid 写回内存并清 dirty，供下一次写回复用。
        // 期间并发写被此锁挡住，故不会覆盖别的写刚打上的 dirty 标记。
        for (pos, id) in ids.iter().enumerate() {
            if let (Some(b), Some(n)) = (all_ids.get(pos), inner.nodes.get_mut(id)) {
                if let Ok(raw) = b32d(b) {
                    n.oid = Some(raw);
                    n.dirty = false;
                }
            }
        }
        Ok(())
    }

    fn with_write<F, T>(&self, f: F) -> FsResult<T>
    where
        F: FnOnce(&mut Inner) -> FsResult<T>,
    {
        let out = {
            let mut inner = self.inner.lock().map_err(|_| FsError::GeneralFailure)?;
            f(&mut inner)?
        };
        self.persist()?;
        Ok(out)
    }

    fn resolve(&self, path: &DavPath) -> FsResult<u64> {
        let segs = segments(path);
        let inner = self.inner.lock().map_err(|_| FsError::GeneralFailure)?;
        let mut cur = 0u64;
        for s in &segs {
            match inner
                .nodes
                .iter()
                .find(|(_, n)| n.parent == cur && &n.name == s)
                .map(|(id, _)| *id)
            {
                Some(id) => cur = id,
                None => return Err(FsError::NotFound),
            }
        }
        Ok(cur)
    }

    fn node_is_dir(&self, id: u64) -> FsResult<bool> {
        if id == 0 {
            return Ok(true);
        }
        let inner = self.inner.lock().map_err(|_| FsError::GeneralFailure)?;
        Ok(inner.nodes.get(&id).map(|n| n.is_dir).unwrap_or(false))
    }

    fn node_len(&self, id: u64) -> FsResult<usize> {
        let inner = self.inner.lock().map_err(|_| FsError::GeneralFailure)?;
        Ok(inner.nodes.get(&id).map(|n| n.data.len()).unwrap_or(0))
    }

    fn file_meta(&self, id: u64) -> FsResult<Box<dyn DavMetaData>> {
        let inner = self.inner.lock().map_err(|_| FsError::GeneralFailure)?;
        let n = inner.nodes.get(&id).ok_or(FsError::NotFound)?;
        Ok(Box::new(Meta {
            len: n.data.len() as u64,
            is_dir: n.is_dir,
            modified: n.modified,
        }))
    }
}

/// `DavPath::as_bytes()` 已逐段百分号解码；这里只按 `/` 切段，不自行解码。
fn segments(path: &DavPath) -> Vec<String> {
    let cow = String::from_utf8_lossy(path.as_bytes());
    cow.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// 返回 (父路径, 末段文件名)。根路径没有名字 → Forbidden。
fn split_target(path: &DavPath) -> FsResult<(DavPath, String)> {
    match segments(path).last() {
        Some(name) => Ok((path.parent(), name.clone())),
        None => Err(FsError::Forbidden),
    }
}

// ---------------------------------------------------------------------------
// DavMetaData / DavDirEntry / DavFile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Meta {
    len: u64,
    is_dir: bool,
    modified: SystemTime,
}

impl DavMetaData for Meta {
    fn len(&self) -> u64 {
        self.len
    }
    fn modified(&self) -> FsResult<SystemTime> {
        Ok(self.modified)
    }
    fn is_dir(&self) -> bool {
        self.is_dir
    }
}

struct DirEntry {
    name: Vec<u8>,
    meta: Meta,
}

impl DavDirEntry for DirEntry {
    fn name(&self) -> Vec<u8> {
        self.name.clone()
    }
    fn metadata(&'_ self) -> FsFuture<'_, Box<dyn DavMetaData>> {
        let m: Box<dyn DavMetaData> = Box::new(self.meta.clone());
        Box::pin(async move { Ok(m) })
    }
}

#[derive(Debug)]
struct VaultFile {
    fs: VaultFs,
    node_id: u64,
    pos: u64,
}

impl VaultFile {
    fn write_at(&mut self, data: &[u8]) -> FsResult<()> {
        self.fs.with_write(|inner| {
            let n = inner
                .nodes
                .get_mut(&self.node_id)
                .ok_or(FsError::NotFound)?;
            let start = self.pos as usize;
            let end = start.saturating_add(data.len());
            if n.data.len() < end {
                n.data.resize(end, 0);
            }
            n.data[start..end].copy_from_slice(data);
            n.modified = SystemTime::now();
            // 明文被改写：下次写回必须分配全新 object_id + 全新 nonce 重加密，
            // 绝不能在原密文对象上原地改动（严格 copy-on-write）。
            n.dirty = true;
            Ok(())
        })?;
        self.pos += data.len() as u64;
        Ok(())
    }
}

impl DavFile for VaultFile {
    fn metadata(&'_ mut self) -> FsFuture<'_, Box<dyn DavMetaData>> {
        let r = self.fs.file_meta(self.node_id);
        Box::pin(async move { r })
    }

    fn write_buf(&'_ mut self, mut buf: Box<dyn Buf + Send>) -> FsFuture<'_, ()> {
        // Box<dyn Buf> 不能调用需 Self: Sized 的 copy_to_bytes，改用 chunk/advance 循环。
        let mut data = Vec::with_capacity(buf.remaining());
        while buf.has_remaining() {
            let c = buf.chunk();
            let n = c.len();
            data.extend_from_slice(c);
            buf.advance(n);
        }
        let r = self.write_at(&data);
        Box::pin(async move { r })
    }

    fn write_bytes(&'_ mut self, buf: Bytes) -> FsFuture<'_, ()> {
        let r = self.write_at(&buf);
        Box::pin(async move { r })
    }

    fn read_bytes(&'_ mut self, count: usize) -> FsFuture<'_, Bytes> {
        let mut read_len = 0usize;
        let r: FsResult<Bytes> = (|| {
            let inner = self
                .fs
                .inner
                .lock()
                .map_err(|_| FsError::GeneralFailure)?;
            let n = inner.nodes.get(&self.node_id).ok_or(FsError::NotFound)?;
            let start = (self.pos as usize).min(n.data.len());
            let end = start.saturating_add(count).min(n.data.len());
            read_len = end - start;
            Ok(Bytes::copy_from_slice(&n.data[start..end]))
        })();
        self.pos += read_len as u64;
        Box::pin(async move { r })
    }

    fn seek(&'_ mut self, pos: SeekFrom) -> FsFuture<'_, u64> {
        let len = self.fs.node_len(self.node_id).unwrap_or(0) as i64;
        let raw = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::End(d) => len + d,
            SeekFrom::Current(d) => self.pos as i64 + d,
        };
        let new_pos = raw.max(0) as u64;
        self.pos = new_pos;
        Box::pin(async move { Ok(new_pos) })
    }

    fn flush(&'_ mut self) -> FsFuture<'_, ()> {
        // with_write 已在每次写时持久化，flush 无需再写一次。
        Box::pin(async move { Ok::<(), FsError>(()) })
    }
}

// ---------------------------------------------------------------------------
// DavFileSystem
// ---------------------------------------------------------------------------

impl DavFileSystem for VaultFs {
    fn open<'a>(
        &'a self,
        path: &'a DavPath,
        options: OpenOptions,
    ) -> FsFuture<'a, Box<dyn DavFile>> {
        let r: FsResult<Box<dyn DavFile>> = (|| {
            let want_write = options.write
                || options.append
                || options.truncate
                || options.create
                || options.create_new;
            match self.resolve(path) {
                Ok(id) if id != 0 => {
                    if options.create_new {
                        return Err(FsError::Exists);
                    }
                    if want_write {
                        self.with_write(|inner| {
                            let n = inner.nodes.get_mut(&id).ok_or(FsError::NotFound)?;
                            if n.is_dir {
                                return Err(FsError::Forbidden);
                            }
                            if options.truncate {
                                n.data.clear();
                                n.modified = SystemTime::now();
                                // truncate 也是明文改动 → 必须重加密（即便内容清空）。
                                n.dirty = true;
                            }
                            Ok(())
                        })?;
                    }
                    let pos = if options.append {
                        self.node_len(id)? as u64
                    } else {
                        0
                    };
                    Ok(Box::new(VaultFile {
                        fs: self.clone(),
                        node_id: id,
                        pos,
                    }) as Box<dyn DavFile>)
                }
                Ok(_) => Err(FsError::Forbidden), // 虚拟根不能当文件打开
                Err(FsError::NotFound) => {
                    if !(options.create || options.create_new) {
                        return Err(FsError::NotFound);
                    }
                    let (parent_path, name) = split_target(path)?;
                    let parent = self.resolve(&parent_path)?;
                    if !self.node_is_dir(parent)? {
                        return Err(FsError::Forbidden);
                    }
                    let id = self.with_write(|inner| {
                        if inner
                            .nodes
                            .values()
                            .any(|n| n.parent == parent && n.name == name)
                        {
                            return Err(FsError::Exists);
                        }
                        let id = inner.next_id;
                        inner.next_id += 1;
                        inner.nodes.insert(
                            id,
                            Node {
                                parent,
                                name,
                                is_dir: false,
                                data: Vec::new(),
                                modified: SystemTime::now(),
                                // 会话内新建，尚无盘上对象 → 首次写回分配全新 oid。
                                oid: None,
                                dirty: true,
                            },
                        );
                        Ok(id)
                    })?;
                    Ok(Box::new(VaultFile {
                        fs: self.clone(),
                        node_id: id,
                        pos: 0,
                    }) as Box<dyn DavFile>)
                }
                Err(e) => Err(e),
            }
        })();
        Box::pin(async move { r })
    }

    fn read_dir<'a>(
        &'a self,
        path: &'a DavPath,
        _meta: ReadDirMeta,
    ) -> FsFuture<'a, FsStream<Box<dyn DavDirEntry>>> {
        let items: Vec<FsResult<Box<dyn DavDirEntry>>> = (|| {
            let dir_id = self.resolve(path)?;
            if !self.node_is_dir(dir_id)? {
                return Err(FsError::Forbidden);
            }
            let inner = self.inner.lock().map_err(|_| FsError::GeneralFailure)?;
            let mut out: Vec<FsResult<Box<dyn DavDirEntry>>> = Vec::new();
            for n in inner.nodes.values().filter(|n| n.parent == dir_id) {
                let meta = Meta {
                    len: n.data.len() as u64,
                    is_dir: n.is_dir,
                    modified: n.modified,
                };
                let e: Box<dyn DavDirEntry> = Box::new(DirEntry {
                    name: n.name.as_bytes().to_vec(),
                    meta,
                });
                out.push(Ok(e));
            }
            Ok(out)
        })()
        .unwrap_or_else(|e| vec![Err(e)]);
        let stream: FsStream<Box<dyn DavDirEntry>> =
            Box::pin(futures_util::stream::iter(items));
        Box::pin(async move { Ok(stream) })
    }

    fn metadata<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, Box<dyn DavMetaData>> {
        let r: FsResult<Box<dyn DavMetaData>> = (|| -> FsResult<Box<dyn DavMetaData>> {
            let id = self.resolve(path)?;
            if id == 0 {
                return Ok(Box::new(Meta {
                    len: 0,
                    is_dir: true,
                    modified: SystemTime::now(),
                }));
            }
            self.file_meta(id)
        })();
        Box::pin(async move { r })
    }

    fn create_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, ()> {
        let r: FsResult<()> = (|| {
            let (parent_path, name) = split_target(path)?;
            let parent = self.resolve(&parent_path)?;
            if !self.node_is_dir(parent)? {
                return Err(FsError::Forbidden);
            }
            self.with_write(|inner| {
                if inner
                    .nodes
                    .values()
                    .any(|n| n.parent == parent && n.name == name)
                {
                    return Err(FsError::Exists);
                }
                let id = inner.next_id;
                inner.next_id += 1;
                inner.nodes.insert(
                    id,
                    Node {
                        parent,
                        name,
                        is_dir: true,
                        data: Vec::new(), // 目录也带空明文对象
                        modified: SystemTime::now(),
                        oid: None,    // 新建目录，首次写回分配全新 oid
                        dirty: true,
                    },
                );
                Ok(())
            })
        })();
        Box::pin(async move { r })
    }

    fn remove_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, ()> {
        let r: FsResult<()> = (|| {
            let id = self.resolve(path)?;
            if id == 0 {
                return Err(FsError::Forbidden);
            }
            self.with_write(|inner| {
                let is_dir = inner.nodes.get(&id).ok_or(FsError::NotFound)?.is_dir;
                let has_children = inner.nodes.values().any(|n| n.parent == id);
                if !is_dir || has_children {
                    return Err(FsError::Forbidden);
                }
                inner.nodes.remove(&id);
                Ok(())
            })
        })();
        Box::pin(async move { r })
    }

    fn remove_file<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, ()> {
        let r: FsResult<()> = (|| {
            let id = self.resolve(path)?;
            if id == 0 {
                return Err(FsError::Forbidden);
            }
            self.with_write(|inner| {
                let is_dir = inner.nodes.get(&id).ok_or(FsError::NotFound)?.is_dir;
                if is_dir {
                    return Err(FsError::Forbidden);
                }
                inner.nodes.remove(&id);
                Ok(())
            })
        })();
        Box::pin(async move { r })
    }

    fn rename<'a>(&'a self, from: &'a DavPath, to: &'a DavPath) -> FsFuture<'a, ()> {
        let r: FsResult<()> = (|| {
            let (to_parent_path, to_name) = split_target(to)?;
            let from_id = self.resolve(from)?;
            if from_id == 0 {
                return Err(FsError::Forbidden);
            }
            let to_parent = self.resolve(&to_parent_path)?;
            if !self.node_is_dir(to_parent)? {
                return Err(FsError::Forbidden);
            }
            self.with_write(|inner| {
                if let Some(tid) = inner
                    .nodes
                    .iter()
                    .find(|(_, n)| n.parent == to_parent && n.name == to_name)
                    .map(|(id, _)| *id)
                {
                    if tid == from_id {
                        return Ok(());
                    }
                    let target_is_dir = inner.nodes.get(&tid).unwrap().is_dir;
                    let target_has_children = inner.nodes.values().any(|n| n.parent == tid);
                    if target_is_dir || target_has_children {
                        return Err(FsError::Exists);
                    }
                    inner.nodes.remove(&tid);
                }
                let n = inner.nodes.get_mut(&from_id).ok_or(FsError::NotFound)?;
                n.parent = to_parent;
                n.name = to_name;
                n.modified = SystemTime::now();
                Ok(())
            })
        })();
        Box::pin(async move { r })
    }
}

// ---------------------------------------------------------------------------
// loopback WebDAV 服务
// ---------------------------------------------------------------------------

/// 构造 WebDAV handler（不绑定端口，便于测试用 0 端口）。
/// 会拒绝云同步根目录内的 vault，并做一次全头认证的解锁。
pub fn build_handler(
    root: &Path,
    secret: &[u8],
    slot: &str,
    device_id: Vec<u8>,
) -> Result<DavHandler<()>, CoreError> {
    ensure_not_sync_root(root)?;
    let u = unwrap_master(root, secret, slot)?;
    let opened = full_open(root, secret, slot)?;
    let fs = VaultFs::from_open(root.to_path_buf(), u, &opened, device_id);
    let cfg: DavConfig<()> = DavHandler::builder();
    Ok(cfg
        .filesystem(Box::new(fs) as Box<dyn GuardedFileSystem<()>>)
        .locksystem(FakeLs::new())
        .methods(DavMethodSet::all())
        .build_handler())
}

/// accept 循环。绑定地址由调用方决定（应始终为 127.0.0.1）；
/// 每个连接再做一次 `peer.ip().is_loopback()` 双保险。
pub async fn run_listener(handler: DavHandler<()>, listener: TcpListener) {
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[mylock-mount] accept 失败: {}", e);
                continue;
            }
        };
        if !peer.ip().is_loopback() {
            eprintln!("[mylock-mount] 拒绝非 loopback 来源 {}", peer);
            continue;
        }
        let dav = handler.clone();
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let dav = dav.clone();
                async move { Ok::<_, Infallible>(dav.handle(req).await) }
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("[mylock-mount] 连接错误: {}", e);
            }
        });
    }
}

fn print_banner(addr: SocketAddr) {
    eprintln!("====================================================================");
    eprintln!(" mylock M3 · loopback-only WebDAV 透明加密文件夹");
    eprintln!(" 监听 http://{}/", addr);
    eprintln!();
    eprintln!(" Windows 挂载：  net use Z: http://{}/", addr);
    eprintln!(" Windows 卸载：  net use Z: /delete",);
    eprintln!(" macOS   挂载：  mount_webdav http://{}/ /mnt/mylock", addr);
    eprintln!();
    eprintln!(" 【安全边界·务必阅读】");
    eprintln!(" - 仅监听 127.0.0.1，但 WebDAV loopback 没有 per-request 认证：");
    eprintln!("   任何能连到该端口的本机进程都能读写【已解锁】的挂载点。");
    eprintln!(" - 解锁态只能「缓解」而非「防止」合法采集者；锁定后库外已产生的");
    eprintln!("   持久副本不保证消失（threat-model.md §3.1）。");
    eprintln!(" - 已拒绝在云同步根目录内挂载；请勿把 vault 放进 OneDrive/Dropbox/");
    eprintln!("   Google Drive/iCloud Drive/坚果云 后直接挂载。");
    eprintln!(" - 这不是 WinFsp/FUSE 驱动级挂载，也不需要安装任何驱动。");
    eprintln!(" - 内核/桌面壳 ≠ 产品发布 ≠ 五端一致性验收 ≠ 第三方安全审计。");
    eprintln!("====================================================================");
}

/// 便捷入口：绑定 `127.0.0.1:port` 后一直服务（Ctrl+C 结束）。
pub async fn serve(
    root: PathBuf,
    secret: Vec<u8>,
    slot: String,
    port: u16,
    device_id: Vec<u8>,
) -> Result<(), CoreError> {
    let handler = build_handler(&root, &secret, &slot, device_id)?;
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
        .await
        .map_err(|e| CoreError::new("E_SERVE", format!("loopback bind 失败: {}", e)))?;
    let addr = listener
        .local_addr()
        .map_err(|e| CoreError::new("E_SERVE", format!("local_addr 失败: {}", e)))?;
    print_banner(addr);
    run_listener(handler, listener).await;
    Ok(())
}
