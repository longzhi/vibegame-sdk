#!/usr/bin/env python3
"""画《记分板》的封面：96×80，写到 examples/scorepad/assets/cover.png。

    python3 tools/covers/scorepad.py

照 tools/game-covers.py 的画法：一个像素一个像素画，颜色照 app 里的，不写字。
桌上一本记分本，四个人的颜色和正字划记；旁边两个骰子，领先的人头上有王冠。
"""

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
spec = importlib.util.spec_from_file_location("game_covers", ROOT / "tools" / "game-covers.py")
gc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gc)
Canvas, art, scale, BAYER, W, H = gc.Canvas, gc.art, gc.scale, gc.BAYER, gc.W, gc.H

# 颜色照 examples/scorepad/src/art.rs。
INK = "#08080c"
WHITE = "#f4f2ea"
SHADE = "#c4c0b8"
PIP = "#201c28"
GOLD = "#f0b820"
GOLD_HI = "#ffec80"
PLAYERS = ["#d83a3a", "#3a6ee0", "#34a448", "#e0b028"]
DARK = ["#4a161e", "#162452", "#123a1c", "#4c3a0e"]

# 小图标，和 art.rs 的 icons() 一样：星、心、方块、草花。
ICONS = [
    ["...x....", "...x....", "..xxx...", "xxxxxxx.", ".xxxxx..", "..xxx...", ".xx.xx..", ".x...x.."],
    [".xx.xx..", "xxxxxxx.", "xxxxxxx.", "xxxxxxx.", ".xxxxx..", "..xxx...", "...x....", "........"],
    ["...x....", "..xxx...", ".xxxxx..", "xxxxxxx.", ".xxxxx..", "..xxx...", "...x....", "........"],
    ["..xxx...", "..xxx...", "xx.x.xx.", "xxxxxxx.", "xx.x.xx.", "...x....", "..xxx...", "........"],
]
CROWN = ["y...y...y", "gg.ggg.gg", "ggggggggg", "gyggyggyg", "ggggggggg"]
PIPS = {1: [(1, 1)], 2: [(0, 0), (2, 2)], 3: [(0, 0), (1, 1), (2, 2)], 4: [(0, 0), (2, 0), (0, 2), (2, 2)],
        5: [(0, 0), (2, 0), (1, 1), (0, 2), (2, 2)], 6: [(0, 0), (2, 0), (0, 1), (2, 1), (0, 2), (2, 2)]}


def die(c, x, y, s, face):
    """和 app 里 draw_die 一样的骰子：黑边、白面、下边一条暗的、点。"""
    c.rect(x + 2, y + 3, s, s, "#0a2414")
    c.rect(x, y + 1, s, s - 2, INK)
    c.rect(x + 1, y, s - 2, s, INK)
    c.rect(x + 1, y + 1, s - 2, s - 2, WHITE)
    c.rect(x + 2, y + s - 4, s - 4, 2, SHADE)
    dot = max(2, s // 7)
    for cx, cy in PIPS[face]:
        c.rect(x + s * (cx + 1) // 4 - dot // 2, y + s * (cy + 1) // 4 - dot // 2 - 1, dot, dot,
               PLAYERS[0] if face == 1 else PIP)


def tally(c, x, y, n, col):
    """正字划记：四竖一斜，一组 5 个。"""
    for g in range(0, n, 5):
        k = min(5, n - g)
        gx = x + (g // 5) * 11
        for i in range(min(k, 4)):
            c.rect(gx + i * 2, y, 1, 6, col)
        if k == 5:
            for i in range(9):
                c.set(gx - 1 + i, y + 5 - i // 2, col)


def scorepad():
    c = Canvas()
    # 牌桌的绒布：中间亮、四边暗。
    for y in range(H):
        for x in range(W):
            d = ((x - 44) ** 2 / 2600 + (y - 38) ** 2 / 1800) * 16
            c.set(x, y, "#1c6a3c" if d + BAYER[y % 4][x % 4] < 14 else "#14502e" if d < 30 else "#0e3a22")

    # 记分本：微微的影子、奶白的纸、上面红色的夹子。
    px, py, pw, ph = 8, 10, 50, 64
    c.rect(px + 3, py + 3, pw, ph, "#0a2414")
    c.rect(px - 1, py - 1, pw + 2, ph + 2, INK)
    c.rect(px, py, pw, ph, "#f6eed8")
    for yy in range(py + 14, py + ph - 2, 12):
        c.rect(px + 2, yy + 10, pw - 4, 1, "#d8ccb0")
    c.rect(px + 12, py, 1, ph, "#e8a0a0")
    c.rect(px + 14, py - 4, 22, 7, INK)
    c.rect(px + 15, py - 3, 20, 5, "#9a9aa8")
    c.rect(px + 16, py - 3, 18, 1, "#d8d8e0")
    # 四个人一行一个：图标、划记。第二个人最多。
    marks = [7, 10, 4, 9]
    for i, (col, n) in enumerate(zip(PLAYERS, marks)):
        ry = py + 8 + i * 12 + 6
        c.blit(art(ICONS[i], {"x": col}), px + 3, ry - 1)
        tally(c, px + 16, ry, n, DARK[i])
    # 领先的那一行：金色的王冠压在纸边。
    cr = scale(art(CROWN, {"g": GOLD, "y": GOLD_HI}), 2)
    shadow = [[INK if v else None for v in row] for row in cr]
    c.blit(shadow, px + pw - 11, py + 21)
    c.blit(cr, px + pw - 12, py + 20)

    # 右边两个骰子，一个刚停，一个还斜着。
    die(c, 70, 12, 22, 5)
    die(c, 72, 44, 18, 3)
    # 几颗筹码：四个人的颜色。
    for i, (x, y) in enumerate([(62, 70), (73, 72), (84, 68), (62, 4)]):
        col = PLAYERS[i]
        c.rect(x, y + 1, 7, 3, INK)
        c.rect(x + 1, y, 5, 4, INK)
        c.rect(x + 1, y + 1, 5, 2, col)
        c.set(x + 3, y + 1, WHITE)
    return c


def main():
    out = ROOT / "examples" / "scorepad" / "assets" / "cover.png"
    scorepad().save(out)
    print(f"{out.relative_to(ROOT)}：{W}×{H}")


if __name__ == "__main__":
    main()
