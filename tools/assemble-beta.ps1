# Assemble the zero-cost, UNSIGNED portable beta package.
# Read-only with respect to source; only writes under dist/.
# No NSIS/MSI, no code signing, no certificate. SmartScreen warning is expected.
#
# FIX (M5 beta hotfix, 2026-11-15): webview2-com-sys 0.39.1 on a NON-msvc target
# (this project builds x86_64-pc-windows-gnu) links WebView2Loader.dll DYNAMICALLY
#   src/lib.rs:19-22  #[cfg_attr(not(target_env = "msvc"), link(name = "WebView2Loader.dll"))]
# MSVC builds static-link WebView2LoaderStatic.lib and need no DLL. This GNU build
# does NOT: the loader DLL MUST sit next to mylock-desktop.exe or Windows refuses to
# start it with "由于找不到 WebView2Loader.dll，无法继续执行代码".
# The first beta package omitted it. This script now copies it AND verifies that
# every non-system DLL named in the exe import table is present in the stage.
#
# ADD (2026-11-15, user m04366): USER-GUIDE.md is bundled so the detailed
# feature/operation guide sits next to README-BETA.md inside the package.
$ErrorActionPreference = 'Stop'
$root = 'D:\Projects\dsh\mylock'
$stage = Join-Path $root 'dist\MyLock-Beta-0.1.0'
$zip   = Join-Path $root 'dist\MyLock-Beta-0.1.0-portable-unsigned.zip'
$objdump = 'C:\mingw64\bin\objdump.exe'

# Do NOT delete the stage directory itself: a user may have dist/ open in Explorer,
# which locks the directory handle even though its files are deletable. Clearing only
# the CONTENTS avoids that lock while still guaranteeing a clean, reproducible stage.
if (Test-Path $stage) {
    Get-ChildItem -LiteralPath $stage -Force | ForEach-Object {
        Remove-Item -LiteralPath $_.FullName -Recurse -Force
    }
} else {
    New-Item -ItemType Directory -Force -Path $stage | Out-Null
}

$desktop = Join-Path $root 'desktop\src-tauri\target\release\mylock-desktop.exe'
$cli     = Join-Path $root 'core\target\release\mylock.exe'
$serve   = Join-Path $root 'core\target\release\mylock-serve.exe'
# Dynamic runtime dependency of mylock-desktop.exe (GNU/non-msvc WebView2 link).
$loader  = Join-Path $root 'desktop\src-tauri\target\release\WebView2Loader.dll'

foreach ($p in @($desktop,$cli,$serve,$loader)) {
    if (-not (Test-Path $p)) { Write-Output "MISSING_SOURCE=$p"; exit 2 }
    Copy-Item -Force $p -Destination $stage
}

# Third-party notices (MPL 3.1/3.2 source-availability + MIT/Apache attribution)
# + BETA notice + the detailed USER-GUIDE requested by the beta reader.
foreach ($f in @('BETA-NOTICE.md','THIRD_PARTY_NOTICES.md','USER-GUIDE.md')) {
    $src = Join-Path $root $f
    if (-not (Test-Path $src)) { Write-Output "MISSING_DOC=$f"; exit 2 }
    Copy-Item -Force $src -Destination $stage
}

# Byte-for-byte check: the staged loader is exactly the one we copied (guards
# against a stale/x86/arm64 DLL being picked up from a different path).
$srcHash = (Get-FileHash $loader -Algorithm SHA256).Hash
$dstHash = (Get-FileHash (Join-Path $stage 'WebView2Loader.dll') -Algorithm SHA256).Hash
if ($srcHash -ne $dstHash) { Write-Output "LOADER_HASH_MISMATCH src=$srcHash dst=$dstHash"; exit 3 }
Write-Output "LOADER_SHA256=$srcHash"

$readme = @'
# MyLock · 封闭内测便携包（0.1.0 · 免签名 · 未审计）

**先读 [BETA-NOTICE.md](BETA-NOTICE.md) 再使用；想看「能干什么、怎么一步步操作」请读 [USER-GUIDE.md](USER-GUIDE.md)。**

