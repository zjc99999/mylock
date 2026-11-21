"""生成「保险柜」文件夹图标 MyLockVault.ico（多尺寸，供 desktop.ini IconFile 引用）。

设计为经典的转轮密码保险柜：钢灰机身 + 内嵌门板 + 组合转盘 + 右侧把手 + 底座。
用 Pillow 在每个目标尺寸上按 256 坐标系等比绘制，避免缩小插值导致线条发虚。
输出：desktop/src-tauri/icons/MyLockVault.ico
"""
from PIL import Image, ImageDraw
import os

OUT = os.path.join(os.path.dirname(__file__), "..", "desktop", "src-tauri", "icons", "MyLockVault.ico")
OUT = os.path.abspath(OUT)

SIZES = [16, 32, 48, 64, 128, 256]


def rr(draw, box, r, fill, outline=None, width=1):
    draw.rounded_rectangle(box, radius=r, fill=fill, outline=outline, width=width)


def draw_icon(size):
    s = size / 256.0
    P = lambda v: int(round(v * s))

    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # 阴影 / 底座
    rr(d, (P(36), P(214), P(220), P(230)), P(6), (12, 18, 28, 160))
    rr(d, (P(52), P(222), P(86), P(236)), P(4), (26, 36, 50, 255))
    rr(d, (P(170), P(222), P(204), P(236)), P(4), (26, 36, 50, 255))

    # 机身（外框 + 立体斜面）
    rr(d, (P(30), P(24), P(226), P(222)), P(18), (43, 58, 77, 255),
       outline=(92, 114, 141, 255), width=max(1, P(3)))
    rr(d, (P(38), P(30), P(218), P(214)), P(14), (58, 78, 102, 255))
    # 顶部高光
    rr(d, (P(44), P(34), P(212), P(58)), P(10), (72, 96, 124, 255))

    # 门板
    rr(d, (P(50), P(44), P(206), P(208)), P(12), (33, 47, 65, 255),
       outline=(104, 128, 156, 255), width=max(1, P(2)))
    rr(d, (P(56), P(50), P(200), P(202)), P(10), (47, 65, 88, 255))

    # 右侧把手
    rr(d, (P(206), P(108), P(216), P(148)), P(4), (140, 160, 184, 255),
       outline=(24, 32, 44, 255), width=max(1, P(1)))
    rr(d, (P(210), P(116), P(214), P(140)), P(2), (92, 112, 138, 255))

    # 组合转盘：外圈 + 刻度 + 内盘 + 指针
    cx, cy, R = P(126), P(126), P(48)
    d.ellipse((cx - R, cy - R, cx + R, cy + R), fill=(20, 30, 44, 255),
              outline=(150, 172, 198, 255), width=max(1, P(3)))
    # 刻度
    import math
    for i in range(12):
        a = math.radians(i * 30 - 90)
        r1, r2 = R - P(3), R - P(11)
        x1, y1 = cx + r1 * math.cos(a), cy + r1 * math.sin(a)
        x2, y2 = cx + r2 * math.cos(a), cy + r2 * math.sin(a)
        d.line((x1, y1, x2, y2), fill=(176, 196, 220, 255), width=max(1, P(2)))
    r_in = R - P(15)
    d.ellipse((cx - r_in, cy - r_in, cx + r_in, cy + r_in),
              fill=(86, 112, 142, 255), outline=(20, 30, 44, 255), width=max(1, P(2)))
    r_hub = P(12)
    d.ellipse((cx - r_hub, cy - r_hub, cx + r_hub, cy + r_hub),
              fill=(28, 40, 56, 255), outline=(168, 190, 214, 255), width=max(1, P(2)))
    # 指针（正上方）
    d.polygon([(cx, cy - r_in + P(2)), (cx - P(6), cy - r_in + P(12)),
               (cx + P(6), cy - r_in + P(12))], fill=(240, 196, 92, 255))

    # 铆钉
    for (px, py) in [(58, 58), (198, 58), (58, 194), (198, 194)]:
        rr(d, (P(px - 4), P(py - 4), P(px + 4), P(py + 4)), P(4), (120, 142, 168, 255))
    return img


def main():
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    frames = [draw_icon(sz) for sz in SIZES]
    frames[-1].save(OUT, format="ICO", sizes=[(sz, sz) for sz in SIZES],
                    append_images=frames[:-1])
    print("WROTE", OUT, os.path.getsize(OUT), "bytes", "sizes=", SIZES)


if __name__ == "__main__":
    main()
