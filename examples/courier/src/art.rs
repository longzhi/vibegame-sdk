//! 调色板和精灵。全是自己画的字符画。
//!
//! 主角小邮差：深蓝帽子上一块金色徽章，浅蓝衬衫，斜挎一个棕色邮包，红鞋。
//! 敌人：发条兵、罐头虫、乌鸦、刺海胆、烟囱里的煤球、关底的邮件大盗。

use vibegame::full::Sprite;

// ---------- 调色板的格 ----------
pub const SKY: u8 = 0;
pub const INK: u8 = 1;
pub const WHITE: u8 = 2;
pub const CAP: u8 = 3;
pub const CAP_HI: u8 = 4;
pub const BADGE: u8 = 5;
pub const SKIN: u8 = 6;
pub const SKIN_LO: u8 = 7;
pub const HAIR: u8 = 8;
pub const SHIRT: u8 = 9;
pub const SHIRT_LO: u8 = 10;
pub const BAG: u8 = 11;
pub const BAG_LO: u8 = 12;
pub const PANTS: u8 = 13;
pub const SHOE: u8 = 14;
pub const SHOE_HI: u8 = 15;
pub const RED: u8 = 16;
pub const RED_HI: u8 = 17;
pub const RED_LO: u8 = 18;
pub const GOLD: u8 = 19;
pub const GOLD_HI: u8 = 20;
pub const GOLD_LO: u8 = 21;
pub const WOOD: u8 = 22;
pub const WOOD_HI: u8 = 23;
pub const WOOD_LO: u8 = 24;
pub const STONE: u8 = 25;
pub const STONE_HI: u8 = 26;
pub const STONE_LO: u8 = 27;
pub const GRAY: u8 = 28;
pub const GRAY_LO: u8 = 29;
pub const NAVY: u8 = 30;
pub const BLUE: u8 = 31;
pub const BLUE_HI: u8 = 32;
pub const GREEN: u8 = 33;
pub const GREEN_HI: u8 = 34;
pub const GREEN_LO: u8 = 35;
pub const PURPLE: u8 = 36;
pub const PURPLE_HI: u8 = 37;
pub const PINK: u8 = 38;
pub const ORANGE: u8 = 39;
pub const CREAM: u8 = 40;
pub const TIN: u8 = 41;
pub const TIN_HI: u8 = 42;
pub const TIN_LO: u8 = 43;
// 背景：每关换一组（看 `Theme`）。
pub const BG_FAR: u8 = 44;
pub const BG_NEAR: u8 = 45;
pub const BG_LIGHT: u8 = 46;
pub const SKY2: u8 = 47;
pub const SKY3: u8 = 48;
pub const CLOUD: u8 = 49;
pub const CLOUD_LO: u8 = 50;
pub const WATER: u8 = 51;
pub const WATER_HI: u8 = 52;
// 地面：每关换一组。
pub const GROUND: u8 = 53;
pub const GROUND_HI: u8 = 54;
pub const GROUND_LO: u8 = 55;
pub const TOP: u8 = 56;
pub const TOP_HI: u8 = 57;
pub const BRICK: u8 = 58;
pub const BRICK_HI: u8 = 59;
pub const BRICK_LO: u8 = 60;
pub const STEAM: u8 = 61;

pub const PALETTE: [(u8, [u8; 3]); 62] = [
    (SKY, [0x6c, 0xb4, 0xf0]),
    (INK, [0x10, 0x10, 0x18]),
    (WHITE, [0xfc, 0xfc, 0xf8]),
    (CAP, [0x24, 0x34, 0x7c]),
    (CAP_HI, [0x44, 0x60, 0xb8]),
    (BADGE, [0xff, 0xd0, 0x30]),
    (SKIN, [0xf8, 0xc4, 0x98]),
    (SKIN_LO, [0xd0, 0x8c, 0x68]),
    (HAIR, [0x5c, 0x34, 0x1c]),
    (SHIRT, [0x8c, 0xc8, 0xf4]),
    (SHIRT_LO, [0x50, 0x90, 0xd0]),
    (BAG, [0xa8, 0x68, 0x30]),
    (BAG_LO, [0x6c, 0x3c, 0x18]),
    (PANTS, [0x34, 0x3c, 0x54]),
    (SHOE, [0xd8, 0x30, 0x30]),
    (SHOE_HI, [0xff, 0xf0, 0xe0]),
    (RED, [0xd8, 0x30, 0x30]),
    (RED_HI, [0xff, 0x7c, 0x70]),
    (RED_LO, [0x84, 0x18, 0x20]),
    (GOLD, [0xf8, 0xb8, 0x20]),
    (GOLD_HI, [0xff, 0xe8, 0x80]),
    (GOLD_LO, [0xb0, 0x70, 0x10]),
    (WOOD, [0xc0, 0x78, 0x38]),
    (WOOD_HI, [0xe8, 0xa8, 0x60]),
    (WOOD_LO, [0x70, 0x40, 0x18]),
    (STONE, [0x9c, 0x98, 0xa8]),
    (STONE_HI, [0xd4, 0xd0, 0xdc]),
    (STONE_LO, [0x5c, 0x58, 0x6c]),
    (GRAY, [0x80, 0x80, 0x88]),
    (GRAY_LO, [0x48, 0x48, 0x54]),
    (NAVY, [0x1c, 0x24, 0x48]),
    (BLUE, [0x30, 0x70, 0xd0]),
    (BLUE_HI, [0x80, 0xc0, 0xff]),
    (GREEN, [0x40, 0xa8, 0x48]),
    (GREEN_HI, [0x90, 0xdc, 0x70]),
    (GREEN_LO, [0x1c, 0x64, 0x30]),
    (PURPLE, [0x80, 0x48, 0xb8]),
    (PURPLE_HI, [0xc0, 0x90, 0xf0]),
    (PINK, [0xff, 0x8c, 0xb8]),
    (ORANGE, [0xf8, 0x88, 0x28]),
    (CREAM, [0xf4, 0xe4, 0xbc]),
    (TIN, [0xb4, 0xb8, 0xc4]),
    (TIN_HI, [0xec, 0xf0, 0xf8]),
    (TIN_LO, [0x6c, 0x70, 0x80]),
    (BG_FAR, [0xa8, 0xd4, 0xf4]),
    (BG_NEAR, [0x88, 0xb8, 0xe0]),
    (BG_LIGHT, [0xff, 0xe8, 0x98]),
    (SKY2, [0x8c, 0xc8, 0xf8]),
    (SKY3, [0xb0, 0xdc, 0xfc]),
    (CLOUD, [0xfc, 0xfc, 0xf8]),
    (CLOUD_LO, [0xc8, 0xdc, 0xf0]),
    (WATER, [0x2c, 0x5c, 0x9c]),
    (WATER_HI, [0x70, 0xa8, 0xe0]),
    (GROUND, [0xb8, 0x80, 0x50]),
    (GROUND_HI, [0xdc, 0xa8, 0x78]),
    (GROUND_LO, [0x7c, 0x50, 0x2c]),
    (TOP, [0xb4, 0xb4, 0xbc]),
    (TOP_HI, [0xe0, 0xe0, 0xe8]),
    (BRICK, [0xb8, 0x58, 0x38]),
    (BRICK_HI, [0xe0, 0x88, 0x60]),
    (BRICK_LO, [0x6c, 0x2c, 0x1c]),
    (STEAM, [0xe8, 0xf0, 0xf8]),
];

