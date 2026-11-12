//! 画面：调色板、每关的配色、字符画。都是自己画的。
//! 主角、敌兵拆成上半身和腿两块，拼起来是一张 24×24：上半身跟着往哪打换，腿跟着跑换。

use vibeplay::full::Sprite;

// ---------- 调色板 ----------
pub const SKY: u8 = 0;
pub const INK: u8 = 1;
pub const WHITE: u8 = 2;
pub const HAIR: u8 = 3;
pub const BAND: u8 = 4;
pub const BAND_HI: u8 = 5;
pub const SKIN: u8 = 6;
pub const SKIN_LO: u8 = 7;
pub const VEST: u8 = 8;
pub const VEST_LO: u8 = 9;
pub const PANTS: u8 = 10;
pub const PANTS_LO: u8 = 11;
pub const BOOT: u8 = 12;
pub const GUN: u8 = 13;
pub const GUN_HI: u8 = 14;
pub const HELM: u8 = 15;
pub const HELM_HI: u8 = 16;
pub const UNI: u8 = 17;
pub const UNI_LO: u8 = 18;
pub const EPANTS: u8 = 19;
pub const EPANTS_LO: u8 = 20;
pub const RED: u8 = 21;
pub const RED_HI: u8 = 22;
pub const RED_LO: u8 = 23;
pub const ORANGE: u8 = 24;
pub const YELLOW: u8 = 25;
pub const YELLOW_HI: u8 = 26;
pub const STEEL: u8 = 27;
pub const STEEL_HI: u8 = 28;
pub const STEEL_LO: u8 = 29;
pub const GRAY: u8 = 30;
pub const GRAY_LO: u8 = 31;
pub const NAVY: u8 = 32;
pub const BLUE: u8 = 33;
pub const BLUE_HI: u8 = 34;
pub const GREEN: u8 = 35;
pub const GREEN_HI: u8 = 36;
pub const GREEN_LO: u8 = 37;
pub const PURPLE: u8 = 38;
pub const PINK: u8 = 39;
pub const CYAN: u8 = 40;
pub const SAND: u8 = 41;
pub const SAND_LO: u8 = 42;
pub const BLACK: u8 = 43;
// 每关换的颜色。
pub const SKY2: u8 = 44;
pub const SKY3: u8 = 45;
pub const BG_FAR: u8 = 46;
pub const BG_MID: u8 = 47;
pub const BG_NEAR: u8 = 48;
pub const BG_DARK: u8 = 49;
pub const LIGHT: u8 = 50;
pub const WATER: u8 = 51;
pub const WATER_HI: u8 = 52;
pub const WATER_LO: u8 = 53;
pub const TOP: u8 = 54;
pub const TOP_HI: u8 = 55;
pub const TOP_LO: u8 = 56;
pub const FILL: u8 = 57;
pub const FILL_HI: u8 = 58;
pub const FILL_LO: u8 = 59;

