# 验证用户核心用例：把「E:\testLock 这样的文件夹（含子目录）」装进保险柜。
# 工作流 = create 空库 -> 起 loopback WebDAV -> net use Z: -> 在 Z: 里重建目录结构并写文件
#          -> 锁定 -> export 验证写回落盘/能否还原层级 -> 扫描密文容器无明文。
# 全部在 %TEMP% 模拟，绝不碰用户真实 E:\testLock。
$ErrorActionPreference = 'Continue'
$pkg = 'D:\Projects\dsh\mylock\dist\MyLock-Beta-0.1.0'
$cli = Join-Path $pkg 'mylock.exe'
$serve = Join-Path $pkg 'mylock-serve.exe'
$base = Join-Path $env:TEMP ("mylock-folder-" + [guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force -Path $base | Out-Null
$pw = 'Demo-Pw-2026'
$vault = Join-Path $base 'testLockVault'   # 新的、不存在的容器路径
$out = Join-Path $base 'decrypted'
$port = 51921
$uri = "http://127.0.0.1:$port/"

# 模拟用户已有的明文文件夹（含子目录）
$src = Join-Path $base 'testLock'
New-Item -ItemType Directory -Force -Path (Join-Path $src '子文件夹') | Out-Null
[System.IO.File]::WriteAllText((Join-Path $src '会议记录.txt'), '机密：季度预算 123 万')
[System.IO.File]::WriteAllText((Join-Path $src '子文件夹\照片说明.txt'), '子目录里的机密')

Write-Output "BASE=$base"
Write-Output ("CLI=" + (Test-Path $cli) + " SERVE=" + (Test-Path $serve))

# 0) 清理可能残留的 Z: 映射
& cmd /c "net use Z: /delete /y >nul 2>&1" | Out-Null

# 1) 建空库（目标目录必须不存在！）
Write-Output "=== STEP1 create EMPTY vault ==="
& $cli create $vault --password $pw 2>&1 | ForEach-Object { Write-Output ("OUT| " + $_) }
Write-Output ("CREATE_EXIT=" + $LASTEXITCODE)

# 2) 起 loopback WebDAV
Write-Output "=== STEP2 serve ==="
$p = Start-Process -FilePath $serve -ArgumentList @('--root',$vault,'--password',$pw,'--port',"$port") -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 2
Write-Output ("SERVE_ALIVE=" + (-not $p.HasExited))

# 3) net use 映射
Write-Output "=== STEP3 net use ==="
$r = & cmd /c "net use Z: `"$uri`" /persistent:no 2>&1"
$r | ForEach-Object { Write-Output ("NETUSE| " + $_) }
Write-Output ("NETUSE_EXIT=" + $LASTEXITCODE)

# 4) 在 Z: 盘里「把整个文件夹拖进去」= 建子目录 + 写嵌套文件 + 顶层文件
Write-Output "=== STEP4 folder drag-in via Z: ==="
try {
  New-Item -ItemType Directory -Force -Path 'Z:\子文件夹' | Out-Null
  Write-Output 'MKDIR=OK'
} catch { Write-Output ('MKDIR_FAIL=' + $_.Exception.Message) }
try {
  [System.IO.File]::WriteAllText('Z:\会议记录.txt', '机密：季度预算 123 万')
  [System.IO.File]::WriteAllText('Z:\子文件夹\照片说明.txt', '子目录里的机密')
  Write-Output 'WRITE=OK'
} catch { Write-Output ('WRITE_FAIL=' + $_.Exception.Message) }
try {
  Write-Output ('READ_TOP=' + [System.IO.File]::ReadAllText('Z:\会议记录.txt'))
  Write-Output ('READ_NESTED=' + [System.IO.File]::ReadAllText('Z:\子文件夹\照片说明.txt'))
} catch { Write-Output ('READ_FAIL=' + $_.Exception.Message) }
Get-ChildItem -Recurse Z:\ -ErrorAction SilentlyContinue | ForEach-Object { Write-Output ('Z_ITEM=' + $_.FullName) }

# 5) 锁定
Write-Output "=== STEP5 lock ==="
& cmd /c "net use Z: /delete /y 2>&1" | ForEach-Object { Write-Output ("UNMAP| " + $_) }
try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
Start-Sleep -Seconds 1
Write-Output 'SERVE_STOPPED=OK'

# 6) export 验证写回是否落盘、层级能否还原
Write-Output "=== STEP6 export ==="
& $cli export $vault --out $out --password $pw 2>&1 | ForEach-Object { Write-Output ("OUT| " + $_) }
Write-Output ("EXPORT_EXIT=" + $LASTEXITCODE)
if (Test-Path $out) {
  Get-ChildItem -Recurse $out -ErrorAction SilentlyContinue | ForEach-Object { Write-Output ('EXPORT_ITEM=' + $_.FullName.Substring($out.Length+1)) }
}

# 7) 密文容器里不应有明文
Write-Output "=== STEP7 ciphertext scan ==="
$hits = 0
Get-ChildItem -LiteralPath $vault -Recurse -File | ForEach-Object {
  $t = [System.Text.Encoding]::UTF8.GetString([System.IO.File]::ReadAllBytes($_.FullName))
  if ($t.Contains('季度预算') -or $t.Contains('子目录里的机密')) { $hits++ }
}
Write-Output ("PLAINTEXT_HITS=" + $hits)
Get-ChildItem -LiteralPath $vault -Recurse -File | ForEach-Object { Write-Output ('VAULT_FILE=' + $_.FullName.Substring($vault.Length+1) + ' (' + $_.Length + 'B)') }

Remove-Item -Recurse -Force $base -ErrorAction SilentlyContinue
Write-Output 'DONE'