## 这个包是什么
- 邀请制封闭内测版，**未经第三方安全审计、未代码签名**。
- **请勿存放不可替代的唯一副本**；不保证数据不丢失。
- 不是产品发布、不是商店上架版、不是合规结论、不是第三方审计通过。
- 首次运行若出现 Windows SmartScreen「未知发布者」警告属预期（未买证书），由内测组织者自行确认后运行。
- **必须整目录拷贝、不要只单独复制 exe**：本目录下的 `WebView2Loader.dll` 必须与 `mylock-desktop.exe` 放在同一目录，否则 Windows 会报「由于找不到 WebView2Loader.dll，无法继续执行代码」并拒绝启动。

## 两个不同的 WebView2 依赖（容易混淆）
| 依赖 | 是否随包 | 说明 |
|---|---|---|
| `WebView2Loader.dll`（加载器） | **随包提供**（本目录内，x64，160880 B） | 本程序用 GNU/MinGW 工具链构建，`webview2-com-sys` 对非 MSVC 目标是**动态链接**该 DLL；MSVC 构建才静态链入、不需要它。缺了它 exe 根本起不来。 |
| WebView2 **Evergreen Runtime**（浏览器运行时） | **不随包**，需系统已安装 | 基于 Edge/Chromium 的运行时，一般 Windows 已随 Edge 装好；未装过的机器须自行安装（微软免费提供，离线安装不额外收费）。 |

> 换句话说：`WebView2Loader.dll` 是「随包的启动桥梁」，Evergreen Runtime 是「系统里的浏览器内核」。前者本包已带，后者靠系统。若报「找不到 WebView2Loader.dll」，几乎一定是**只拷了 exe、没把整个文件夹一起拷走**。

## 三个 exe 与一个 DLL
| 文件 | 用途 |
|---|---|
| `mylock-desktop.exe` | 图形界面：① 第一次用「新建保险柜」（把原 CLI Step 1 收进界面，含使用前必读提示，建好返回 Recovery Key）；② 以后「解锁并映射」自动 `net use` 到指定盘符（默认 Z:）→ 在盘符里像普通文件夹一样编辑 →「锁定并断开」。解锁前会预检 `vault.meta`，没建库会给人话提示而不是底层 `E_FORMAT` 串。 |
| `WebView2Loader.dll` | 上面 GUI 的动态加载器，**必须与 exe 同目录**，勿删。 |
| `mylock.exe` | 独立恢复 CLI：`info` / `verify` / `create` / `export` / `export-object` / `change-password` / `rotate-recovery` / `rotate-keys`。**即使桌面壳不可用也能打开密文容器**（纯 Rust，不依赖 WebView2）。 |
| `mylock-serve.exe` | 独立 loopback WebDAV 服务（备用/排障）。桌面壳 M5 起已改为**进程内引擎**，正常内测不需要单独运行它；`mylock mount` 明确拒绝并指向本工具。 |
| `USER-GUIDE.md` | **详细功能与操作说明**：完整示例「把 `E:\testLock` 装进保险柜」、GUI/CLI 全命令、排错表、内测清单。看不懂本 README 就读它。 |

