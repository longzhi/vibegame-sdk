#!/usr/bin/env python3
"""画《桌面时钟》的封面：96×80 的像素画，写到 examples/deskclock/assets/cover.png。

照 tools/game-covers.py 的画法：像素一个一个用代码画，颜色、点阵字、鱼照 app 里的。
上半是点阵的大字钟（薄荷色主题），下半是白天的鱼缸。封面上不写名字。

    python3 tools/covers/deskclock.py
"""

from pathlib import Path

from PIL import Image

W, H = 96, 80
ROOT = Path(__file__).resolve().parent.parent.parent
OUT = ROOT / "examples" / "deskclock" / "assets" / "cover.png"

# 4×4 的有序抖动表：两种颜色过渡时一格一格交错。
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]


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

    def gradient(self, y0, y1, colors, x0=0, x1=W):
        """y0 到 y1 一段竖着的渐变，几种颜色之间用抖动过渡。"""
        n = len(colors) - 1
        for y in range(y0, y1):
            t = (y - y0) / max(1, y1 - y0 - 1) * n
            i = min(int(t), n - 1)
            f = t - i
            for x in range(x0, x1):
                c = colors[i + 1] if f * 16 > BAYER[y % 4][x % 4] else colors[i]
                self.set(x, y, c)

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


def flip(spr):
    return [list(reversed(r)) for r in spr]


def scale(spr, k):
    return [[c for c in r for _ in range(k)] for r in spr for _ in range(k)]


# 和 app 里 DIGITS 一样的点阵字：5×8。
DIGITS = {
    "0": [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    "1": ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "2": [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
    "3": [".###.", "#...#", "....#", "..##.", "....#", "....#", "#...#", ".###."],
    "4": ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#.", "...#."],
    "5": ["#####", "#....", "####.", "....#", "....#", "....#", "#...#", ".###."],
    "6": ["..##.", ".#...", "#....", "####.", "#...#", "#...#", "#...#", ".###."],
    "7": ["#####", "....#", "...#.", "...#.", "..#..", "..#..", "..#..", "..#.."],
    "8": [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", "#...#", ".###."],
    "9": [".###.", "#...#", "#...#", "#...#", ".####", "....#", "...#.", ".##.."],
}

# 薄荷色主题：背景、亮的点、暗的点。
BG, ON, OFF = "#0e161a", "#84f0bc", "#1a322c"

# 和 app 里一样的鱼，朝右。
FISH = ["....ff.....", "ff.fbbsb...", "fbfbbbsbbb.", "fbbbbbsbbeb", "fbfbbbsbbb.", "ff..bbsb...", "....ff....."]


def digit(c, ch, x, y):
    """一个点阵字：一个点 2×2，点和点隔 1 个像素。"""
    for r, row in enumerate(DIGITS[ch]):
        for k, bit in enumerate(row):
            c.rect(x + k * 3, y + r * 3, 2, 2, ON if bit == "#" else OFF)


def plant(c, x, h, lean):
    """一棵水草：茎往一边弯，隔几个像素一片叶子。"""
    for i in range(h):
        dx = lean * i * i // (h * h)
        y = 74 - i
        c.set(x + dx, y, "#247840")
        c.set(x + dx + 1, y, "#48b058")
        if i % 5 == 2 and i > 2:
            side = -1 if (i // 5) % 2 else 2
            c.set(x + dx + side, y - 1, "#48b058")
            c.set(x + dx + side * 2 - (1 if side > 0 else 0), y - 2, "#48b058")


def deskclock():
    c = Canvas(BG)
    # 上半：「10:08」点阵大字，冒号亮着。
    x = 11
    for i, ch in enumerate("1008"):
        digit(c, ch, x, 6)
        x += 17 if i != 1 else 0
        if i == 1:
            c.rect(x + 17, 12, 2, 2, ON)
            c.rect(x + 17, 21, 2, 2, ON)
            x += 25
    # 下半：鱼缸。玻璃、上面的灯、白天的水。
    c.rect(2, 34, 92, 46, "#465460")
    c.rect(2, 34, 92, 2, "#fafadc")
    c.gradient(37, 74, ["#78c8f0", "#48a0e0", "#2878c8", "#1c58a8"], 4, 92)
    # 斜着的光。
    for y in range(37, 60):
        for x in range(4, 92):
            if (x + (y - 37) // 2) % 30 < 3 and (x + y) % 2 == 0:
                c.set(x, y, "#b0e6fc")
    # 沙子和小石子。
    for y in range(74, 80):
        for x in range(4, 92):
            c.set(x, y, "#b09460" if (y - 74) * 3 > BAYER[y % 4][x % 4] + 4 else "#d6ba80")
    for x, y in [(10, 77), (30, 78), (52, 76), (70, 78), (84, 77)]:
        c.rect(x, y, 2, 1, "#827468")
    # 石头和水草。
    rock = art(
        ["...hhhh...", ".hhrrrrhh.", "hrrrrrrrrr", "rrrrrr..rr", "rrrrrrr.rr"],
        {"h": "#9696a6", "r": "#686878"},
    )
    c.blit(rock, 44, 70)
    plant(c, 9, 30, 3)
    plant(c, 15, 20, -2)
    plant(c, 80, 32, -3)
    plant(c, 86, 22, 2)
    # 气泡从石头里冒出来。
    for x, y, big in [(50, 64, 0), (49, 56, 1), (51, 47, 1), (49, 40, 1)]:
        if big:
            c.rect(x, y, 3, 1, "#dcf4ff")
            c.rect(x, y + 2, 3, 1, "#dcf4ff")
            c.set(x - 1, y + 1, "#dcf4ff")
            c.set(x + 3, y + 1, "#dcf4ff")
        else:
            c.rect(x, y, 2, 2, "#dcf4ff")
    # 三条鱼：橙的、黄的、蓝的，两倍大一条。
    fish = [
        ({"b": "#ff8028", "f": "#ffdcc8", "s": "#ffffff", "e": "#101018"}, 2, 22, 42, False),
        ({"b": "#ffd440", "f": "#e87828", "s": "#282838", "e": "#101018"}, 1, 62, 52, True),
        ({"b": "#50a0ff", "f": "#ffdc40", "s": "#1838a0", "e": "#101018"}, 1, 24, 62, False),
    ]
    for keys, k, x, y, left in fish:
        spr = scale(art(FISH, keys), k)
        c.blit(flip(spr) if left else spr, x, y)
    return c


def main():
    deskclock().save(OUT)
    print(f"{OUT.relative_to(ROOT)}：{W}×{H}")


if __name__ == "__main__":
    main()
