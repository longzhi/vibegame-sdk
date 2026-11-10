#!/usr/bin/env python3
"""画游戏的封面：96×80 的像素画，放在 examples/<游戏>/assets/cover.png。

封面是专门画的，不是游戏截图：一张图说清这款游戏是什么、是什么气氛。
像素一个一个用代码画，颜色和精灵照游戏里的，不从网上拿图。

    python3 tools/game-covers.py            # 画全部
    python3 tools/game-covers.py ironcamp   # 只画一款
"""

import math
import sys
from pathlib import Path

from PIL import Image

W, H = 96, 80
ROOT = Path(__file__).resolve().parent.parent

# 4×4 的有序抖动表：两种颜色过渡时一格一格交错，像那个年代的画法。
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

    def gradient(self, y0, y1, colors):
        """y0 到 y1 一段竖着的渐变，几种颜色之间用抖动过渡。"""
        n = len(colors) - 1
        for y in range(y0, y1):
            t = (y - y0) / max(1, y1 - y0 - 1) * n
            i = min(int(t), n - 1)
            f = t - i
            for x in range(W):
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


def rotate(spr):
    """顺时针转 90 度。"""
    h, w = len(spr), len(spr[0])
    return [[spr[h - 1 - x][y] for x in range(h)] for y in range(w)]


def flip(spr):
    return [list(reversed(r)) for r in spr]


def scale(spr, k):
    return [[c for c in r for _ in range(k)] for r in spr for _ in range(k)]


