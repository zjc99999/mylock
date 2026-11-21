# MyLock 桌面端到端探针（M5 / 零成本研发第 1 项）
# 目的：验证 M3 报告宣称但从未实测的 `net use Z: http://127.0.0.1:<port>/`
#       是否真能在 Windows 上映射出盘符，并通过盘符读写、锁定、拷盘仍密文。
#
# 这是只读诊断脚本：不修改产品代码，只造临时 vault 与临时盘符映射。
# 诚实边界：即使 net use 成功，也仍是 loopback WebDAV，非 WinFsp 驱动级挂载；
#           WebClient 服务状态 / 是否需要管理员属本机环境事实，不是产品保证。

$ErrorActionPreference = 'Continue'
$env:PATH = "C:\Users\zjc_9\.cargo\bin;C:\mingw64\bin;$env:PATH"

$base = Join-Path $env:TEMP ("mylock-e2e-" + [guid]::NewGuid().ToString('N').Substring(0,8))
$root = Join-Path $base "vault"
$seed = Join-Path $base "seed.txt"
$cwdWrite = Join-Path $base "cwd-write.txt"
$out  = Join-Path $base "exported"
$copy = Join-Path $base "vault-copy"
New-Item -ItemType Directory -Force -Path $base | Out-Null
[System.IO.File]::WriteAllText($seed, "seed-from-disk-visible-only-when-unlocked")

$cli   = "D:\Projects\dsh\mylock\core\target\debug\mylock.exe"
$serve = "D:\Projects\dsh\mylock\core\target\debug\mylock-serve.exe"
$pw    = "E2E-Probe-Pw-2026"
$port  = 51919
$uri   = "http://127.0.0.1:$port/"

Write-Output "BASE=$base"
Write-Output "ROOT=$root"
Write-Output "URI=$uri"

# --- 0) WebClient 服务：net use 到 WebDAV 需要它 ---
Write-Output "=== STEP 0: WebClient service ==="
$wc = Get-Service WebClient -ErrorAction SilentlyContinue
Write-Output ("WEBCLIENT_BEFORE=" + $(if ($wc) { "$($wc.Status)/$($wc.StartType)" } else { "MISSING" }))
try {
    Start-Service WebClient -ErrorAction Stop
    Write-Output "WEBCLIENT_START=OK"
} catch {
    Write-Output ("WEBCLIENT_START=FAIL: " + $_.Exception.Message)
}
$wc = Get-Service WebClient -ErrorAction SilentlyContinue
Write-Output ("WEBCLIENT_AFTER=" + $(if ($wc) { "$($wc.Status)/$($wc.StartType)" } else { "MISSING" }))

# --- 1) 用独立恢复 CLI 建库 ---
Write-Output "=== STEP 1: create vault ==="
& $cli create $root --password $pw --file $seed 2>&1 | ForEach-Object { Write-Output $_ }
Write-Output "CREATE_EXIT=$LASTEXITCODE"

# --- 2) 启动 loopback WebDAV 服务 ---
Write-Output "=== STEP 2: start mylock-serve ==="
$p = Start-Process -FilePath $serve `
    -ArgumentList @("--root", $root, "--password", $pw, "--port", "$port") `
    -PassThru `
    -RedirectStandardOutput (Join-Path $base "serve.out") `
    -RedirectStandardError  (Join-Path $base "serve.err") `
    -WindowStyle Hidden
Write-Output "SERVE_PID=$($p.Id)"
Start-Sleep -Seconds 2
Write-Output ("SERVE_ALIVE=" + (-not $p.HasExited))

# --- 3) curl 基线（M3 已验证过的路径） ---
Write-Output "=== STEP 3: curl baseline ==="
& curl.exe -s -o NUL -w "PROPFIND_STATUS=%{http_code}`n" -X PROPFIND -H "Depth: 1" $uri 2>&1 | ForEach-Object { Write-Output $_ }

