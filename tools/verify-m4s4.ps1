# M4 第 4 点最终复验（可重放，只读验证 + 编译/测试，不写产品代码）
# 用法: powershell -NoProfile -ExecutionPolicy Bypass -File tools/verify-m4s4.ps1
$ErrorActionPreference = "Continue"
$env:PATH = "C:\Users\zjc_9\.cargo\bin;C:\mingw64\bin;$env:PATH"
$repo = Split-Path -Parent $PSScriptRoot
$py = "C:\Python312\python.exe"
$core = Join-Path $repo "core"
$desktop = Join-Path $repo "desktop\src-tauri"

function Run-Step {
    param([string]$Name, [string]$WorkDir, [string]$Cmd)
    Write-Output "===== STEP: $Name ====="
    Push-Location $WorkDir
    try {
        & cmd /c "$Cmd" 2>&1
        Write-Output "STEP_EXIT($Name)=$LASTEXITCODE"
    } finally {
        Pop-Location
    }
}

Run-Step "cargo-test-workspace-locked" $core "cargo test --workspace --locked"
Run-Step "cargo-build-workspace-locked" $core "cargo build --workspace --locked"
Run-Step "check-vectors" $repo "`"$py`" tools\check_vectors.py"
Run-Step "compute-fingerprint" $repo "`"$py`" tools\compute_fingerprint.py"
Run-Step "gate-selftest" $repo "`"$py`" tools\gate_selftest.py"
Run-Step "license-audit" $repo "`"$py`" tools\audit_licenses.py"
Run-Step "desktop-cargo-check" $desktop "cargo check"

Write-Output "===== VERIFY_M4S4_DONE ====="
