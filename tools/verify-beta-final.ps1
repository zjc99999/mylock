# 最终交付物强验证：解压最终 ZIP 到全新干净目录 -> 从该目录启动 GUI -> 窗口标题判据
# 只读验证脚本：不写产品代码，不改任何源/文档；仅创建并清理临时目录。
$ErrorActionPreference = 'Continue'
$root = 'D:\Projects\dsh\mylock'
$zip  = Join-Path $root 'dist\MyLock-Beta-0.1.0-portable-unsigned.zip'
$dest = Join-Path $root 'dist\_final_verify_extract'

Write-Output "ZIP=$zip"
if (-not (Test-Path $zip)) { Write-Output 'FATAL: zip missing'; exit 1 }
$zipInfo = Get-Item $zip
Write-Output "ZIP_SIZE=$($zipInfo.Length)"

# 1) 列 ZIP 条目
Add-Type -AssemblyName System.IO.Compression.FileSystem
$z = [System.IO.Compression.ZipFile]::OpenRead($zip)
$entries = @($z.Entries | ForEach-Object { $_.FullName })
$z.Dispose()
Write-Output "ZIP_ENTRIES=$($entries.Count)"
foreach ($e in $entries) { Write-Output "ENTRY=$e" }
$hasLoader = $entries -contains 'WebView2Loader.dll'
$hasGui    = $entries -contains 'mylock-desktop.exe'
$hasGuide  = $entries -contains 'USER-GUIDE.md'
Write-Output "HAS_LOADER=$hasLoader HAS_GUI=$hasGui HAS_GUIDE=$hasGuide"
if (-not ($hasLoader -and $hasGui -and $hasGuide)) { Write-Output 'FATAL: loader/gui/USER-GUIDE.md missing in zip'; exit 2 }

# 2) 干净解压（先删旧的）
if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
[System.IO.Compression.ZipFile]::ExtractToDirectory($zip, $dest)
Write-Output "EXTRACTED_TO=$dest"

# 3) 校验随包 DLL 哈希
$loaderPath = Join-Path $dest 'WebView2Loader.dll'
$loaderHash = (Get-FileHash -Algorithm SHA256 $loaderPath).Hash
Write-Output "EXTRACTED_LOADER_SHA256=$loaderHash"
if ($loaderHash -ne '86545B66CDB0603BC26B626FB9AD610CB6E71F28D468F5EA66DF23B03DDA96D5') {
  Write-Output 'FATAL: loader hash mismatch'; exit 3
}

# 4) 从解压目录启动 GUI（强判据：主窗口标题）
$exe = Join-Path $dest 'mylock-desktop.exe'
$p = Start-Process -FilePath $exe -WorkingDirectory $dest -PassThru
Start-Sleep -Seconds 5
$p.Refresh()
Write-Output "PID=$($p.Id) ALIVE_AFTER_5S=$(-not $p.HasExited)"

Add-Type @"
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class WinEnumFinal {
  public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder s, int n);
  public static List<string> ForPid(uint target) {
    var result = new List<string>();
    EnumWindows((h, l) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid == target && IsWindowVisible(h)) {
        var sb = new StringBuilder(512); GetWindowText(h, sb, sb.Capacity);
        if (sb.Length > 0) result.Add("HWND=" + h.ToInt64() + " TITLE=" + sb.ToString());
      }
      return true;
    }, IntPtr.Zero);
    return result;
  }
}
"@
$wins = [WinEnumFinal]::ForPid([uint32]$p.Id)
Write-Output "VISIBLE_WINDOWS=$($wins.Count)"
foreach ($w in $wins) { Write-Output "WINDOW=$w" }

$titleHit = $false
foreach ($w in $wins) { if ($w -match 'MyLock') { $titleHit = $true } }
Write-Output "MAIN_WINDOW_TITLE_OK=$titleHit"

if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force; Start-Sleep -Seconds 1 }
Write-Output "STOPPED_OK=$(-not (Get-Process -Id $p.Id -ErrorAction SilentlyContinue))"

# 5) 清理临时解压目录
Start-Sleep -Seconds 1
try { Remove-Item -Recurse -Force $dest; Write-Output "CLEANED=$(-not (Test-Path $dest))" }
catch { Write-Output "CLEANED=False (busy, harmless)" }

if ($titleHit -and $hasLoader -and $hasGui -and $hasGuide) { Write-Output 'VERIFY_BETA_FINAL=PASS' }
else { Write-Output 'VERIFY_BETA_FINAL=FAIL'; exit 4 }
