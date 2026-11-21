"""从 M3 已生成的多尺寸 icon.ico 派生 Tauri bundle 需要的 PNG 图标。

M3 阶段已用 Pillow 生成 desktop/src-tauri/icons/icon.ico（含 16/32/48/64/128/256 六帧）。
Tauri 2 的 bundle.icon 需要 PNG 列表 + ICO，本脚本补齐三个 PNG，避免手写无效占位图。
诚实边界：这是构建脚手架用的自生成占位图标，不是正式产品美术资源。
"""
from pathlib import Path
from PIL import Image

HERE = Path(__file__).resolve().parent
ICONS = HERE.parent / "desktop" / "src-tauri" / "icons"
ICO = ICONS / "icon.ico"

im = Image.open(ICO)
# 取最大帧（256x256）作为缩放基准
frame = max(im.ico.sizes())
im.ico.getimage(frame).convert("RGBA").save(ICONS / "128x128@2x.png", sizes=[(256, 256)])
base256 = Image.open(ICONS / "128x128@2x.png").convert("RGBA")
base256.resize((128, 128), Image.LANCZOS).save(ICONS / "128x128.png")
base256.resize((32, 32), Image.LANCZOS).save(ICONS / "32x32.png")

for name in ["32x32.png", "128x128.png", "128x128@2x.png", "icon.ico"]:
    p = ICONS / name
    print(f"{name}: {p.stat().st_size} bytes")
print("ICON_GEN=OK")
