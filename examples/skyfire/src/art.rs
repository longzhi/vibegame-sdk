//! 调色板、精灵、地图块。精灵用字符画，地图块用代码按格子画（一格 16 像素）。

use vibeplay::full::Sprite;

pub const SKY: u8 = 0;
pub const INK: u8 = 1;
pub const WHITE: u8 = 2;
pub const RED: u8 = 3;
pub const RED_HI: u8 = 4;
pub const RED_LO: u8 = 5;
pub const ORANGE: u8 = 6;
pub const YELLOW: u8 = 7;
pub const YELLOW_HI: u8 = 8;
pub const BLUE: u8 = 9;
pub const BLUE_HI: u8 = 10;
pub const NAVY: u8 = 11;
pub const SEA: u8 = 12;
pub const SEA_HI: u8 = 13;
pub const SAND: u8 = 14;
pub const SAND_LO: u8 = 15;
pub const GRASS: u8 = 16;
pub const GRASS_LO: u8 = 17;
pub const ROCK: u8 = 18;
pub const ROCK_LO: u8 = 19;
pub const OLIVE: u8 = 20;
pub const OLIVE_HI: u8 = 21;
pub const OLIVE_LO: u8 = 22;
pub const PURPLE: u8 = 23;
pub const PURPLE_HI: u8 = 24;
pub const STEEL: u8 = 25;
pub const STEEL_HI: u8 = 26;
pub const STEEL_LO: u8 = 27;
pub const NIGHT: u8 = 28;
pub const CLOUD: u8 = 29;
pub const CLOUD_HI: u8 = 30;
pub const METAL: u8 = 31;
pub const METAL_HI: u8 = 32;
pub const METAL_LO: u8 = 33;
pub const PINK: u8 = 34;
pub const PINK_HI: u8 = 35;
pub const CYAN: u8 = 36;
pub const GREEN: u8 = 37;
pub const GRAY: u8 = 38;
pub const DUNE: u8 = 39;
pub const LAMP: u8 = 40;
/// 被打中闪一下的白。
pub const FLASH: u8 = 41;

pub const PALETTE: [(u8, [u8; 3]); 42] = [
    (SKY, [0x10, 0x0c, 0x24]),
    (INK, [0x0c, 0x08, 0x14]),
    (WHITE, [0xf4, 0xf0, 0xe6]),
    (RED, [0xd8, 0x30, 0x30]),
    (RED_HI, [0xff, 0x70, 0x58]),
    (RED_LO, [0x80, 0x18, 0x28]),
    (ORANGE, [0xff, 0x94, 0x20]),
    (YELLOW, [0xff, 0xd2, 0x3f]),
    (YELLOW_HI, [0xff, 0xf4, 0xa0]),
    (BLUE, [0x30, 0x70, 0xe0]),
    (BLUE_HI, [0x8c, 0xd0, 0xff]),
    (NAVY, [0x18, 0x24, 0x58]),
    (SEA, [0x1c, 0x4c, 0x90]),
    (SEA_HI, [0x48, 0x88, 0xc8]),
    (SAND, [0xe8, 0xc8, 0x80]),
    (SAND_LO, [0xc0, 0x98, 0x58]),
    (GRASS, [0x48, 0xa0, 0x48]),
    (GRASS_LO, [0x2c, 0x70, 0x38]),
    (ROCK, [0x88, 0x80, 0x78]),
    (ROCK_LO, [0x58, 0x50, 0x50]),
    (OLIVE, [0x70, 0x88, 0x38]),
    (OLIVE_HI, [0xa8, 0xc0, 0x58]),
    (OLIVE_LO, [0x40, 0x50, 0x20]),
    (PURPLE, [0x80, 0x40, 0xb8]),
    (PURPLE_HI, [0xc0, 0x88, 0xf0]),
    (STEEL, [0x78, 0x88, 0xa0]),
    (STEEL_HI, [0xb8, 0xc8, 0xd8]),
    (STEEL_LO, [0x40, 0x48, 0x60]),
    (NIGHT, [0x0c, 0x10, 0x30]),
    (CLOUD, [0x30, 0x38, 0x68]),
    (CLOUD_HI, [0x58, 0x64, 0x98]),
    (METAL, [0x48, 0x40, 0x50]),
    (METAL_HI, [0x70, 0x68, 0x78]),
    (METAL_LO, [0x28, 0x22, 0x30]),
    (PINK, [0xff, 0x48, 0xa0]),
    (PINK_HI, [0xff, 0xb8, 0xe0]),
    (CYAN, [0x40, 0xf0, 0xe0]),
    (GREEN, [0x60, 0xe0, 0x60]),
    (GRAY, [0x98, 0x98, 0xa8]),
    (DUNE, [0xd4, 0xa8, 0x68]),
    (LAMP, [0xff, 0x60, 0x40]),
    (FLASH, [0xff, 0xff, 0xff]),
];

