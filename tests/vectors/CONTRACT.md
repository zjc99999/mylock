# G4 五端 + 恢复 CLI 逐字节一致性契约（M4 第 1 点）

> 状态：**基础设施工厂已落地（2026-11-14）**；「五端全部绿灯」尚未达成，本文件给出
> 每端的激活条件与验收判据，绝不伪造移动端绿灯。
> 上游：[vault-format-spec.md](../../vault-format-spec.md) v1.1.1 §9（G4 定义）、
> [crypto-review-m1.md](../../crypto-review-m1.md) §9【9】（G4 只约束固定向量）、
> [产品分析与概要设计.md](../../产品分析与概要设计.md) v1.9、
> [m3-report.md](../../m3-report.md) §8（硬门槛对照）。

---

## 1. 契约是什么（冻结，不可单方面修改）

| 项 | 值 |
|---|---|
| 唯一逐字节契约 | `tests/vectors/test-vectors.json`（归档副本；`poc/test-vectors.json` 为产出源，两份字节必须相同） |
| 字节数 | 13499 B |
| SHA-256 | `07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939` |
| EOL | CRLF × 190，**无裸 LF**；`.gitattributes` 标 `-text` 强制 Git 不做归一化 |
| 派生输出 | 30 个 `derived_outputs`（IKM/KEK/wrapped blob/versionMac/object_key/对象文件 hex/manifest hex/AAD hex/canonical hex/RK display 等） |
| G4 指纹 | `d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa`（30 项按 key 排序拼 `k=v\n` 后的 SHA-256，8254 B 输入） |
| 负向向量 | 13 条（`negative_vectors`），含 `kdf_oob_m4tib`、`slot_swap`、`orphan_object`（警告非致命）等 |
| 硬性 KDF 约束 | `argon2id(m=65536, t=3, p=4)`——`p=1`（libsodium `crypto_pwhash`）无法复现，跨端解锁会得 `E_AUTH_FAILED` |
| 格式版本 | `format = 1`；改动任何字节级约定必须升 `format` 并重新冻结全部锚 |

**G4 的准确含义**（crypto-review-m1.md §9【9】裁定）：只要求「固定输入的测试向量」
产出逐字节相同密文；**不要求运行时确定性密文**——正常运行使用 CSPRNG 24B nonce，
重复加密同一文件得到不同密文是安全属性，不是缺陷。

---

## 2. 门禁基础设施（本轮已落地并实测）

| 组件 | 路径 | 职责 |
|---|---|---|
| 归档副本 | `tests/vectors/test-vectors.json` | 受门禁保护的**正式归档**；`check_vectors.py` 与 `conformance.rs` 从它读，CI 门禁 4 也从它算指纹 |
| 产出源 | `poc/test-vectors.json` | 由 `poc/mylock_poc.py` 生成；现有两个 Rust 测试（`vectors.rs`、`mount.rs`）仍从这里读，靠下面两道「字节相同」门禁覆盖，故不改动这两个已全绿的测试文件 |
| 完整性锚 | `tests/vectors/SHA256SUMS` | 两份副本的 SHA-256 记录（2 条） |
| EOL 防护 | `.gitattributes` | 两个副本 + 锚文件标 `-text`，禁 CRLF→LF 归一化 |
| 可移植校验器 | `tools/check_vectors.py` | 三平台 CI 共用：锚 / **两副本字节相同** / EOL / JSON 结构 / KDF 约束，任一失败非零退出 |
| 指纹门禁 | `tools/compute_fingerprint.py` | 从归档契约独立算 G4 指纹并**断言**等于冻结常量（非零退出即红），与 Rust 侧互为交叉验证 |
| Rust 门禁 | `core/crates/mylock-core/tests/conformance.rs` | 4 个测试：归档 SHA-256 锚 + `poc == archive` 字节相同、契约指纹、**本平台独立复算指纹**、负向/KDF 不变量 |
| 负向自检 | `tools/gate_selftest.py` | 4 种篡改（单字节翻转 / CRLF→LF / 截断 / 陈旧锚）+ 好路径，验证门禁真能拒绝漂移 |
| CI | `.github/workflows/vectors.yml` | Windows / macOS / Linux 三桌面平台矩阵 |

**为什么「三平台各自独立算同一指纹」等价于跨平台一致性**：每个 runner 用各自的
CPU / OS / 编译器 / Rust 工具链读同一份 13499 B 字节 → 跑 `derive_outputs` → 得到
30 个派生串 → 算 SHA-256 → 与冻结常量比对。三份独立绿灯 = 同一字节输入在三套工具链
上产出同一字节输出，无需 runner 之间通信或共享状态。

**门禁是双向的**：`tools/check_vectors.py`、`tools/compute_fingerprint.py` 与 `conformance.rs`
都必须能拒绝漂移。`tools/gate_selftest.py` 在临时仓库树上做 4 种篡改（单字节翻转、
CRLF→LF 归一化、截断到 1000 B、陈旧 `SHA256SUMS`）+ 1 条未篡改好路径，实测
5/5 全部按预期被拒 / 通过（见 `m4-report.md` §4）；不是只测「好路径」。

---

## 3. 五端 + 恢复 CLI 激活条件与验收判据（诚实状态表）

