$ErrorActionPreference = 'Continue'
$env:JAVA_HOME = 'C:\Program Files\Java\jdk-26.0.1'
$env:ANDROID_HOME = 'C:\Users\zjc_9\AppData\Local\Android\Sdk'
$env:ANDROID_SDK_ROOT = 'C:\Users\zjc_9\AppData\Local\Android\Sdk'
$env:NDK_HOME = 'C:\Users\zjc_9\AppData\Local\Android\Sdk\ndk\30.0.16248370'
$env:PATH = 'C:\Users\zjc_9\.cargo\bin;' + $env:PATH
Write-Host "=== build start $(Get-Date -Format o) ==="
$log = 'D:\Projects\dsh\mylock\desktop\src-tauri\build-android-debug.log'
cmd /c "cargo tauri android build --apk --target aarch64 --debug > `"$log`" 2>&1"
$code = $LASTEXITCODE
Write-Host "=== build exit=$code $(Get-Date -Format o) ==="
if (Test-Path $log) {
  Write-Host '--- tail ---'
  Get-Content $log -Tail 40
}
exit $code
