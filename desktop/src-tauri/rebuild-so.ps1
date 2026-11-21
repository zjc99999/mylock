$ErrorActionPreference = 'Continue'
$env:JAVA_HOME = 'C:\Program Files\Java\jdk-26.0.1'
$env:ANDROID_HOME = 'C:\Users\zjc_9\AppData\Local\Android\Sdk'
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
$env:NDK_HOME = 'C:\Users\zjc_9\AppData\Local\Android\Sdk\ndk\30.0.16248370'
$env:PATH = "C:\Windows\System32;C:\Windows;C:\Users\zjc_9\.cargo\bin;$env:PATH"
Set-Location 'D:\Projects\dsh\mylock\desktop\src-tauri'
Write-Output "=== rebuilding aarch64 .so via tauri (symlink failure expected) ==="
& 'C:\Windows\System32\cmd.exe' /c "cargo tauri android build --apk --target aarch64 --debug > rebuild-so.log 2>&1"
Write-Output "tauri exit=$LASTEXITCODE"
$log = Get-Content 'rebuild-so.log' -Raw
if ($log -match 'Finished.*profile') { Write-Output "COMPILE_OK" } else { Write-Output "COMPILE_UNKNOWN" }
$so = 'D:\Projects\dsh\mylock\desktop\src-tauri\target\aarch64-linux-android\debug\libmylock_mobile.so'
if (Test-Path $so) { $f = Get-Item $so; Write-Output "SO size=$($f.Length) mtime=$($f.LastWriteTime)" }
