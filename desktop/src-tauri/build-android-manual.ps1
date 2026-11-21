# MyLock Android APK 手动构建（绕过 Windows symlink 权限限制）
# 前置：rebuild-so.ps1 已编译并把 libmylock_mobile.so 复制到 jniLibs/arm64-v8a
$ErrorActionPreference = 'Continue'
$env:JAVA_HOME = 'C:\Program Files\Java\jdk-26.0.1'
$env:ANDROID_HOME = 'C:\Users\zjc_9\AppData\Local\Android\Sdk'
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
$env:NDK_HOME = 'C:\Users\zjc_9\AppData\Local\Android\Sdk\ndk\30.0.16248370'
$env:MYLOCK_SKIP_RUST_BUILD = '1'
$env:PATH = "C:\Windows\System32;C:\Windows;$env:JAVA_HOME\bin;$env:PATH"
$android = 'D:\Projects\dsh\mylock\desktop\src-tauri\gen\android'
Set-Location $android
Write-Output "=== gradle assembleArm64Debug (MYLOCK_SKIP_RUST_BUILD=1) ==="
& 'C:\Windows\System32\cmd.exe' /c "gradlew.bat assembleArm64Debug --no-daemon > assemble-arm64-debug.log 2>&1"
Write-Output "gradle exit=$LASTEXITCODE"
if (Test-Path 'assemble-arm64-debug.log') {
  Get-Content 'assemble-arm64-debug.log' -Tail 40
}
$apk = Get-ChildItem -Path $android -Recurse -Filter '*.apk' -ErrorAction SilentlyContinue
foreach ($a in $apk) { Write-Output "APK: $($a.FullName)  $($a.Length) bytes" }
