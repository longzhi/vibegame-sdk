//! 颜色和精灵。小狗是自己画的：奶油色，一只棕耳朵，背上一块棕斑，红项圈。

use vibeplay::full::Sprite;

pub const BG: u8 = 0;
pub const INK: u8 = 1;
pub const CREAM: u8 = 2;
pub const BROWN: u8 = 3;
pub const RED: u8 = 4;
pub const PINK: u8 = 5;
pub const WHITE: u8 = 6;
pub const WALL: u8 = 7;
pub const WALL2: u8 = 8;
pub const FLOOR: u8 = 9;
pub const FLOOR2: u8 = 10;
pub const SKY: u8 = 11;
pub const NIGHT: u8 = 12;
pub const MOON: u8 = 13;
pub const WALL_N: u8 = 14;
pub const WALL2_N: u8 = 15;
pub const FLOOR_N: u8 = 16;
pub const FLOOR2_N: u8 = 17;
pub const BAR_BG: u8 = 18;
pub const FOOD: u8 = 19;
pub const FUN: u8 = 20;
pub const PANEL: u8 = 21;
pub const GRASS: u8 = 22;
pub const GRASS2: u8 = 23;
pub const TREE: u8 = 24;
pub const PATH: u8 = 25;
pub const GRAY: u8 = 26;
pub const CUSHION: u8 = 27;
pub const CUSHION_N: u8 = 28;
pub const BOWL: u8 = 29;
pub const KIBBLE: u8 = 30;
pub const CREAM_N: u8 = 31;
pub const BROWN_N: u8 = 32;
pub const RED_N: u8 = 33;
pub const ZZZ: u8 = 34;
pub const FRAME: u8 = 35;
pub const BOWL_N: u8 = 36;

pub const PALETTE: [(u8, [u8; 3]); 37] = [
    (BG, [20, 16, 28]),
    (INK, [26, 20, 36]),
    (CREAM, [246, 224, 178]),
    (BROWN, [164, 100, 60]),
    (RED, [216, 60, 60]),
    (PINK, [240, 140, 160]),
    (WHITE, [255, 248, 232]),
    (WALL, [124, 180, 168]),
    (WALL2, [108, 164, 152]),
    (FLOOR, [176, 120, 72]),
    (FLOOR2, [148, 96, 56]),
    (SKY, [140, 208, 248]),
    (NIGHT, [28, 36, 80]),
    (MOON, [248, 232, 144]),
    (WALL_N, [44, 66, 74]),
    (WALL2_N, [38, 58, 66]),
    (FLOOR_N, [70, 50, 40]),
    (FLOOR2_N, [56, 40, 32]),
    (BAR_BG, [60, 52, 72]),
    (FOOD, [240, 160, 48]),
    (FUN, [240, 96, 156]),
    (PANEL, [42, 34, 54]),
    (GRASS, [88, 184, 72]),
    (GRASS2, [60, 140, 56]),
    (TREE, [44, 108, 60]),
    (PATH, [216, 184, 120]),
    (GRAY, [168, 160, 180]),
    (CUSHION, [106, 90, 168]),
    (CUSHION_N, [58, 50, 102]),
    (BOWL, [72, 120, 208]),
    (KIBBLE, [140, 80, 48]),
    (CREAM_N, [150, 138, 120]),
    (BROWN_N, [100, 70, 48]),
    (RED_N, [130, 44, 44]),
    (ZZZ, [200, 192, 240]),
    (FRAME, [240, 232, 210]),
    (BOWL_N, [44, 70, 120]),
];

/// 夜里屋里暗：小狗的颜色换成暗的。
pub const DIM: [(u8, u8); 4] = [(CREAM, CREAM_N), (BROWN, BROWN_N), (RED, RED_N), (WHITE, CREAM_N)];

fn dog(rows: &[&str]) -> Sprite {
    Sprite::art(rows, &[('k', INK), ('c', CREAM), ('b', BROWN), ('r', RED), ('p', PINK), ('w', WHITE)])
}

/// 头，朝右，10×6。`eyes` 为 false 是闭着眼。
pub fn head(eyes: bool) -> Sprite {
    if eyes {
        dog(&[".kkk......", "kbbck.....", "kbccckkk..", "kbcwkcccck", ".kcccccckk", "..kcccpk.."])
    } else {
        dog(&[".kkk......", "kbbck.....", "kbccckkk..", "kbckkcccck", ".kcccccckk", "..kccck..."])
    }
}