/// 一关的样子：天、远景、地面的颜色。画的时候改调色板里这几格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// 白天的街道：蓝天、远处一排房子、灰色人行道。
    Street,
    /// 下水道：黑底、蓝灰色的砖。
    Sewer,
    /// 傍晚的屋顶：橙红的天、红瓦。
    Roof,
    /// 夜里的港口：深蓝的天、月亮、灯。
    Harbor,
    /// 夜里的屋顶：紫色的天、星星。
    NightRoof,
    /// 钟楼里：灰石头、齿轮。
    Tower,
}

impl Theme {
    /// 这几格换成什么颜色：天三层、远景两层、灯、云两层、水两层、地面五格、砖三格。
    pub fn colors(self) -> [(u8, [u8; 3]); 18] {
        let c = |v: [u32; 18]| {
            let idx = [SKY, SKY2, SKY3, BG_FAR, BG_NEAR, BG_LIGHT, CLOUD, CLOUD_LO, WATER, WATER_HI, GROUND, GROUND_HI, GROUND_LO, TOP, TOP_HI, BRICK, BRICK_HI, BRICK_LO];
            let mut out = [(0u8, [0u8; 3]); 18];
            for i in 0..18 {
                out[i] = (idx[i], [(v[i] >> 16) as u8, (v[i] >> 8) as u8, v[i] as u8]);
            }
            out
        };
        match self {
            Theme::Street => c([
                0x6cb4f0, 0x8cc8f8, 0xb0dcfc, 0xa8c8e8, 0x7898c8, 0xffe898, 0xfcfcf8, 0xc8dcf0, 0x2c5c9c, 0x70a8e0, 0xb88050, 0xdca878, 0x7c502c, 0xb4b4bc, 0xe0e0e8, 0xb85838, 0xe08860,
                0x6c2c1c,
            ]),
            Theme::Sewer => c([
                0x08080c, 0x0c0c14, 0x10101c, 0x1c2030, 0x283048, 0x6cc070, 0x384058, 0x283048, 0x2c6c40, 0x60b070, 0x485878, 0x7088a8, 0x283450, 0x5c6c8c, 0x90a0c0, 0x506080, 0x7890b0,
                0x283048,
            ]),
            Theme::Roof => c([
                0xf07848, 0xf8a058, 0xffc870, 0x9c4860, 0x6c3050, 0xffe090, 0xffd0a0, 0xe89078, 0x2c5c9c, 0x70a8e0, 0x8c4430, 0xc06848, 0x5c2418, 0xc04838, 0xf07858, 0xb85838, 0xe08860,
                0x6c2c1c,
            ]),
            Theme::Harbor => c([
                0x101838, 0x18244c, 0x243464, 0x283860, 0x18203c, 0xffd860, 0x485880, 0x303c60, 0x1c3c78, 0x5088d0, 0x6c5848, 0x9c8068, 0x3c3028, 0x7c7068, 0xb0a498, 0x804838, 0xb07058,
                0x482018,
            ]),
            Theme::NightRoof => c([
                0x281848, 0x3c2460, 0x583478, 0x40285c, 0x281838, 0xfff0b0, 0x6c5890, 0x4c3c70, 0x1c3c78, 0x5088d0, 0x5c3448, 0x8c5068, 0x341c2c, 0x7c4058, 0xb06880, 0x804838, 0xb07058,
                0x482018,
            ]),
            Theme::Tower => c([
                0x201c24, 0x2c2830, 0x38343c, 0x484450, 0x302c38, 0xff9c40, 0x585460, 0x403c48, 0xc04020, 0xff9040, 0x6c6878, 0x9c98a8, 0x3c3848, 0x807c8c, 0xb4b0c0, 0x705c50, 0x988070,
                0x40342c,
            ]),
        }
    }
}

// ---------- 主角 ----------

pub const HERO_KEYS: [(char, u8); 13] = [
    ('c', CAP),
    ('C', CAP_HI),
    ('y', BADGE),
    ('h', HAIR),
    ('s', SKIN),
    ('S', SKIN_LO),
    ('k', INK),
    ('j', SHIRT),
    ('J', SHIRT_LO),
    ('b', BAG),
    ('B', BAG_LO),
    ('p', PANTS),
    ('f', SHOE),
];

const HEAD: [&str; 8] = [
    "................",
    "......ccccc.....",
    ".....cCCCCcc....",
    "....ccccccycc...",
    "....cccccccccccc",
    "....hhssskss....",
    "....hsssssssss..",
    ".....hsssssSS...",
];

