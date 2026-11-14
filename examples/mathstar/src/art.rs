//! 调色板和精灵。精灵用字符画。

use vibeplay::full::Sprite;

pub const SKY: u8 = 0;
pub const INK: u8 = 1;
pub const WHITE: u8 = 2;
pub const YELLOW: u8 = 3;
pub const YELLOW_HI: u8 = 4;
pub const ORANGE: u8 = 5;
pub const PINK: u8 = 6;
pub const GREEN: u8 = 7;
pub const GREEN_HI: u8 = 8;
pub const RED: u8 = 9;
pub const RED_HI: u8 = 10;
pub const GRAY: u8 = 11;
pub const SKY_HI: u8 = 12;
pub const CYAN: u8 = 13;
pub const PURPLE: u8 = 14;
pub const HILL: u8 = 15;
pub const HILL_HI: u8 = 16;

pub const PALETTE: [(u8, [u8; 3]); 17] = [
    (SKY, [0x24, 0x30, 0x6c]),
    (INK, [0x10, 0x0c, 0x24]),
    (WHITE, [0xfa, 0xf6, 0xec]),
    (YELLOW, [0xff, 0xd2, 0x3f]),
    (YELLOW_HI, [0xff, 0xf4, 0xa0]),
    (ORANGE, [0xff, 0x94, 0x20]),
    (PINK, [0xff, 0x80, 0xb0]),
    (GREEN, [0x40, 0xc0, 0x68]),
    (GREEN_HI, [0xa8, 0xf0, 0xb8]),
    (RED, [0xe8, 0x48, 0x58]),
    (RED_HI, [0xff, 0xa0, 0xa0]),
    (GRAY, [0x78, 0x84, 0xb8]),
    (SKY_HI, [0x34, 0x44, 0x8c]),
    (CYAN, [0x70, 0xd8, 0xff]),
    (PURPLE, [0x9c, 0x6c, 0xe0]),
    (HILL, [0x2c, 0x8c, 0x70]),
    (HILL_HI, [0x48, 0xb0, 0x84]),
];

const STAR: [&str; 9] =
    ["....o....", "...oyo...", "...oyo...", "oooyyyooo", ".oyyhyyo.", "..oyyyo..", "..oyoyo..", ".oyo.oyo.", ".oo...oo."];

/// 一颗星：答对一道得一颗。
pub fn star() -> Sprite {
    Sprite::art(&STAR, &[('o', ORANGE), ('y', YELLOW), ('h', YELLOW_HI)])
}

/// 还没得的星：灰的空心。
pub fn star_empty() -> Sprite {
    Sprite::art(
        &["....g....", "...g.g...", "...g.g...", "gggg.gggg", ".g.....g.", "..g...g..", "..g.g.g..", ".g.g.g.g.", ".gg...gg."],
        &[('g', GRAY)],
    )
}

/// 主角「星星」：带笑脸的大星星。`blink` 时闭眼。
pub fn buddy(blink: bool) -> Sprite {
    let (e1, e2) = if blink { ("..oyyyyyyyyyo..", "..oyiiyyyiiyo..") } else { ("..oyyiyyyiyyo..", "..oyyiyyyiyyo..") };
    Sprite::art(
        &[
            ".......o.......",
            "......oyo......",
            "......oyo......",
            ".....oyhyo.....",
            "oooooyhyyyooooo",
            "oyyyyhyyyyyyyyo",
            ".oyyyyyyyyyyyo.",
            e1,
            e2,
            "..opyiyyyiypo..",
            "...oyyiiiyyo...",
            "...oyyyoyyyo...",
            "..oyyyo.oyyyo..",
            "..oyyo...oyyo..",
            ".oyyo.....oyyo.",
            ".ooo.......ooo.",
        ],
        &[('o', ORANGE), ('y', YELLOW), ('h', YELLOW_HI), ('i', INK), ('p', PINK)],
    )
}

/// 大数字和算式记号，5×7 的字符画。自带的字里 0 中间有一道斜线，孩子容易看成别的，所以自己画。
const DIGITS: [(char, [&str; 7]); 15] = [
    ('0', [".xxx.", "xx.xx", "xx.xx", "xx.xx", "xx.xx", "xx.xx", ".xxx."]),
    ('1', ["..xx.", ".xxx.", "..xx.", "..xx.", "..xx.", "..xx.", ".xxxx"]),
    ('2', [".xxx.", "xx.xx", "...xx", "..xx.", ".xx..", "xx...", "xxxxx"]),
    ('3', ["xxxx.", "...xx", "...xx", ".xxx.", "...xx", "...xx", "xxxx."]),
    ('4', ["...xx", "..xxx", ".x.xx", "x..xx", "xxxxx", "...xx", "...xx"]),
    ('5', ["xxxxx", "xx...", "xxxx.", "...xx", "...xx", "xx.xx", ".xxx."]),
    ('6', [".xxx.", "xx...", "xx...", "xxxx.", "xx.xx", "xx.xx", ".xxx."]),
    ('7', ["xxxxx", "...xx", "..xx.", "..xx.", ".xx..", ".xx..", ".xx.."]),
    ('8', [".xxx.", "xx.xx", "xx.xx", ".xxx.", "xx.xx", "xx.xx", ".xxx."]),
    ('9', [".xxx.", "xx.xx", "xx.xx", ".xxxx", "...xx", "...xx", ".xxx."]),
    ('+', [".....", "..x..", "..x..", "xxxxx", "..x..", "..x..", "....."]),
    ('-', [".....", ".....", ".....", "xxxxx", ".....", ".....", "....."]),
    ('=', [".....", ".....", "xxxxx", ".....", "xxxxx", ".....", "....."]),
    ('/', ["...xx", "...xx", "..xx.", "..xx.", ".xx..", "xx...", "xx..."]),
    ('?', [".xxx.", "xx.xx", "...xx", "..xx.", "..xx.", ".....", "..xx."]),
];

/// 一串大数字画成一张精灵（颜色格 `c`），字之间空一格，空格占一格。再 `scaled` 放大。
pub fn numbers(s: &str, c: u8) -> Sprite {
    let mut rows = vec![String::new(); 7];
    for (i, ch) in s.chars().enumerate() {
        if i > 0 {
            rows.iter_mut().for_each(|r| r.push('.'));
        }
        match DIGITS.iter().find(|(d, _)| *d == ch) {
            Some((_, art)) => rows.iter_mut().zip(art).for_each(|(r, a)| r.push_str(a)),
            None => rows.iter_mut().for_each(|r| r.push('.')),
        }
    }
    let rows: Vec<&str> = rows.iter().map(|r| r.as_str()).collect();
    Sprite::art(&rows, &[('x', c)])
}
