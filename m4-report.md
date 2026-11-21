# M4 第 1 点阶段报告：向量归档与跨平台门禁基础设施

> 日期：2026-11-14
> 授权：用户 m02428 / m02430「你逐项进行吧，先执行第1点」——对应 M3 汇报「七、下一步」第 1 条
> 上游：[产品分析与概要设计.md](产品分析与概要设计.md) **v1.9**（M4 第 1 点当时为 v1.5；M4 第 2 点后升 v1.6、M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step4-report.md](m4-step4-report.md)）/ [vault-format-spec.md](vault-format-spec.md) v1.1.1 §9（G4 定义） / [crypto-review-m1.md](crypto-review-m1.md) §9【9】（G4 只约束固定向量） / [m3-report.md](m3-report.md) §8（硬门槛对照） / [tests/vectors/CONTRACT.md](tests/vectors/CONTRACT.md)（本轮契约）
> 状态：**已完成（门禁基础设施 + 三桌面 CI 配置 + 五端契约定义）**；**不是 G4 五端验收通过**。

---

## 1. 本轮做什么、不做什么

### 做了

| # | 交付 | 目的 |
|---|---|---|
| 1 | 把 `poc/test-vectors.json` 归档为 `tests/vectors/test-vectors.json` | 给「唯一逐字节契约」一个受保护、可被 CI 引用的正式副本 |
| 2 | `tests/vectors/SHA256SUMS` + `.gitattributes` | 完整性锚 + 禁 Git EOL 归一化（源与归档都是纯 CRLF × 190，Linux/macOS 默认转 LF 会改字节、破坏锚） |
| 3 | `tools/check_vectors.py` | 三平台共用的可移植门禁：锚 / 两副本字节相同 / EOL / JSON 结构 / KDF 硬约束，任一失败非零退出 |
| 4 | `tools/compute_fingerprint.py` | 从归档契约独立算 G4 指纹并**断言**等于冻结常量（真门禁，非零退出即红） |
| 5 | `core/crates/mylock-core/tests/conformance.rs` | 4 个 Rust 门禁测试，其中 `this_platform_reproduces_g4_fingerprint` 在**本平台真跑** `derive_outputs` 复算 30 项并命中冻结指纹 |
| 6 | `tools/gate_selftest.py` | 门禁负向自检：4 种篡改 + 1 好路径，证明门禁真能拒绝漂移，而非只测好路径 |
| 7 | `.github/workflows/vectors.yml` | Windows / macOS / Linux 三桌面矩阵 CI，4 道门禁 + 汇总 gate |
| 8 | `tests/vectors/CONTRACT.md` | G4 五端 + 恢复 CLI 的激活条件、验收判据、诚实状态表 |
| 9 | `core/crates/mylock-core/Cargo.toml` dev-dep 追加 `sha2 = "0.11.1"` | conformance.rs 复用 [dependencies] 已有的同版本 crate，`Cargo.lock` 无需变、`--locked` 可用 |

### 明确不做（诚实边界，必须保留）

- **不宣称 G4 五端验收通过**。完整 G4 需要 Android/iOS 客户端与 macOS/Linux runner；本机只有 Windows。
- **不伪造移动端/其他平台绿灯**。macOS / Linux 是「CI 已配置、未实跑」；Android / iOS 🔴 未实现。CI `needs` 不含它们。
- **本仓库当前无 `.git`**，`.github/workflows/vectors.yml` 是配置，推送到远端并启用 Actions 后才会实跑；本机仅实测 Windows 路径。
- **不改动已全绿的 `vectors.rs` / `mount.rs`**（它们仍读 `poc/` 产出源）。两副本字节相同由两道独立门禁保证（`check_vectors.py` ② 与 `conformance.rs::archive_sha256_anchor_matches`），改 389 行测试文件收益边际、风险更高。
- G4 只约束**固定测试向量**逐字节一致；正常运行的 CSPRNG 24B nonce 产生不同密文是安全属性，不是缺陷（crypto-review-m1.md §9【9】）。
- 内核实现 / 桌面壳 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计。