const SMALL_BODY: [&str; 4] = ["......jjjjB.....", "....bjjjjBjs....", "...bbbjjBjjs....", "...bBbjBjjjj...."];
const SMALL_LEGS: [[&str; 4]; 5] = [
    // 站
    ["....ppppppp.....", "....ppp..ppp....", "...fff....fff...", "...ffff...ffff.."],
    // 走一
    ["....ppppppp.....", "...ppp....pp....", "..fff......ff...", "..ff.......fff.."],
    // 走二（两脚并拢）
    ["....pppppp......", ".....pppp.......", ".....ffff.......", ".....fffff......"],
    // 走三
    ["....ppppppp.....", ".....pp.ppp.....", "....fff..fff....", "...fff....ff...."],
    // 跳：一条腿收起来
    ["....pppppppp....", "...pp....ppp....", "..ff.......fff..", "..f..........f.."],
];

/// 把几段字符画的行接成一张。
fn stack<'a>(parts: &[&[&'a str]]) -> Vec<&'a str> {
    parts.iter().flat_map(|p| p.iter().copied()).collect()
}

/// 小个子：站、走三张、跳、刹车。
pub fn hero_small() -> Vec<Sprite> {
    let mut out = Vec::new();
    for legs in &SMALL_LEGS {
        let rows = stack(&[&HEAD, &SMALL_BODY, legs]);
        out.push(Sprite::art(&rows, &HERO_KEYS));
    }
    // 跳的那张举起一只手。
    let mut jump = stack(&[&HEAD, &SMALL_BODY, &SMALL_LEGS[4]]);
    jump[7] = ".....hsssssSS.ss";
    jump[8] = "......jjjjBjjs..";
    jump[9] = "....bjjjjBjj....";
    out[4] = Sprite::art(&jump, &HERO_KEYS);
    // 刹车：身子往后仰，脚往前蹬。
    let skid = stack(&[
        &HEAD,
        &["......jjjjB.....", "...sbjjjjBj.....", "...bbbjjBjj.....", "...bBbjBjjjj...."],
        &["....ppppppp.....", "....pp...pppp...", "...ff.......fff.", "..fff.......fff."],
    ]);
    out.push(Sprite::art(&skid, &HERO_KEYS));
    out
}

const BIG_TORSO: [&str; 12] = [
    "......ssss......",
    ".....jjjjjjB....",
    "....jjjjjjBjj...",
    "...jjjjjjBjjjj..",
    "...jJjjjBjjjJj..",
    "...jJjjBjjjjJj..",
    "..bbbBjjjjjjJs..",
    "..bbbbjjjjjjjs..",
    "..bBbbJJJJJJJ...",
    "..bbbbpppppp....",
    "...bbppppppp....",
    "....ppppppppp...",
];
const BIG_LEGS: [[&str; 12]; 5] = [
    [
        "....pppp.pppp...",
        "....ppp...ppp...",
        "....ppp...ppp...",
        "....ppp...ppp...",
        "....ppp...ppp...",
        "....ppp...ppp...",
        "....ppp...ppp...",
        "....ppp...ppp...",
        "....ppp...ppp...",
        "...ffff..ffff...",
        "...fffff.fffff..",
        "................",
    ],
    [
        "....pppp..ppp...",
        "...pppp....ppp..",
        "...ppp.....ppp..",
        "..ppp.......ppp.",
        "..ppp.......ppp.",
        "..ppp........pp.",
        ".ppp.........pp.",
        ".ppp.........ff.",
        "ffff.........fff",
        "ffff..........ff",
        "................",
        "................",
    ],
    [
        "....ppppppp.....",
        ".....ppppp......",
        ".....ppppp......",
        ".....pppp.......",
        ".....pppp.......",
        ".....pppp.......",
        ".....pppp.......",
        ".....pppp.......",
        ".....pppp.......",
        "....fffff.......",
        "....ffffff......",
        "................",
    ],
    [
        "....pppp.ppp....",
        "....ppp..ppp....",
        "....ppp...ppp...",
        "...ppp....ppp...",
        "...ppp.....ppp..",
        "...ppp.....ppp..",
        "...ppp.....ppp..",
        "..ppp.......pp..",
        "..fff.......fff.",
        ".ffff.......ffff",
        "................",
        "................",
    ],
    [
        "....pppppppp....",
        "...ppp....ppp...",
        "..ppp......ppp..",
        "..pp........pp..",
        ".ppp........ppp.",
        ".ff..........ff.",
        ".fff.........fff",
        "................",
        "................",
        "................",
        "................",
        "................",
    ],
];

/// 大个子（16×32）：站、走三张、跳、刹车、蹲。
pub fn hero_big() -> Vec<Sprite> {
    let mut out = Vec::new();
    for legs in &BIG_LEGS {
        let rows = stack(&[&HEAD, &BIG_TORSO, legs]);
        out.push(Sprite::art(&rows, &HERO_KEYS));
    }
    let mut jump = stack(&[&HEAD, &BIG_TORSO, &BIG_LEGS[4]]);
    jump[6] = "....hsssssssss.s";
    jump[7] = ".....hsssssSS.ss";
    jump[8] = "......ssss...js.";
    jump[9] = ".....jjjjjjBjj..";
    out[4] = Sprite::art(&jump, &HERO_KEYS);
    let mut skid = stack(&[&HEAD, &BIG_TORSO, &BIG_LEGS[0]]);
    for (i, r) in [
        "....pppp..pppp..",
        "....ppp....ppp..",
        "...ppp......ppp.",
        "...ppp......ppp.",
        "..ppp........pp.",
        "..ppp........pp.",
        "..pp.........pp.",
        "..pp........fff.",
        ".fff........ffff",
        ".ffff...........",
        "................",
        "................",
    ]
    .iter()
    .enumerate()
    {
        skid[20 + i] = r;
    }
    out.push(Sprite::art(&skid, &HERO_KEYS));
    // 蹲：头压低，身子缩成一团（还是 16×32，上面空着）。
    let mut duck: Vec<&str> = vec!["................"; 10];
    duck.extend(stack(&[&HEAD, &["....bjjjjjjBs...", "...bbbjjjjBjs...", "...bBbjjjBjjj...", "...bbbpppppppp..", "...pppppppppppp.", "...ff.pppppp.ff.", "..fff........fff", "................"][..], &["................", "................"][..]]));
    duck.truncate(32);
    out.push(Sprite::art(&duck, &HERO_KEYS));
    out
}

