#!/usr/bin/env python3
"""画《口算星星》的封面：96×80 的像素画，写到 examples/mathstar/assets/cover.png。

    python3 tools/covers/mathstar.py

照 tools/game-covers.py 的画法：像素一个一个用代码画，颜色和精灵照 app 里的（src/art.rs）。
封面上不写名字：家上封面底下有名字，跟着系统的语言换。
"""

import math
from pathlib import Path

from PIL import Image

W, H = 96, 80
ROOT = Path(__file__).resolve().parent.parent.parent

# 4×4 的有序抖动表：两种颜色过渡时一格一格交错。
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]

# 颜色照 src/art.rs 的 PALETTE。
SKY = "#24306c"
SKY_HI = "#34448c"
INK = "#100c24"
WHITE = "#faf6ec"
YELLOW = "#ffd23f"
YELLOW_HI = "#fff4a0"
ORANGE = "#ff9420"
PINK = "#ff80b0"
GREEN = "#40c068"
CYAN = "#70d8ff"
PURPLE = "#9c6ce0"
GRAY = "#7884b8"
HILL = "#2c8c70"
HILL_HI = "#48b084"


def hexc(s):
    s = s.lstrip("#")
    return tuple(int(s[i : i + 2], 16) for i in (0, 2, 4))


class Canvas:
    def __init__(self, bg="#000000"):
        self.px = [[hexc(bg)] * W for _ in range(H)]

    def set(self, x, y, c):
        if 0 <= x < W and 0 <= y < H and c is not None:
            self.px[y][x] = hexc(c) if isinstance(c, str) else c

    def rect(self, x, y, w, h, c):
        for yy in range(y, y + h):
            for xx in range(x, x + w):
                self.set(xx, yy, c)

    def gradient(self, y0, y1, colors):
        """y0 到 y1 一段竖着的渐变，几种颜色之间用抖动过渡。"""
        n = len(colors) - 1
        for y in range(y0, y1):
            t = (y - y0) / max(1, y1 - y0 - 1) * n
            i = min(int(t), n - 1)
            f = t - i
            for x in range(W):
                self.set(x, y, colors[i + 1] if f * 16 > BAYER[y % 4][x % 4] else colors[i])

    def blit(self, spr, x, y):
        for yy, row in enumerate(spr):
            for xx, c in enumerate(row):
                if c is not None:
                    self.set(x + xx, y + yy, c)

    def outline(self, spr, x, y, c):
        """精灵四周描一圈 `c`：压在花背景上也看得清。"""
        h, w = len(spr), len(spr[0])
        for yy in range(-1, h + 1):
            for xx in range(-1, w + 1):
                inside = 0 <= yy < h and 0 <= xx < w and spr[yy][xx] is not None
                if not inside and any(
                    0 <= yy + dy < h and 0 <= xx + dx < w and spr[yy + dy][xx + dx] is not None
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
                ):
                    self.set(x + xx, y + yy, c)

    def save(self, path):
        im = Image.new("RGB", (W, H))
        im.putdata([c for row in self.px for c in row])
        im.save(path)
        return im


def art(rows, keys):
    """字符画变成精灵：一格一个颜色，不在 keys 里的字符是透明。"""
    return [[keys.get(ch) for ch in r] for r in rows]


def scale(spr, k):
    return [[c for c in row for _ in range(k)] for row in spr for _ in range(k)]


# 和 src/art.rs 的 buddy() 一样。
BUDDY = [
    ".......o.......",
    "......oyo......",
    "......oyo......",
    ".....oyhyo.....",
    "oooooyhyyyooooo",
    "oyyyyhyyyyyyyyo",
    ".oyyyyyyyyyyyo.",
    "..oyyiyyyiyyo..",
    "..oyyiyyyiyyo..",
    "..opyiyyyiypo..",
    "...oyyiiiyyo...",
    "...oyyyoyyyo...",
    "..oyyyo.oyyyo..",
    "..oyyo...oyyo..",
    ".oyyo.....oyyo.",
    ".ooo.......ooo.",
]
# 和 src/art.rs 的 star() 一样。
STAR = ["....o....", "...oyo...", "...oyo...", "oooyyyooo", ".oyyhyyo.", "..oyyyo..", "..oyoyo..", ".oyo.oyo.", ".oo...oo."]
KEYS = {"o": ORANGE, "y": YELLOW, "h": YELLOW_HI, "i": INK, "p": PINK}

# 算式的记号，5×5：加、减、等于。是记号，不是名字。
PLUS = ["..x..", "..x..", "xxxxx", "..x..", "..x.."]
MINUS = [".....", ".....", "xxxxx", ".....", "....."]
EQUALS = [".....", "xxxxx", ".....", "xxxxx", "....."]


def mathstar():
    c = Canvas()
    c.gradient(0, H, [INK, SKY, SKY_HI])
    # 小星星一闪一闪。
    for i in range(30):
        x, y = (i * 83) % W, (i * 47) % 60
        c.set(x, y, YELLOW_HI if i % 3 == 0 else GRAY)
    # 底下一道小山。
    for x in range(W):
        h = 9 + int(3 * math.sin(x / 9))
        c.rect(x, H - h, 1, h, HILL)
        c.set(x, H - h, HILL_HI)
    # 飘着的算式记号，放大两倍，各一种颜色。
    for spr, col, x, y in ((PLUS, PINK, 8, 12), (MINUS, CYAN, 76, 16), (EQUALS, GREEN, 10, 46), (PLUS, PURPLE, 78, 46)):
        s = scale(art(spr, {"x": col}), 2)
        c.outline(s, x, y, INK)
        c.blit(s, x, y)
    # 三颗得到的星，排成一道弧。
    star = scale(art(STAR, KEYS), 1)
    for x, y in ((26, 8), (43, 3), (60, 8)):
        c.outline(star, x, y, INK)
        c.blit(star, x, y)
    # 主角放大三倍，站在山上。
    b = scale(art(BUDDY, KEYS), 3)
    bx, by = (W - len(b[0])) // 2, 22
    c.outline(b, bx, by, INK)
    c.blit(b, bx, by)
    return c


def main():
    out = ROOT / "examples" / "mathstar" / "assets" / "cover.png"
    mathstar().save(out)
    print(f"{out.relative_to(ROOT)}：{W}×{H}")


if __name__ == "__main__":
    main()
