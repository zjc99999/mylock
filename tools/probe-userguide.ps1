# 为「软件功能与操作说明」文档做真实性探针：验证便携包内 CLI 的真实输出与行为。
# 不碰用户真实数据（E:\testLock 绝不操作），全部在 %TEMP% 下模拟。
$ErrorActionPreference = 'Continue'
$cli = 'D:\Projects\dsh\mylock\dist\MyLock-Beta-0.1.0\mylock.exe'
$base = Join-Path $env:TEMP ("mylock-guide-" + [guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Force -Path $base | Out-Null
$pw = 'Demo-Pw-2026'
$src = Join-Path $base 'testLock'          # 模拟用户的明文文件夹
$vault = Join-Path $base 'testLockVault'   # 新建的密文容器
$out = Join-Path $base 'decrypted'
New-Item -ItemType Directory -Force -Path $src | Out-Null
[System.IO.File]::WriteAllText((Join-Path $src '会议记录.txt'), '这是机密会议记录明文')
[System.IO.File]::WriteAllText((Join-Path $src 'report.docx'), 'FAKE-DOCX-PLAINTEXT')
New-Item -ItemType Directory -Force -Path (Join-Path $src '子文件夹') | Out-Null
[System.IO.File]::WriteAllText((Join-Path $src '子文件夹\照片说明.txt'), '子目录里的文件')

function Run([string]$label, [string[]]$argv) {
  Write-Output "=== $label ==="
  & $cli @argv 2>&1 | ForEach-Object { Write-Output ("OUT| " + $_) }
  Write-Output ("EXIT| " + $LASTEXITCODE)
}

Write-Output "BASE=$base"
Write-Output ("CLI_EXISTS=" + (Test-Path $cli))
Run 'help' @('--help')
Run 'create new vault (flat files)' @('create', $vault, '--password', $pw, '--file', (Join-Path $src '会议记录.txt'), '--file', (Join-Path $src 'report.docx'))
Run 'info' @('info', $vault)
Run 'verify --json' @('verify', $vault, '--json')
Run 'export' @('export', $vault, '--out', $out, '--password', $pw)
if (Test-Path $out) { Get-ChildItem $out | ForEach-Object { Write-Output ("EXPORT_FILE=" + $_.Name + " " + $_.Length + "B") } }
Run 'wrong password' @('export', $vault, '--out', (Join-Path $base 'no'), '--password', 'WrongPw')
Run 'change-password' @('change-password', $vault, '--password', $pw, '--new', 'NewPw-2026')
Run 'export with new password' @('export', $vault, '--out', (Join-Path $base 'out2'), '--password', 'NewPw-2026')
Run 'rotate-keys' @('rotate-keys', $vault, (Join-Path $base 'testLockVault-rotated'), '--password', 'NewPw-2026', '--new', 'Rotated-2026')
Run 'export rotated with OLD password (should FAIL)' @('export', (Join-Path $base 'testLockVault-rotated'), '--out', (Join-Path $base 'no2'), '--password', 'NewPw-2026')

# 危险行为验证：create 到「已存在的非空目录」会不会清空原内容
$danger = Join-Path $base 'danger-existing'
New-Item -ItemType Directory -Force -Path $danger | Out-Null
[System.IO.File]::WriteAllText((Join-Path $danger '原明文.txt'), '我会被删掉吗')
Run 'create INTO existing non-empty dir' @('create', $danger, '--password', $pw)
Write-Output ("ORIGINAL_STILL_EXISTS=" + (Test-Path (Join-Path $danger '原明文.txt')))
Write-Output ("VAULT_META_EXISTS=" + (Test-Path (Join-Path $danger 'vault.meta')))

# --file 传目录会怎样
Run 'create with --file pointing at a DIRECTORY' @('create', (Join-Path $base 'vault-dirfile'), '--password', $pw, '--file', (Join-Path $src '子文件夹'))

Remove-Item -Recurse -Force $base -ErrorAction SilentlyContinue
Write-Output 'DONE'
