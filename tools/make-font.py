#!/usr/bin/env python3
"""给 vibegame 库烘自带的点阵字（SDK 的维护者用，写游戏的人不用跑）。

字体是缝合像素字体 12px 等宽（Fusion Pixel Font，TakWolf，OFL-1.1）：
  https://github.com/TakWolf/fusion-pixel-font/releases
  下 fusion-pixel-font-12px-monospaced-bdf-v<版本>.zip。字体文件不进仓库，许可证在 vibegame/fonts/fusion-pixel-font/。

  python3 tools/make-font.py <字体 .zip 或 .bdf>

带的字：可打印的 ASCII、GB2312 里的全部字（6763 个汉字和常用的符号）。写 vibegame/src/font.rs。
"""
import pathlib, re, sys, zipfile

if len(sys.argv) != 2:
    sys.exit(__doc__)
src = pathlib.Path(sys.argv[1])
if src.suffix == ".zip":
    with zipfile.ZipFile(src) as z:
        name = next(n for n in z.namelist() if n.endswith("zh_hans.bdf"))
        bdf = z.read(name).decode("utf-8")
        font_name = name
else:
    bdf = src.read_text(encoding="utf-8")
    font_name = src.name


want = {chr(c) for c in range(0x20, 0x7F)}
for hi in range(0xA1, 0xF8):
    for lo in range(0xA1, 0xFF):
        try:
            want.add(bytes([hi, lo]).decode("gb2312"))
        except UnicodeDecodeError:
            pass
# 读 BDF：FONTBOUNDINGBOX 定 12 行的框，基线在框底往上 descent 行。
m = re.search(r"^FONTBOUNDINGBOX (\d+) (\d+) (-?\d+) (-?\d+)", bdf, re.M)
box_h, descent = int(m.group(2)), -int(m.group(4))
if box_h != 12:
    sys.exit(f"要 12px 的字体，这个是 {box_h}px")
glyphs = {}
for chunk in bdf.split("STARTCHAR")[1:]:
    enc = int(re.search(r"^ENCODING (-?\d+)", chunk, re.M).group(1))
    if enc < 0 or chr(enc) not in want:
        continue
    dw = int(re.search(r"^DWIDTH (\d+)", chunk, re.M).group(1))
    w, h, xo, yo = map(int, re.search(r"^BBX (-?\d+) (-?\d+) (-?\d+) (-?\d+)", chunk, re.M).groups())
    bits = chunk.split("BITMAP")[1].split("ENDCHAR")[0].split()
    rows = [0] * 12
    top = box_h - descent - (yo + h)
    for r, hexrow in enumerate(bits):
        y = top + r
        if not 0 <= y < 12:
            continue
        v = int(hexrow, 16)
        nbits = len(hexrow) * 4
        for x in range(w):
            if v >> (nbits - 1 - x) & 1 and 0 <= xo + x < 16:
                rows[y] |= 0x8000 >> (xo + x)
    glyphs[chr(enc)] = (dw, rows)

missing = sorted(want - set(glyphs))
if missing:
    print("字体里没有：" + "".join(missing), file=sys.stderr)


def lit(ch):
    return {"'": "\\'", "\\": "\\\\"}.get(ch, ch)


lines = [
    "// 由 tools/make-font.py 生成，不要手改。",
    f"// 字形来自缝合像素字体 12px 等宽（{font_name}），https://github.com/TakWolf/fusion-pixel-font",
    "// Copyright (c) 2022, TakWolf。SIL Open Font License 1.1，全文和它带的上游许可证在 vibegame/fonts/fusion-pixel-font/。",
    "use qapp::full::{Font, Glyph};",
    "",
    "/// 自带的点阵字：12 像素高，ASCII 和 GB2312 的全部字。",
    "#[rustfmt::skip]",
    "pub static FONT: Font = Font { glyphs: &[",
]
for ch in sorted(glyphs):
    dw, rows = glyphs[ch]
    lines.append(f"    Glyph {{ ch: '{lit(ch)}', w: {dw}, rows: [{', '.join(f'0x{r:04x}' for r in rows)}] }},")
lines.append("] };")
out = pathlib.Path(__file__).resolve().parent.parent / "vibegame" / "src" / "font.rs"
out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"{out}：{len(glyphs)} 个字")
