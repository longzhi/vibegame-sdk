#!/usr/bin/env python3
"""画《节拍盒》的封面：96×80 的像素画，写到 examples/beatbox/assets/cover.png。

照 tools/game-covers.py 的画法：一个像素一个像素画，颜色照 app 里的调色板。封面上不写字。

    python3 tools/covers/beatbox.py
"""

import math
from pathlib import Path

from PIL import Image

W, H = 96, 80
ROOT = Path(__file__).resolve().parent.parent.parent
OUT = ROOT / "examples" / "beatbox" / "assets" / "cover.png"

# 4×4 的有序抖动表。
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]

# 颜色和 src/lib.rs 的调色板一样。
BG = "#121022"
PANEL = "#221e3c"
CELL = "#303054"
HEAD = "#545080"
WHITE = "#f8f8f0"
ROW = ["#e85448", "#f4c438", "#50c8e8", "#70d870"]
ROW_HI = ["#ffaa96", "#ffec96", "#b4f0ff", "#c4ffb4"]

# 一段节奏：和「摇滚」那个现成的一样。
PATTERN = ["x...x.x.", "..x...x.", "xxxxxxxx", "x..x.x.x"]
HEAD_STEP = 4


def hexc(s):
    s = s.lstrip("#")
    return tuple(int(s[i : i + 2], 16) for i in (0, 2, 4))


class Canvas:
    def __init__(self, bg):
        self.px = [[hexc(bg)] * W for _ in range(H)]

    def set(self, x, y, c):
        if 0 <= x < W and 0 <= y < H:
            self.px[y][x] = hexc(c)

    def rect(self, x, y, w, h, c):
        for yy in range(y, y + h):
            for xx in range(x, x + w):
                self.set(xx, yy, c)

    def gradient(self, y0, y1, colors):
        n = len(colors) - 1
        for y in range(y0, y1):
            t = (y - y0) / max(1, y1 - y0 - 1) * n
            i = min(int(t), n - 1)
            f = t - i
            for x in range(W):
                self.set(x, y, colors[i + 1] if f * 16 > BAYER[y % 4][x % 4] else colors[i])

    def save(self, path):
        im = Image.new("RGB", (W, H))
        im.putdata([c for row in self.px for c in row])
        im.save(path)


def speaker(c, cx, cy):
    """一个圆喇叭：外圈深、里面一圈圈。"""
    for y in range(cy - 9, cy + 10):
        for x in range(cx - 9, cx + 10):
            d = math.hypot(x - cx, y - cy)
            if d < 3:
                c.set(x, y, "#141020")
            elif d < 5:
                c.set(x, y, "#5c5890")
            elif d < 7:
                c.set(x, y, "#3a3664")
            elif d < 9:
                c.set(x, y, "#141020")


def note(c, x, y, col):
    """一个八分音符：头 3×2，杆 4 高，尾巴一点。"""
    c.rect(x, y + 4, 3, 2, col)
    c.rect(x + 2, y, 1, 4, col)
    c.set(x + 3, y + 1, col)
    c.set(x + 4, y + 2, col)


def main():
    c = Canvas(BG)
    c.gradient(0, H, ["#0c0a1c", "#161430", "#241e48"])
    # 盒子：一块圆角的面板。
    bx, by, bw, bh = 4, 18, 88, 58
    c.rect(bx + 1, by, bw - 2, bh, PANEL)
    c.rect(bx, by + 1, bw, bh - 2, PANEL)
    c.rect(bx + 1, by + 1, bw - 2, 1, "#3a3462")
    # 上面两个喇叭，中间一排小灯。
    speaker(c, 18, 12)
    speaker(c, 78, 12)
    for i in range(5):
        c.rect(36 + i * 5, 10, 3, 3, ROW[i % 4] if i != 2 else WHITE)
    # 格子：八列四行，一格 8 像素。放到第 5 列，那一列亮。
    gx, gy = 9, 24
    for r, row in enumerate(PATTERN):
        for s, ch in enumerate(row):
            x, y = gx + s * 10, gy + r * 12
            on = ch == "x"
            head = s == HEAD_STEP
            if on:
                col = ROW_HI[r] if head else ROW[r]
            else:
                col = HEAD if head else CELL
            c.rect(x, y, 8, 10, col)
            if on:
                c.rect(x, y, 8, 1, ROW_HI[r])
    # 指着这一列的小三角。
    tx = gx + HEAD_STEP * 10 + 4
    for i in range(3):
        c.rect(tx - 3 + i, 20 + i, 6 - 2 * i, 1, WHITE)
    # 光标：白框框住一格。
    cx, cy = gx + 2 * 10 - 1, gy + 3 * 12 - 1
    for x in range(cx, cx + 10):
        c.set(x, cy, WHITE)
        c.set(x, cy + 11, WHITE)
    for y in range(cy, cy + 12):
        c.set(cx, y, WHITE)
        c.set(cx + 9, y, WHITE)
    # 飘出来的音符。
    note(c, 4, 2, ROW_HI[1])
    note(c, 88, 1, ROW_HI[2])
    note(c, 44, 0, ROW_HI[3])
    c.save(OUT)
    print(f"{OUT.relative_to(ROOT)}：{W}×{H}")


if __name__ == "__main__":
    main()