pub const PALETTE: [(u8, [u8; 3]); 44] = [
    (SKY, [0x40, 0x80, 0xd0]),
    (INK, [0x10, 0x0c, 0x14]),
    (WHITE, [0xfc, 0xfc, 0xf8]),
    (HAIR, [0x3c, 0x24, 0x18]),
    (BAND, [0xe0, 0x28, 0x28]),
    (BAND_HI, [0xff, 0x70, 0x60]),
    (SKIN, [0xf0, 0xb0, 0x80]),
    (SKIN_LO, [0xb8, 0x74, 0x50]),
    (VEST, [0x5c, 0x7c, 0x38]),
    (VEST_LO, [0x38, 0x50, 0x20]),
    (PANTS, [0x30, 0x48, 0x90]),
    (PANTS_LO, [0x1c, 0x2c, 0x60]),
    (BOOT, [0x40, 0x30, 0x28]),
    (GUN, [0x50, 0x54, 0x60]),
    (GUN_HI, [0xa8, 0xb0, 0xbc]),
    (HELM, [0x48, 0x5c, 0x40]),
    (HELM_HI, [0x78, 0x90, 0x68]),
    (UNI, [0x9c, 0x94, 0x84]),
    (UNI_LO, [0x68, 0x60, 0x54]),
    (EPANTS, [0x6c, 0x5c, 0x48]),
    (EPANTS_LO, [0x48, 0x3c, 0x30]),
    (RED, [0xd8, 0x28, 0x20]),
    (RED_HI, [0xff, 0x80, 0x60]),
    (RED_LO, [0x80, 0x10, 0x18]),
    (ORANGE, [0xff, 0x98, 0x20]),
    (YELLOW, [0xff, 0xd8, 0x30]),
    (YELLOW_HI, [0xff, 0xf8, 0xb0]),
    (STEEL, [0x80, 0x88, 0x98]),
    (STEEL_HI, [0xc8, 0xd0, 0xdc]),
    (STEEL_LO, [0x48, 0x4c, 0x5c]),
    (GRAY, [0x70, 0x70, 0x78]),
    (GRAY_LO, [0x38, 0x38, 0x44]),
    (NAVY, [0x18, 0x1c, 0x3c]),
    (BLUE, [0x28, 0x60, 0xd8]),
    (BLUE_HI, [0x78, 0xb8, 0xff]),
    (GREEN, [0x30, 0x98, 0x40]),
    (GREEN_HI, [0x80, 0xd8, 0x60]),
    (GREEN_LO, [0x14, 0x50, 0x28]),
    (PURPLE, [0x78, 0x38, 0xa8]),
    (PINK, [0xff, 0x88, 0xc0]),
    (CYAN, [0x60, 0xf0, 0xf0]),
    (SAND, [0xd8, 0xb8, 0x80]),
    (SAND_LO, [0x98, 0x78, 0x48]),
    (BLACK, [0x00, 0x00, 0x00]),
];

/// 每关的场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Jungle,
    Falls,
    Snow,
    Fort,
}

impl Theme {
    pub fn colors(self) -> [(u8, [u8; 3]); 16] {
        let c = |v: [u32; 16]| {
            let idx = [SKY, SKY2, SKY3, BG_FAR, BG_MID, BG_NEAR, BG_DARK, LIGHT, WATER, WATER_HI, WATER_LO, TOP, TOP_HI, TOP_LO, FILL, FILL_HI];
            let mut out = [(0u8, [0u8; 3]); 16];
            for i in 0..16 {
                out[i] = (idx[i], [(v[i] >> 16) as u8, (v[i] >> 8) as u8, v[i] as u8]);
            }
            out
        };
        match self {
            // 傍晚的丛林：橙红的天，远山发紫，近处的树墨绿。
            Theme::Jungle => c([
                0x2c1c4c, 0x8c3c5c, 0xf08850, 0x6c3c6c, 0x2c4c3c, 0x18382c, 0x0c1c18, 0xffd890, 0x1c4c8c, 0x60a0e0, 0x10305c, 0x58b040, 0xa0e070, 0x2c6c24, 0x6c5038,
                0x98785c,
            ]),
            // 瀑布：白天，崖是灰褐的石头，水是亮蓝。
            Theme::Falls => c([
                0x3c78c8, 0x70a8e8, 0xa8d8f8, 0x5c5c6c, 0x6c6058, 0x4c4440, 0x2c2828, 0xffffff, 0x3c88e0, 0xc8f0ff, 0x1c58b0, 0x7c9c50, 0xb0d078, 0x4c6430, 0x786c60,
                0xa09888,
            ]),
            // 雪原的夜：深蓝的天，雪山，地上是雪。
            Theme::Snow => c([
                0x0c1028, 0x182448, 0x2c3c6c, 0x4c5c8c, 0x8c9cc8, 0x3c4870, 0x1c2440, 0xf0f8ff, 0x3c5c8c, 0x9cc0e8, 0x1c3058, 0xe8f0fc, 0xffffff, 0xa8b8d8, 0x5c6880,
                0x8c98b0,
            ]),
            // 要塞里：暗的铁墙，红灯，地上是铁格板。
            Theme::Fort => c([
                0x140c14, 0x24141c, 0x341c24, 0x3c2c34, 0x2c2430, 0x1c1820, 0x0c0810, 0xff5030, 0xa02818, 0xff8040, 0x601010, 0x8c94a0, 0xd0d8e0, 0x4c5060, 0x4c4450,
                0x6c6470,
            ]),
        }
    }
}