/// 身子，朝右，18×10。两张：尾巴翘着、尾巴放下（摇尾巴就是两张换着画）。
pub fn body(wag: bool) -> Sprite {
    let top: [&str; 5] = if wag {
        ["..k.......krrrrk..", ".kck....kkcccccck.", ".kck...kccccccccck", "..kck.kcbbccccccck", "...kkkcbbbcccccck."]
    } else {
        ["..........krrrrk..", "........kkcccccck.", ".kk....kccccccccck", "kcckk.kcbbccccccck", ".kkcckcbbbcccccck."]
    };
    let mut rows = top.to_vec();
    rows.extend([".....kcbbcccccccck", ".....kccccccccccck", ".....kcckcckcckcck", ".....kcck.kck.kcck", ".....kkkk.kkk.kkkk"]);
    dog(&rows)
}

/// 低头吃饭的身子：项圈让头挡住，背上不露出来。
pub fn body_eat() -> Sprite {
    dog(&[
        "..k...............",
        ".kck....kkkkkkkk..",
        ".kck...kcccccccckk",
        "..kck.kcbbccccccck",
        "...kkkcbbbcccccck.",
        ".....kcbbcccccccck",
        ".....kccccccccccck",
        ".....kcckcckcckcck",
        ".....kcck.kck.kcck",
        ".....kkkk.kkk.kkkk",
    ])
}

/// 月牙，7×7。
pub fn moon() -> Sprite {
    Sprite::art(&["..mmm..", ".mm....", "mm.....", "mm.....", "mm.....", ".mm....", "..mmm.."], &[('m', MOON)])
}

/// 走路的身子：腿换一种摆法。
pub fn body_step() -> Sprite {
    dog(&[
        "..k.......krrrrk..",
        ".kck....kkcccccck.",
        ".kck...kccccccccck",
        "..kck.kcbbccccccck",
        "...kkkcbbbcccccck.",
        ".....kcbbcccccccck",
        ".....kccccccccccck",
        "....kcckcck.kcckck",
        "...kcck..kck.kck.k",
        "...kkk...kkk.kkk..",
    ])
}

/// 趴着睡的身子，20×6。
pub fn lying() -> Sprite {
    dog(&[
        "..kk.....kkkkkkkk...",
        ".kcck..kkccccccccck.",
        "..kcckkcbbccccccccck",
        "...kcccbbbccccccccck",
        "...kccccbcccccccccck",
        "....kkkkkkkkkkkkkkk.",
    ])
}

/// 饭碗，14×7。`left` 是剩几成：0 到 3。
pub fn bowl(left: u32) -> Sprite {
    let top: [&str; 2] = match left {
        3 => ["....ooooo.....", "..ooooooooo..."],
        2 => [".....oo.......", "..oooooo.ooo.."],
        1 => ["..............", "....oo...o...."],
        _ => ["..............", ".............."],
    };
    Sprite::art(
        &[top[0], top[1], "kkkkkkkkkkkkkk", "kbbbbbbbbbbbbk", ".kbwbbbbbbbbk.", "..kbbbbbbbbk..", "...kkkkkkkk..."],
        &[('k', INK), ('b', BOWL), ('w', WHITE), ('o', KIBBLE)],
    )
}

/// 一颗心：遛完狗冒出来。
pub fn heart() -> Sprite {
    Sprite::art(&[".pp.pp.", "pppwppp", "ppppppp", ".ppppp.", "..ppp..", "...p..."], &[('p', PINK), ('w', WHITE)])
}

/// 一棵树，遛狗时往后退。
pub fn tree() -> Sprite {
    Sprite::art(
        &[
            "....tttt....",
            "..tttgtttt..",
            ".tttttttgtt.",
            "ttgttttttttt",
            "tttttttgtttt",
            ".tttgtttttt.",
            "..tttttttt..",
            ".....bb.....",
            ".....bb.....",
            ".....bb.....",
            "....bbbb....",
        ],
        &[('t', TREE), ('g', GRASS2), ('b', BROWN)],
    )
}
