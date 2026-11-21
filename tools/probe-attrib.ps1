# 验证 Windows 文件属性（隐藏/系统）对程序化读写/删除的影响。
# 目的：确认给密文容器加 attrib +h +s 不会破坏 mylock-core 的 fs::read / fs::write /
# fs::remove_dir_all / fs::rename。用 Python（底层同为 Win32 CreateFile/DeleteFile）
# 近似 Rust std::fs 行为。
$ErrorActionPreference = 'Continue'
$py = 'C:\Python312\python.exe'
$tmp = Join-Path $env:TEMP 'mylock_attrib_probe'
if (Test-Path $tmp) { cmd /c "attrib -s -h -r `"$tmp`" /s /d >nul 2>&1"; Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue }
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

# 造一个「保险柜」：desktop.ini(UTF-16LE) + icon + vault.meta + d/ 子目录 + manifest
$ini = Join-Path $tmp 'desktop.ini'
$icon = Join-Path $tmp 'MyLockVault.ico'
$meta = Join-Path $tmp 'vault.meta'
$man = Join-Path $tmp 'manifest'
$dd = Join-Path $tmp 'd'
$utf16 = New-Object System.Text.UnicodeEncoding($false, $true)
[System.IO.File]::WriteAllText($ini, "[.ShellClassInfo]`r`nIconFile=MyLockVault.ico`r`nIconIndex=0`r`nInfoTip=MyLock 保险柜：请勿删除内部文件`r`n", $utf16)
[System.IO.File]::WriteAllBytes($icon, [byte[]](1,2,3,4))
[System.IO.File]::WriteAllText($meta, 'MYVAULT-META')
[System.IO.File]::WriteAllText($man, 'MANIFEST-BLOB')
New-Item -ItemType Directory -Force -Path $dd | Out-Null
[System.IO.File]::WriteAllText((Join-Path $dd 'obj'), 'OBJECT-BLOB')

Write-Output "=== BEFORE ATTRIB ==="
cmd /c "attrib `"$tmp`""
cmd /c "attrib `"$meta`""

# 施加装饰属性：文件夹 +s；根级文件 +s +h
# 注意：attrib 一条命令带多个引号路径会报 "Parameter format not correct"，
# 必须逐文件单独调用（实测结论）。
cmd /c "attrib +s `"$tmp`""
foreach ($f in @($ini, $icon, $meta, $man, $dd)) {
  cmd /c "attrib +s +h `"$f`""
}

Write-Output "=== AFTER ATTRIB ==="
cmd /c "attrib `"$tmp`""
cmd /c "attrib `"$meta`""
cmd /c "attrib `"$man`""

# Python 模拟 Rust std::fs 的读 / 覆盖写 / rename / remove_dir_all
$probe = @'
import os, sys
tmp = sys.argv[1]
def rd(p):
    with open(p,'rb') as f: return f.read()
print("READ_META_OK", rd(os.path.join(tmp,'vault.meta')) == b'MYVAULT-META')
print("READ_MANIFEST_OK", rd(os.path.join(tmp,'manifest')) == b'MANIFEST-BLOB')
print("READ_INI_BOM", rd(os.path.join(tmp,'desktop.ini'))[:2] == b'\xff\xfe')
# 覆盖写（模拟 manifest tmp+rename）
with open(os.path.join(tmp,'manifest.tmp'),'wb') as f: f.write(b'NEW-MANIFEST')
os.replace(os.path.join(tmp,'manifest.tmp'), os.path.join(tmp,'manifest'))
print("RENAME_OVERWRITE_OK", rd(os.path.join(tmp,'manifest')) == b'NEW-MANIFEST')
# 新写 d/ 下对象
with open(os.path.join(tmp,'d','obj2'),'wb') as f: f.write(b'OBJ2')
print("WRITE_IN_D_OK", os.path.exists(os.path.join(tmp,'d','obj2')))
'@
$probeFile = Join-Path $env:TEMP 'mylock_attrib_probe.py'
[System.IO.File]::WriteAllText($probeFile, $probe, (New-Object System.Text.UTF8Encoding($false)))
Write-Output "=== PYTHON FS OPS ==="
& $py $probeFile $tmp
Write-Output "PY_EXIT=$LASTEXITCODE"

# 检查 rename 后 manifest 的属性是否丢失（决定性：决定要不要幂等重放）
Write-Output "=== ATTRIB AFTER RENAME ==="
cmd /c "attrib `"$man`""

# 清理：先去属性再删
cmd /c "attrib -s -h -r `"$tmp`" /s /d >nul 2>&1"
Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
Remove-Item -Force $probeFile -ErrorAction SilentlyContinue
Write-Output "CLEANED=$(-not (Test-Path $tmp))"
Write-Output "DONE"