// ---------- 主角 ----------

/// 两个突击队员：雷系红头巾、绿背心；霆系蓝头巾、沙色背心。
pub const HERO_KEYS: [(char, u8); 16] = [
    ('k', INK),
    ('a', HAIR),
    ('h', BAND),
    ('H', BAND_HI),
    ('s', SKIN),
    ('S', SKIN_LO),
    ('v', VEST),
    ('V', VEST_LO),
    ('p', PANTS),
    ('P', PANTS_LO),
    ('b', BOOT),
    ('g', GUN),
    ('G', GUN_HI),
    ('w', WHITE),
    ('m', HELM),
    ('M', HELM_HI),
];

/// 霆的颜色：把雷的换掉。
pub const TING: [(u8, u8); 6] = [(BAND, BLUE), (BAND_HI, BLUE_HI), (VEST, SAND), (VEST_LO, SAND_LO), (PANTS, GREEN_LO), (PANTS_LO, GRAY_LO)];
/// 敌兵的颜色：背心、裤子换成灰的。
pub const ENEMY: [(u8, u8); 4] = [(VEST, UNI), (VEST_LO, UNI_LO), (PANTS, EPANTS), (PANTS_LO, EPANTS_LO)];

/// 头：头发、头巾（后面飘一截）。
const HEAD: [&str; 7] = [
    "......kkkk..............",
    ".....kaaaak.............",
    "....kaaaaaak............",
    "...hkhhhhhHk............",
    "..hh.kassksk............",
    ".h...kssssSk............",
    "......kssSk.............",
];
/// 敌兵的头：钢盔。
const HELMET: [&str; 7] = [
    "......kkkkk.............",
    ".....kmmMMmk............",
    "....kmmmmMmmk...........",
    "....kkkkkkkkkk..........",
    ".....kssksk.............",
    ".....kssssSk............",
    "......kssSk.............",
];

/// 身子（第 7 到 15 行），往哪打一张。往下斜着的枪伸到腿那里。
const BODY_FWD: [&str; 9] = [
    ".....kvvvvvkk...........",
    "....kvVvvvvssskkkkkkkk..",
    "....kvVvvvSSskgGGGGGGGk.",
    "....kvVvvvvkkkgkkkkkkkk.",
    "....kVVVVVVk..kk........",
    "....kppppppk............",
    "",
    "",
    "",
];
const BODY_UP: [&str; 9] = [
    ".....kvvvvvsssk.........",
    "....kvVvvvSssSk.........",
    "....kvVvvvkkkk..........",
    "....kvVvvvvk............",
    "....kVVVVVVk............",
    "....kppppppk............",
    "",
    "",
    "",
];
/// 往上打时枪在头旁边竖着（第 0 到 6 行）。
const GUN_UP: [&str; 7] = [
    "...........kGk..........",
    "...........kgk..........",
    "...........kgk..........",
    "...........kgk..........",
    "...........kgk..........",
    "...........kgk..........",
    "...........kgk..........",
];
const BODY_DIAG_UP: [&str; 9] = [
    ".....kvvvvvkkssk........",
    "....kvVvvvvsssk.........",
    "....kvVvvvSSkk..........",
    "....kvVvvvvk............",
    "....kVVVVVVk............",
    "....kppppppk............",
    "",
    "",
    "",
];
const GUN_DIAG_UP: [&str; 7] = [
    "....................kk..",
    "...................kGk..",
    "..................kgk...",
    ".................kgk....",
    "................kgk.....",
    "...............kgk......",
    "..............kgk.......",
];
const BODY_DIAG_DOWN: [&str; 9] = [
    ".....kvvvvvk............",
    "....kvVvvvvsssk.........",
    "....kvVvvvSSsssk........",
    "....kvVvvvvk.kgk........",
    "....kVVVVVVk..kgk.......",
    "....kppppppk...kgk......",
    "................kgk.....",
    ".................kGk....",
    "..................kk....",
];
/// 敌兵跑：两手一前一后，没拿枪。
const BODY_RUN: [&str; 9] = [
    ".....kvvvvvk............",
    "....kvVvvvvvk...........",
    "...kskVvvvvsk...........",
    "...kSkVvvvvssk..........",
    "....kVVVVVVk............",
    "....kppppppk............",
    "",
    "",
    "",
];