/// 倒下：正面，两手举起来，眼睛打叉。
pub const HERO_DIE: [&str; 16] = [
    "................",
    "......ccccc.....",
    ".....cCCCCCc....",
    "....cccccyccc...",
    "...cccccccccccc.",
    "..s.hssssssh.s..",
    "..s.skskksks.s..",
    "..ss.sskkss.ss..",
    "...sjjjjjjjjs...",
    "....jjBjjBjj....",
    "....bbjBBjjj....",
    "....pppppppp....",
    "....ppp..ppp....",
    "....ppp..ppp....",
    "...fff....fff...",
    "...fff....fff...",
];

// ---------- 敌人（都朝左画，往右走时翻过来） ----------

/// 发条兵：高帽子，蓝外套，背上一把金钥匙。
pub const WALKER: [[&str; 16]; 2] = [
    [
        "......rrrr......",
        ".....rrRRrr.....",
        ".....rRRRRr.....",
        ".....rrrrrr.....",
        "....kkkkkkkk....",
        ".....ssssss.....",
        ".....kssksS.....",
        ".....ssssss.....",
        "....bbbbbbbbg...",
        "...sbbyybbbbgg..",
        "...sbbbbbbbbg...",
        "....bbyybbbb....",
        "....ww....ww....",
        "....ww....ww....",
        "...kkk....kkk...",
        "...kkk....kkk...",
    ],
    [
        "......rrrr......",
        ".....rrRRrr.....",
        ".....rRRRRr.....",
        ".....rrrrrr.....",
        "....kkkkkkkk....",
        ".....ssssss.....",
        ".....kssksS.....",
        ".....ssssss.....",
        "....bbbbbbbb.g..",
        "...sbbyybbbbggg.",
        "...sbbbbbbbb.g..",
        "....bbyybbbb....",
        ".....ww..ww.....",
        "......wwww......",
        ".....kkkkk......",
        "....kkk.kkk.....",
    ],
];
pub const WALKER_FLAT: [&str; 6] = ["...rrrrrrrrrr...", "..kkkkkkkkkkkk..", "..ssksssssksss..", "..bbbbyybbbbbbg.", "..bbbbbbbbbbbbgg", "..kkkk....kkkk.."];
pub const WALKER_KEYS: [(char, u8); 8] = [('r', RED), ('R', RED_HI), ('k', INK), ('s', SKIN), ('S', SKIN_LO), ('b', BLUE), ('y', GOLD), ('g', GOLD_HI)];

/// 罐头虫：一只铁皮罐头，标签上两只眼睛，头上两根须，底下几条腿。
pub const CANBUG: [[&str; 16]; 2] = [
    [
        "...k......k.....",
        "....k....k......",
        "....gggggggg....",
        "...gGGGGGGGGg...",
        "...gwwwwwwwwg...",
        "...gwkkwwkkwg...",
        "...gwkwwwkwwg...",
        "...gwwwwwwwwg...",
        "...grrrrrrrrg...",
        "...gRRRRRRRRg...",
        "...gwwwwwwwwg...",
        "...gGGGGGGGGg...",
        "....gggggggg....",
        "...k.k....k.k...",
        "..k..k....k..k..",
        "..k...k..k...k..",
    ],
    [
        "....k.....k.....",
        "....k....k......",
        "....gggggggg....",
        "...gGGGGGGGGg...",
        "...gwwwwwwwwg...",
        "...gwkkwwkkwg...",
        "...gwkwwwkwwg...",
        "...gwwwwwwwwg...",
        "...grrrrrrrrg...",
        "...gRRRRRRRRg...",
        "...gwwwwwwwwg...",
        "...gGGGGGGGGg...",
        "....gggggggg....",
        "....k.k..k.k....",
        "...k..k..k..k...",
        "....k.k..k.k....",
    ],
];
/// 缩进罐头里：盖子扣上，没有腿。`peek` 那张露出眼睛（快出来了）。
pub const CAN: [[&str; 16]; 2] = [
    [
        "................",
        "................",
        "................",
        "....gggggggg....",
        "...gGGGGGGGGg...",
        "...gTTTTTTTTg...",
        "...gwwwwwwwwg...",
        "...gwwwwwwwwg...",
        "...grrrrrrrrg...",
        "...gRRRRRRRRg...",
        "...gwwwwwwwwg...",
        "...gwwwwwwwwg...",
        "...gTTTTTTTTg...",
        "...gGGGGGGGGg...",
        "....gggggggg....",
        "................",
    ],
    [
        "................",
        "................",
        "................",
        "....gggggggg....",
        "...gGGGGGGGGg...",
        "...gTTTTTTTTg...",
        "...gwkkwwkkwg...",
        "...gwwwwwwwwg...",
        "...grrrrrrrrg...",
        "...gRRRRRRRRg...",
        "...gwwwwwwwwg...",
        "...gwwwwwwwwg...",
        "...gTTTTTTTTg...",
        "...gGGGGGGGGg...",
        "....gggggggg....",
        "...k.k....k.k...",
    ],
];
pub const CAN_KEYS: [(char, u8); 8] = [('k', INK), ('g', TIN_LO), ('G', TIN_HI), ('T', TIN), ('w', CREAM), ('r', RED), ('R', RED_HI), ('.', 0)];

/// 乌鸦：两张，翅膀上、下。
pub const CROW: [[&str; 16]; 2] = [
    [
        "........kk......",
        ".......kkkk.....",
        "......kkKKkk....",
        ".....kkKKKKkk...",
        "...kkkkkKKKkk...",
        "..kkkkkkkKkkk...",
        ".yykrkkkkkkkkk..",
        "yyykkkkkkkkkkkkk",
        "..kkkkkkkkkkkkk.",
        "....kkkkkkkkk...",
        ".....kkkkkk.....",
        "......y..y......",
        "......y..y......",
        "................",
        "................",
        "................",
    ],
    [
        "................",
        "................",
        "................",
        "................",
        "...kkkk.........",
        "..kkkkkk........",
        ".yykrkkkkkkkkk..",
        "yyykkkkkkkkkkkkk",
        "..kkkkkKKKkkkkk.",
        "....kkKKKKKkk...",
        ".....kKKKKkk....",
        "......kKKkk.....",
        "......ykk.y.....",
        "......y..y......",
        "................",
        "................",
    ],
];
pub const CROW_KEYS: [(char, u8); 4] = [('k', NAVY), ('K', GRAY_LO), ('y', GOLD), ('r', RED)];