# --- 4) 关键：真正尝试 net use 映射盘符（多种 URL 形式逐一试） ---
Write-Output "=== STEP 4: net use mapping candidates ==="
$candidates = @(
    "http://127.0.0.1:$port/",
    "http://localhost:$port/",
    "\\127.0.0.1@$port\DavWWWRoot",
    "\\localhost@$port\DavWWWRoot"
)
$mapped = $null
foreach ($cand in $candidates) {
    Write-Output ("--- CANDIDATE=" + $cand)
    $r = & cmd /c "net use Z: `"$cand`" 2>&1"
    $code = $LASTEXITCODE
    $r | ForEach-Object { Write-Output ("NETUSE_OUT=" + $_) }
    Write-Output "NETUSE_EXIT=$code"
    if ($code -eq 0) { $mapped = $cand; break }
    # 失败后清理半残留映射再试下一个
    & cmd /c "net use Z: /delete /y >nul 2>&1" | Out-Null
}
Write-Output ("MAPPED=" + $(if ($mapped) { $mapped } else { "NONE" }))

# --- 5) 若映射成功，通过盘符做真实读写 ---
if ($mapped) {
    Write-Output "=== STEP 5: drive-letter IO ==="
    Start-Sleep -Seconds 1
    Write-Output ("Z_EXISTS=" + (Test-Path Z:\) )
    try {
        Get-ChildItem Z:\ -ErrorAction Stop | ForEach-Object { Write-Output ("Z_ITEM=" + $_.Name) }
    } catch { Write-Output ("Z_LIST_FAIL=" + $_.Exception.Message) }
    try {
        [System.IO.File]::WriteAllText("Z:\written-from-drive.txt", "hello-from-Z-drive-2026")
        Write-Output "Z_WRITE=OK"
    } catch { Write-Output ("Z_WRITE_FAIL=" + $_.Exception.Message) }
    try {
        $seedViaZ = [System.IO.File]::ReadAllText("Z:\seed.txt")
        Write-Output ("Z_READ_SEED=" + $seedViaZ)
    } catch { Write-Output ("Z_READ_SEED_FAIL=" + $_.Exception.Message) }
    try {
        $wViaZ = [System.IO.File]::ReadAllText("Z:\written-from-drive.txt")
        Write-Output ("Z_READ_NEW=" + $wViaZ)
    } catch { Write-Output ("Z_READ_NEW_FAIL=" + $_.Exception.Message) }
}

# --- 6) 断开映射、锁定（停 serve） ---
Write-Output "=== STEP 6: unmap + lock ==="
& cmd /c "net use Z: /delete /y 2>&1" | ForEach-Object { Write-Output ("UNMAP_OUT=" + $_) }
Write-Output "UNMAP_EXIT=$LASTEXITCODE"
try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
Write-Output "SERVE_STOPPED=OK"

# --- 7) 写回是否真的落到密文容器：CLI export ---
Write-Output "=== STEP 7: CLI export (proves disk write-back) ==="
& $cli export $root --out $out --password $pw 2>&1 | ForEach-Object { Write-Output $_ }
Write-Output "EXPORT_EXIT=$LASTEXITCODE"
if (Test-Path (Join-Path $out "written-from-drive.txt")) {
    Write-Output ("EXPORT_NEW=" + [System.IO.File]::ReadAllText((Join-Path $out "written-from-drive.txt")))
} else { Write-Output "EXPORT_NEW=MISSING" }
if (Test-Path (Join-Path $out "seed.txt")) {
    Write-Output ("EXPORT_SEED=" + [System.IO.File]::ReadAllText((Join-Path $out "seed.txt")))
} else { Write-Output "EXPORT_SEED=MISSING" }

# --- 8) 整盘复制到另一路径（模拟拷到移动硬盘）仍是保险柜 ---
Write-Output "=== STEP 8: copy vault to another path, verify still ciphertext ==="
Copy-Item -LiteralPath $root -Destination $copy -Recurse -Force
& $cli verify $copy --json 2>&1 | ForEach-Object { Write-Output $_ }
Write-Output "VERIFY_COPY_EXIT=$LASTEXITCODE"
# 密文容器里不应出现任何明文片段
$plainHits = 0
Get-ChildItem -LiteralPath $copy -Recurse -File | ForEach-Object {
    $bytes = [System.IO.File]::ReadAllBytes($_.FullName)
    $txt = [System.Text.Encoding]::ASCII.GetString($bytes)
    if ($txt.Contains("seed-from-disk-visible-only-when-unlocked") -or
        $txt.Contains("hello-from-Z-drive-2026")) { $plainHits++ }
}
Write-Output "PLAINTEXT_HITS_IN_COPY=$plainHits"
Get-ChildItem -LiteralPath $copy -Recurse -File | ForEach-Object { Write-Output ("COPY_FILE=" + $_.FullName.Substring($copy.Length+1) + " (" + $_.Length + "B)") }

# --- 9) 输出 serve 日志并清理 ---
Write-Output "SERVE_STDOUT_BEGIN"
Get-Content (Join-Path $base "serve.out") -ErrorAction SilentlyContinue | ForEach-Object { Write-Output $_ }
Write-Output "SERVE_STDOUT_END"
Write-Output "SERVE_STDERR_BEGIN"
Get-Content (Join-Path $base "serve.err") -ErrorAction SilentlyContinue | ForEach-Object { Write-Output $_ }
Write-Output "SERVE_STDERR_END"
Remove-Item -Recurse -Force $base -ErrorAction SilentlyContinue
Write-Output "DONE"