/// 胯（第 13 到 15 行）：身子和腿接起来。
const HIPS: [&str; 3] = ["....kppppppk....", "....kpPppPpk....", "....kppkkppk...."];

/// 腿（第 16 到 23 行）：站、跑三张。
const LEGS: [[&str; 8]; 4] = [
    [
        "....kppk.kppk...",
        "....kpPk.kpPk...",
        "....kppk.kppk...",
        "....kpPk.kpPk...",
        "....kppk.kppk...",
        "...kbbbk.kbbbk..",
        "...kbbbbkkbbbbk.",
        "...kkkkk.kkkkkk.",
    ],
    [
        "...kppk..kpppk..",
        "..kppk....kppk..",
        "..kpPk....kpPk..",
        ".kppk......kppk.",
        ".kbbk......kbbbk",
        "kbbbk......kbbbb",
        "kkkk........kkkk",
        "................",
    ],
    [
        "....kppkkppk....",
        "....kppk.kpPk...",
        "...kbbbk.kppk...",
        "...kbbbbk.kppk..",
        "....kkkk..kbbbk.",
        "..........kbbbbk",
        "...........kkkkk",
        "................",
    ],
    [
        "....kppk.kppk...",
        "...kppk...kppk..",
        "..kppk.....kpk..",
        "..kpPk.....kpPk.",
        ".kbbbk.....kbbk.",
        ".kbbbbk...kbbbk.",
        "..kkkkk...kkkk..",
        "................",
    ],
];

/// 趴下：一张 32×8，头在右边，枪往前。
pub const PRONE: [&str; 8] = [
    "..................kkkk..........",
    ".................kaaaak.........",
    "...............hkhhhhHk.........",
    "..............hh.kassksk........",
    "kkkkkkkkkkkkkkkkkkkssssskkkkkkkk",
    "kbbkppppppPpvvvvvvvVSSskgGGGGGGk",
    "kbbkpppPpppppvvVvvvvvkkkgkkkkkkk",
    "kkkkkkkkkkkkkkkkkkkkkk..kk......",
];
/// 跳起来翻跟头：缩成一团。转着画四个方向。
pub const SPIN: [&str; 16] = [
    "................",
    "......kkkk......",
    "....kkaaaakk....",
    "...khhhhhhhHk...",
    "..kaasssssvvvk..",
    "..kassksssvVVk..",
    ".kssssssSvvvVVk.",
    ".kSssssSvvvvVpk.",
    ".kvvSSvvvvvpppk.",
    ".kvvvvvvvvpppbk.",
    "..kvVVvvpppbbk..",
    "..kpppppppbbbk..",
    "...kpppppbbbk...",
    "....kkbbbbbk....",
    "......kkkkk.....",
    "................",
];
/// 倒在地上，脸朝天。
pub const DOWNED: [&str; 6] = [
    "........................",
    "..................kkkk..",
    "kkkkkkkkkkkkkkkkkkaaaak.",
    "kbbkppppPpppvvvvvvsssSk.",
    "kbbkppPppppvvvVvvsskssk.",
    "kkkkkkkkkkkkkkkkkkkkkk..",
];