## 快速内测流程（组织者）
1. 把**本目录整个**拷给受邀用户（不要放公开下载链接，也不要只发单个 exe）。
2. **第一次使用**：在 GUI 填「Vault 根目录」+「主口令」→ 点「新建保险柜」（会弹使用前必读确认框）→ **立刻离线抄存返回的 Recovery Key**。目标必须是**全新空路径**（建库会清空已存在目录）；旧备用方式 `mylock.exe create <全新空目录> --password <口令>` 仍可用。**别把已有保险柜的路径拿去「新建」，也别把没建库的路径拿去「解锁」。**
3. **日常使用**：点「解锁并映射」→ 出现 `Z:` 盘 → 把文件拖进 `Z:\`（这就是「加密」；从 `Z:` 拷出就是「解密」）→ 用完点「锁定并断开」。完整逐步示例见 [USER-GUIDE.md](USER-GUIDE.md) §4。
4. 体验「锁定 → 把整个 vault 目录拷到 U 盘 → 仍是一堆密文文件、不会自动变明文」。
5. **密文容器文件夹保护（提示性，不是防删机制）**：新建/每次解锁会自动把 `vault.meta`/`manifest`/`d/` 等内部零件设为「隐藏+系统」、写 `desktop.ini`（UTF-16LE）让删除/移动时 Windows 弹「You Are Deleting a System Folder」警告、把文件夹图标换成保险柜、并留一份可见的「⚠ 请勿删除.txt」。A 级依据：[Microsoft Learn · Customize Folders with Desktop.ini](https://learn.microsoft.com/en-us/windows/win32/shell/how-to-customize-folders-with-desktop-ini)（**刻意不写 `ConfirmFileOp=0`**，以保留删除警告）。**这只是 shell 层提示，不保证数据不丢失**；删了内部零件仍会导致无法解密，请勿存放不可替代的唯一副本。
6. **明确告知**：解锁态下任何本机进程都能访问该 loopback 挂载点（无 per-request 认证）；不要放在 OneDrive/Dropbox/坚果云等云同步根目录内；解锁态只缓解风险、不防已攻陷设备。

## 反馈
只走主动渠道（访谈 / 问卷 / 用户主动发消息）。本包**无埋点、无遥测、无崩溃上报、无账号、无云同步**。

## 许可
见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。无 GPL 家族强 copyleft（已核验 0），但存在 MPL-2.0 文件级弱 copyleft。`WebView2Loader.dll` 为微软 WebView2 SDK 随 `webview2-com-sys`（MIT）分发的二进制组件，随包再分发用于加载系统 WebView2 Runtime。

## 明确没有
NSIS/MSI 安装包、代码签名、第三方审计、律师签发、商店上架、Android/iOS 端、G4 五端一致性绿灯。真要上架或收费那天，A（审计）/ B（律师）/ C（签名包）/ D（商店申报）四道闸门一个不能跳。
'@
Set-Content -LiteralPath (Join-Path $stage 'README-BETA.md') -Value $readme -Encoding UTF8

# ---- Deterministic import-table gate -----------------------------------------
# For every non-system DLL in the exe's PE import table, a file of that name MUST
# exist in the stage. This is the regression gate that would have caught the
# missing WebView2Loader.dll without needing a GUI launch.
$systemDlls = @(
  'advapi32.dll','bcryptprimitives.dll','combase.dll','comctl32.dll','dwmapi.dll',
  'gdi32.dll','imm32.dll','kernel32.dll','msvcrt.dll','ntdll.dll','ole32.dll',
  'oleaut32.dll','propsys.dll','rpcrt4.dll','shell32.dll','shlwapi.dll','user32.dll',
  'userenv.dll','ws2_32.dll'
)
if (Test-Path $objdump) {
    $imports = & $objdump -p (Join-Path $stage 'mylock-desktop.exe') 2>&1 |
        Select-String -Pattern 'DLL Name:' |
        ForEach-Object { ($_.Line -replace '.*DLL Name:\s*','').Trim() } |
        Sort-Object -Unique
    $missing = @()
    foreach ($dll in $imports) {
        if ($dll -match '^api-ms-win-' -or $dll -match '^ext-ms-win-') { continue }
        if ($systemDlls -contains $dll.ToLowerInvariant()) { continue }
        $staged = Join-Path $stage $dll
        Write-Output "IMPORT_DLL=$dll STAGED=$(Test-Path $staged)"
        if (-not (Test-Path $staged)) { $missing += $dll }
    }
    if ($missing.Count -gt 0) {
        Write-Output ("MISSING_RUNTIME_DLL=" + ($missing -join ','))
        exit 4
    }
    Write-Output 'IMPORT_GATE=PASS'
} else {
    if (-not (Test-Path (Join-Path $stage 'WebView2Loader.dll'))) {
        Write-Output 'MISSING_RUNTIME_DLL=WebView2Loader.dll'; exit 4
    }
    Write-Output 'IMPORT_GATE=SKIPPED_OBJDUMP (loader presence checked)'
}

if (Test-Path $zip) { Remove-Item -Force $zip }
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($stage, $zip)

Write-Output "STAGE=$stage"
Get-ChildItem -File $stage | ForEach-Object { Write-Output ("FILE={0} SIZE={1}" -f $_.Name, $_.Length) }
$zi = Get-Item $zip
Write-Output ("ZIP={0} SIZE={1}" -f $zi.FullName, $zi.Length)
Write-Output 'ASSEMBLE_BETA_OK'