| # | 端 | 实现载体 | 当前状态 | 激活条件（满足才算该端绿灯） | 验收判据 |
|---|---|---|---|---|---|
| 1 | **Rust 内核（Windows）** | `core/crates/mylock-core` + `tests/vectors.rs` + `tests/conformance.rs` | ✅ **已验收**（M2 起，本轮再加 conformance 4 测试） | 已满足 | `cargo test --workspace` 全绿；`derive_outputs` 30/30 逐字节一致；指纹命中 |
| 2 | **恢复 CLI（Windows）** | `core/crates/mylock-cli`（`mylock` 9 命令） | ✅ **已验收**（M2 CLI 8 步冒烟 + M3 网络冒烟 `export` 一致） | 已满足；恢复 CLI 与内核共用同一 crate，无独立 KDF 路径 | `mylock export` 取回明文与 `export` 段逐字节一致 |
| 3 | **Windows 桌面层** | `core/crates/mylock-mount`（WebDAV）+ `desktop/src-tauri` | ✅ **已验收**（M3：21/0、真实 loopback 冒烟、桌面壳 `cargo check`） | 已满足；透明文件夹不改变容器字节 | 写入后 `full_open` 回读一致、`vault.meta` 头不变 |
| 4 | **macOS（同一份 Rust 源码）** | 同 `mylock-core`，`x86_64-apple-darwin` / `aarch64-apple-darwin` | 🟡 **CI 门禁已配置，未实跑**（本机无 macOS runner） | 在 GitHub Actions `macos-latest` 上 `cargo test --workspace` + `check_vectors.py` 双绿 | 见 §4 CI；指纹命中 `G4_FINGERPRINT` |
| 5 | **Linux（同一份 Rust 源码）** | 同 `mylock-core`，`x86_64-unknown-linux-gnu` | 🟡 **CI 门禁已配置，未实跑**（本机无 Linux runner） | 在 GitHub Actions `ubuntu-latest` 上双绿 | 同上 |
| 6 | **Android** | 计划：JNI/UniFFI 绑定同一 Rust crate | 🔴 **未实现** | ①有 Android 端实现；②在 CI 的 Android target 或设备上跑同一份 `test-vectors.json` 复现；③`p=4` 的 Argon2id 在 Android 上可用（不可用则须评审而非静默改 `p=1`） | 30/30 逐字节 + 13 负向 + 指纹命中 |
| 7 | **iOS** | 计划：Swift 绑定同一 Rust crate | 🔴 **未实现** | ①有 iOS 端实现；②在 CI 的 iOS target / 模拟器上跑同一份向量；③Secure Enclave 仅作本机 wrap，跨端主密钥仍是可移植软件密钥（A 级硬约束，见 mobile-vault-research.md） | 同上 |

> **口径纪律**：本轮交付的是「门禁基础设施 + 三桌面平台 CI 配置 + 契约定义」，
> **不是**「G4 五端验收通过」。移动端两行保持 🔴，直到有真实实现与真实绿灯。
> 任何对外材料不得把「CI 配置完成」写成「五端一致已验收」。

---

## 4. CI 判定规则

`.github/workflows/vectors.yml` 的 `g4-vectors` job 在三平台矩阵上执行：

1. `python3 tools/check_vectors.py`（setup-python 保证三平台都有 `python3`）——非绿即 fail；
2. `cargo test --workspace --locked`——**全部**测试（含 `vectors.rs` 8 集成 + `conformance.rs` 4 + `mount.rs` 6 + 7 KAT 单测）必须 0 failed；
3. `cargo build --workspace --locked`（编译所有二进制，确保门禁不止跑测试目标）；
4. `python3 tools/compute_fingerprint.py`（独立算指纹并断言冻结常量，与 Rust 侧交叉验证）。

`mylock-serve` 的 loopback WebDAV 网络冒烟见 `tools/m3-smoke.ps1`（M3 已实测），
它不属于向量逐字节门禁，故不进本工作流；各平台等价性由同一份 Rust 源码保证。

`g4-gate` 汇总 job 依赖三平台全绿；**不把 Android/iOS 计入** `needs`（它们尚未实现，
写进去只会得到一个永远红或伪造绿的门禁）。

---

## 5. 变更向量的纪律（摩擦力是刻意的）

任何一次向量/格式变更必须**同一次提交**里更新全部 6 处，并写明原因：

1. `poc/test-vectors.json`（产出源，由 `poc/mylock_poc.py` 重跑）
2. `tests/vectors/test-vectors.json`（归档副本，字节必须相同）
3. `tests/vectors/SHA256SUMS`
4. `tools/check_vectors.py` 的 `VECTOR_SHA256` / `FROZEN_CRLF_LINES`
5. `tools/compute_fingerprint.py` 的 `EXPECTED_FINGERPRINT` / `EXPECTED_ENTRIES` / `EXPECTED_INPUT_BYTES`
6. `core/crates/mylock-core/tests/conformance.rs` 的 `EXPECTED_SHA256` / `G4_FINGERPRINT`

（`tools/gate_selftest.py` 的陈旧锚用例也引用了同一常量字面量，同提交一并核对。）

并同步升 `vault-format-spec.md` 的 `format` 版本号、记录到变更日志；**禁止**自动修复、
禁止静默重生锚。任一处漏改，门禁会红——这正是目的。

---

## 6. 诚实边界

- 本文件是**工程契约与门禁定义**，不是第三方安全审计、不是安全认证。
- 三桌面平台 CI 是**配置**（本仓库当前无 `.git`，工作流在推送到远端后才会实跑）；
  本机只实测了 Windows 路径。macOS/Linux/Android/iOS 的绿灯均未被本机观察到。
- G4 只约束固定测试向量；正常运行的随机 nonce 不产生确定性密文。
- 「内核实现 / 桌面壳 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计」的既有口径继续保留。