/// 主角的战机「烈风」：红白，机头朝上。`flame` 换着画尾焰。
pub fn hero(flame: bool) -> Sprite {
    let tail = if flame { "......oyyo......" } else { ".......oo......." };
    Sprite::art(
        &[
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
            tail,
            "................",
        ],
        &[('w', WHITE), ('r', RED), ('R', RED_LO), ('b', BLUE_HI), ('o', ORANGE), ('y', YELLOW)],
    )
}

/// 小飞镖：绿色的小战斗机，机头朝下。
pub fn dart() -> Sprite {
    Sprite::art(
        &["....kk....", "...kggk...", "k..kggk..k", "kgkgggggkg", ".kgGGGGgk.", "..kgyygk..", "...kggk...", "....kk...."],
        &[('k', OLIVE_LO), ('g', OLIVE), ('G', OLIVE_HI), ('y', YELLOW)],
    )
}

/// 俯冲机：紫色，翅膀往后掠。
pub fn swoop() -> Sprite {
    Sprite::art(
        &[
            "pp..........pp",
            "PPp........pPP",
            ".PPp..kk..pPP.",
            "..PPpkppkpPP..",
            "...PPppppPP...",
            "....PpwwpP....",
            ".....pwwp.....",
            "......pp......",
        ],
        &[('p', PURPLE), ('P', PURPLE_HI), ('k', INK), ('w', CYAN)],
    )
}

/// 轰炸机：大的，墨绿，四个发动机。
pub fn bomber() -> Sprite {
    Sprite::art(
        &[
            "...........kkkkkkkkkk...........",
            "..........kggggggggggk..........",
            "...kk....kgGGGGGGGGGGgk....kk...",
            "..kggk..kgGGGGGGGGGGGGgk..kggk..",
            "kkkggkkkkgGGGGGGGGGGGGgkkkkggkkk",
            "kggggggggggGGGGGGGGGGggggggggggk",
            "kGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGk",
            "kggggggggggGGGGGGGGGGggggggggggk",
            ".kkkkoo.kkkgGGGGGGGGgkkk.oo.kkk.",
            "......o...kgGGGwwGGGgk...o......",
            "..........kgGGwbbwGGgk..........",
            "..........kgGGwbbwGGgk..........",
            "...........kgGGwwGGgk...........",
            "...........kgGGGGGGgk...........",
            "............kgGGGGgk............",
            ".............kggggk.............",
            "..............kggk..............",
            "...............kk...............",
        ],
        &[('k', OLIVE_LO), ('g', OLIVE), ('G', OLIVE_HI), ('w', STEEL_HI), ('b', BLUE), ('o', ORANGE)],
    )
}

/// 运输机：橙色，打下来掉东西。
pub fn carrier() -> Sprite {
    Sprite::art(
        &[
            "......kkkkkk......",
            ".....kooooook.....",
            "kkkkkkoyyyyokkkkkk",
            "kooooooyyyyoooooook",
            "kkkkkkoyyyyokkkkkk",
            ".....kooooook.....",
            "......kowwok......",
            "......koooook.....",
            ".......kkkk.......",
        ],
        &[('k', RED_LO), ('o', ORANGE), ('y', YELLOW), ('w', WHITE)],
    )
}

/// 浮雷：一圈刺，打爆了往八个方向崩子弹。
pub fn mine(lit: bool) -> Sprite {
    let c = if lit { LAMP } else { RED_LO };
    Sprite::art(
        &["....k....", ".k.kGk.k.", "..kGGGk..", ".kGGcGGk.", "kGGcccGGk", ".kGGcGGk.", "..kGGGk..", ".k.kGk.k.", "....k...."],
        &[('k', INK), ('G', STEEL), ('c', c)],
    )
}

