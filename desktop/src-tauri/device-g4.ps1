# MyLock 真机 G4 辅助：等待设备 → 安装 APK → 启动 App → 抓 logcat
# 用法：手机开启 USB 调试并连接后，运行本脚本；随后在 App 内点「G4 自检 / 含负向自检」。
$ErrorActionPreference = 'Continue'
$adb = 'C:\Users\zjc_9\AppData\Local\Android\Sdk\platform-tools\adb.exe'
$apk = 'D:\Projects\dsh\mylock\dist\mylock-android-arm64-v0.1.0-alpha-debug.apk'
$pkg = 'com.mylock.mobile.alpha'

Write-Output "=== 等待设备（最多 90s）==="
$ready = $false
for ($i = 0; $i -lt 30; $i++) {
    $out = & $adb devices 2>&1 | Out-String
    if ($out -match '(?m)^\S+\s+device$') { $ready = $true; break }
    Start-Sleep -Seconds 3
}
if (-not $ready) { Write-Output "未检测到设备。请在手机开启 USB 调试后重试。"; exit 2 }
& $adb devices 2>&1 | ForEach-Object { Write-Output $_ }

Write-Output "=== 安装 APK ==="
& $adb install -r $apk 2>&1 | ForEach-Object { Write-Output $_ }

Write-Output "=== 启动 App ==="
& $adb shell monkey -p $pkg -c android.intent.category.LAUNCHER 1 2>&1 | ForEach-Object { Write-Output $_ }

Write-Output "=== 清空 logcat 后启动实时抓取（另开窗口可运行 adb logcat）==="
& $adb logcat -c 2>&1 | Out-Null
Write-Output "设备就绪。请在 App 内点 G4 自检 与 含负向自检，核对返回的 JSON："
Write-Output "  contract_sha256_ok=true, derived_match=30, fingerprint_ok=true,"
Write-Output "  argon2_ok=true(p=4), negative_total=13, negative_passed=13, ok=true"
