# 把 core workspace 构建出的 mylock-serve 复制成 Tauri externalBin 约定命名。
#
# Tauri 2 (tauri-utils external_binaries / tauri-build copy_binaries) 按
#   "binary-name{-target-triple}{.exe}"
# 查找 sidecar：本仓库配置为 bundle.externalBin = ["binaries/mylock-serve"]，
# 故 Windows/gnu host 上需要
#   desktop/src-tauri/binaries/mylock-serve-x86_64-pc-windows-gnu.exe
# tauri-build 在 build script 阶段去掉 "-<triple>" 后把文件复制到 target 目录，
# 运行期桌面壳再从 resource_dir / current_exe 同目录定位 mylock-serve.exe。
#
# 诚实边界：这只是「本机 stage 二进制以便 cargo check / 打包」的脚本；
# 生成的 binaries/ 不入库（见 .gitignore），发布前需自备签名证书，见 desktop/SIGNING.md。

$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\zjc_9\.cargo\bin;C:\mingw64\bin;$env:PATH"

$repo    = Split-Path -Parent $PSScriptRoot          # ...\mylock
$desktop = Join-Path $repo "desktop"                # ...\mylock\desktop
$src     = Join-Path $repo "core\target\debug\mylock-serve.exe"

if (-not (Test-Path -LiteralPath $src)) {
    throw "找不到 $src ；请先在 core/ 运行: cargo build -p mylock-mount --bin mylock-serve"
}

$hostLine = (& rustc -vV | Select-String '^host:').ToString()
$triple   = $hostLine.Split(':', 2)[1].Trim()

$dstDir = Join-Path $desktop "src-tauri\binaries"
$dst    = Join-Path $dstDir ("mylock-serve-{0}.exe" -f $triple)

New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
Copy-Item -LiteralPath $src -Destination $dst -Force

Write-Host "SRC=$src"
Write-Host "TRIPLE=$triple"
Write-Host "SIDECAR=$dst"
Write-Host "BYTES=$((Get-Item -LiteralPath $dst).Length)"