/// 往哪打。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aim {
    Fwd,
    Up,
    DiagUp,
    DiagDown,
}

/// 拼一张 24×24：腿在下面，上面压头和身子。
fn compose(head: &[&str], body: &[&str], gun: Option<&[&str]>, legs: &[&str]) -> Sprite {
    let mut rows: Vec<String> = vec![String::new(); 24];
    for (i, r) in head.iter().enumerate() {
        rows[i] = r.to_string();
    }
    for (i, r) in HIPS.iter().enumerate() {
        rows[13 + i] = r.to_string();
    }
    for (i, r) in legs.iter().enumerate() {
        rows[16 + i] = r.to_string();
    }
    for (i, r) in body.iter().enumerate() {
        rows[7 + i] = overlay(&rows[7 + i], r);
    }
    if let Some(g) = gun {
        for (i, r) in g.iter().enumerate() {
            rows[i] = overlay(&rows[i], r);
        }
    }
    let refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
    Sprite::art(&refs, &HERO_KEYS)
}

/// 两行字符画叠起来：上面那行不是 `.` 的字符盖住下面的。
fn overlay(under: &str, over: &str) -> String {
    let (a, b): (Vec<char>, Vec<char>) = (under.chars().collect(), over.chars().collect());
    (0..a.len().max(b.len()))
        .map(|i| match b.get(i) {
            Some(&c) if c != '.' => c,
            _ => *a.get(i).unwrap_or(&'.'),
        })
        .collect()
}

/// 主角的样子：`[往哪打][腿]`，腿 0 是站，1 到 3 是跑。
pub fn hero_poses() -> Vec<Vec<Sprite>> {
    [Aim::Fwd, Aim::Up, Aim::DiagUp, Aim::DiagDown]
        .iter()
        .map(|a| {
            let (body, gun): (&[&str], Option<&[&str]>) = match a {
                Aim::Fwd => (&BODY_FWD, None),
                Aim::Up => (&BODY_UP, Some(&GUN_UP)),
                Aim::DiagUp => (&BODY_DIAG_UP, Some(&GUN_DIAG_UP)),
                Aim::DiagDown => (&BODY_DIAG_DOWN, None),
            };
            LEGS.iter().map(|l| compose(&HEAD, body, gun, l)).collect()
        })
        .collect()
}

/// 敌兵：跑三张、端枪站一张、端枪往上斜一张、往下斜一张。
pub fn soldier_poses() -> Vec<Sprite> {
    let mut v: Vec<Sprite> = (1..4).map(|i| compose(&HELMET, &BODY_RUN, None, &LEGS[i])).collect();
    v.push(compose(&HELMET, &BODY_FWD, None, &LEGS[0]));
    v.push(compose(&HELMET, &BODY_DIAG_UP, Some(&GUN_DIAG_UP), &LEGS[0]));
    v.push(compose(&HELMET, &BODY_DIAG_DOWN, None, &LEGS[0]));
    v.into_iter().map(|s| s.recolor(&ENEMY)).collect()
}

/// 转 90 度（顺时针）。
pub fn rotate(s: &Sprite) -> Sprite {
    let mut px = vec![0; s.px.len()];
    for y in 0..s.h {
        for x in 0..s.w {
            let (nx, ny) = (s.h - 1 - y, x);
            px[(ny * s.h + nx) as usize] = s.at(x, y);
        }
    }
    Sprite { w: s.h, h: s.w, px }
}

/// 上下翻过来。
pub fn flip_v(s: &Sprite) -> Sprite {
    let mut px = Vec::with_capacity(s.px.len());
    for y in (0..s.h).rev() {
        for x in 0..s.w {
            px.push(s.at(x, y));
        }
    }
    Sprite { w: s.w, h: s.h, px }
}

// ---------- 敌人 ----------