/// 地面炮台的底座。炮管照方向另外画。
pub fn turret_base() -> Sprite {
    Sprite::art(
        &[
            "....kkkkkkkk....",
            "..kkssssssssk...",
            ".ksSSSSSSSSssk..",
            ".ksSSkkkkSSSsk..",
            "ksSSkmmmmkSSSsk.",
            "ksSkmmmmmmkSSsk.",
            "ksSkmmmmmmkSSsk.",
            "ksSkmmmmmmkSSsk.",
            "ksSkmmmmmmkSSsk.",
            "ksSSkmmmmkSSSsk.",
            ".ksSSkkkkSSSsk..",
            ".ksSSSSSSSSssk..",
            "..kksssssssk....",
            "....kkkkkkk.....",
        ],
        &[('k', INK), ('s', STEEL_LO), ('S', STEEL), ('m', METAL_LO)],
    )
}

/// 东西：P 加火力，B 加炸弹，星是分。
pub fn item(kind: u8) -> Sprite {
    let (c, hi) = match kind {
        0 => (RED, RED_HI),
        1 => (BLUE, BLUE_HI),
        _ => (YELLOW, YELLOW_HI),
    };
    Sprite::art(
        &["...kkkk...", "..kHHHHk..", ".kHccccHk.", "kHccccccHk", "kHccccccHk", "kHccccccHk", "kHccccccHk", ".kHccccHk.", "..kHHHHk..", "...kkkk..."],
        &[('k', INK), ('H', hi), ('c', c)],
    )
}

/// 一格地图块：底色 `base` 上撒点 `dots` 色的斑，`seed` 定斑的位置。
fn speckle(base: u8, dots: u8, seed: u32, every: u32) -> Sprite {
    let mut px = vec![base; 256];
    let mut s = seed.wrapping_mul(2654435761) | 1;
    for p in px.iter_mut() {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        if s.is_multiple_of(every) {
            *p = dots;
        }
    }
    Sprite { w: 16, h: 16, px }
}

fn rows(rows: &[&str], colors: &[(char, u8)]) -> Sprite {
    Sprite::art(rows, colors)
}