/// 刺海胆：紫色的一团，一身白刺，踩不得。
pub const SPIKY: [[&str; 16]; 2] = [
    [
        "......w..w......",
        "..w...w..w...w..",
        "...w.pppppp.w...",
        "....ppPPPPpp....",
        "ww.pPPPPPPPPp.ww",
        "..ppPPPPPPPPpp..",
        "..pPkkPPPkkPPp..",
        "wwpPkwPPPkwPPpww",
        "..pPPPPPPPPPPp..",
        "..ppPPkkkkPPpp..",
        "ww.pPPPPPPPPp.ww",
        "....pppppppp....",
        "...w.pppppp.w...",
        "..w..o....o..w..",
        "....oo....oo....",
        "................",
    ],
    [
        ".......w..w.....",
        "...w...w..w..w..",
        "....w.pppppp.w..",
        "....ppPPPPpp....",
        ".w.pPPPPPPPPp.w.",
        "w.ppPPPPPPPPpp.w",
        "..pPkkPPPkkPPp..",
        ".wpPkwPPPkwPPpw.",
        "w.pPPPPPPPPPPp.w",
        "..ppPPkkkkPPpp..",
        ".w.pPPPPPPPPp.w.",
        "....pppppppp....",
        "..w..pppppp..w..",
        "....o......o....",
        "...oo......oo...",
        "................",
    ],
];
pub const SPIKY_KEYS: [(char, u8); 5] = [('w', WHITE), ('p', PURPLE), ('P', PURPLE_HI), ('k', INK), ('o', ORANGE)];

/// 煤球：黑乎乎一团，白眼睛，从烟囱里冒出来。
pub const SOOT: [[&str; 16]; 2] = [
    [
        "....k..k..k.....",
        ".....kkkkkk.....",
        "...kkkkkkkkkk...",
        "..kkkkkkkkkkkk..",
        ".kkkwwkkkkwwkkk.",
        ".kkkwrkkkkwrkkk.",
        "kkkkwwkkkkwwkkkk",
        "kkkkkkkkkkkkkkkk",
        "kkkkkkrrrrkkkkkk",
        ".kkkkkkkkkkkkkk.",
        ".kkkkkkkkkkkkkk.",
        "..kkkkkkkkkkkk..",
        "...kkkkkkkkkk...",
        ".....kkkkkk.....",
        "................",
        "................",
    ],
    [
        "...k..k..k..k...",
        ".....kkkkkk.....",
        "...kkkkkkkkkk...",
        "..kkkkkkkkkkkk..",
        ".kkkwwkkkkwwkkk.",
        ".kkkrwkkkkrwkkk.",
        "kkkkwwkkkkwwkkkk",
        "kkkkkkkkkkkkkkkk",
        "kkkkkrrrrrrkkkkk",
        ".kkkkkkkkkkkkkk.",
        ".kkkkkkkkkkkkkk.",
        "..kkkkkkkkkkkk..",
        "...kkkkkkkkkk...",
        ".....kkkkkk.....",
        "................",
        "................",
    ],
];
pub const SOOT_KEYS: [(char, u8); 3] = [('k', INK), ('w', WHITE), ('r', ORANGE)];

/// 邮件大盗（32×32）：黑眼罩、条纹衫、背一只鼓鼓的麻袋。两张：站、跳起来扔信。
pub const BOSS: [[&str; 32]; 2] = [
    [
        "................................",
        "..............kkkkkk............",
        "............kkccccccKk..........",
        "...........kccccccccccK.........",
        "..........kccccccccccccK........",
        "..........kkkkkkkkkkkkkkkk......",
        "..........kssssssssssssk........",
        ".........kskkkkkkkkkkkksk.......",
        ".........ksk.wkkkk.wkkksk.......",
        ".........kskkkkkkkkkkkksk.......",
        "..........kssssssssssssk........",
        "..........ksssshhhhsssk.........",
        "...........kssssssssssk.........",
        "............kkkkkkkkkk..........",
        "..........kwwwwwwwwwwwwk........",
        "...nnnn..kwkkkkkkkkkkkkwk.......",
        "..nNNNNn.kwwwwwwwwwwwwwwkss.....",
        ".nNNNNNNnkkkkkkkkkkkkkkkkss.....",
        ".nNNNNNNNkwwwwwwwwwwwwwwk.......",
        ".nNNNNNNNkkkkkkkkkkkkkkkk.......",
        ".nNNNNNNnkwwwwwwwwwwwwwwk.......",
        "..nNNNNn.kkkkkkkkkkkkkkkk.......",
        "...nnnn...kppppppppppppk........",
        "..........kppppppppppppk........",
        "..........kpppppk.kppppk........",
        "..........kpppppk.kppppk........",
        "..........kpppppk.kppppk........",
        "..........kpppppk.kppppk........",
        ".........kkkkkkk...kkkkkk.......",
        "........kffffffk...kffffffk.....",
        "........kkkkkkkk...kkkkkkkk.....",
        "................................",
    ],
    [
        "................................",
        "..............kkkkkk............",
        "............kkccccccKk..........",
        "...........kccccccccccK.........",
        "..........kccccccccccccK........",
        "..........kkkkkkkkkkkkkkkk......",
        "..........kssssssssssssk........",
        ".........kskkkkkkkkkkkksk..ss...",
        ".........ksk.wkkkk.wkkksk.ss....",
        ".........kskkkkkkkkkkkksk.s.....",
        "..........kssssssssssssk.ss.....",
        "..........ksssshhhhsssk.kk......",
        "...........kssssssssssk.kw......",
        "............kkkkkkkkkk.kw.......",
        "..........kwwwwwwwwwwwwkw.......",
        "...nnnn..kwkkkkkkkkkkkkwk.......",
        "..nNNNNn.kwwwwwwwwwwwwwwk.......",
        ".nNNNNNNnkkkkkkkkkkkkkkkk.......",
        ".nNNNNNNNkwwwwwwwwwwwwwwk.......",
        ".nNNNNNNNkkkkkkkkkkkkkkkk.......",
        ".nNNNNNNnkwwwwwwwwwwwwwwk.......",
        "..nNNNNn.kkkkkkkkkkkkkkkk.......",
        "...nnnn...kppppppppppppk........",
        "..........kppppppppppppk........",
        ".........kpppppk...kppppk.......",
        "........kpppppk.....kppppk......",
        ".......kpppppk.......kppppk.....",
        "......kkkkkkk.........kkkkkk....",
        ".....kffffffk.........kffffffk..",
        ".....kkkkkkkk.........kkkkkkkk..",
        "................................",
        "................................",
    ],
];
pub const BOSS_KEYS: [(char, u8); 10] = [('k', INK), ('c', GRAY_LO), ('K', GRAY), ('s', SKIN), ('w', WHITE), ('h', SKIN_LO), ('n', WOOD_LO), ('N', WOOD), ('p', NAVY), ('f', RED)];