---

## 2. 冻结常量（写死在多处，摩擦力是刻意的）

| 项 | 值 |
|---|---|
| 向量字节数 | `13499` B（`poc/test-vectors.json` 与 `tests/vectors/test-vectors.json` 逐字节相同） |
| 向量 SHA-256 | `07af0e8b2a6d3b1afa9e43da57131a5d2bf6d1f752d5a842c6d3d7d13bee9939` |
| EOL | CRLF × 190，**无裸 LF**；`.gitattributes` 标 `-text` |
| 派生输出 | 30 个 `derived_outputs` |
| 负向向量 | 13 条，全部 `pass:true`；`summary.negative_vectors_matched == "13/13"` |
| 硬 KDF 约束 | `argon2id(m=65536, t=3, p=4)`；`p=1`（libsodium `crypto_pwhash`）无法复现 |
| 格式版本 | `format = 1` |
| **G4 指纹** | `d45b8659b9a4e0e6f67a7df80d08388723eeb7c0a59c05704a78332bb39302fa` |
| 指纹算法 | 30 个 `derived_outputs` 按 key 排序拼 `k=v\n`，UTF-8 输入共 `8254` B，取 SHA-256 |

变更一次向量必须**同一次提交**更新 6 处（详见 [CONTRACT.md §5](tests/vectors/CONTRACT.md)）：产出源、归档副本、`SHA256SUMS`、`check_vectors.py` 两常量、`compute_fingerprint.py` 三常量、`conformance.rs` 两常量，并升 `format`、写明原因。**禁止自动修复 / 静默重生锚**。

---

## 3. 门禁架构（为什么三平台独立算同一指纹 = 跨平台一致）

```
   同一份 13499 B 字节契约（SHA-256 锚 + CRLF 防护）
                    │
      ┌─────────────┼─────────────┐
      ▼             ▼             ▼
  windows-latest  macos-latest  ubuntu-latest   （各自 CPU/OS/编译器/Rust host）
      │             │             │
  check_vectors.py（可移植 Python 门禁，三份都一样）
  cargo test --workspace --locked  →  conformance.rs::this_platform_reproduces_g4_fingerprint
      │             │             │   （各自跑 derive_outputs → 30 串 → SHA-256 → 比冻结常量）
      └─────────────┼─────────────┘
                    ▼
        g4-gate：三份独立绿灯 = 同一字节输入在三套工具链产出同一字节输出
        （无需 runner 间通信 / 共享状态）
                    │
        Android / iOS 不进 needs（未实现，不伪造绿灯）
```

四道 CI 门禁：
1. `python3 tools/check_vectors.py`——完整性/结构/KDF；
2. `cargo test --workspace --locked`——全部 Rust 测试（7 KAT + 8 vectors + 4 conformance + 6 mount）；
3. `cargo build --workspace --locked`——确保门禁不止跑测试目标；
4. `python3 tools/compute_fingerprint.py`——从归档侧独立算指纹并断言（与 Rust 侧交叉验证）。

CI 刻意各平台用 runner 默认 host（本机验证过的是 `x86_64-pc-windows-gnu`，而 `windows-latest` 默认是 MSVC），让 CI 覆盖一条**不同于本机**的工具链，反而扩大 G4 证据面；且不强制 GNU 以避免在 CI 上拼装 MinGW 路径。

---

## 4. 本机实测结果（全部为 EXIT 真值）

> pwsh 把 cargo 的 stderr 进度行渲染成 `NativeCommandError` 是显示假象，以 `EXIT=` 为准。

