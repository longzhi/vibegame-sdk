//! 调色板和精灵：每个人的颜色、小图标、王冠、骰子。

use vibeplay::full::{Frame, Sprite};

pub const BG: u8 = 0;
pub const INK: u8 = 1;
pub const WHITE: u8 = 2;
pub const GRAY: u8 = 3;
pub const DIM: u8 = 4;
pub const PANEL: u8 = 5;
pub const GOLD: u8 = 6;
pub const GOLD_HI: u8 = 7;
pub const PIP: u8 = 8;
pub const SHADE: u8 = 9;
/// 六个人的颜色：主色从 10 起，亮的从 16 起，暗的从 22 起。
pub const BASE: u8 = 10;
pub const LIGHT: u8 = 16;
pub const DARK: u8 = 22;

pub const MAX_PLAYERS: usize = 6;

const PLAYERS: [([u8; 3], [u8; 3], [u8; 3]); MAX_PLAYERS] = [
    ([0xd8, 0x3a, 0x3a], [0xff, 0x9a, 0x8a], [0x4a, 0x16, 0x1e]),
    ([0x3a, 0x6e, 0xe0], [0x9c, 0xc4, 0xff], [0x16, 0x24, 0x52]),
    ([0x34, 0xa4, 0x48], [0x9c, 0xe8, 0x8a], [0x12, 0x3a, 0x1c]),
    ([0xe0, 0xb0, 0x28], [0xff, 0xec, 0x98], [0x4c, 0x3a, 0x0e]),
    ([0x9a, 0x50, 0xd0], [0xd8, 0xb0, 0xff], [0x34, 0x1a, 0x4c]),
    ([0xf0, 0x7a, 0x28], [0xff, 0xc4, 0x90], [0x52, 0x26, 0x0e]),
];

pub fn palette(f: &mut Frame) {
    for (i, rgb) in [
        (BG, [0x14, 0x14, 0x1c]),
        (INK, [0x08, 0x08, 0x0c]),
        (WHITE, [0xf4, 0xf2, 0xea]),
        (GRAY, [0x9a, 0x98, 0xa4]),
        (DIM, [0x5a, 0x58, 0x66]),
        (PANEL, [0x26, 0x26, 0x32]),
        (GOLD, [0xf0, 0xb8, 0x20]),
        (GOLD_HI, [0xff, 0xec, 0x80]),
        (PIP, [0x20, 0x1c, 0x28]),
        (SHADE, [0xc4, 0xc0, 0xb8]),
    ] {
        f.palette[i as usize] = rgb;
    }
    for (p, (base, light, dark)) in PLAYERS.iter().enumerate() {
        f.palette[BASE as usize + p] = *base;
        f.palette[LIGHT as usize + p] = *light;
        f.palette[DARK as usize + p] = *dark;
    }
}

/// 每个人的小图标，8×8：星、心、方块、草花、月亮、闪电。画成颜色格 1，用的时候 `recolor`。
pub fn icons() -> [Sprite; MAX_PLAYERS] {
    let rows: [[&str; 8]; MAX_PLAYERS] = [
        ["...x....", "...x....", "..xxx...", "xxxxxxx.", ".xxxxx..", "..xxx...", ".xx.xx..", ".x...x.."],
        [".xx.xx..", "xxxxxxx.", "xxxxxxx.", "xxxxxxx.", ".xxxxx..", "..xxx...", "...x....", "........"],
        ["...x....", "..xxx...", ".xxxxx..", "xxxxxxx.", ".xxxxx..", "..xxx...", "...x....", "........"],
        ["..xxx...", "..xxx...", "xx.x.xx.", "xxxxxxx.", "xx.x.xx.", "...x....", "..xxx...", "........"],
        ["..xxxx..", ".xxx....", "xxx.....", "xxx.....", "xxx.....", ".xxx....", "..xxxx..", "........"],
        ["....xx..", "...xx...", "..xx....", ".xxxxx..", "...xx...", "..xx....", ".xx.....", ".x......"],
    ];
    rows.map(|r| Sprite::art(&r, &[('x', 1)]))
}

/// 领先的人头上的小王冠。
pub fn crown() -> Sprite {
    Sprite::art(&["y...y...y", "gg.ggg.gg", "ggggggggg", "gyggyggyg", "ggggggggg"], &[('g', GOLD), ('y', GOLD_HI)])
}

/// 骰子一面上的点在哪：3×3 的格子里哪几格有点。
pub fn pips(face: u8) -> &'static [(i32, i32)] {
    match face {
        1 => &[(1, 1)],
        2 => &[(0, 0), (2, 2)],
        3 => &[(0, 0), (1, 1), (2, 2)],
        4 => &[(0, 0), (2, 0), (0, 2), (2, 2)],
        5 => &[(0, 0), (2, 0), (1, 1), (0, 2), (2, 2)],
        _ => &[(0, 0), (2, 0), (0, 1), (2, 1), (0, 2), (2, 2)],
    }
}

/// 画一个骰子：左上角 (x, y)，边长 `s` 像素。`tilt` 不是 0 时画成转着的样子（一边压扁）。
pub fn draw_die(f: &mut Frame, x: i32, y: i32, s: i32, face: u8, tilt: i32) {
    let w = s - tilt.abs() * 4;
    let x = x + (s - w) / 2;
    // 影子、黑边、白面、下边一条暗的。
    f.rect(x + 2, y + 3, w, s, INK);
    f.rect(x, y + 1, w, s - 2, INK);
    f.rect(x + 1, y, w - 2, s, INK);
    f.rect(x + 1, y + 1, w - 2, s - 2, WHITE);
    f.rect(x + 2, y + s - 4, w - 4, 2, SHADE);
    let cell = s / 4;
    let dot = (s / 7).max(2);
    for &(cx, cy) in pips(face) {
        let px = x + w * (cx + 1) / 4 - dot / 2;
        let py = y + cell * (cy + 1) - dot / 2 - 1;
        f.rect(px, py, dot, dot, if face == 1 { BASE } else { PIP });
    }
}
