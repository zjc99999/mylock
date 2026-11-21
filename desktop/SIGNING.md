# 安装包 / 代码签名说明（M4 第 2 点 · 诚实边界）

> 状态：**本仓库不提供代码签名证书，也不产出「已签名」安装包。**
> 本文档只给出「发布前要做什么」和「当前配置里哪些字段与签名相关」，不宣称已完成签名。

## 一、M4 第 2 点已经做了什么

| 项 | 状态 | 说明 |
|---|---|---|
| `bundle.active` | ✅ 由 `false` 改为 `true` | `desktop/src-tauri/tauri.conf.json` |
| `bundle.targets` | ✅ `["nsis","msi"]` | Windows 安装包类型，明确而非 `"all"` |
| `bundle.icon` | ✅ 4 个图标（32/128/128@2x PNG + ICO） | 由 `tools/make_icons.py` 从 M3 的多尺寸 ICO 派生，属**占位图标非产品美术** |
| `bundle.externalBin` | ✅ `["binaries/mylock-serve"]` | `mylock-serve` 作为 Tauri sidecar 打进安装包 |
| sidecar stage | ✅ `tools/prepare-sidecar.ps1` | 把 `core/target/debug/mylock-serve.exe` 复制成 `binaries/mylock-serve-<triple>.exe`；tauri-build 去掉 triple 落到 target 目录 |
| 运行期解析 | ✅ `resolve_serve_bin()` | `app.path().resource_dir()` → 同目录 → PATH 四级；前端可用 `serveBin` 覆盖 |
| 桌面壳编译 | ✅ `cargo check` EXIT=0 | build script 成功把 `mylock-serve.exe` 复制到 `target/debug/` |

**未做**：没有生成 `.msi` / `.exe`（NSIS）安装包成品，没有跑 `tauri build`，没有对任何二进制签名。
本仓库当前**无 `.git`、无 CI 打包 job、无证书**，只有本机 `cargo check` 级别的验证。

## 二、发布前必须补齐（需持有证书的人操作）

Tauri 2 的 Windows 签名配置位于 `tauri.conf.json` 的 `bundle.windows`（字段来自
`tauri-utils-2.10.2/src/config.rs::WindowsConfig`，camelCase、`deny_unknown_fields`）：

```jsonc
"bundle": {
  "active": true,
  "targets": ["nsis", "msi"],
  "externalBin": ["binaries/mylock-serve"],
  "windows": {
    // 方式 A：证书指纹（signtool 从系统证书库取）
    "certificateThumbprint": "<SHA1>",
    "digestAlgorithm": "sha256",
    "timestampUrl": "http://timestamp.digicert.com",
    // 方式 B：自定义签名命令（须含 %1 占位符，会被替换为待签名文件路径）
    // "signCommand": "signtool sign /f C:\\path\\cert.pfx /p <pwd> /tr http://timestamp.digicert.com /td sha256 /fd sha256 %1"
  }
}
```

要点：

1. **证书自备**：OV/EV 代码签名证书（或至少自签测试证书）。私钥/密码**绝不入库**，
   通过环境变量或 CI secret 注入；本仓库不提供任何证书或密码。
2. **时间戳**：`timestampUrl`（或 `tsp`）建议开启，否则证书过期后已签名安装包会失效。
3. **digest**：`sha256`；避免 SHA-1。
4. **签名覆盖**：主程序 exe + sidecar `mylock-serve.exe` + 生成的 NSIS/MSI 安装包都要签；
   不要只签安装器外壳。
5. **构建命令**（在有证书的机器上）：
   ```
   # 1) 构建 sidecar
   cd core && cargo build --release -p mylock-mount --bin mylock-serve
   # 2) stage（脚本默认取 debug，release 需自行调整 $src 或加参数）
   powershell -File tools/prepare-sidecar.ps1
   # 3) 打包并签名
   cd desktop/src-tauri && cargo tauri build
   ```
6. **校验**：`signtool verify /pa <installer>`、检查 SmartScreen/Defender 告警、检查
   `mylock-serve.exe` 的 Authenticode 签名确实存在。

## 三、诚实边界（唯一冻结措辞，勿删改）

- 安装包**尚未代码签名**；发布前须自备证书并完成签名校验。
- 本阶段是 **Tauri sidecar 化 + 打包配置 + 证书说明**，不是「已产出可分发安装包」。
- 桌面壳是**可编译脚手架**：未做安装包成品签名/公证、未做自动更新、未做崩溃上报
  （后者与「无遥测」定位冲突，若要做须单独评审）。
- 内核/桌面壳 ≠ 产品发布 ≠ 五端一致性验收（G4 仍只完成 Windows 单端）≠ 第三方安全审计。
- 仍是 loopback-only WebDAV，**非** WinFsp/FUSE 驱动级挂载；WebDAV loopback
  **无 per-request 认证**；解锁态只缓解不消除采集风险；拒绝在云同步根目录内挂载。
- `bundle.icon` 是自生成占位图标，非产品美术资源。

## 四、文档接口

- 上游：[m3-architecture.md](../m3-architecture.md)、[m3-report.md](../m3-report.md)（安全边界冻结措辞）
- 本阶段报告：[m4-step2-report.md](../m4-step2-report.md)
- 主文档：[产品分析与概要设计.md](../产品分析与概要设计.md)（v1.9）
- sidecar 脚本：[tools/prepare-sidecar.ps1](../tools/prepare-sidecar.ps1)、图标：[tools/make_icons.py](../tools/make_icons.py)