/// 蹲在沙袋后面扔手雷的兵：16×16，蹲着、举手扔两张。
pub const LOBBER: [[&str; 16]; 2] = [
    [
        "................",
        "................",
        "................",
        "................",
        "......kkkkk.....",
        ".....kmmMMmk....",
        "....kmmmmMmmk...",
        "....kkkkkkkkkk..",
        ".....kssksk.....",
        ".....kssssSk....",
        "....kvvvvvvk....",
        "...kvVvvvvvvk...",
        "...kvVvvvvsssk..",
        "..kppppppppk....",
        "..kpPppkbbbbk...",
        "..kkkkkkkkkkk...",
    ],
    [
        "..........kk....",
        ".........kssk...",
        ".........kssk...",
        ".........ksk....",
        "......kkkksk....",
        ".....kmmMMmk....",
        "....kmmmmMmmk...",
        "....kkkkkkkkkk..",
        ".....kssksk.....",
        ".....kssssSk....",
        "....kvvvvvvk....",
        "...kvVvvvvvk....",
        "...kvVvvvvvk....",
        "..kppppppppk....",
        "..kpPppkbbbbk...",
        "..kkkkkkkkkkk...",
    ],
];

/// 沙袋：16×8，挡在兵前面。
pub const SANDBAG: [&str; 8] = [
    "..kkkkk.kkkkk...",
    ".kyyyYykyyyYyk..",
    "kyyyyyyyyyyyyyk.",
    "kkkkkkkkkkkkkkk.",
    "kyyYykyyyyYyyyk.",
    "kyyyyyykyyyyyyyk",
    "kooooooooooooook",
    "kkkkkkkkkkkkkkkk",
];
pub const SANDBAG_KEYS: [(char, u8); 4] = [('k', INK), ('y', SAND), ('Y', WHITE), ('o', SAND_LO)];

/// 墙上的炮台底座：16×16，圆的铁盘。炮管另外画。
pub const TURRET: [&str; 16] = [
    "kkkkkkkkkkkkkkkk",
    "kzzzzzzzzzzzzzzk",
    "kzzzkkkkkkkzzzzk",
    "kzzkssSSSSskzzzk",
    "kzksSSSSSSSskzzk",
    "kzksSSsssSSsskzk",
    "kkssSsrrrsSssskk",
    "kkssSsrRrsSssskk",
    "kkssSsrrrsSssskk",
    "kkssSssssSSssskk",
    "kzksSSSSSSSsskzk",
    "kzkssSSSSSssskzk",
    "kzzksssssssskzzk",
    "kzzzkkkkkkkkzzzk",
    "kzzzzzzzzzzzzzzk",
    "kkkkkkkkkkkkkkkk",
];
pub const TURRET_KEYS: [(char, u8); 6] = [('k', INK), ('z', STEEL_LO), ('s', STEEL), ('S', STEEL_HI), ('r', RED), ('R', RED_HI)];

/// 地堡：关着、开着两张。开着能打，打坏了掉勋章。
pub const PILLBOX: [[&str; 16]; 2] = [
    [
        "....kkkkkkkk....",
        "..kksSSSSSSskk..",
        ".ksSSSSSSSSSSsk.",
        ".kSSssssssssSSk.",
        "kSSskkkkkkkksSSk",
        "kSskzzzzzzzzksSk",
        "kSskzkzkzkzkksSk",
        "kSskzzzzzzzzksSk",
        "kSskzkzkzkzkksSk",
        "kSskzzzzzzzzksSk",
        "kSSskkkkkkkksSSk",
        "kSSSssssssssSSSk",
        "kssSSSSSSSSSSssk",
        "kzzssssssssssszk",
        "kzzzzzzzzzzzzzzk",
        "kkkkkkkkkkkkkkkk",
    ],
    [
        "....kkkkkkkk....",
        "..kksSSSSSSskk..",
        ".ksSSSSSSSSSSsk.",
        ".kSSssssssssSSk.",
        "kSSskkkkkkkksSSk",
        "kSsk..kkkk..ksSk",
        "kSsk.krrrrk.ksSk",
        "kSsk.krRwrk.ksSk",
        "kSsk.krRRrk.ksSk",
        "kSsk..kkkk..ksSk",
        "kSSskkkkkkkksSSk",
        "kSSSssssssssSSSk",
        "kssSSSSSSSSSSssk",
        "kzzssssssssssszk",
        "kzzzzzzzzzzzzzzk",
        "kkkkkkkkkkkkkkkk",
    ],
];