def side_tank(body, hi, lo, accent, gun="#70707a", gun_hi="#b0b0b8"):
    """侧面看的坦克，朝右，44×22：履带、负重轮、车身、炮塔、炮管，外面一圈黑边。"""
    w, h = 46, 24
    s = [[None] * w for _ in range(h)]

    def put(x, y, col):
        if 0 <= x < w and 0 <= y < h:
            s[y][x] = col

    # 炮管：从炮塔伸到右边，头上一节粗的。
    for x in range(26, 44):
        put(x, 6, gun_hi)
        put(x, 7, gun)
        put(x, 8, "#45454c")
    for y in range(5, 10):
        put(43, y, gun)
        put(42, y, gun_hi if y < 7 else gun)
    # 炮塔：上面圆角，左上亮，右下暗。
    for y in range(3, 11):
        for x in range(10, 28):
            if (y == 3 and (x < 13 or x > 24)) or (y == 4 and (x < 11 or x > 26)):
                continue
            col = body
            if y <= 4 or x <= 11:
                col = hi
            if y >= 9 or x >= 26:
                col = lo
            put(x, y, col)
    for x in range(14, 19):
        put(x, 2, "#45454c")  # 舱盖
    put(21, 6, "#08080c")
    put(22, 6, "#08080c")  # 观察窗
    # 车身：前后斜着。
    for y in range(11, 17):
        for x in range(2 + (16 - y) // 2, 42 - (y - 11)):
            col = body
            if y == 11:
                col = hi
            if y == 16:
                col = lo
            if y == 14:
                col = accent
            put(x, y, col)
    # 履带和轮子。
    for y in range(16, 23):
        for x in range(3, 41):
            if (y in (16, 22)) and (x < 5 or x > 38):
                continue
            put(x, y, "#2c2c30" if y not in (16,) else "#707074")
    for x in range(4, 40, 2):
        put(x, 22, "#707074")
    for cx in (8, 15, 22, 29, 35):
        for yy in range(-2, 3):
            for xx in range(-2, 3):
                if xx * xx + yy * yy <= 5:
                    put(cx + xx, 19 + yy, "#5a5a62")
        put(cx, 19, "#b0b0b8")
    # 外面一圈黑边。
    edge = [[None] * w for _ in range(h)]
    for y in range(h):
        for x in range(w):
            if s[y][x] is None and any(0 <= x + dx < w and 0 <= y + dy < h and s[y + dy][x + dx] is not None for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                edge[y][x] = "#08080c"
    return [[s[y][x] if s[y][x] is not None else edge[y][x] for x in range(w)] for y in range(h)]


# ---------------------------------------------------------------- 铁壳守营

TANK_PLAYER = [
    ".......gg.......",
    ".......gg.......",
    "tt.....gg.....tt",
    "TTbbbbbggbbbbbTT",
    "tthhhbbggbbdddtt",
    "TThbbckkkkcbbdTT",
    "tthbckhhhhkcbdtt",
    "TThbkhhhhhdkbdTT",
    "tthbkhhhhhdkbdtt",
    "TThbkhhhhddkbdTT",
    "tthbckddddkcbdtt",
    "TThbbckkkkcbbdTT",
    "tthbbbbbbbbbbdtt",
    "TTddddddddddddTT",
    "tt............tt",
    "TT............TT",
]
TANK_ENEMY = [
    ".......gg.......",
    ".......gg.......",
    "tttt...gg...tttt",
    "TTTTbbbggbbbTTTT",
    "ttttbhhggddbtttt",
    "TTTTkkkkkkkkTTTT",
    "ttttbhckkcdbtttt",
    "TTTTbhkhhkdbTTTT",
    "ttttbhkhhkdbtttt",
    "TTTTbhckkcdbTTTT",
    "ttttkkkkkkkktttt",
    "TTTTbhhhhddbTTTT",
    "ttttbddddddbtttt",
    "TTTTbbbbbbbbTTTT",
    "tttt........tttt",
    "TTTT........TTTT",
]
BOOM = [
    "...y........y...",
    ".y...yyyyyy...y.",
    "....yoooooooy...",
    "..yyoffffffooyy.",
    ".yooffwwwwffffoy",
    ".yoffwwwwwwwffoy",
    "yoofwwwwwwwwwfooy",
    "yofwwwwwwwwwwwfoy",
    "yofwwwwwwwwwwwfoy",
    "yoofwwwwwwwwwfooy",
    ".yoffwwwwwwwffoy",
    ".yooffwwwwffffoy",
    "..yyoffffffooyy.",
    "....yoooooooy...",
    ".y...yyyyyy...y.",
    "...y........y...",
]
TENT = [
    "......cc......",
    ".....cccC.....",
    "....ccccCc....",
    "...cccc.cCc...",
    "..cccc...cCc..",
    "..ccc.kkk.cCc.",
    ".ccc..kkk..cCc",
    "cccccckkkcccCC",
]


def ironcamp():
    c = Canvas()
    # 黄昏的天：上面深紫，往下到地平线变成橙、金。
    c.gradient(0, 46, ["#1c1030", "#3a1a4c", "#7a2c5c", "#c44c4c", "#f08c3c", "#ffc860"])
    # 太阳：半个，横着切几道（那个年代的海报画法）。
    for y in range(28, 46):
        for x in range(54, 90):
            d = math.hypot(x - 72, y - 46)
            if d <= 15 and not (y > 36 and (y - 36) % 3 == 0):
                c.set(x, y, "#ffe08a" if d < 12 else "#ffb860")
    # 远山两层。
    for x in range(W):
        h1 = 40 + int(3 * math.sin(x / 7.0) + 2 * math.sin(x / 3.1 + 1))
        for y in range(h1, 50):
            c.set(x, y, "#5a2a50")
        h2 = 44 + int(2 * math.sin(x / 5.0 + 2) + math.sin(x / 2.3))
        for y in range(h2, 50):
            c.set(x, y, "#3a1c3c")
    # 地面。
    c.gradient(50, 80, ["#3a2c22", "#2a2018", "#1a140f"])
    # 营地：帐篷、旗杆、风里的红旗（在墙后面，左边）。
    tent = art(TENT, {"c": "#e8d49c", "C": "#a88c58", "k": "#141010"})
    c.blit(tent, 2, 42)
    for y in range(12, 43):
        c.set(9, y, "#2a1a14")
    c.set(9, 11, "#ffd840")
    for x in range(14):
        wave = int(round(1.6 * math.sin(x / 2.2)))
        for y in range(10):
            yy = 13 + y + wave
            col = "#d83030" if y not in (0, 9) else "#801818"
            if y == 2 and 2 < x < 11:
                col = "#ff7868"
            c.set(10 + x, yy, col)
    # 矮砖墙横在营地前面，中间被炸开一个口子。
    brick = art(
        ["hbbmhbbb", "bbbmbbbb", "bddmbddd", "mmmmmmmm", "bbhbbbmh", "bbbbbbmb", "dddbddmd", "mmmmmmmm"],
        {"b": "#a84c24", "h": "#d88048", "d": "#5c2c18", "m": "#5c2c18"},
    )
    gap = set()
    for y in range(50, 66):
        for x in range(54, 74):
            edge = 2.5 * math.sin(y * 1.7) + 1.5 * math.sin(y * 0.9 + 1)
            if 57 + edge < x < 71 - edge and y > 50 + abs(x - 64) // 3:
                gap.add((x, y))
    for by in range(50, 66, 8):
        for bx in range(0, W, 8):
            for yy in range(8):
                for xx in range(8):
                    x, y = bx + xx, by + yy
                    if y < 64 and (x, y) not in gap:
                        c.set(x, y, brick[yy][xx])
    for x in range(W):
        if (x, 50) not in gap:
            c.set(x, 50, "#e8a070")
        c.set(x, 64, "#100c08")
    # 碎砖块。
    for x, y in [(52, 67), (56, 70), (62, 66), (70, 69), (75, 66), (48, 74), (66, 75)]:
        c.rect(x, y, 2, 2, "#a84c24")
        c.set(x, y, "#d88048")
    # 敌车从口子里冲进来，朝左，正被打中炸开。
    enemy = flip(side_tank("#8c5cc8", "#c49cf0", "#50307c", "#ffd840"))
    c.blit(enemy, 58, 50)
    # 我方坦克在前面，朝右开火。
    me = side_tank("#5c9c3c", "#a4d468", "#34602e", "#f09028")
    c.blit(me, 2, 54)
    # 炮口的火光。
    for dx, dy, col in [(0, 0, "#ffffff"), (1, 0, "#ffffff"), (2, 0, "#ffe060"), (3, 0, "#ff9028"), (1, -1, "#ffe060"), (1, 1, "#ffe060"), (2, -2, "#ff9028"), (2, 2, "#ff9028"), (0, -1, "#ffffff"), (0, 1, "#ffffff")]:
        c.set(47 + dx, 61 + dy, col)
    # 炮弹拖着光飞过去。
    for x in range(51, 66):
        t = (x - 51) / 15
        col = "#ff9028" if t < 0.4 else ("#ffe060" if t < 0.85 else "#ffffff")
        c.set(x, 61 - (x - 51) // 6, col)
    boom = art(BOOM, {"w": "#ffffff", "y": "#ffd840", "o": "#f09028", "f": "#ff6c20"})
    c.blit(boom, 64, 48)
    # 烟：灰色的团往上飘。
    for x, y, r in [(78, 42, 4), (83, 35, 3), (76, 33, 2), (88, 30, 2)]:
        for yy in range(-r, r + 1):
            for xx in range(-r, r + 1):
                if xx * xx + yy * yy <= r * r:
                    c.set(x + xx, y + yy, "#9a8a94" if yy < 0 else "#6a5c6c")
    return c


# ---------------------------------------------------------------- 从游戏的 Rust 源码里读字符画和调色板

import re


def rust_consts(path):
    """读 art.rs 里的字符画常量：名字 → 行的列表；二维的数组是几张的列表。"""
    src = (ROOT / path).read_text()
    out = {}
    for m in re.finditer(r"const (\w+): \[[^=]*= ", src):
        i = m.end()
        if src[i] != "[":
            continue
        depth, j = 0, i
        while True:
            if src[j] == "[":
                depth += 1
            elif src[j] == "]":
                depth -= 1
                if depth == 0:
                    break
            elif src[j] == '"':
                j = src.index('"', j + 1)
            j += 1
        body = src[i + 1 : j]
        if "'" in body:
            out[m.group(1)] = dict(re.findall(r"\('(.)', (\w+)\)", body))
        elif body.lstrip().startswith("//") or body.lstrip().startswith("["):
            out[m.group(1)] = [re.findall(r'"([^"]*)"', g) for g in re.findall(r"\[([^\[\]]*)\]", body)]
        else:
            out[m.group(1)] = re.findall(r'"([^"]*)"', body)
    return out


def rust_palette(path):
    """读 art.rs 的调色板：常量名 → 颜色。"""
    src = (ROOT / path).read_text()
    idx = {m.group(1): int(m.group(2)) for m in re.finditer(r"pub const (\w+): u8 = (\d+);", src)}
    pal = {}
    for m in re.finditer(r"\((\w+), \[0x(\w\w), 0x(\w\w), 0x(\w\w)\]\)", src):
        pal[m.group(1)] = (int(m.group(2), 16), int(m.group(3), 16), int(m.group(4), 16))
    return idx, pal


def keyed(rows, keys, pal):
    """字符画加上「字符 → 调色板名」，变成精灵。"""
    return art(rows, {k: pal[v] for k, v in keys.items()})


# ---------------------------------------------------------------- 疾风邮差


def courier():
    a = rust_consts("examples/courier/src/art.rs")
    _, pal = rust_palette("examples/courier/src/art.rs")
    c = Canvas()
    # 清晨：上面浅蓝，往下到地平线是暖黄。
    c.gradient(0, 58, ["#5aa4ec", "#7cbcf4", "#a8d4f8", "#f8e0a0", "#ffd080"])
    # 太阳刚从屋顶后面出来。
    for y in range(30, 58):
        for x in range(0, 40):
            d = math.hypot(x - 18, y - 52)
            if d < 13:
                c.set(x, y, "#fff4c0" if d < 10 else "#ffe08a")
    # 远处一排房子：浅色的墙，尖屋顶，有的带烟囱。
    houses = [(-4, 18, 40, "#e8c8a0", "#b85848"), (13, 16, 36, "#d8b0a0", "#8c4c58"), (28, 20, 42, "#f0d8b0", "#c86848"),
              (47, 15, 34, "#d0b8c8", "#7c4c68"), (61, 18, 39, "#e8c8a0", "#b85848"), (78, 20, 36, "#d8b0a0", "#a05040")]
    for hx, hw, top, wall, roof in houses:
        mid = hx + hw // 2
        for x in range(hx, hx + hw):
            peak = top - (hw // 2 - abs(x - mid)) // 2
            for y in range(peak, top + 1):
                c.set(x, y, roof)
            c.set(x, peak, "#ffe0b0")
            for y in range(top + 1, 66):
                c.set(x, y, wall)
            if x in (hx, hx + hw - 1):
                for y in range(top + 1, 66):
                    c.set(x, y, "#806060")
        # 窗：两排，亮着暖光。
        for wx in range(hx + 3, hx + hw - 4, 6):
            for wy in (top + 5, top + 14):
                if wy + 4 < 66:
                    c.rect(wx, wy, 3, 4, "#ffe890")
                    c.rect(wx, wy + 4, 3, 1, "#806060")
        if hw >= 18:
            c.rect(mid + 3, top - 10, 3, 7, "#705050")
    # 人行道。
    for y in range(66, 80):
        for x in range(W):
            col = "#b4b4bc"
            if y == 66:
                col = "#e0e0e8"
            elif y == 69:
                col = "#5c586c"
            elif y > 69:
                col = "#b88050" if (x // 8 + (y > 74)) % 2 else "#a87040"
            c.set(x, y, col)
    # 包裹箱：右上，刚被顶过，一张邮票跳出来。
    mail = keyed(a["T_MAIL"][0], a["TILE_KEYS"], {**pal, "WOOD_HI": pal["WHITE"]})
    c.blit(mail, 70, 8)
    stamp = keyed(a["STAMP"][0], a["STAMP_KEYS"], pal)
    c.blit(stamp, 72, -4 + 4)
    # 风：几道白线从左边拖过来。
    for y, x0, n in [(22, 0, 26), (30, 4, 22), (38, 0, 30), (46, 6, 18), (54, 2, 24)]:
        for x in range(x0, x0 + n):
            if (x + y) % 9 != 0:
                c.set(x, y, "#ffffff" if x > x0 + n // 3 else "#d8ecff")
    # 信从邮包里飞出来。
    for x, y in [(12, 16), (6, 34), (20, 48), (2, 24)]:
        c.rect(x, y, 7, 5, "#ffffff")
        c.set(x, y, "#9c9ca8")
        c.set(x + 6, y, "#9c9ca8")
        for i in range(4):
            c.set(x + i, y + i // 2, "#9c9ca8")
            c.set(x + 6 - i, y + i // 2, "#9c9ca8")
    # 小邮差：大个子跳起来那张，放大两倍。
    hk = a["HERO_KEYS"]
    rows = a["HEAD"] + a["BIG_TORSO"] + a["BIG_LEGS"][4]
    hero = keyed(rows, hk, pal)
    # 黑边，压在花背景上看得清。
    big = scale(hero, 2)
    h, w = len(big), len(big[0])
    for y in range(h):
        for x in range(w):
            if big[y][x] is None and any(0 <= y + dy < h and 0 <= x + dx < w and big[y + dy][x + dx] is not None for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                c.set(28 + x, 4 + y, "#101018")
    c.blit(big, 28, 4)
    # 脚下的影子：人在半空。
    for x in range(36, 54):
        c.set(x, 67, "#8c8c98")
    for x in range(39, 51):
        c.set(x, 68, "#8c8c98")
    # 地上被踩扁的发条兵。
    flat = keyed(a["WALKER_FLAT"], a["WALKER_KEYS"], pal)
    c.blit(flat, 72, 60)
    return c


# ---------------------------------------------------------------- 雷霆突击


def thunder():
    a = rust_consts("examples/thunder/src/art.rs")
    _, pal = rust_palette("examples/thunder/src/art.rs")
    c = Canvas()
    # 夜里的火光：上面深紫，往下烧成红、橙。
    c.gradient(0, 64, ["#140c28", "#2c1438", "#5c1c34", "#a8302c", "#e86828", "#ffb040"])
    # 后面一团大火：圆的，里亮外暗，边上抖开。
    for y in range(0, 70):
        for x in range(W):
            d = math.hypot((x - 50) * 0.9, (y - 40) * 1.1)
            n = BAYER[y % 4][x % 4] / 16 * 4
            if d + n < 12:
                c.set(x, y, "#fff8c0")
            elif d + n < 19:
                c.set(x, y, "#ffd840")
            elif d + n < 27:
                c.set(x, y, "#ff9020")
            elif d + n < 34:
                c.set(x, y, "#d03820")
    # 飞起来的碎片。
    for x, y in [(20, 14), (78, 10), (30, 6), (70, 24), (14, 30), (86, 34)]:
        c.rect(x, y, 2, 2, "#ffd840")
        c.set(x + 2, y + 1, "#ff9020")
    # 林子的黑影：一排棕榈。
    for x in range(W):
        h = 52 + int(4 * math.sin(x / 5)) + (x * 7919 % 5)
        for y in range(h, 66):
            c.set(x, y, "#100810")
    for tx, lean in [(6, 1), (88, -1)]:
        for y in range(18, 64):
            dx = (64 - y) ** 2 // 260 * lean
            c.rect(tx + dx, y, 3, 1, "#100810")
        top = tx + (46 ** 2 // 260) * lean
        for ang in [200, 230, 260, 290, 320, 345, 20]:
            for k in range(14):
                px_ = top + 1 + int(k * math.cos(math.radians(ang)))
                py_ = 18 + int(k * math.sin(math.radians(ang))) + k * k // 24
                c.rect(px_, py_, 2, 2, "#100810")
    # 地面。
    for y in range(66, 80):
        for x in range(W):
            c.set(x, y, "#2c1c18" if (x // 6 + y) % 5 else "#3c2820")
    c.rect(0, 66, W, 1, "#583828")
    # 两个队员背靠背：雷朝右上打，霆朝左打。放大两倍。
    def hero(body, gun):
        rows = ["." * 24 for _ in range(24)]
        def put(at, src):
            for i, r in enumerate(src):
                old = rows[at + i].ljust(24, ".")
                rows[at + i] = "".join(r[j] if j < len(r) and r[j] != "." else old[j] for j in range(24))
        put(0, a["HEAD"])
        put(13, a["HIPS"])
        put(16, a["LEGS"][0])
        put(7, [r for r in body if r])
        if gun:
            put(0, gun)
        return rows
    keys = a["HERO_KEYS"]
    ting = {"BAND": "BLUE", "BAND_HI": "BLUE_HI", "VEST": "SAND", "VEST_LO": "SAND_LO", "PANTS": "GREEN_LO", "PANTS_LO": "GRAY_LO"}
    lei = scale(keyed(hero(a["BODY_DIAG_UP"], a["GUN_DIAG_UP"]), keys, pal), 2)
    ting_pal = {k: pal[ting.get(k, k)] for k in pal}
    tg = scale(flip(keyed(hero(a["BODY_FWD"], None), keys, ting_pal)), 2)
    def outline(spr, ox, oy):
        h, w = len(spr), len(spr[0])
        for y in range(h):
            for x in range(w):
                if spr[y][x] is None and any(0 <= y + dy < h and 0 <= x + dx < w and spr[y + dy][x + dx] is not None for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    c.set(ox + x, oy + y, "#000000")
    # 两道子弹：雷的往右上，霆的往左。
    for i in range(3):
        c.rect(89 + i * 3, 29 - i * 3, 2, 2, "#fff8c0")
    for x in range(0, 8, 4):
        c.rect(x, 50, 3, 2, "#fff8c0")
    outline(tg, 4, 32)
    c.blit(tg, 4, 32)
    outline(lei, 40, 32)
    c.blit(lei, 40, 32)
    # 枪口的火。
    for x, y in [(84, 34), (2, 48)]:
        c.rect(x, y + 1, 5, 1, "#fff8c0")
        c.rect(x + 2, y - 1, 1, 5, "#fff8c0")
        c.set(x + 2, y + 1, "#ffffff")
    return c


# ---------------------------------------------------------------- 烈空战机


def skyfire():
    c = Canvas()
    # 从上往下看的海：远处深，近处亮，抖动过渡。
    c.gradient(0, 80, ["#0c1030", "#182458", "#1c4c90", "#2860a8"])
    # 浪花：短短的亮线，越往下越密。
    for i in range(40):
        x, y = (i * 37) % W, (i * 53) % 80
        if y > 20 or i % 3 == 0:
            c.rect(x, y, 3, 1, "#4888c8")
    # 左下一块岸：沙一圈，里面是草。
    for y in range(44, 80):
        edge = 20 - (y - 44) // 3 + int(2 * math.sin(y / 3))
        for x in range(0, max(0, 36 - edge)):
            c.set(x, y, "#48a048" if x < 30 - edge else "#e8c880")
    for x, y in [(4, 70), (10, 62), (6, 76), (14, 72)]:
        c.rect(x, y, 2, 2, "#2c7038")
    # 上面压着一艘大战舰的船头：灰的，两座炮塔。
    for k in range(18):
        c.rect(22 + k * 2, 18 + k, 52 - k * 4, 1, "#404860")
    c.rect(18, 0, 60, 18, "#404860")
    c.rect(20, 0, 56, 16, "#7888a0")
    c.rect(22, 2, 52, 1, "#b8c8d8")
    for tx in (26, 58):
        c.rect(tx, 5, 12, 10, "#0c0814")
        c.rect(tx + 1, 6, 10, 8, "#7888a0")
        c.rect(tx + 5, 14, 2, 4, "#0c0814")
    c.rect(43, 2, 10, 13, "#0c0814")
    c.rect(44, 3, 8, 11, "#ff7058")
    c.rect(46, 5, 4, 7, "#fff4a0")
    # 敌人打下来的粉色子弹。
    for x, y in [(30, 30), (66, 32), (40, 40), (58, 44), (20, 38), (76, 40)]:
        c.rect(x - 1, y, 3, 1, "#ff48a0")
        c.rect(x, y - 1, 1, 3, "#ff48a0")
        c.set(x, y, "#ffb8e0")
    # 右边炸开一架小飞机。
    for y in range(20, 40):
        for x in range(70, 96):
            d = math.hypot(x - 82, y - 30) + BAYER[y % 4][x % 4] / 16 * 3
            if d < 4:
                c.set(x, y, "#ffffff")
            elif d < 7:
                c.set(x, y, "#fff4a0")
            elif d < 9:
                c.set(x, y, "#ff9420")
            elif d < 10:
                c.set(x, y, "#d83030")
    # 主角「烈风」放大两倍，往上打两排子弹。
    # 字符画和游戏里 art.rs 的 hero 一样，尾焰多一行。
    hero = [
        ".......ww.......",
        "......wbbw......",
        "......wbbw......",
        "......wrrw......",
        ".....wrrrrw.....",
        "....wrrRRrrw....",
        "..wwrrrRRrrrww..",
        ".wrrrrrRRrrrrrw.",
        "wrrwwrrRRrrwwrrw",
        "www..wrRRrw..www",
        "......wrrw......",
        ".....wrrrrw.....",
        "....wrr..rrw....",
        "....www..www....",
        "......oyyo......",
        ".......yy.......",
    ]
    keys = {"w": "#f4f0e6", "r": "#d83030", "R": "#801828", "b": "#8cd0ff", "o": "#ff9420", "y": "#ffd23f"}
    spr = scale(art(hero, keys), 2)
    hx, hy = 32, 46
    for y in range(len(spr)):
        for x in range(len(spr[0])):
            if spr[y][x] is None and any(
                0 <= y + dy < len(spr) and 0 <= x + dx < len(spr[0]) and spr[y + dy][x + dx] is not None
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
            ):
                c.set(hx + x, hy + y, "#0c0814")
    c.blit(spr, hx, hy)
    for y in range(22, 44, 7):
        for dx in (10, 20):
            c.rect(hx + dx, y, 2, 4, "#ffd23f")
            c.set(hx + dx, y, "#fff4a0")
    return c


COVERS = {"ironcamp": ironcamp, "courier": courier, "thunder": thunder, "skyfire": skyfire}


def main():
    names = sys.argv[1:] or list(COVERS)
    for n in names:
        out = ROOT / "examples" / n / "assets" / "cover.png"
        COVERS[n]().save(out)
        print(f"{out.relative_to(ROOT)}：{W}×{H}")


if __name__ == "__main__":
    main()
