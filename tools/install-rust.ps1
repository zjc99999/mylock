$ErrorActionPreference = 'Continue'
$root = 'D:\Projects\dsh\mylock\tools'
$log  = Join-Path $root 'rust-install.log'
function Log($m) { Add-Content -Path $log -Value ("[{0}] {1}" -f (Get-Date -Format o), $m) }

Set-Content -Path $log -Value 'mylock M2: rust toolchain install (v2)'
$rustup = Join-Path $env:USERPROFILE '.cargo\bin\rustup.exe'
$cargo  = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$rustc  = Join-Path $env:USERPROFILE '.cargo\bin\rustc.exe'

# rustup shim already present (from interrupted first run). Install the toolchain
# directly via cmd /c so rustup's stderr info/warn lines never become PowerShell errors.
$out = Join-Path $root 'rustup-install.out'
$err = Join-Path $root 'rustup-install.err'
Log 'installing toolchain stable-x86_64-pc-windows-gnu via rustup'
cmd /c "`"$rustup`" toolchain install stable-x86_64-pc-windows-gnu --profile minimal > `"$out`" 2> `"$err`""
Log ("rustup install exit={0}" -f $LASTEXITCODE)
if (Test-Path $out) { Get-Content $out | ForEach-Object { Log ("OUT: {0}" -f $_) } }
if (Test-Path $err) { Get-Content $err | ForEach-Object { Log ("ERR: {0}" -f $_) } }

Log 'setting default toolchain'
cmd /c "`"$rustup`" default stable-x86_64-pc-windows-gnu > `"$out`" 2> `"$err`""
Log ("rustup default exit={0}" -f $LASTEXITCODE)
if (Test-Path $out) { Get-Content $out | ForEach-Object { Log ("OUT: {0}" -f $_) } }
if (Test-Path $err) { Get-Content $err | ForEach-Object { Log ("ERR: {0}" -f $_) } }

# ensure GCC (mingw) linker is reachable
$env:PATH = "C:\mingw64\bin;$env:PATH"

if (Test-Path $cargo) {
    Log ("cargo at {0}" -f $cargo)
    cmd /c "`"$rustc`" --version >> `"$out`" 2>&1"
    cmd /c "`"$cargo`" --version >> `"$out`" 2>&1"
    Get-Content $out | Select-Object -Last 4 | ForEach-Object { Log ("VER: {0}" -f $_) }
    Log 'INSTALL_OK'
} else {
    Log 'INSTALL_FAILED: cargo.exe missing'
    exit 3
}