/// 飞过去的补给舱：16×10，两边有小翅膀。
pub const POD: [[&str; 10]; 2] = [
    [
        "......kkkk......",
        "....kksSSskk....",
        "kk.ksSSSSSSsk.kk",
        "kSkkSSrrrrSSkkSk",
        "kSSksSrRRrSsksSk",
        "kSSksSrrrrSsksSk",
        "kSkkssSSSSsskkSk",
        "kk.ksssssssk..kk",
        "....kkssskk.....",
        "......kkk.......",
    ],
    [
        "......kkkk......",
        "....kksSSskk....",
        "...ksSSSSSSsk...",
        "kkkkSSrrrrSSkkkk",
        "kSSksSrRRrSsksSk",
        "kSSksSrrrrSsksSk",
        "kkkkssSSSSsskkkk",
        "...ksssssssk....",
        "....kkssskk.....",
        "......kkk.......",
    ],
];

/// 勋章：14×12，翅膀加中间一块牌子，牌子上印字母（字母另外画）。
pub const MEDAL: [&str; 12] = [
    "....kkkkkk....",
    "kk.kyyyyyyk.kk",
    "kwkkyYYYYykkwk",
    "kwwkyYyyyykkwk",
    "kwwkyYyyyykwwk",
    ".kwkyYyyyykwk.",
    ".kwkyYyyyykwk.",
    "..kkyYyyyykk..",
    "...kyyyyyyk...",
    "...kooooook...",
    "....koooook...",
    ".....kkkkk....",
];
pub const MEDAL_KEYS: [(char, u8); 5] = [('k', INK), ('y', YELLOW), ('Y', YELLOW_HI), ('o', ORANGE), ('w', WHITE)];

/// 勋章上的字母，5×5：S 散弹、M 机枪、L 激光、F 火球、R 快、B 护盾。
pub const LETTERS: [(char, [&str; 5]); 6] = [
    ('S', [".xxx.", "x....", ".xxx.", "....x", "xxxx."]),
    ('M', ["x...x", "xx.xx", "x.x.x", "x...x", "x...x"]),
    ('L', ["x....", "x....", "x....", "x....", "xxxxx"]),
    ('F', ["xxxxx", "x....", "xxxx.", "x....", "x...."]),
    ('R', ["xxxx.", "x...x", "xxxx.", "x..x.", "x...x"]),
    ('B', ["xxxx.", "x...x", "xxxx.", "x...x", "xxxx."]),
];

/// 滚下来的石头：16×16。
pub const BOULDER: [&str; 16] = [
    ".....kkkkkk.....",
    "...kkSSSSssskk..",
    "..kSSSSssssssk..",
    ".kSSSsssssszssk.",
    ".kSSssssssszzsk.",
    "kSSsssszsssssszk",
    "kSssssszzsssszzk",
    "kSssssssssssszzk",
    "ksssszssssssszzk",
    "ksssszzsssssszzk",
    "kssssssssszzzzzk",
    ".ksssssssszzzzk.",
    ".kzssssszzzzzzk.",
    "..kzzzzzzzzzzk..",
    "...kkzzzzzzkk...",
    ".....kkkkkk.....",
];
pub const ROCK_KEYS: [(char, u8); 4] = [('k', INK), ('S', FILL_HI), ('s', FILL), ('z', FILL_LO)];