// ---------- 道具 ----------

/// 加急包裹：牛皮纸盒子，红丝带，两边一对小翅膀。
pub const PARCEL: [&str; 16] = [
    "................",
    "................",
    "ww............ww",
    "www..bbrrbb..www",
    "wwwwbbbrrbbbwwww",
    ".wwwBbbrrbbbBww.",
    "..wBbbbrrbbbbB..",
    "...rrrrrrrrrrr..",
    "...rrrrrrrrrrr..",
    "...BbbbrrbbbbB..",
    "...BbbbrrbbbbB..",
    "...BbbbrrbbbbB..",
    "...BbbbrrbbbbB..",
    "...BBBBrrBBBBB..",
    "................",
    "................",
];
pub const PARCEL_KEYS: [(char, u8); 4] = [('w', WHITE), ('b', WOOD_HI), ('B', WOOD), ('r', RED)];

/// 纸飞机：拿到以后能扔。
pub const PLANE_ITEM: [&str; 16] = [
    "................",
    "................",
    "..............ww",
    "...........wwwwk",
    "........wwwwwwk.",
    ".....wwwwwwwwk..",
    "..wwwwwwwwwwk...",
    "kkkkkkkkwwwk....",
    "..ggggwwwwk.....",
    "....ggwwwk......",
    "......gwk.......",
    ".......gk.......",
    "........k.......",
    "................",
    "................",
    "................",
];
pub const PLANE_KEYS: [(char, u8); 3] = [('w', WHITE), ('k', INK), ('g', CLOUD_LO)];

/// 扔出去的纸飞机：8×6。
pub const PLANE_SHOT: [&str; 6] = ["......ww", "...wwwwk", "wwwwwwk.", "kkkkwk..", "..ggk...", "....k..."];

/// 金邮票：齿边的金色邮票，中间一道闪电。拿到了一阵子谁也碰不了你。
pub const GOLD_STAMP: [&str; 16] = [
    ".y.y.y.y.y.y.y..",
    "yYYYYYYYYYYYYYy.",
    ".YggggggggggggY.",
    "yYggggggWWggggYy",
    ".YgggggWWgggggY.",
    "yYggggWWggggggYy",
    ".YgggWWWWWWgggY.",
    "yYggggggWWggggYy",
    ".YgggggWWgggggY.",
    "yYggggWWggggggYy",
    ".YgggWWgggggggY.",
    "yYggggggggggggYy",
    ".YYYYYYYYYYYYYY.",
    "y.y.y.y.y.y.y.y.",
    "................",
    "................",
];
pub const GOLD_KEYS: [(char, u8); 4] = [('y', GOLD_LO), ('Y', GOLD), ('g', GOLD_HI), ('W', WHITE)];

/// 一封粉色的信，封口一颗红心：多一条命。
pub const LOVE: [&str; 16] = [
    "................",
    "................",
    "................",
    ".kkkkkkkkkkkkkk.",
    ".kppppppppppppk.",
    ".kpkppppppppkpk.",
    ".kppkpppppPkppk.",
    ".kpppkrrrrkpppk.",
    ".kppppRrrrrpppk.",
    ".kppppprrrppppk.",
    ".kpppppprppppPk.",
    ".kppppppppppppk.",
    ".kkkkkkkkkkkkkk.",
    "................",
    "................",
    "................",
];
pub const LOVE_KEYS: [(char, u8); 5] = [('k', RED_LO), ('p', PINK), ('P', WHITE), ('r', RED), ('R', RED_HI)];

/// 邮票：三张，转起来（正面、侧一点、侧面）。12×14。
pub const STAMP: [[&str; 14]; 3] = [
    [
        ".w.w.w.w.w..",
        "wbbbbbbbbbbw",
        ".bBBBBBBBBb.",
        "wbBwwwwwwBbw",
        ".bBwBBBBwBb.",
        "wbBwBwwBwBbw",
        ".bBwBwwBwBb.",
        "wbBwBBBBwBbw",
        ".bBwwwwwwBb.",
        "wbBBBBBBBBbw",
        ".bbbbbbbbbb.",
        "w.w.w.w.w.w.",
        "............",
        "............",
    ],
    [
        "...w.w.w....",
        "...bbbbbw...",
        "...bBBBb....",
        "...bwwwbw...",
        "...bwBwb....",
        "...bwBwbw...",
        "...bwBwb....",
        "...bwwwbw...",
        "...bBBBb....",
        "...bbbbbw...",
        "...w.w.w....",
        "............",
        "............",
        "............",
    ],
    [
        ".....w......",
        ".....bw.....",
        ".....b......",
        ".....bw.....",
        ".....b......",
        ".....bw.....",
        ".....b......",
        ".....bw.....",
        ".....b......",
        ".....bw.....",
        ".....w......",
        "............",
        "............",
        "............",
    ],
];
pub const STAMP_KEYS: [(char, u8); 3] = [('w', WHITE), ('b', BLUE), ('B', BLUE_HI)];