/// 每关的地图块，下标就是地图里格子的值（0 不画，露出底色）。
/// 第一关：1 浪、2 沙、3 草、4 礁石。第二关：1 沙丘、2 石头、3 路、4 废墟。
/// 第三关：1 云、2 亮的云、3 星。第四关：1 甲板、2 通风口、3 灯、4 管子。
pub fn tiles(stage: usize) -> Vec<Sprite> {
    let blank = Sprite { w: 16, h: 16, px: vec![0; 256] };
    match stage {
        0 => vec![
            blank,
            rows(
                &[
                    "................",
                    "...hhh..........",
                    "..h...h.........",
                    "................",
                    "................",
                    "..........hhh...",
                    ".........h...h..",
                    "................",
                    "................",
                    "....hhh.........",
                    "...h...h........",
                    "................",
                    "................",
                    "...........hhh..",
                    "..........h...h.",
                    "................",
                ],
                &[('h', SEA_HI)],
            ),
            speckle(SAND, SAND_LO, 3, 9),
            speckle(GRASS, GRASS_LO, 5, 6),
            speckle(ROCK, ROCK_LO, 7, 4),
        ],
        1 => vec![
            blank,
            rows(
                &[
                    "................",
                    "................",
                    "......dddd......",
                    "....dd....dd....",
                    "..dd........dd..",
                    "................",
                    "................",
                    "................",
                    "................",
                    "dd............dd",
                    "..dd........dd..",
                    "....dddddddd....",
                    "................",
                    "................",
                    "................",
                    "................",
                ],
                &[('d', SAND_LO)],
            ),
            speckle(ROCK, ROCK_LO, 11, 5),
            rows(
                &[
                    "kssssssssssssssk",
                    "kssssssssssssssk",
                    "ksssssssyssssssk",
                    "kssssssssssssssk",
                    "kssssssssssssssk",
                    "ksssssssyssssssk",
                    "kssssssssssssssk",
                    "kssssssssssssssk",
                    "ksssssssyssssssk",
                    "kssssssssssssssk",
                    "kssssssssssssssk",
                    "ksssssssyssssssk",
                    "kssssssssssssssk",
                    "kssssssssssssssk",
                    "ksssssssyssssssk",
                    "kssssssssssssssk",
                ],
                &[('k', ROCK_LO), ('s', GRAY), ('y', SAND)],
            ),
            rows(
                &[
                    "kkkkkkkk..kkkkkk",
                    "krrrrrrk..krrrrk",
                    "krRRRRrk..krRRrk",
                    "krRRRRrk..krRRrk",
                    "krrrrrrk..krrrrk",
                    "kkkkkkkk..kkkkkk",
                    "................",
                    "..kkkkkkkkkk....",
                    "..krrrrrrrrk....",
                    "..krRRRRRRrk....",
                    "..krrrrrrrrk....",
                    "..kkkkkkkkkk....",
                    "................",
                    "kkkkk.....kkkkkk",
                    "krrrk.....krrrrk",
                    "kkkkk.....kkkkkk",
                ],
                &[('k', ROCK_LO), ('r', SAND_LO), ('R', DUNE)],
            ),
        ],
        2 => vec![blank, speckle(CLOUD, CLOUD_HI, 13, 7), speckle(CLOUD_HI, CLOUD, 17, 9)],
        _ => vec![
            blank,
            rows(
                &[
                    "kmmmmmmmmmmmmmmk",
                    "mMMMMMMMMMMMMMMm",
                    "mMmmmmmmmmmmmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmMMMMMMMMMmMMm",
                    "mMmmmmmmmmmmmMMm",
                    "mMMMMMMMMMMMMMMm",
                    "mMMkMMMMMMMMkMMm",
                    "mMMMMMMMMMMMMMMm",
                    "kmmmmmmmmmmmmmmk",
                ],
                &[('k', INK), ('m', METAL_LO), ('M', METAL)],
            ),
            rows(
                &[
                    "kkkkkkkkkkkkkkkk",
                    "kmmmmmmmmmmmmmmk",
                    "kmkkkkkkkkkkkkmk",
                    "kmmmmmmmmmmmmmmk",
                    "kmkkkkkkkkkkkkmk",
                    "kmmmmmmmmmmmmmmk",
                    "kmkkkkkkkkkkkkmk",
                    "kmmmmmmmmmmmmmmk",
                    "kmkkkkkkkkkkkkmk",
                    "kmmmmmmmmmmmmmmk",
                    "kmkkkkkkkkkkkkmk",
                    "kmmmmmmmmmmmmmmk",
                    "kmkkkkkkkkkkkkmk",
                    "kmmmmmmmmmmmmmmk",
                    "kmmmmmmmmmmmmmmk",
                    "kkkkkkkkkkkkkkkk",
                ],
                &[('k', INK), ('m', METAL_HI)],
            ),
            rows(
                &[
                    "kmmmmmmmmmmmmmmk",
                    "mMMMMMMMMMMMMMMm",
                    "mMMMMMMMMMMMMMMm",
                    "mMMMMMkkkkMMMMMm",
                    "mMMMMkllllkMMMMm",
                    "mMMMkllLLllkMMMm",
                    "mMMMklLLLLlkMMMm",
                    "mMMMklLLLLlkMMMm",
                    "mMMMkllLLllkMMMm",
                    "mMMMMkllllkMMMMm",
                    "mMMMMMkkkkMMMMMm",
                    "mMMMMMMMMMMMMMMm",
                    "mMMMMMMMMMMMMMMm",
                    "mMMMMMMMMMMMMMMm",
                    "mMMMMMMMMMMMMMMm",
                    "kmmmmmmmmmmmmmmk",
                ],
                &[('k', INK), ('m', METAL_LO), ('M', METAL), ('l', RED_LO), ('L', LAMP)],
            ),
            rows(
                &[
                    "....kSSSSSSk....",
                    "....kShhhhSk....",
                    "....kShhhhSk....",
                    "....kSSSSSSk....",
                    "....kSSSSSSk....",
                    "....kShhhhSk....",
                    "....kShhhhSk....",
                    "...kkSSSSSSkk...",
                    "...kSSSSSSSSk...",
                    "...kkSSSSSSkk...",
                    "....kShhhhSk....",
                    "....kShhhhSk....",
                    "....kSSSSSSk....",
                    "....kSSSSSSk....",
                    "....kShhhhSk....",
                    "....kSSSSSSk....",
                ],
                &[('k', INK), ('S', STEEL_LO), ('h', STEEL)],
            ),
        ],
    }
}
