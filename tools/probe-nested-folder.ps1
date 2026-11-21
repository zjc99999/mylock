# 补测：把「整个文件夹」拖进 Z: 形成 Z:\testLock\... 子树的真实行为 + export 摊平情况。
# 全部在 %TEMP%，不碰用户真实 E:\testLock。
$ErrorActionPreference = 'Continue'
$pkg = 'D:\Projects\dsh\mylock\dist\MyLock-Beta-0.1.0'
$cli = Join-Path $pkg 'mylock.exe'
$serve = Join-Path $pkg 'mylock-serve.exe'
$base = Join-Path $env:TEMP ("mylock-nested-" + [guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force -Path $base | Out-Null
$pw = 'Demo-Pw-2026'
$vault = Join-Path $base 'testLockVault'
$out = Join-Path $base 'decrypted'
$port = 51922
$uri = "http://127.0.0.1:$port/"

& cmd /c "net use Z: /delete /y >nul 2>&1" | Out-Null
& $cli create $vault --password $pw 2>&1 | ForEach-Object { Write-Output ("CREATE| " + $_) }
$p = Start-Process -FilePath $serve -ArgumentList @('--root',$vault,'--password',$pw,'--port',"$port") -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 2
$r = & cmd /c "net use Z: `"$uri`" /persistent:no 2>&1"
Write-Output ("NETUSE_EXIT=" + $LASTEXITCODE)

# 模拟「把 E:\testLock 整个文件夹拖进 Z:」= Z:\testLock\ 子树
New-Item -ItemType Directory -Force -Path 'Z:\testLock\子文件夹' | Out-Null
[System.IO.File]::WriteAllText('Z:\testLock\会议记录.txt', '机密：季度预算 123 万')
[System.IO.File]::WriteAllText('Z:\testLock\report.docx', 'FAKE-DOCX')
[System.IO.File]::WriteAllText('Z:\testLock\子文件夹\照片说明.txt', '子目录里的机密')
Write-Output ('READ1=' + [System.IO.File]::ReadAllText('Z:\testLock\会议记录.txt'))
Write-Output ('READ2=' + [System.IO.File]::ReadAllText('Z:\testLock\子文件夹\照片说明.txt'))
Write-Output '=== Z: TREE ==='
Get-ChildItem -Recurse 'Z:\' -ErrorAction SilentlyContinue | ForEach-Object { Write-Output ('Z_ITEM=' + $_.FullName) }

& cmd /c "net use Z: /delete /y >nul 2>&1" | Out-Null
try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
Start-Sleep -Seconds 1

Write-Output '=== EXPORT (flatten check) ==='
& $cli export $vault --out $out --password $pw 2>&1 | ForEach-Object { Write-Output ("OUT| " + $_) }
Write-Output ("EXPORT_EXIT=" + $LASTEXITCODE)
Get-ChildItem -Recurse $out -ErrorAction SilentlyContinue | ForEach-Object { Write-Output ('EXPORT_ITEM=' + $_.FullName.Substring($out.Length+1) + ' (' + $_.Length + 'B)') }

Write-Output '=== VERIFY ==='
& $cli verify $vault --json 2>&1 | ForEach-Object { Write-Output ("OUT| " + $_) }

$hits = 0
Get-ChildItem -LiteralPath $vault -Recurse -File | ForEach-Object {
  $t = [System.Text.Encoding]::UTF8.GetString([System.IO.File]::ReadAllBytes($_.FullName))
  if ($t.Contains('季度预算') -or $t.Contains('子目录里的机密') -or $t.Contains('FAKE-DOCX')) { $hits++ }
}
Write-Output ("PLAINTEXT_HITS=" + $hits)
Remove-Item -Recurse -Force $base -ErrorAction SilentlyContinue
Write-Output 'DONE'