/// 坦克：40×24，炮管往左。
pub const TANK: [&str; 24] = [
    "........................................",
    "........................................",
    "..................kkkkkkkkkk............",
    ".................kSSSSSSSSSSk...........",
    "................kSSsssssssssSk..........",
    "kkkkkkkkkkkkkkkkkSssssssssssssk.........",
    "kSSSSSSSSSSSSSSSkSssssskkksssssk........",
    "kssssssssssssssskSssssskrkssssssk.......",
    "kkkkkkkkkkkkkkkkkSssssskkksssssssk......",
    "................kzsssssssssssssssk......",
    "..........kkkkkkkzzzzzzzzzzzzzzzzzkkkkk.",
    ".........kSSSSSSSSSSSSSSSSSSSSSSSSSSSSSk",
    "........kSssssssssssssssssssssssssssssssk",
    ".......kSsssssssssssssssssssssssssssssssk",
    ".......kzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzk",
    "......kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk",
    ".....kgggggggggggggggggggggggggggggggggk",
    "....kgkkkgkkkgkkkgkkkgkkkgkkkgkkkgkkkgk.",
    "....kgkSkgkSkgkSkgkSkgkSkgkSkgkSkgkSkgk.",
    "....kgkkkgkkkgkkkgkkkgkkkgkkkgkkkgkkkgk.",
    ".....kgggggggggggggggggggggggggggggggk..",
    "......kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk..",
    "........................................",
    "........................................",
];
pub const TANK_KEYS: [(char, u8); 6] = [('k', INK), ('S', GREEN_HI), ('s', GREEN), ('z', GREEN_LO), ('g', GRAY), ('r', RED)];

// ---------- 地图块 ----------

pub const TILE_KEYS: [(char, u8); 10] = [('k', INK), ('t', TOP), ('T', TOP_HI), ('d', TOP_LO), ('f', FILL), ('F', FILL_HI), ('l', FILL_LO), ('w', WATER), ('W', WATER_HI), ('o', WATER_LO)];

/// 能站上去的台子顶：草、雪、铁格板（看每关的颜色）。从下面能跳上来。
pub const T_LEDGE: [&str; 16] = [
    "TTtTTTTtTTTTTtTT",
    "tTttTtttTttTtttT",
    "ttttdtttttdttttt",
    "tdtttdttdtttdttd",
    "dkdttkdddtkdtdkd",
    "kfkdkfkkdkfkdkfk",
    "fffkfffffkfffkff",
    "fFfffffFffffFfff",
    "ffflfffffflfffff",
    "fflfffFfffflfffF",
    "ffffffffffffffff",
    "flffffflfFffflff",
    "ffffFffffffffffl",
    "fFfflffffffFffff",
    "ffffffffflffffff",
    "lfffffFfffffflff",
];
/// 台子下面的土、石头、铁墙。
pub const T_FILL: [&str; 16] = [
    "ffffFffffffffffl",
    "fFfflffffffFffff",
    "ffffffffflffffff",
    "lfffffFfffffflff",
    "fffFfffflfffffff",
    "ffffflffffffFfff",
    "fFffffffffflffff",
    "ffflfffFffffffff",
    "ffffffffffffflff",
    "flffFfffflffffff",
    "fffffffffffFffff",
    "ffFfflffffffffll",
    "ffffffffFfffffff",
    "lffffffflffffFff",
    "fffFffffffffffff",
    "ffffflfffffflfff",
];
/// 桥：一节一节，主角走上去就一节一节炸掉。
pub const T_BRIDGE: [&str; 16] = [
    "kkkkkkkkkkkkkkkk",
    "TTTTTTTTTTTTTTTT",
    "tttttttttttttttt",
    "dkdkdkdkdkdkdkdk",
    "kk.kk.kk.kk.kk.k",
    "k...k..k...k...k",
    "k..k....k...k..k",
    "k.k......k...k.k",
    "kk........k...kk",
    "kkkkkkkkkkkkkkkk",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];