| 检查 | 命令 | 结果 |
|---|---|---|
| 完整性门禁 | `python3 tools/check_vectors.py` | `EXIT=0`：`sha256=07af…9939` / `bytes=13499` / `CRLF x190 (no bare LF)` / `derived=30` / `negative=13` / `copies=byte-identical` / `SHA256SUMS 2/2` |
| 指纹门禁 | `python3 tools/compute_fingerprint.py` | `EXIT=0`：`ENTRIES=30 INPUT_BYTES=8254 G4_FINGERPRINT=d45b8659…02fa` + `OK: G4 fingerprint matches frozen contract` |
| 门禁负向自检 | `python3 tools/gate_selftest.py` | `GATE_SELFTEST=PASS`（5/5，见下表） |
| Rust conformance | `cargo test -p mylock-core --test conformance` | `EXIT=0`，**4 passed / 0 failed**（2.61s） |
| 锁文件兼容 | `cargo build -p mylock-core --tests`（workdir `core/`） | `EXIT=0`，`Finished dev profile in 0.07s`；`Cargo.lock` 无需变，`--locked` 可用 |
| 全量回归 | `cargo test --workspace --locked` | `EXIT=0`，**25 passed / 0 failed**：7 KAT(8.43s) + 4 conformance(2.61s) + 8 vectors.rs(59.61s) + 6 mount.rs(9.16s) + 各 bin/lib/doc-tests 0 |
| 全量编译 | `cargo build --workspace --locked` | `EXIT=0`（见 §7 最终复验） |

**负向自检 5/5**（`tools/gate_selftest.py`，在临时仓库树上用 `MYLOCK_VECTOR_ROOT` 覆盖）：

| 用例 | 期望 | 实测 |
|---|---|---|
| 单字节翻转（`b[100]^1`） | 被拒 | `FAIL: archive SHA-256 drifted: got ffd278ce…`，exit=1 |
| CRLF → LF 归一化 | 被拒 | `got 42498090… (13309 B)`，exit=1 |
| 截断到 1000 B | 被拒 | `got b36bc700… (1000 B)`，exit=1 |
| 陈旧 `SHA256SUMS`（锚换成 64 个 0） | 被拒 | `FAIL: SHA256SUMS record for poc/test-vectors.json is missing or wrong`，exit=1 |
| 未篡改真实仓库 | 通过 | `OK: vector archive integrity`，exit=0 |

一次失败并已修：`cargo build -p mylock-core --tests` 首次因 `workdir` 用了仓库根（无 `Cargo.toml`）→ `EXIT=101 error: could not find Cargo.toml`；改 `workdir=core/` 后通过。

---

## 5. 交付物清单

```
mylock/
├── .gitattributes                         # 源/归档/锚 标 -text，禁 EOL 归一化
├── .gitignore                            # core/target、desktop target、*.log 等
├── .github/workflows/vectors.yml          # 三桌面平台 CI（4 门禁 + g4-gate 汇总）
├── tests/vectors/
│   ├── test-vectors.json                  # 归档副本（13499 B，与 poc/ 逐字节相同）
│   ├── SHA256SUMS                        # 2 条锚记录
│   └── CONTRACT.md                       # G4 五端 + 恢复 CLI 契约（6 节）
├── tools/
│   ├── check_vectors.py                  # 可移植完整性门禁
│   ├── compute_fingerprint.py            # G4 指纹真门禁（断言冻结常量）
│   └── gate_selftest.py                  # 门禁负向自检
├── core/crates/mylock-core/
│   ├── Cargo.toml                        # dev-deps 追加 sha2 = "0.11.1"
│   └── tests/conformance.rs              # 4 个 G4 跨平台门禁测试
└── m4-report.md                          # 本报告
```

主文档 [产品分析与概要设计.md](产品分析与概要设计.md) 已升 **v1.5**（M4 第 1 点当时；M4 第 2 点后升 **v1.6**，见 [m4-step2-report.md](m4-step2-report.md)；M4 第 3 点后升 **v1.7**，见 [m4-step3-report.md](m4-step3-report.md)；M4 第 4 点后升 **v1.8**、M5 后升 **v1.9**，见 [m4-step4-report.md](m4-step4-report.md)）（表头 / 配套文件 / §8.2 里程碑新增 M4 第 1 点行 / §9.3 维护约定）；`crypto-review-m1.md` / `vault-format-spec.md` / `m2-report.md` / `m3-architecture.md` / `m3-report.md` / `poc-report-m1.md` / `tests/vectors/CONTRACT.md` 的上游版本引用已全部对齐（历史「升 v1.4」注记保留为历史记录）。