/// 邮件大盗扔的信：8×6。
pub const LETTER: [&str; 6] = ["kkkkkkkk", "kwkwwkwk", "kwwkkwwk", "kwwwwwwk", "kwwwwwwk", "kkkkkkkk"];

// ---------- 地图块 ----------

pub const TILE_KEYS: [(char, u8); 18] = [
    ('g', GROUND),
    ('G', GROUND_HI),
    ('d', GROUND_LO),
    ('t', TOP),
    ('T', TOP_HI),
    ('k', INK),
    ('w', WOOD),
    ('W', WOOD_HI),
    ('l', WOOD_LO),
    ('y', GOLD),
    ('Y', GOLD_HI),
    ('o', GOLD_LO),
    ('s', STONE),
    ('S', STONE_HI),
    ('z', STONE_LO),
    ('r', BRICK),
    ('R', BRICK_HI),
    ('m', BRICK_LO),
];

/// 地面最上面一格：人行道（每关换颜色）。
pub const T_TOP: [&str; 16] = [
    "TTTTTTTTTTTTTTTT",
    "TttttttTTttttttt",
    "tttttttzTttttttz",
    "zzzzzzzzzzzzzzzz",
    "ggggGgggggggGggg",
    "gggGggdggggGgggd",
    "ggggggdgggggggdg",
    "dddddddddddddddd",
    "gggGgggggGgggggg",
    "ggGggggdgGgggggd",
    "gggggggdgggggggd",
    "dddddddddddddddd",
    "ggggGgggggggGggg",
    "gggGggdggggGgggd",
    "ggggggdgggggggdg",
    "dddddddddddddddd",
];
/// 地面下面的格。
pub const T_FILL: [&str; 16] = [
    "ggggGgggggggGggg",
    "gggGggdggggGgggd",
    "ggggggdgggggggdg",
    "dddddddddddddddd",
    "gggGgggggGgggggg",
    "ggGggggdgGgggggd",
    "gggggggdgggggggd",
    "dddddddddddddddd",
    "ggggGgggggggGggg",
    "gggGggdggggGgggd",
    "ggggggdgggggggdg",
    "dddddddddddddddd",
    "gggGgggggGgggggg",
    "ggGggggdgGgggggd",
    "gggggggdgggggggd",
    "dddddddddddddddd",
];
/// 旧木箱：大个子顶得碎。
pub const T_CRATE: [&str; 16] = [
    "kkkkkkkkkkkkkkkk",
    "kWWWWWWWWWWWWWlk",
    "kWlwwwwwwwwwwllk",
    "kWwlwwwwwwwwlwlk",
    "kWwwlwwwwwwlwwlk",
    "kWwwwlwwwwlwwwlk",
    "kWwwwwlwwlwwwwlk",
    "kWwwwwwllwwwwwlk",
    "kWwwwwwllwwwwwlk",
    "kWwwwwlwwlwwwwlk",
    "kWwwwlwwwwlwwwlk",
    "kWwwlwwwwwwlwwlk",
    "kWwlwwwwwwwwlwlk",
    "kWlwwwwwwwwwwllk",
    "klllllllllllllkk",
    "kkkkkkkkkkkkkkkk",
];
/// 包裹箱：金色，上面印一个信封，顶了出东西。三张，信封一闪一闪。
pub const T_MAIL: [[&str; 16]; 3] = [
    [
        "kkkkkkkkkkkkkkkk",
        "kYYYYYYYYYYYYYok",
        "kYkyyyyyyyyyykok",
        "kYyyyyyyyyyyyyok",
        "kYykkkkkkkkkkyok",
        "kYykWkWWWWkWkyok",
        "kYykWWkWWkWWkyok",
        "kYykWWWkkWWWkyok",
        "kYykWWWWWWWWkyok",
        "kYykWWWWWWWWkyok",
        "kYykkkkkkkkkkyok",
        "kYyyyyyyyyyyyyok",
        "kYyyyyyyyyyyyyok",
        "kYkyyyyyyyyyykok",
        "kooooooooooooook",
        "kkkkkkkkkkkkkkkk",
    ],
    [
        "kkkkkkkkkkkkkkkk",
        "kYYYYYYYYYYYYYok",
        "kYkyyyyyyyyyykok",
        "kYyyyyyyyyyyyyok",
        "kYykkkkkkkkkkyok",
        "kYykYkYYYYkYkyok",
        "kYykYYkYYkYYkyok",
        "kYykYYYkkYYYkyok",
        "kYykYYYYYYYYkyok",
        "kYykYYYYYYYYkyok",
        "kYykkkkkkkkkkyok",
        "kYyyyyyyyyyyyyok",
        "kYyyyyyyyyyyyyok",
        "kYkyyyyyyyyyykok",
        "kooooooooooooook",
        "kkkkkkkkkkkkkkkk",
    ],
    [
        "kkkkkkkkkkkkkkkk",
        "kYYYYYYYYYYYYYok",
        "kYkyyyyyyyyyykok",
        "kYyyyyyyyyyyyyok",
        "kYykkkkkkkkkkyok",
        "kYykokooookokyok",
        "kYykookookookyok",
        "kYykoookkoookyok",
        "kYykooooooookyok",
        "kYykooooooookyok",
        "kYykkkkkkkkkkyok",
        "kYyyyyyyyyyyyyok",
        "kYyyyyyyyyyyyyok",
        "kYkyyyyyyyyyykok",
        "kooooooooooooook",
        "kkkkkkkkkkkkkkkk",
    ],
];
/// 顶空了的箱子。
pub const T_USED: [&str; 16] = [
    "kkkkkkkkkkkkkkkk",
    "kzzzzzzzzzzzzzzk",
    "kzkzzzzzzzzzzkzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kzkzzzzzzzzzzkzk",
    "kzzzzzzzzzzzzzzk",
    "kkkkkkkkkkkkkkkk",
];
/// 石块：台阶。
pub const T_STONE: [&str; 16] = [
    "SSSSSSSSSSSSSSSz",
    "SSsssssssssssszz",
    "SsSssssssssssszz",
    "SssSsssssssszzzz",
    "SsssSssssssszzzz",
    "SssssSsssszzzzzz",
    "SsssssSsszzzzzzz",
    "SssssssSzzzzzzzz",
    "SssssssSzzzzzzzz",
    "SsssssSsszzzzzzz",
    "SssssSsssszzzzzz",
    "SsssSssssssszzzz",
    "SssSsssssssszzzz",
    "SsSssssssssssszz",
    "SSsssssssssssszz",
    "Szzzzzzzzzzzzzzz",
];
/// 烟囱：左顶、右顶、左身、右身。
pub const T_CHIMNEY: [[&str; 16]; 4] = [
    [
        "kkkkkkkkkkkkkkkk",
        "kSSSSSSSSSSSSSSS",
        "kSssssssssssssss",
        "kzzzzzzzzzzzzzzz",
        "kkkkkkkkkkkkkkkk",
        "..kRRRRRRmRRRRRR",
        "..kRrrrrrmRrrrrr",
        "..kmmmmmmmmmmmmm",
        "..kRRRmRRRRRRRmR",
        "..kRrrmRrrrrrrmR",
        "..kmmmmmmmmmmmmm",
        "..kRRRRRRmRRRRRR",
        "..kRrrrrrmRrrrrr",
        "..kmmmmmmmmmmmmm",
        "..kRRRmRRRRRRRmR",
        "..kRrrmRrrrrrrmR",
    ],
    [
        "kkkkkkkkkkkkkkkk",
        "SSSSSSSSSSSSSSzk",
        "sssssssssssssszk",
        "zzzzzzzzzzzzzzzk",
        "kkkkkkkkkkkkkkkk",
        "RRRmRRRRRRmRk...",
        "rrrmRrrrrrmrk...",
        "mmmmmmmmmmmmk...",
        "RRRRRRmRRRRmk...",
        "rrrrrrmRrrrmk...",
        "mmmmmmmmmmmmk...",
        "RRRmRRRRRRmRk...",
        "rrrmRrrrrrmrk...",
        "mmmmmmmmmmmmk...",
        "RRRRRRmRRRRmk...",
        "rrrrrrmRrrrmk...",
    ],
    [
        "..kRRRRRRmRRRRRR",
        "..kRrrrrrmRrrrrr",
        "..kmmmmmmmmmmmmm",
        "..kRRRmRRRRRRRmR",
        "..kRrrmRrrrrrrmR",
        "..kmmmmmmmmmmmmm",
        "..kRRRRRRmRRRRRR",
        "..kRrrrrrmRrrrrr",
        "..kmmmmmmmmmmmmm",
        "..kRRRmRRRRRRRmR",
        "..kRrrmRrrrrrrmR",
        "..kmmmmmmmmmmmmm",
        "..kRRRRRRmRRRRRR",
        "..kRrrrrrmRrrrrr",
        "..kmmmmmmmmmmmmm",
        "..kRRRmRRRRRRRmR",
    ],
    [
        "RRRmRRRRRRmRk...",
        "rrrmRrrrrrmrk...",
        "mmmmmmmmmmmmk...",
        "RRRRRRmRRRRmk...",
        "rrrrrrmRrrrmk...",
        "mmmmmmmmmmmmk...",
        "RRRmRRRRRRmRk...",
        "rrrmRrrrrrmrk...",
        "mmmmmmmmmmmmk...",
        "RRRRRRmRRRRmk...",
        "rrrrrrmRrrrmk...",
        "mmmmmmmmmmmmk...",
        "RRRmRRRRRRmRk...",
        "rrrmRrrrrrmrk...",
        "mmmmmmmmmmmmk...",
        "RRRRRRmRRRRmk...",
    ],
];
/// 砖墙（下水道的顶、钟楼的墙）：实心，打不碎。
pub const T_BRICK: [&str; 16] = [
    "RRRRRRRmRRRRRRRm",
    "RrrrrrrmRrrrrrrm",
    "RrrrrrrmRrrrrrrm",
    "mmmmmmmmmmmmmmmm",
    "RRRmRRRRRRRmRRRR",
    "rrrmRrrrrrrmRrrr",
    "rrrmRrrrrrrmRrrr",
    "mmmmmmmmmmmmmmmm",
    "RRRRRRRmRRRRRRRm",
    "RrrrrrrmRrrrrrrm",
    "RrrrrrrmRrrrrrrm",
    "mmmmmmmmmmmmmmmm",
    "RRRmRRRRRRRmRRRR",
    "rrrmRrrrrrrmRrrr",
    "rrrmRrrrrrrmRrrr",
    "mmmmmmmmmmmmmmmm",
];
/// 会动的踏板：铁架子，16×8。
pub const T_LIFT: [&str; 8] = ["kkkkkkkkkkkkkkkk", "kSSSSSSSSSSSSSSk", "kszszszszszszszk", "kzzzzzzzzzzzzzzk", "k.k..k..k..k..kk", "kk....kk....kk.k", "k.k..k..k..k..kk", "kkkkkkkkkkkkkkkk"];
/// 蒸汽口：地上一块铁格栅。
pub const T_VENT: [&str; 16] = [
    "kkkkkkkkkkkkkkkk",
    "kSSSSSSSSSSSSSSk",
    "kSkzkzkzkzkzkzzk",
    "kSkzkzkzkzkzkzzk",
    "kSkzkzkzkzkzkzzk",
    "kSzzzzzzzzzzzzzk",
    "kzzzzzzzzzzzzzzk",
    "kkkkkkkkkkkkkkkk",
    "ggggGgggggggGggg",
    "gggGggdggggGgggd",
    "ggggggdgggggggdg",
    "dddddddddddddddd",
    "gggGgggggGgggggg",
    "ggGggggdgGgggggd",
    "gggggggdgggggggd",
    "dddddddddddddddd",
];
