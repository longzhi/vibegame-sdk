#!/usr/bin/env python3
"""画《像素小狗》的封面：96×80 的像素画，写到 examples/pixelpup/assets/cover.png。

    python3 tools/covers/pixelpup.py

照 tools/game-covers.py 的画法：像素一个一个用代码画，颜色和精灵照 app 里的（src/art.rs）。
封面上不写名字：家上封面底下有名字，跟着系统的语言换。
"""

from pathlib import Path

from PIL import Image

W, H = 96, 80
ROOT = Path(__file__).resolve().parent.parent.parent

# 4×4 的有序抖动表：两种颜色过渡时一格一格交错。
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]

# 颜色照 src/art.rs 的 PALETTE。
INK = "#1a1424"
CREAM = "#f6e0b2"
BROWN = "#a4643c"
RED = "#d83c3c"
PINK = "#f08ca0"
WHITE = "#fff8e8"
WALL = "#7cb4a8"
WALL2 = "#6ca498"
FLOOR = "#b07848"
FLOOR2 = "#946038"
SKY = "#8cd0f8"
SKY_LO = "#b8e4fc"
FRAME = "#f0e8d2"
CUSHION = "#6a5aa8"
BOWL = "#4878d0"
KIBBLE = "#8c5030"

DOG = {"k": INK, "c": CREAM, "b": BROWN, "r": RED, "p": PINK, "w": WHITE}
# 照 art.rs 的 head(true) 和 body(true)。
HEAD = [".kkk......", "kbbck.....", "kbccckkk..", "kbcwkcccck", ".kcccccckk", "..kcccpk.."]
BODY = [
    "..k.......krrrrk..",
    ".kck....kkcccccck.",
    ".kck...kccccccccck",
    "..kck.kcbbccccccck",
    "...kkkcbbbcccccck.",
    ".....kcbbcccccccck",
    ".....kccccccccccck",
    ".....kcckcckcckcck",
    ".....kcck.kck.kcck",
    ".....kkkk.kkk.kkkk",
]
BOWL_ART = ["...ooooooo....", "kkkkkkkkkkkkkk", "kbbbbbbbbbbbbk", ".kbwbbbbbbbbk.", "..kbbbbbbbbk..", "...kkkkkkkk..."]
HEART = [".pp.pp.", "pppwppp", "ppppppp", ".ppppp.", "..ppp..", "...p..."]


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

    def gradient(self, x0, y0, w, y1, colors):
        """一块竖着的渐变，两种颜色之间用抖动过渡。"""
        for y in range(y0, y1):
            f = (y - y0) / max(1, y1 - y0 - 1)
            for x in range(x0, x0 + w):
                self.set(x, y, colors[1] if f * 16 > BAYER[y % 4][x % 4] else colors[0])

    def blit(self, spr, x, y):
        for yy, row in enumerate(spr):
            for xx, c in enumerate(row):
                if c is not None:
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


def cover():
    c = Canvas(WALL)
    # 墙：竖条纹。
    for x in range(0, W, 12):
        c.rect(x, 0, 6, 58, WALL2)
    # 窗：蓝天，一朵云。
    c.rect(6, 6, 32, 28, FRAME)
    c.gradient(8, 8, 28, 32, [SKY, SKY_LO])
    c.rect(11, 14, 10, 3, WHITE)
    c.rect(14, 12, 5, 2, WHITE)
    c.rect(21, 8, 2, 24, FRAME)
    c.rect(8, 19, 28, 2, FRAME)
    # 地板。
    c.rect(0, 58, W, 22, FLOOR)
    c.rect(0, 57, W, 1, INK)
    for y in (65, 73):
        c.rect(0, y, W, 1, FLOOR2)
    for x, y in ((14, 58), (61, 58), (37, 66), (84, 66), (22, 74), (70, 74)):
        c.rect(x, y, 1, 7, FLOOR2)
    # 垫子、小狗、饭碗、一颗心。
    c.rect(10, 70, 58, 6, CUSHION)
    c.rect(12, 68, 54, 2, CUSHION)
    k = 3
    dx, dy = 12, 26
    c.blit(scale(art(BODY, DOG), k), dx, dy + 6 * k)
    c.blit(scale(art(HEAD, DOG), k), dx + 8 * k, dy)
    c.blit(scale(art(BOWL_ART, {"k": INK, "b": BOWL, "w": WHITE, "o": KIBBLE}), 2), 66, 62)
    c.blit(scale(art(HEART, {"p": PINK, "w": WHITE}), 2), 70, 10)
    return c


def main():
    out = ROOT / "examples" / "pixelpup" / "assets" / "cover.png"
    cover().save(out)
    print(f"{out.relative_to(ROOT)}：{W}×{H}")


if __name__ == "__main__":
    main()