---

## 6. 硬门槛对照（沿用 m3-report.md §8 口径）

| # | 硬门槛 | 状态 |
|---|---|---|
| ① | 五端（Win/mac/Linux/Android/iOS）+ 恢复 CLI 对固定向量逐字节一致 | **部分**：Windows 内核/恢复 CLI/桌面层 ✅；macOS/Linux 🟡 CI 已配置未实跑；Android/iOS 🔴 未实现。**本轮是门禁基础设施，不是 ① 验收通过** |
| ② | 30 个 derived_outputs 逐字节一致 + 13 负向向量 | ✅（M2 起，本轮再加 conformance 双重锚定） |
| ③ | 主文档同步 | ✅ v1.5 + 七份配套文档版本对齐（M4 第 2 点后已再升 v1.6，M4 第 3 点后升 v1.7，M4 第 4 点后升 v1.8、M5 后升 v1.9，见 m4-step2-report.md / m4-step3-report.md / m4-step4-report.md） |
| ④ | 三桌面 CI 门禁配置 | ✅ 已落盘（须推送启用 Actions 后实跑） |
| ⑤ | 第三方安全审计 | ⏳ 未做（上架前） |

---

## 7. 诚实边界（唯一冻结措辞）

- 本轮交付 = 「向量归档 + 完整性锚 + EOL/漂移门禁 + 三桌面平台 CI **配置** + 五端契约定义」；**不是 G4 五端验收通过**。
- 本机只实测了 Windows 路径；macOS / Linux / Android / iOS 的绿灯均未被本机观察到，Android / iOS 未实现。
- 本仓库当前无 `.git`，YAML 工作流在推送到远端并启用 Actions 后才会实跑；本机未做 YAML 1.2 解析器校验（缺 PyYAML），已逐行人工核对 `on:` / `GIT_CONFIG_VALUE_0: "false"`（字符串引号）等易错点。
- G4 只约束固定测试向量；运行时随机 nonce 不产生确定性密文。
- 「内核实现 / 桌面壳 ≠ 产品发布 ≠ 五端验收 ≠ 第三方审计」的既有口径继续保留。

---

## 8. 下一步（需新授权）

1. 推送到远端启用 Actions，取得 macOS / Linux 真实绿灯（补齐 G4 桌面部分）。
2. M4 移动 α：Android / iOS 端实现 + 在真实 runner/设备上复现同一份向量（激活条件见 [CONTRACT.md §3](tests/vectors/CONTRACT.md)）。
3. 增量写回优化 / Tauri sidecar 化 + 安装包签名 / WinFsp 增强路径（另评 GPL-3.0 法务）。
4. 上架前：第三方审计、隐私政策与安全白皮书、上架法务核验。

---

## 9. 文档接口

本报告与 [tests/vectors/CONTRACT.md](tests/vectors/CONTRACT.md)、[.github/workflows/vectors.yml](.github/workflows/vectors.yml)、[产品分析与概要设计.md](产品分析与概要设计.md) v1.9（M4 第 1 点当时为 v1.5；M4 第 3 点后升 v1.7、M4 第 4 点后升 v1.8、M5 后升 v1.9，见 [m4-step3-report.md](m4-step3-report.md) / [m4-step4-report.md](m4-step4-report.md)）、[m4-step2-report.md](m4-step2-report.md)、[m3-report.md](m3-report.md)、[m2-report.md](m2-report.md)、[poc-report-m1.md](poc-report-m1.md) 相互引用。**M4 第 1 点的「非 G4 五端验收通过」口径以 CONTRACT.md §3/§6 与本报告 §1/§7 为冻结措辞**；参数总表与十条对外诚实声明仍以 `crypto-review-m1.md` §8 为唯一冻结措辞，本报告不新增、不矛盾。
