$ErrorActionPreference = 'Continue'
$env:PATH = "C:\Users\zjc_9\.cargo\bin;C:\mingw64\bin;$env:PATH"
$base = Join-Path $env:TEMP ("mylock-m3-smoke-" + [guid]::NewGuid().ToString('N').Substring(0,8))
$root = Join-Path $base "vault"
$seed = Join-Path $base "seed.txt"
$out  = Join-Path $base "exported"
New-Item -ItemType Directory -Force -Path $base | Out-Null
[System.IO.File]::WriteAllText($seed, "seed from disk")

$cli = "D:\Projects\dsh\mylock\core\target\debug\mylock.exe"
$serve = "D:\Projects\dsh\mylock\core\target\debug\mylock-serve.exe"
$pw = "SmokeTest-Pw-2026"
$port = 51918

Write-Output "BASE=$base"
Write-Output "ROOT=$root"

# 1) create vault with one seed file via the independent recovery CLI
& $cli create $root --password $pw --file $seed 2>&1 | ForEach-Object { Write-Output $_ }
Write-Output "CREATE_EXIT=$LASTEXITCODE"

# 2) start loopback WebDAV server as a child process
$p = Start-Process -FilePath $serve -ArgumentList @("--root", $root, "--password", $pw, "--port", "$port") -PassThru -RedirectStandardOutput (Join-Path $base "serve.out") -RedirectStandardError (Join-Path $base "serve.err") -WindowStyle Hidden
Write-Output "SERVE_PID=$($p.Id)"
Start-Sleep -Seconds 2

# 3) curl: PROPFIND root, PUT a new file, GET it back, GET seed
$curl = "curl.exe"
$baseUri = "http://127.0.0.1:$port/"
& $curl -s -o NUL -w "PROPFIND_STATUS=%{http_code}`n" -X PROPFIND -H "Depth: 1" $baseUri 2>&1 | ForEach-Object { Write-Output $_ }
& $curl -s -o NUL -w "PUT_STATUS=%{http_code}`n" -X PUT --data-binary "written over loopback" ("${baseUri}loop.txt") 2>&1 | ForEach-Object { Write-Output $_ }
$got = & $curl -s ("${baseUri}loop.txt") 2>&1
Write-Output "GET_LOOP=$got"
$seedGot = & $curl -s ("${baseUri}seed.txt") 2>&1
Write-Output "GET_SEED=$seedGot"

# 4) export from the encrypted container via the CLI (proves writes really landed on disk)
& $cli export $root --out $out --password $pw 2>&1 | ForEach-Object { Write-Output $_ }
Write-Output "EXPORT_EXIT=$LASTEXITCODE"
if (Test-Path (Join-Path $out "loop.txt")) {
  Write-Output ("EXPORT_LOOP=" + [System.IO.File]::ReadAllText((Join-Path $out "loop.txt")))
} else {
  Write-Output "EXPORT_LOOP=MISSING"
}

# 5) cleanup
try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
Write-Output "SERVE_STDOUT_BEGIN"
Get-Content (Join-Path $base "serve.out") -ErrorAction SilentlyContinue | ForEach-Object { Write-Output $_ }
Write-Output "SERVE_STDOUT_END"
Remove-Item -Recurse -Force $base -ErrorAction SilentlyContinue
Write-Output "DONE"
