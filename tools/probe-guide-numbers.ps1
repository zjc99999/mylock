# 精确补测：为 USER-GUIDE §4 Step 5 拿「内容放到 Z: 根」的 3 记录容器的真实 info / verify 输出。
# 全部 %TEMP%，不碰用户真实 E:\testLock。
$ErrorActionPreference = 'Continue'
$pkg = 'D:\Projects\dsh\mylock\dist\MyLock-Beta-0.1.0'
$cli = Join-Path $pkg 'mylock.exe'
$serve = Join-Path $pkg 'mylock-serve.exe'
$base = Join-Path $env:TEMP ("mylock-nums-" + [guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force -Path $base | Out-Null
$pw = 'Demo-Pw-2026'
$vault = Join-Path $base 'testLockVault'
$port = 51923
$uri = "http://127.0.0.1:$port/"
& cmd /c "net use Z: /delete /y >nul 2>&1" | Out-Null
& $cli create $vault --password $pw 2>&1 | ForEach-Object { Write-Output ("CREATE| " + $_) }
$p = Start-Process -FilePath $serve -ArgumentList @('--root',$vault,'--password',$pw,'--port',"$port") -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 2
& cmd /c "net use Z: `"$uri`" /persistent:no 2>&1" | Out-Null
New-Item -ItemType Directory -Force -Path 'Z:\子文件夹' | Out-Null
[System.IO.File]::WriteAllText('Z:\会议记录.txt', '机密：季度预算 123 万')
[System.IO.File]::WriteAllText('Z:\子文件夹\照片说明.txt', '子目录里的机密')
& cmd /c "net use Z: /delete /y >nul 2>&1" | Out-Null
try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
Start-Sleep -Seconds 1
Write-Output '=== INFO ==='
& $cli info $vault 2>&1 | ForEach-Object { Write-Output ("OUT| " + $_) }
Write-Output ("INFO_EXIT=" + $LASTEXITCODE)
Write-Output '=== VERIFY --json ==='
& $cli verify $vault --json 2>&1 | ForEach-Object { Write-Output ("OUT| " + $_) }
Write-Output ("VERIFY_EXIT=" + $LASTEXITCODE)
Remove-Item -Recurse -Force $base -ErrorAction SilentlyContinue
Write-Output 'DONE'
