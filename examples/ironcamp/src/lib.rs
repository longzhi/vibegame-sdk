//! 游戏《铁壳守营》：俯视的坦克战。角色、地图、画面、声音都是自己做的。
//!
//! 开一辆小坦克守住营地。敌车从上面三个口子出来，一关二十辆，打完过关。营地被打中，或者命用完，游戏结束。
//! 砖墙一打一条，钢墙打不动（火力升满能打），水过不去，草丛挡住看不见，冰上会滑。
//! 闪红光的敌车打中了掉道具：升火力、全屏炸、护盾、营地加钢、定住敌车。
//! 十字键开，A 或 B 打，START 开始、暂停。
//!
//! 场地 160×160（20×20 个 8 像素的小格），右边 80 像素是记分板。砖墙按 4 像素一小块记，打掉一条是 16 像素宽、4 像素深。
//! 只做了一号车。二号车的出生点、命和分数都留了位置（`players`），等系统能接第二个手柄再开。

mod levels;

use vibegame::FONT;
use levels::{LEVELS, MIXES};
use vibegame::full::{key, tone, tone_wav, Channel, Frame, Keys, Music, Song, Sprite, Track, W};
use std::sync::Mutex;

/// 场地多大（像素）。
const FIELD: i32 = 160;
/// 一个小格 8 像素，场地 20×20 格。
const CELL: i32 = 8;
const CELLS: usize = 20;
/// 砖墙一小块 4 像素，场地 40×40 块。
const MINI: i32 = 4;
const MINIS: usize = 40;
/// 速度和位置的小数：16 份是一个像素。
const SUB: i32 = 16;
/// 一关几辆敌车，场上最多几辆。
#[cfg(test)]
const PER_STAGE: usize = 20;
const MAX_ALIVE: usize = 4;
/// 第几辆敌车闪红光、带道具（从 0 数）。
const CARRIERS: [usize; 3] = [3, 10, 17];
/// 营地的位置（左上角，像素）。
const BASE: (i32, i32) = (72, 144);
/// 敌车的三个出口，一号车、二号车的出生点。
const GATES: [(i32, i32); 3] = [(0, 0), (72, 0), (144, 0)];
const HOMES: [(i32, i32); 2] = [(48, 144), (96, 144)];
/// 营地外面一圈砖的小格。
const RING: [(usize, usize); 8] = [(8, 17), (9, 17), (10, 17), (11, 17), (8, 18), (8, 19), (11, 18), (11, 19)];
/// 存档：开到第几关、最高分。
const SAVE: &str = "progress";

// 调色板的格。
const BG: u8 = 0;
const INK: u8 = 1;
const WHITE: u8 = 2;
const PANEL: u8 = 3;
const PANEL_LO: u8 = 4;
const BRICK: u8 = 5;
const BRICK_HI: u8 = 6;
const MORTAR: u8 = 7;
const STEEL: u8 = 8;
const STEEL_HI: u8 = 9;
const STEEL_LO: u8 = 10;
const WATER: u8 = 11;
const WATER_HI: u8 = 12;
const LEAF: u8 = 13;
const LEAF_HI: u8 = 14;
const LEAF_LO: u8 = 15;
const ICE: u8 = 16;
const ICE_LO: u8 = 17;
const TREAD: u8 = 18;
const TREAD_HI: u8 = 19;
const BARREL: u8 = 20;
const P_MID: u8 = 21;
const P_HI: u8 = 22;
const P_LO: u8 = 23;
const ORANGE: u8 = 24;
const E_MID: u8 = 25;
const E_HI: u8 = 26;
const E_LO: u8 = 27;
const RED: u8 = 28;
const R_HI: u8 = 29;
const R_LO: u8 = 30;
const GOLD: u8 = 31;
const V_MID: u8 = 32;
const V_HI: u8 = 33;
const V_LO: u8 = 34;
const Y_MID: u8 = 35;
const Y_HI: u8 = 36;
const Y_LO: u8 = 37;
const CANVAS: u8 = 38;
const CANVAS_LO: u8 = 39;
const FIRE: u8 = 40;
const GRAY: u8 = 41;
const SKYBLUE: u8 = 42;

const PALETTE: [(u8, [u8; 3]); 43] = [
    (BG, [0x12, 0x12, 0x1a]),
    (INK, [0x08, 0x08, 0x0c]),
    (WHITE, [0xf4, 0xf4, 0xf0]),
    (PANEL, [0x6c, 0x6c, 0x74]),
    (PANEL_LO, [0x48, 0x48, 0x50]),
    (BRICK, [0xa8, 0x4c, 0x24]),
    (BRICK_HI, [0xd8, 0x80, 0x48]),
    (MORTAR, [0x5c, 0x2c, 0x18]),
    (STEEL, [0xa0, 0xa4, 0xb4]),
    (STEEL_HI, [0xe0, 0xe4, 0xf0]),
    (STEEL_LO, [0x58, 0x5c, 0x6c]),
    (WATER, [0x24, 0x54, 0xc4]),
    (WATER_HI, [0x6c, 0xa4, 0xf4]),
    (LEAF, [0x2c, 0x8c, 0x34]),
    (LEAF_HI, [0x74, 0xc8, 0x44]),
    (LEAF_LO, [0x14, 0x50, 0x20]),
    (ICE, [0xd0, 0xe8, 0xf4]),
    (ICE_LO, [0x94, 0xbc, 0xd4]),
    (TREAD, [0x2c, 0x2c, 0x30]),
    (TREAD_HI, [0x70, 0x70, 0x74]),
    (BARREL, [0x50, 0x50, 0x58]),
    (P_MID, [0x5c, 0x9c, 0x3c]),
    (P_HI, [0xa4, 0xd4, 0x68]),
    (P_LO, [0x34, 0x60, 0x24]),
    (ORANGE, [0xf0, 0x90, 0x28]),
    (E_MID, [0xa8, 0xa8, 0xa0]),
    (E_HI, [0xe0, 0xe0, 0xd8]),
    (E_LO, [0x64, 0x64, 0x60]),
    (RED, [0xd8, 0x30, 0x30]),
    (R_HI, [0xff, 0x88, 0x78]),
    (R_LO, [0x80, 0x18, 0x18]),
    (GOLD, [0xff, 0xd8, 0x40]),
    (V_MID, [0x8c, 0x5c, 0xc8]),
    (V_HI, [0xc4, 0x9c, 0xf0]),
    (V_LO, [0x50, 0x30, 0x7c]),
    (Y_MID, [0xd8, 0xb0, 0x30]),
    (Y_HI, [0xff, 0xe8, 0x88]),
    (Y_LO, [0x8c, 0x68, 0x10]),
    (CANVAS, [0xe8, 0xd4, 0x9c]),
    (CANVAS_LO, [0xa8, 0x8c, 0x58]),
    (FIRE, [0xff, 0x6c, 0x20]),
    (GRAY, [0x9c, 0x9c, 0xa4]),
    (SKYBLUE, [0x78, 0xc8, 0xf0]),
];

// 坦克字符画里的四格是占位的，画的时候换成哪一种车的颜色。
const T_BODY: u8 = 250;
const T_HI: u8 = 251;
const T_LO: u8 = 252;
const T_ACC: u8 = 253;
const TANK_KEYS: [(char, u8); 8] = [('t', TREAD), ('T', TREAD_HI), ('g', BARREL), ('k', INK), ('b', T_BODY), ('h', T_HI), ('d', T_LO), ('c', T_ACC)];

/// 一号车：圆顶，前面一块装甲，橙色的边。
const ART_PLAYER: [&str; 16] = [
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
];
/// 普通敌车：方车身，圆炮塔。
const ART_BASIC: [&str; 16] = [
    ".......gg.......",
    ".......gg.......",
    "tt.....gg.....tt",
    "TT.....gg.....TT",
    "ttbbbbbggbbbbbtt",
    "TTbhhhhggddddbTT",
    "ttbhhkkkkkkddbtt",
    "TTbhkhhhhhdkdbTT",
    "ttbhkhhhhhdkdbtt",
    "TTbhkhhhhddkdbTT",
    "ttbhkddddddkdbtt",
    "TTbhhkkkkkkddbTT",
    "ttbddddddddddbtt",
    "TTbbbbbbbbbbbbTT",
    "tt............tt",
    "TT............TT",
];
/// 快车：尖头，车身窄。
const ART_FAST: [&str; 16] = [
    "................",
    ".......gg.......",
    ".......gg.......",
    "tt.....gg.....tt",
    "TT....bggb....TT",
    "tt...bhggdb...tt",
    "TT..bhhkkddb..TT",
    "tt.bhhkhhkddb.tt",
    "TTbhhkhhhhkddbTT",
    "ttbhhkhhhdkddbtt",
    "TTbhhhkddkdddbTT",
    "ttbhhhhkkddddbtt",
    "TTbbbbbbbbbbbbTT",
    "tt.cc......cc.tt",
    "TT............TT",
    "................",
];
/// 炮车：粗炮管，方炮塔，子弹快。
const ART_POWER: [&str; 16] = [
    "......gggg......",
    "......gggg......",
    "tt....gggg....tt",
    "TT....gggg....TT",
    "ttbbbbggggbbbbtt",
    "TTbhhkkkkkkddbTT",
    "ttbhhkhhhhkddbtt",
    "TTbhhkhcchkddbTT",
    "ttbhhkhcchkddbtt",
    "TTbhhkhhhdkddbTT",
    "ttbhhkkkkkkddbtt",
    "TTbhhhhhhddddbTT",
    "ttbddddddddddbtt",
    "TTbbbbbbbbbbbbTT",
    "tt............tt",
    "TT............TT",
];
/// 重甲车：宽履带，车身上一块块钢板，要打四下。
const ART_ARMOR: [&str; 16] = [
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
];

const ART_BRICK: [&str; 8] = ["hbbmhbbb", "bbbmbbbb", "bddmbddd", "mmmmmmmm", "bbhbbbmh", "bbbbbbmb", "dddbddmd", "mmmmmmmm"];
const ART_STEEL: [&str; 8] = ["wwwwwwwd", "wssssssd", "wswwwddd", "wswssdsd", "wswssdsd", "wsdddssd", "wssssssd", "dddddddd"];
const ART_WATER: [[&str; 8]; 2] = [
    ["bbbbbbbb", "bwwbbbbb", "wbbwbbbb", "bbbbbbbb", "bbbbbwwb", "bbbbwbbw", "bbbbbbbb", "bbbbbbbb"],
    ["bbbbbbbb", "bbbbbwwb", "bbbbwbbw", "bbbbbbbb", "bwwbbbbb", "wbbwbbbb", "bbbbbbbb", "bbbbbbbb"],
];
const ART_GRASS: [&str; 8] = ["lgLglgLg", "gLgglLgl", "Lglgg.gL", "glLgLglg", "gLg.gLgL", "LglgglLg", "glgLgLgl", "gLglLggL"];
const ART_ICE: [&str; 8] = ["iiiiiiil", "iiiiilii", "iiiilii.", "iiiliiii", "iilii.ii", "ilii.iii", "liiiiiii", "iiiiiiii"];

/// 营地：一顶帐篷，上面插一面旗。
const ART_CAMP: [&str; 16] = [
    ".......kr.......",
    ".......krrrr....",
    ".......krrr.....",
    ".......k........",
    ".......k........",
    "......ccc.......",
    ".....cccCc......",
    "....cccccCc.....",
    "...cccc.ccCc....",
    "..cccc...ccCc...",
    "..ccc.kkk.cCc...",
    ".ccc..kkk..cCc..",
    ".cc...kkk...cCc.",
    "ccccccccccccccCc",
    "kkkkkkkkkkkkkkkk",
    "................",
];
/// 营地被打中：帐篷塌了，旗倒了。
const ART_RUIN: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "........k.......",
    ".rrr...k........",
    ".rrrkkk.........",
    "..kk..ccC.......",
    "....cCcccCccc...",
    "..cCccgcccgcCc..",
    "kkkkkkkkkkkkkkkk",
    "................",
];

/// 道具，16×16，带一圈框。
const ART_BONUS: [[&str; 16]; 5] = [
    // 升火力：往上的箭头
    [
        "kkkkkkkkkkkkkkkk",
        "kwwwwwwwwwwwwwwk",
        "kw.....yy.....wk",
        "kw....yyyy....wk",
        "kw...yyyyyy...wk",
        "kw..yyyyyyyy..wk",
        "kw.yyyyyyyyyy.wk",
        "kw....yyyy....wk",
        "kw....yyyy....wk",
        "kw....yyyy....wk",
        "kw....yyyy....wk",
        "kw....oooo....wk",
        "kw....oooo....wk",
        "kw............wk",
        "kwwwwwwwwwwwwwwk",
        "kkkkkkkkkkkkkkkk",
    ],
    // 全屏炸：一颗炸弹
    [
        "kkkkkkkkkkkkkkkk",
        "kwwwwwwwwwwwwwwk",
        "kw.........oy.wk",
        "kw........o...wk",
        "kw.....kkko...wk",
        "kw....kkkkk...wk",
        "kw...kkkkkkk..wk",
        "kw..kkwkkkkkk.wk",
        "kw..kwkkkkkkk.wk",
        "kw..kkkkkkkkk.wk",
        "kw..kkkkkkkkk.wk",
        "kw...kkkkkkk..wk",
        "kw....kkkkk...wk",
        "kw............wk",
        "kwwwwwwwwwwwwwwk",
        "kkkkkkkkkkkkkkkk",
    ],
    // 护盾：一面盾
    [
        "kkkkkkkkkkkkkkkk",
        "kwwwwwwwwwwwwwwk",
        "kw..ssssssss..wk",
        "kw..sbbbbbbs..wk",
        "kw..sbbyybbs..wk",
        "kw..sbyyyybs..wk",
        "kw..sbbyybbs..wk",
        "kw..sbbyybbs..wk",
        "kw..sbbbbbbs..wk",
        "kw...sbbbbs...wk",
        "kw....sbbs....wk",
        "kw.....ss.....wk",
        "kw............wk",
        "kw............wk",
        "kwwwwwwwwwwwwwwk",
        "kkkkkkkkkkkkkkkk",
    ],
    // 营地加钢：一把铲子插在钢墙上
    [
        "kkkkkkkkkkkkkkkk",
        "kwwwwwwwwwwwwwwk",
        "kw......oo....wk",
        "kw......oo....wk",
        "kw......oo....wk",
        "kw.....ssss...wk",
        "kw.....ssss...wk",
        "kw......ss....wk",
        "kw..SSSSSSSSS.wk",
        "kw..SsssSsssS.wk",
        "kw..SSSSSSSSS.wk",
        "kw..SsssSsssS.wk",
        "kw..SSSSSSSSS.wk",
        "kw............wk",
        "kwwwwwwwwwwwwwwk",
        "kkkkkkkkkkkkkkkk",
    ],
    // 定住敌车：一只钟
    [
        "kkkkkkkkkkkkkkkk",
        "kwwwwwwwwwwwwwwk",
        "kw....yyyy....wk",
        "kw..yywwwwyy..wk",
        "kw.ywwwkwwwwy.wk",
        "kw.ywwwkwwwwy.wk",
        "kwywwwwkwwwwwywk",
        "kwywwwwkkkkwwywk",
        "kwywwwwwwwwwwywk",
        "kw.ywwwwwwwwy.wk",
        "kw.ywwwwwwwwy.wk",
        "kw..yywwwwyy..wk",
        "kw....yyyy....wk",
        "kw............wk",
        "kwwwwwwwwwwwwwwk",
        "kkkkkkkkkkkkkkkk",
    ],
];

/// 出车时闪的光，四张由小到大。
const ART_SPARK: [[&str; 9]; 4] = [
    [".........", ".........", ".........", "....w....", "...www...", "....w....", ".........", ".........", "........."],
    [".........", ".........", "....w....", "....w....", "..wwwww..", "....w....", "....w....", ".........", "........."],
    ["....w....", "....w....", ".w..w..w.", "...www...", "wwwwwwwww", "...www...", ".w..w..w.", "....w....", "....w...."],
    ["w...w...w", ".w..w..w.", "..w.w.w..", "...www...", "wwwwwwwww", "...www...", "..w.w.w..", ".w..w..w.", "w...w...w"],
];
/// 炸开：三张，越来越大。大的是放大两倍画。
const ART_BOOM: [[&str; 16]; 3] = [
    [
        "................",
        "................",
        "................",
        "................",
        "......w..w......",
        ".....wyyyyw.....",
        "......yooy......",
        ".....yoffoy.....",
        ".....yoffoy.....",
        "......yooy......",
        ".....wyyyyw.....",
        "......w..w......",
        "................",
        "................",
        "................",
        "................",
    ],
    [
        "................",
        "..w.....w....w..",
        "...y..yyyy..y...",
        "....yyooooyy....",
        "..yyoooffoooyy..",
        "..yooffffffooy..",
        ".yooffwwwwffooy.",
        ".yoffwwwwwwffoy.",
        ".yoffwwwwwwffoy.",
        ".yooffwwwwffooy.",
        "..yooffffffooy..",
        "..yyoooffoooyy..",
        "....yyooooyy....",
        "...y..yyyy..y...",
        "..w....w.....w..",
        "................",
    ],
    [
        "...y........y...",
        ".y...yyyyyy...y.",
        "....yoooooooy...",
        "..yyoffffffooyy.",
        ".yooffwwwwffffoy",
        ".yoffww..wwwffoy",
        "yoofw......wfooy",
        "yofw........wfoy",
        "yofw........wfoy",
        "yoofw......wfooy",
        ".yoffww..wwwffoy",
        ".yooffwwwwffffoy",
        "..yyoffffffooyy.",
        "....yoooooooy...",
        ".y...yyyyyy...y.",
        "...y........y...",
    ],
];
const BOOM_KEYS: [(char, u8); 4] = [('w', WHITE), ('y', GOLD), ('o', ORANGE), ('f', FIRE)];
/// 护盾：车外面一圈闪的线，两张换着画。
const ART_SHIELD: [[&str; 16]; 2] = [
    [
        "w.w.w.w.w.w.w.w.",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        ".w.w.w.w.w.w.w.w",
    ],
    [
        ".w.w.w.w.w.w.w.w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w...............",
        "...............w",
        "w.w.w.w.w.w.w.w.",
    ],
];
const ART_SHOT: [&str; 4] = [".ww.", "wyyw", "wyyw", ".ww."];
/// 记分板上的小车（还剩几辆敌车、几条命）和旗子（第几关）。
const ART_ICON: [&str; 7] = ["...g...", "t.bgb.t", "tbbbbbt", "tbhkdbt", "tbbbbbt", "tbbbbbt", "t.....t"];
const ART_FLAG: [&str; 10] = ["k.........", "krrrrrr...", "krrrrrrrr.", "krrrrrrr..", "krrrrrr...", "k.........", "k.........", "k.........", "k.........", "kkk......."];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    fn step(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Right => (1, 0),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
        }
    }
    fn vertical(self) -> bool {
        matches!(self, Dir::Up | Dir::Down)
    }
    fn turns(self) -> usize {
        self as usize
    }
}

/// 车的种类。速度是每帧走几份（16 份一个像素）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Player,
    Basic,
    Fast,
    Power,
    Armor,
}

impl Kind {
    fn speed(self) -> i32 {
        match self {
            Kind::Player => 12,
            Kind::Basic => 6,
            Kind::Fast => 16,
            Kind::Power => 9,
            Kind::Armor => 7,
        }
    }
    fn hp(self) -> u8 {
        if self == Kind::Armor {
            4
        } else {
            1
        }
    }
    fn points(self) -> u32 {
        match self {
            Kind::Player => 0,
            Kind::Basic => 100,
            Kind::Fast => 200,
            Kind::Power => 300,
            Kind::Armor => 400,
        }
    }
    /// 记分时的顺序：普通、快车、炮车、重甲。
    fn slot(self) -> usize {
        match self {
            Kind::Player | Kind::Basic => 0,
            Kind::Fast => 1,
            Kind::Power => 2,
            Kind::Armor => 3,
        }
    }
    fn art(self) -> &'static [&'static str; 16] {
        match self {
            Kind::Player => &ART_PLAYER,
            Kind::Basic => &ART_BASIC,
            Kind::Fast => &ART_FAST,
            Kind::Power => &ART_POWER,
            Kind::Armor => &ART_ARMOR,
        }
    }
    const ENEMIES: [Kind; 4] = [Kind::Basic, Kind::Fast, Kind::Power, Kind::Armor];
}

/// 小格上是什么。砖墙另外按小块记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ground {
    Open,
    Steel,
    Water,
    Grass,
    Ice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bonus {
    Power,
    Bomb,
    Shield,
    Shovel,
    Clock,
}

const BONUSES: [Bonus; 5] = [Bonus::Power, Bonus::Bomb, Bonus::Shield, Bonus::Shovel, Bonus::Clock];

/// 一辆车。位置是左上角（像素），`acc` 攒着不够一个像素的那点。
#[derive(Debug, Clone)]
pub struct Tank {
    pub id: u32,
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub dir: Dir,
    acc: i32,
    pub hp: u8,
    /// 闪红光：打中了掉道具。
    pub carrier: bool,
    /// 还在出车的光里：还有几帧。这时不挡路、不能打。
    pub spawn: u32,
    /// 护盾还有几帧。
    pub shield: u32,
    /// 火力：0 普通，1 子弹快，2 能连打两发，3 能打穿钢墙。
    pub level: u8,
    moving: bool,
    anim: u8,
    /// 敌车：再走几帧换方向。
    turn: u32,
    /// 一号车在冰上松了键，还要滑几帧。
    slide: u32,
    /// 被打中闪白，还有几帧。
    flash: u32,
}

impl Tank {
    fn new(id: u32, kind: Kind, (x, y): (i32, i32)) -> Tank {
        let dir = if kind == Kind::Player { Dir::Up } else { Dir::Down };
        Tank { id, kind, x, y, dir, acc: 0, hp: kind.hp(), carrier: false, spawn: 48, shield: 0, level: 0, moving: false, anim: 0, turn: 60, slide: 0, flash: 0 }
    }
    fn active(&self) -> bool {
        self.spawn == 0
    }
    fn is_player(&self) -> bool {
        self.kind == Kind::Player
    }
}

#[derive(Debug, Clone)]
pub struct Shot {
    pub x: i32,
    pub y: i32,
    pub dir: Dir,
    pub owner: u32,
    pub from_player: bool,
    pub speed: i32,
    /// 能打穿钢墙（火力升满）。
    pub pierce: bool,
    pub alive: bool,
}

#[derive(Debug, Clone)]
struct Boom {
    x: i32,
    y: i32,
    t: u32,
    big: bool,
}

#[derive(Debug, Clone)]
struct Popup {
    x: i32,
    y: i32,
    text: String,
    t: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Title,
    /// 选关：上下换关，A 开始。
    Select,
    Play,
    /// 一关打完（或者输了）数分。
    Tally,
    Over,
    AllClear,
}

/// 自己的随机数：每关从同一个数开始，同样的按键跑出来一样。
#[derive(Debug, Clone)]
struct Rng(u32);

impl Rng {
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next() % n.max(1)
    }
}

/// 精灵：每种车四个方向、两张（履带一前一后）。
struct Art {
    tanks: Vec<[[Sprite; 2]; 4]>,
    brick: Sprite,
    steel: Sprite,
    water: [Sprite; 2],
    grass: Sprite,
    ice: Sprite,
    camp: Sprite,
    ruin: Sprite,
    bonus: Vec<Sprite>,
    spark: Vec<Sprite>,
    boom: Vec<Sprite>,
    shield: [Sprite; 2],
    shot: Sprite,
    icon: Sprite,
    life: Sprite,
    flag: Sprite,
}

/// 顺时针转一下（一号车画的是朝上的，别的方向转出来）。
fn rotate(s: &Sprite) -> Sprite {
    // 新的宽是原来的高。新的 (x, y) 取原来的 (y, 高 - 1 - x)。
    let mut px = vec![0; (s.w * s.h) as usize];
    for y in 0..s.w {
        for x in 0..s.h {
            px[(y * s.h + x) as usize] = s.at(y, s.h - 1 - x);
        }
    }
    Sprite { w: s.h, h: s.w, px }
}

impl Art {
    fn new() -> Art {
        let tank = |rows: &[&str; 16]| -> [[Sprite; 2]; 4] {
            let a = Sprite::art(rows, &TANK_KEYS);
            // 第二张：履带的深浅换一下，开起来像在转。
            let b = a.recolor(&[(TREAD, TREAD_HI), (TREAD_HI, TREAD)]);
            let mut out: Vec<[Sprite; 2]> = Vec::new();
            let (mut a, mut b) = (a, b);
            for _ in 0..4 {
                out.push([a.clone(), b.clone()]);
                a = rotate(&a);
                b = rotate(&b);
            }
            [out[0].clone(), out[1].clone(), out[2].clone(), out[3].clone()]
        };
        let kinds = [Kind::Player, Kind::Basic, Kind::Fast, Kind::Power, Kind::Armor];
        let bonus_keys = [('k', INK), ('w', WHITE), ('y', GOLD), ('o', ORANGE), ('s', STEEL_HI), ('S', STEEL_LO), ('b', SKYBLUE)];
        let water = |i: usize| Sprite::art(&ART_WATER[i], &[('b', WATER), ('w', WATER_HI)]);
        Art {
            tanks: kinds.iter().map(|k| tank(k.art())).collect(),
            brick: Sprite::art(&ART_BRICK, &[('b', BRICK), ('h', BRICK_HI), ('d', MORTAR), ('m', MORTAR)]),
            steel: Sprite::art(&ART_STEEL, &[('w', STEEL_HI), ('s', STEEL), ('d', STEEL_LO)]),
            water: [water(0), water(1)],
            grass: Sprite::art(&ART_GRASS, &[('g', LEAF), ('L', LEAF_HI), ('l', LEAF_LO)]),
            ice: Sprite::art(&ART_ICE, &[('i', ICE), ('l', ICE_LO), ('.', WHITE)]),
            camp: Sprite::art(&ART_CAMP, &[('k', INK), ('r', RED), ('c', CANVAS), ('C', CANVAS_LO)]),
            ruin: Sprite::art(&ART_RUIN, &[('k', INK), ('r', RED), ('c', GRAY), ('C', CANVAS_LO), ('g', PANEL_LO)]),
            bonus: ART_BONUS.iter().map(|r| Sprite::art(r, &bonus_keys)).collect(),
            spark: ART_SPARK.iter().map(|r| Sprite::art(r, &[('w', WHITE)])).collect(),
            boom: ART_BOOM.iter().map(|r| Sprite::art(r, &BOOM_KEYS)).collect(),
            shield: [Sprite::art(&ART_SHIELD[0], &[('w', WHITE)]), Sprite::art(&ART_SHIELD[1], &[('w', SKYBLUE)])],
            shot: Sprite::art(&ART_SHOT, &[('w', WHITE), ('y', GOLD)]),
            icon: Sprite::art(&ART_ICON, &[('t', INK), ('g', INK), ('b', INK), ('h', INK), ('k', PANEL), ('d', INK)]),
            life: Sprite::art(&ART_ICON, &[('t', TREAD), ('g', BARREL), ('b', P_MID), ('h', P_HI), ('k', INK), ('d', P_LO)]),
            flag: Sprite::art(&ART_FLAG, &[('k', INK), ('r', RED)]),
        }
    }
}

/// 车身换成哪一套颜色：一号车绿，敌车灰；重甲车按还剩几下换色；带道具的闪红。
fn tank_colors(t: &Tank, tick: u32) -> [(u8, u8); 4] {
    let set = |b, h, d, c| [(T_BODY, b), (T_HI, h), (T_LO, d), (T_ACC, c)];
    if t.flash > 0 {
        return set(WHITE, WHITE, E_HI, WHITE);
    }
    if t.carrier && (tick / 8).is_multiple_of(2) {
        return set(RED, R_HI, R_LO, GOLD);
    }
    match t.kind {
        Kind::Player => set(P_MID, P_HI, P_LO, ORANGE),
        Kind::Armor => match t.hp {
            4 => set(V_MID, V_HI, V_LO, GOLD),
            3 => set(Y_MID, Y_HI, Y_LO, RED),
            2 => set(P_MID, P_HI, P_LO, GOLD),
            _ => set(E_MID, E_HI, E_LO, RED),
        },
        Kind::Fast => set(E_MID, E_HI, E_LO, ORANGE),
        Kind::Power => set(E_MID, E_HI, E_LO, RED),
        Kind::Basic => set(E_MID, E_HI, E_LO, E_LO),
    }
}

fn overlap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

/// 一段由自己生成的噪声，慢慢变小：炸开的声音。`tenths` 是长几个十分之一秒。
fn boom_wav(tenths: u32, seed: u32) -> Vec<u8> {
    let rate = 8000u32;
    let n = (rate * tenths / 10) as usize;
    let mut s = seed;
    let mut data = Vec::with_capacity(n * 2);
    let mut low = 0i32;
    for i in 0..n {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let noise = (s >> 16) as i32 % 0x8000 - 0x4000;
        // 低一点的轰：噪声抹平一些。
        low = (low * 3 + noise) / 4;
        let fade = (n - i) as i32 * 1000 / n as i32;
        let v = (low * fade / 1000 * 2).clamp(-32768, 32767) as i16;
        data.extend_from_slice(&v.to_le_bytes());
    }
    let mut w = Vec::with_capacity(44 + data.len());
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&rate.to_le_bytes());
    w.extend_from_slice(&(rate * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(data.len() as u32).to_le_bytes());
    w.extend_from_slice(&data);
    w
}

/// 开局的一小段：自己写的号角。
fn fanfare() -> Song {
    let lead = Track::parse(Channel::Square1, 50, "G4 C5 E5 G5:2 r E5 F5 A5:2 G5:2 C6:4").unwrap();
    let bass = Track::parse(Channel::Triangle, 60, "C3:2 G2:2 C3:2 F2:2 G2:2 C3:4").unwrap();
    Song::new(6, false, vec![lead, bass])
}

fn game_over_tune() -> Song {
    let lead = Track::parse(Channel::Square1, 50, "E5:2 D5:2 C5:2 A4:2 B4:2 G4:6").unwrap();
    let bass = Track::parse(Channel::Triangle, 60, "A2:4 F2:4 G2:4 C2:4").unwrap();
    Song::new(8, false, vec![lead, bass])
}

fn victory_tune() -> Song {
    let lead = Track::parse(Channel::Square1, 50, "C5 E5 G5 C6 r G5 C6:2 E6 D6 C6 G5 A5:2 B5:2 C6:6").unwrap();
    let bass = Track::parse(Channel::Triangle, 60, "C3:2 G3:2 C3:2 G3:2 F3:2 G3:2 C3:6").unwrap();
    Song::new(6, false, vec![lead, bass])
}

pub struct Game {
    pub keys: Keys,
    pub frame: Frame,
    music: Music,
    art: Art,
    pub mode: Mode,
    timer: u32,
    tick: u32,
    rng: Rng,
    /// 第几关（从 0 数）。
    pub stage: usize,
    /// 开到了第几关（从 0 数），存在存档里。
    pub unlocked: usize,
    pub hi: u32,
    /// 几个人玩。现在只有 1。
    players: usize,
    pub score: [u32; 2],
    /// 还剩几条命（不算场上这条）。
    pub lives: [u32; 2],
    /// 一号车、二号车死了以后再出来还要等几帧。
    respawn: [u32; 2],
    /// 下一次加命的分。
    next_life: [u32; 2],
    pub bricks: Vec<[bool; MINIS]>,
    pub ground: [[Ground; CELLS]; CELLS],
    pub base_alive: bool,
    pub tanks: Vec<Tank>,
    pub shots: Vec<Shot>,
    booms: Vec<Boom>,
    popups: Vec<Popup>,
    /// 场上的道具和它的位置。
    pub bonus: Option<(Bonus, i32, i32)>,
    /// 还没出来的敌车。
    pub queue: Vec<Kind>,
    spawned: usize,
    /// 这关打掉的：普通、快车、炮车、重甲。
    pub killed: [u32; 4],
    spawn_timer: u32,
    gate: usize,
    /// 敌车被定住还有几帧。
    pub freeze: u32,
    /// 营地外面那圈是钢的还有几帧。
    pub shovel: u32,
    ring_steel: bool,
    /// 输了：从这时起数帧。
    pub over: Option<u32>,
    /// 打完了：从这时起数帧。
    clear: Option<u32>,
    paused: bool,
    next_id: u32,
    /// 排着的短音：第几帧响、声道、频率、毫秒、音量。
    jingle: Vec<(u32, Channel, u32, u32, u32)>,
    /// 一号车上一帧按的方向：两个方向一起按时听后按的那个。
    last_dir: Option<Dir>,
    boom_small: Vec<u8>,
    boom_big: Vec<u8>,
}

impl Default for Game {
    fn default() -> Self {
        let mut frame = Frame::default();
        for (i, rgb) in PALETTE {
            frame.palette[i as usize] = rgb;
        }
        let mut g = Game {
            keys: Keys::default(),
            frame,
            music: Music::default(),
            art: Art::new(),
            mode: Mode::Title,
            timer: 0,
            tick: 0,
            rng: Rng(0x2468_ace1),
            stage: 0,
            unlocked: 0,
            hi: 20000,
            players: 1,
            score: [0; 2],
            lives: [2; 2],
            respawn: [0; 2],
            next_life: [20000; 2],
            bricks: vec![[false; MINIS]; MINIS],
            ground: [[Ground::Open; CELLS]; CELLS],
            base_alive: true,
            tanks: Vec::new(),
            shots: Vec::new(),
            booms: Vec::new(),
            popups: Vec::new(),
            bonus: None,
            queue: Vec::new(),
            spawned: 0,
            killed: [0; 4],
            spawn_timer: 0,
            gate: 0,
            freeze: 0,
            shovel: 0,
            ring_steel: false,
            over: None,
            clear: None,
            paused: false,
            next_id: 10,
            jingle: Vec::new(),
            last_dir: None,
            boom_small: boom_wav(3, 0x1357_9bdf),
            boom_big: boom_wav(7, 0x2468_9ace),
        };
        if let Some(b) = vibegame::load(SAVE) {
            let s = String::from_utf8_lossy(&b);
            let mut it = s.split(',');
            if let Some(u) = it.next().and_then(|v| v.trim().parse::<usize>().ok()) {
                g.unlocked = u.min(LEVELS.len() - 1);
            }
            if let Some(h) = it.next().and_then(|v| v.trim().parse::<u32>().ok()) {
                g.hi = g.hi.max(h);
            }
        }
        g.stage = g.unlocked;
        g
    }
}

impl Game {
    fn save(&self) {
        // 存不了（开发机上没给目录）也接着玩，只是下次从头。
        let _ = vibegame::save(SAVE, format!("{},{}", self.unlocked, self.hi).as_bytes());
    }

    fn sfx(&mut self, ch: Channel, hz: u32, ms: u32, vol: u32) {
        self.music.sfx(ch, hz, ms, vol);
    }

    /// 几个音接着响：每个隔 `gap` 帧。
    fn arpeggio(&mut self, ch: Channel, notes: &[u32], gap: u32, vol: u32) {
        for (i, hz) in notes.iter().enumerate() {
            self.jingle.push((self.tick + i as u32 * gap, ch, *hz, gap * 1000 / 60 + 20, vol));
        }
    }

    fn run_jingle(&mut self) {
        let now = self.tick;
        let due: Vec<_> = self.jingle.iter().filter(|j| j.0 <= now).cloned().collect();
        self.jingle.retain(|j| j.0 > now);
        for (_, ch, hz, ms, vol) in due {
            self.music.sfx(ch, hz, ms, vol);
        }
    }

    // ---------- 地图 ----------

    fn set_brick_cell(&mut self, cx: usize, cy: usize, on: bool) {
        for my in cy * 2..cy * 2 + 2 {
            for mx in cx * 2..cx * 2 + 2 {
                self.bricks[my][mx] = on;
            }
        }
    }

    /// 读第 `i` 关的地图，盖上营地和它外面一圈砖。
    pub fn load_map(&mut self, i: usize) {
        self.bricks = vec![[false; MINIS]; MINIS];
        self.ground = [[Ground::Open; CELLS]; CELLS];
        for (ty, row) in LEVELS[i].iter().enumerate() {
            for (tx, ch) in row.chars().enumerate() {
                for dy in 0..2 {
                    for dx in 0..2 {
                        let (cx, cy) = (tx * 2 + dx, ty * 2 + dy);
                        let half = match ch {
                            '<' | 'l' => dx == 0,
                            '>' | 'r' => dx == 1,
                            '^' | 't' => dy == 0,
                            'v' | 'b' => dy == 1,
                            _ => true,
                        };
                        if !half {
                            continue;
                        }
                        match ch {
                            '#' | '<' | '>' | '^' | 'v' => self.set_brick_cell(cx, cy, true),
                            '@' | 'l' | 'r' | 't' | 'b' => self.ground[cy][cx] = Ground::Steel,
                            '~' => self.ground[cy][cx] = Ground::Water,
                            '%' => self.ground[cy][cx] = Ground::Grass,
                            '-' => self.ground[cy][cx] = Ground::Ice,
                            _ => {}
                        }
                    }
                }
            }
        }
        // 营地那块清空，外面一圈砖。
        for cy in 17..20 {
            for cx in 8..12 {
                self.ground[cy][cx] = Ground::Open;
                self.set_brick_cell(cx, cy, false);
            }
        }
        for &(cx, cy) in &RING {
            self.set_brick_cell(cx, cy, true);
        }
        self.ring_steel = false;
    }

    /// 营地外面那圈：换成钢墙，或者换回整块的砖。
    fn set_ring(&mut self, steel: bool) {
        if steel == self.ring_steel {
            return;
        }
        self.ring_steel = steel;
        for &(cx, cy) in &RING {
            if steel {
                self.ground[cy][cx] = Ground::Steel;
                self.set_brick_cell(cx, cy, false);
            } else {
                self.ground[cy][cx] = Ground::Open;
                self.set_brick_cell(cx, cy, true);
            }
        }
    }

    fn ground_at(&self, x: i32, y: i32) -> Ground {
        if !(0..FIELD).contains(&x) || !(0..FIELD).contains(&y) {
            return Ground::Steel;
        }
        self.ground[(y / CELL) as usize][(x / CELL) as usize]
    }

    /// 这块地方车开不进去：出了场地、有砖、钢、水，或者是营地。
    pub fn blocked(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        if x < 0 || y < 0 || x + w > FIELD || y + h > FIELD {
            return true;
        }
        for my in y / MINI..=(y + h - 1) / MINI {
            for mx in x / MINI..=(x + w - 1) / MINI {
                if self.bricks[my as usize][mx as usize] {
                    return true;
                }
            }
        }
        for cy in y / CELL..=(y + h - 1) / CELL {
            for cx in x / CELL..=(x + w - 1) / CELL {
                if matches!(self.ground[cy as usize][cx as usize], Ground::Steel | Ground::Water) {
                    return true;
                }
            }
        }
        overlap((x, y, w, h), (BASE.0, BASE.1, 16, 16))
    }

    // ---------- 开局 ----------

    fn start_stage(&mut self) {
        self.mode = Mode::Play;
        self.timer = 0;
        self.load_map(self.stage);
        self.rng = Rng(0x2468_ace1 ^ (self.stage as u32).wrapping_mul(0x9e37_79b9));
        // 二十辆车排好次序：按配比放进去再打乱。
        let mix = MIXES[self.stage];
        let mut q: Vec<Kind> = Vec::new();
        for (k, n) in Kind::ENEMIES.iter().zip(mix) {
            q.extend(std::iter::repeat_n(*k, n as usize));
        }
        for i in (1..q.len()).rev() {
            let j = self.rng.below(i as u32 + 1) as usize;
            q.swap(i, j);
        }
        // 后出来的先取：倒过来放，pop 取的就是第一辆。
        q.reverse();
        self.queue = q;
        self.spawned = 0;
        self.killed = [0; 4];
        self.spawn_timer = 30;
        self.gate = 1;
        self.tanks.clear();
        self.shots.clear();
        self.booms.clear();
        self.popups.clear();
        self.bonus = None;
        self.freeze = 0;
        self.shovel = 0;
        self.over = None;
        self.clear = None;
        self.paused = false;
        self.base_alive = true;
        for p in 0..self.players {
            self.respawn[p] = 0;
            self.spawn_player(p);
        }
        self.music.play(fanfare());
    }

    /// 新开一局：分数、命清零，火力从头来。
    fn new_game(&mut self) {
        self.score = [0; 2];
        self.lives = [2; 2];
        self.next_life = [20000; 2];
        self.start_stage();
    }

    fn spawn_player(&mut self, p: usize) {
        let mut t = Tank::new(p as u32, Kind::Player, HOMES[p]);
        t.shield = 180;
        // 过关时火力留着：带到下一关。
        if let Some(old) = self.tanks.iter().find(|o| o.id == p as u32) {
            t.level = old.level;
        }
        self.tanks.retain(|o| o.id != p as u32);
        self.tanks.push(t);
    }

    fn player(&self, p: usize) -> Option<&Tank> {
        self.tanks.iter().find(|t| t.id == p as u32)
    }

    fn enemies_alive(&self) -> usize {
        self.tanks.iter().filter(|t| !t.is_player()).count()
    }

    // ---------- 一帧 ----------

    pub fn step(&mut self) {
        self.tick += 1;
        self.timer += 1;
        self.music.tick();
        self.run_jingle();
        match self.mode {
            Mode::Title => {
                if self.keys.pressed(key::START) || self.keys.pressed(key::A) {
                    self.mode = Mode::Select;
                    self.stage = self.unlocked;
                    self.timer = 0;
                    self.sfx(Channel::Square2, 880, 60, 40);
                }
            }
            Mode::Select => {
                if self.keys.pressed(key::UP) || self.keys.pressed(key::RIGHT) {
                    self.stage = (self.stage + 1).min(self.unlocked);
                    self.sfx(Channel::Square2, 660, 30, 30);
                }
                if self.keys.pressed(key::DOWN) || self.keys.pressed(key::LEFT) {
                    self.stage = self.stage.saturating_sub(1);
                    self.sfx(Channel::Square2, 520, 30, 30);
                }
                if self.keys.pressed(key::B) {
                    self.mode = Mode::Title;
                    self.timer = 0;
                } else if self.keys.pressed(key::A) || self.keys.pressed(key::START) {
                    self.new_game();
                }
            }
            Mode::Play => self.play(),
            Mode::Tally => {
                // 一行一行出来，全出来以后按 A 或等一会儿。
                let rows = 5;
                if self.timer.is_multiple_of(30) && self.timer / 30 <= rows {
                    self.sfx(Channel::Square2, 1200, 30, 30);
                }
                let done = self.timer > (rows + 1) * 30;
                if done && (self.keys.pressed(key::A) || self.keys.pressed(key::START) || self.timer > (rows + 1) * 30 + 180) {
                    self.after_tally();
                }
            }
            Mode::Over | Mode::AllClear => {
                if self.timer > 60 && (self.keys.pressed(key::A) || self.keys.pressed(key::START)) || self.timer > 360 {
                    self.mode = Mode::Title;
                    self.timer = 0;
                }
            }
        }
        self.draw();
    }

    fn after_tally(&mut self) {
        let best = self.score.iter().copied().max().unwrap_or(0);
        if best > self.hi {
            self.hi = best;
        }
        if self.over.is_some() {
            self.mode = Mode::Over;
            self.timer = 0;
            self.music.play(game_over_tune());
        } else if self.stage + 1 >= LEVELS.len() {
            self.mode = Mode::AllClear;
            self.timer = 0;
            self.music.play(victory_tune());
        } else {
            self.stage += 1;
            self.unlocked = self.unlocked.max(self.stage);
            self.start_stage();
        }
        self.save();
    }

    fn play(&mut self) {
        if self.keys.pressed(key::START) && self.over.is_none() {
            self.paused = !self.paused;
            self.sfx(Channel::Square2, 880, 40, 40);
        }
        if self.paused {
            return;
        }
        self.freeze = self.freeze.saturating_sub(1);
        if self.shovel > 0 {
            self.shovel -= 1;
            // 快到时间了闪：钢、砖换着来。
            let steel = self.shovel > 0 && (self.shovel > 180 || (self.shovel / 16).is_multiple_of(2));
            self.set_ring(steel);
        }
        self.spawn_enemies();
        for p in 0..self.players {
            self.drive_player(p);
        }
        self.drive_enemies();
        self.move_shots();
        for t in self.tanks.iter_mut() {
            t.spawn = t.spawn.saturating_sub(1);
            t.shield = t.shield.saturating_sub(1);
            t.flash = t.flash.saturating_sub(1);
        }
        self.pick_bonus();
        self.booms.retain_mut(|b| {
            b.t += 1;
            b.t < if b.big { 30 } else { 12 }
        });
        self.popups.retain_mut(|p| {
            p.t += 1;
            p.t < 50
        });
        self.engine();
        // 输了：「游戏结束」从下面升上来，过一会儿数分。
        if let Some(t) = self.over.as_mut() {
            *t += 1;
            if *t > 200 {
                self.mode = Mode::Tally;
                self.timer = 0;
            }
            return;
        }
        if self.queue.is_empty() && self.enemies_alive() == 0 {
            let t = self.clear.get_or_insert(0);
            *t += 1;
            if *t > 150 {
                self.mode = Mode::Tally;
                self.timer = 0;
            }
        }
    }

    /// 发动机：开着和停着两种低音，在三角波上。开局的号角响着时不发。
    fn engine(&mut self) {
        if self.music.playing() || !self.tick.is_multiple_of(6) || self.over.is_some() {
            return;
        }
        let Some(p) = self.player(0) else { return };
        let (hz, vol) = if p.moving { (if self.tick.is_multiple_of(12) { 92 } else { 98 }, 34) } else { (if self.tick.is_multiple_of(12) { 55 } else { 58 }, 22) };
        tone(Channel::Triangle, hz, 120, vol);
    }

    fn spawn_enemies(&mut self) {
        self.spawn_timer = self.spawn_timer.saturating_sub(1);
        if self.queue.is_empty() || self.enemies_alive() >= MAX_ALIVE || self.spawn_timer > 0 {
            return;
        }
        // 三个口子轮着出；口子上有车就等下一帧。
        let at = GATES[self.gate];
        if self.tanks.iter().any(|t| overlap((t.x, t.y, 16, 16), (at.0, at.1, 16, 16))) {
            return;
        }
        let kind = self.queue.pop().unwrap();
        let mut t = Tank::new(self.next_id, kind, at);
        self.next_id += 1;
        t.carrier = CARRIERS.contains(&self.spawned);
        t.turn = 30 + self.rng.below(60);
        self.spawned += 1;
        self.tanks.push(t);
        self.gate = (self.gate + 1) % GATES.len();
        self.spawn_timer = 190u32.saturating_sub(self.stage as u32 * 5).max(60);
    }

    /// 车 `i` 想往 `dir` 开 `speed` 份。转弯时顺着原来的方向对齐到 8 像素，好钻进路口。回开没开动。
    fn drive(&mut self, i: usize, dir: Dir, speed: i32) -> bool {
        let t = &self.tanks[i];
        let (mut x, mut y) = (t.x, t.y);
        if dir != t.dir {
            if dir.vertical() != t.dir.vertical() {
                let (sx, sy) = if t.dir.vertical() { (x, (y + 4) / CELL * CELL) } else { ((x + 4) / CELL * CELL, y) };
                if !self.blocked(sx, sy, 16, 16) && !self.tank_in_way(i, sx, sy) {
                    (x, y) = (sx, sy);
                }
            }
            let t = &mut self.tanks[i];
            t.dir = dir;
            t.acc = 0;
        }
        let t = &mut self.tanks[i];
        (t.x, t.y) = (x, y);
        t.acc += speed;
        let mut moved = false;
        let (dx, dy) = dir.step();
        while self.tanks[i].acc >= SUB {
            self.tanks[i].acc -= SUB;
            let (nx, ny) = (self.tanks[i].x + dx, self.tanks[i].y + dy);
            if self.blocked(nx, ny, 16, 16) || self.tank_in_way(i, nx, ny) {
                self.tanks[i].acc = 0;
                break;
            }
            self.tanks[i].x = nx;
            self.tanks[i].y = ny;
            moved = true;
        }
        let t = &mut self.tanks[i];
        t.moving = moved;
        if moved && self.tick.is_multiple_of(4) {
            t.anim ^= 1;
        }
        moved
    }

    /// 别的车挡在 (x, y) 上。本来就叠着的不算（出车时叠上了，让它能开出来）。
    fn tank_in_way(&self, i: usize, x: i32, y: i32) -> bool {
        let me = &self.tanks[i];
        self.tanks.iter().enumerate().any(|(j, o)| {
            j != i && o.active() && overlap((x, y, 16, 16), (o.x, o.y, 16, 16)) && !overlap((me.x, me.y, 16, 16), (o.x, o.y, 16, 16))
        })
    }

    fn drive_player(&mut self, p: usize) {
        if self.respawn[p] > 0 {
            self.respawn[p] -= 1;
            if self.respawn[p] == 0 && self.over.is_none() {
                self.spawn_player(p);
            }
            return;
        }
        let Some(i) = self.tanks.iter().position(|t| t.id == p as u32) else { return };
        if !self.tanks[i].active() || self.over.is_some() {
            self.tanks[i].moving = false;
            return;
        }
        // 方向：几个一起按时听后按下的那个。
        let k = &self.keys;
        let all = [(key::UP, Dir::Up), (key::DOWN, Dir::Down), (key::LEFT, Dir::Left), (key::RIGHT, Dir::Right)];
        if let Some(&(_, d)) = all.iter().find(|(b, _)| k.pressed(*b)) {
            self.last_dir = Some(d);
        }
        let held: Vec<Dir> = all.iter().filter(|(b, _)| k.held(*b)).map(|(_, d)| *d).collect();
        let want = match self.last_dir {
            Some(d) if held.contains(&d) => Some(d),
            _ => held.first().copied(),
        };
        let fire = k.pressed(key::A) || k.pressed(key::B);
        let speed = Kind::Player.speed();
        let t = &self.tanks[i];
        let on_ice = self.ground_at(t.x + 8, t.y + 8) == Ground::Ice;
        match want {
            Some(d) => {
                self.drive(i, d, speed);
                self.tanks[i].slide = if on_ice { 28 } else { 0 };
            }
            None if self.tanks[i].slide > 0 => {
                self.tanks[i].slide -= 1;
                let d = self.tanks[i].dir;
                if !self.drive(i, d, speed) {
                    self.tanks[i].slide = 0;
                }
            }
            None => self.tanks[i].moving = false,
        }
        if fire {
            self.fire(i);
        }
    }

    fn drive_enemies(&mut self) {
        let frozen = self.freeze > 0;
        let n = self.tanks.len();
        for i in 0..n {
            if i >= self.tanks.len() {
                break;
            }
            let t = &self.tanks[i];
            if t.is_player() || !t.active() {
                continue;
            }
            if frozen {
                self.tanks[i].moving = false;
                continue;
            }
            let (kind, dir) = (t.kind, t.dir);
            let moved = self.drive(i, dir, kind.speed());
            let t = &mut self.tanks[i];
            t.turn = t.turn.saturating_sub(1);
            if !moved || t.turn == 0 {
                // 撞了墙有时先打一发（打穿砖墙），再换方向。
                if !moved && self.rng.below(3) == 0 {
                    self.fire(i);
                }
                let d = self.pick_dir(i);
                let turn = 40 + self.rng.below(120);
                let t = &mut self.tanks[i];
                t.dir = d;
                t.turn = turn;
            }
            // 隔一会儿打一发：后面的关打得勤，炮车更勤。
            let rate = 70u32.saturating_sub(self.stage as u32 * 2).max(28) / if kind == Kind::Power { 2 } else { 1 };
            if self.rng.below(rate) == 0 {
                self.fire(i);
            }
        }
    }

    /// 敌车换个方向：后面的关更常朝营地或一号车开，别的时候多往下走。
    fn pick_dir(&mut self, i: usize) -> Dir {
        let (x, y) = (self.tanks[i].x, self.tanks[i].y);
        let aim = 20 + self.stage as u32 * 2;
        if self.rng.below(100) < aim {
            let chase = self.rng.below(3) == 0;
            let (tx, ty) = match self.player(0) {
                Some(p) if chase => (p.x, p.y),
                _ => BASE,
            };
            let (dx, dy) = (tx - x, ty - y);
            return if dx.abs() > dy.abs() && self.rng.below(2) == 0 {
                if dx > 0 {
                    Dir::Right
                } else {
                    Dir::Left
                }
            } else if dy > 0 {
                Dir::Down
            } else {
                Dir::Up
            };
        }
        match self.rng.below(9) {
            0..=3 => Dir::Down,
            4..=5 => Dir::Left,
            6..=7 => Dir::Right,
            _ => Dir::Up,
        }
    }

    /// 车 `i` 打一发。一号车火力 2 以上能同时有两发在飞，别的车一发。
    fn fire(&mut self, i: usize) {
        let t = &self.tanks[i];
        if !t.active() {
            return;
        }
        let max = if t.is_player() && t.level >= 2 { 2 } else { 1 };
        if self.shots.iter().filter(|s| s.owner == t.id && s.alive).count() >= max {
            return;
        }
        let (dx, dy) = t.dir.step();
        let fast = (t.is_player() && t.level >= 1) || t.kind == Kind::Power;
        let s = Shot {
            x: t.x + 6 + dx * 6,
            y: t.y + 6 + dy * 6,
            dir: t.dir,
            owner: t.id,
            from_player: t.is_player(),
            speed: if fast { 5 } else { 3 },
            pierce: t.is_player() && t.level >= 3,
            alive: true,
        };
        let player = t.is_player();
        self.shots.push(s);
        if player {
            self.sfx(Channel::Square2, 523, 40, 40);
        }
    }

    fn move_shots(&mut self) {
        let mut shots = std::mem::take(&mut self.shots);
        for s in shots.iter_mut() {
            let (dx, dy) = s.dir.step();
            for _ in 0..s.speed {
                if !s.alive {
                    break;
                }
                s.x += dx;
                s.y += dy;
                self.hit(s);
            }
        }
        // 两发对着撞上：一号车的和敌车的，一起没了。
        for a in 0..shots.len() {
            for b in a + 1..shots.len() {
                if shots[a].alive && shots[b].alive && shots[a].from_player != shots[b].from_player && overlap((shots[a].x - 1, shots[a].y - 1, 6, 6), (shots[b].x, shots[b].y, 4, 4)) {
                    shots[a].alive = false;
                    shots[b].alive = false;
                }
            }
        }
        shots.retain(|s| s.alive);
        // 打的时候可能又打了新的（没有这种情况，留着稳妥）。
        shots.append(&mut self.shots);
        self.shots = shots;
    }

    /// 子弹走了一个像素以后看撞了什么。
    fn hit(&mut self, s: &mut Shot) {
        let rect = (s.x, s.y, 4, 4);
        if s.x < 0 || s.y < 0 || s.x + 4 > FIELD || s.y + 4 > FIELD {
            s.alive = false;
            self.booms.push(Boom { x: s.x + 2, y: s.y + 2, t: 0, big: false });
            if s.from_player {
                self.sfx(Channel::Square2, 1568, 30, 30);
            }
            return;
        }
        if self.base_alive && overlap(rect, (BASE.0, BASE.1, 16, 16)) {
            s.alive = false;
            self.lose_base();
            return;
        }
        if self.wall_in(rect) {
            s.alive = false;
            self.break_wall(s);
            return;
        }
        for i in 0..self.tanks.len() {
            let t = &self.tanks[i];
            if t.id == s.owner || !t.active() || !overlap(rect, (t.x, t.y, 16, 16)) {
                continue;
            }
            if s.from_player && !t.is_player() {
                s.alive = false;
                self.damage(i);
                return;
            }
            if !s.from_player && t.is_player() {
                s.alive = false;
                if t.shield == 0 {
                    self.kill_player(i);
                }
                return;
            }
            // 一号车打二号车：双人时定住一会儿（等双人做）。敌车的子弹穿过敌车。
        }
    }

    fn wall_in(&self, (x, y, w, h): (i32, i32, i32, i32)) -> bool {
        for my in y / MINI..=(y + h - 1) / MINI {
            for mx in x / MINI..=(x + w - 1) / MINI {
                if self.bricks[my as usize][mx as usize] {
                    return true;
                }
            }
        }
        for cy in y / CELL..=(y + h - 1) / CELL {
            for cx in x / CELL..=(x + w - 1) / CELL {
                if self.ground[cy as usize][cx as usize] == Ground::Steel {
                    return true;
                }
            }
        }
        false
    }

    /// 子弹撞墙：前面那排砖打掉 16 像素宽一条（子弹在中间）；火力满的连钢墙一起打掉。
    fn break_wall(&mut self, s: &Shot) {
        let (cx, cy) = (s.x + 2, s.y + 2);
        let front = match s.dir {
            Dir::Up => s.y,
            Dir::Down => s.y + 3,
            Dir::Left => s.x,
            Dir::Right => s.x + 3,
        };
        let (lo, hi) = if s.dir.vertical() { ((cx - 8).max(0), (cx + 7).min(FIELD - 1)) } else { ((cy - 8).max(0), (cy + 7).min(FIELD - 1)) };
        let depth = if s.pierce { 2 } else { 1 };
        let mut broke = false;
        let mut steel = false;
        for d in 0..depth {
            let line = front / MINI + if matches!(s.dir, Dir::Up | Dir::Left) { -d } else { d };
            if !(0..MINIS as i32).contains(&line) {
                continue;
            }
            for m in lo / MINI..=hi / MINI {
                let (mx, my) = if s.dir.vertical() { (m, line) } else { (line, m) };
                let cell = &mut self.bricks[my as usize][mx as usize];
                if *cell {
                    *cell = false;
                    broke = true;
                }
            }
        }
        let cline = front / CELL;
        for c in lo / CELL..=hi / CELL {
            let (gx, gy) = if s.dir.vertical() { (c, cline) } else { (cline, c) };
            let g = &mut self.ground[gy as usize][gx as usize];
            if *g == Ground::Steel {
                steel = true;
                if s.pierce {
                    *g = Ground::Open;
                    broke = true;
                }
            }
        }
        self.booms.push(Boom { x: cx, y: cy, t: 0, big: false });
        if s.from_player {
            if broke {
                self.sfx(Channel::Noise, 1800, 60, 45);
            } else if steel {
                self.sfx(Channel::Square2, 1568, 30, 30);
            }
        }
    }

    /// 一号车的子弹打中敌车 `i`。
    fn damage(&mut self, i: usize) {
        let t = &mut self.tanks[i];
        if t.carrier {
            t.carrier = false;
            self.drop_bonus();
        }
        let t = &mut self.tanks[i];
        t.hp = t.hp.saturating_sub(1);
        if t.hp > 0 {
            t.flash = 6;
            self.sfx(Channel::Square2, 300, 40, 40);
            return;
        }
        let t = self.tanks.remove(i);
        self.killed[t.kind.slot()] += 1;
        self.add_score(0, t.kind.points());
        self.booms.push(Boom { x: t.x + 8, y: t.y + 8, t: 0, big: true });
        self.popups.push(Popup { x: t.x + 2, y: t.y + 4, text: t.kind.points().to_string(), t: 0 });
        tone_wav(&self.boom_small);
    }

    fn kill_player(&mut self, i: usize) {
        let t = self.tanks.remove(i);
        let p = t.id as usize;
        self.booms.push(Boom { x: t.x + 8, y: t.y + 8, t: 0, big: true });
        tone_wav(&self.boom_big);
        if self.lives[p] > 0 {
            self.lives[p] -= 1;
            self.respawn[p] = 60;
        } else if (0..self.players).all(|q| self.player(q).is_none() && (self.lives[q] == 0 || q == p)) {
            self.over = Some(0);
        }
    }

    fn lose_base(&mut self) {
        self.base_alive = false;
        self.booms.push(Boom { x: BASE.0 + 8, y: BASE.1 + 8, t: 0, big: true });
        tone_wav(&self.boom_big);
        if self.over.is_none() {
            self.over = Some(0);
        }
    }

    fn add_score(&mut self, p: usize, n: u32) {
        self.score[p] += n;
        if self.score[p] >= self.next_life[p] {
            self.next_life[p] += 20000;
            self.lives[p] += 1;
            self.arpeggio(Channel::Square1, &[1047, 1568, 2093], 5, 45);
        }
    }

    fn drop_bonus(&mut self) {
        let kind = BONUSES[self.rng.below(BONUSES.len() as u32) as usize];
        // 找个地方：营地附近不放。
        let mut at = (0, 0);
        for _ in 0..20 {
            let (x, y) = (self.rng.below(19) as i32 * CELL, self.rng.below(17) as i32 * CELL);
            if !overlap((x, y, 16, 16), (56, 128, 48, 32)) {
                at = (x, y);
                break;
            }
        }
        self.bonus = Some((kind, at.0, at.1));
        self.arpeggio(Channel::Square1, &[784, 988, 1175, 1568], 3, 40);
    }

    fn pick_bonus(&mut self) {
        let Some((kind, bx, by)) = self.bonus else { return };
        let Some(i) = self.tanks.iter().position(|t| t.is_player() && t.active() && overlap((t.x, t.y, 16, 16), (bx, by, 16, 16))) else { return };
        self.bonus = None;
        let p = self.tanks[i].id as usize;
        self.apply(kind, i);
        self.add_score(p, 500);
        self.popups.push(Popup { x: bx, y: by + 4, text: "500".into(), t: 0 });
        self.arpeggio(Channel::Square1, &[523, 659, 784, 1047, 1319], 3, 45);
    }

    /// 道具起作用。`i` 是捡到的那辆一号车。
    pub fn apply(&mut self, kind: Bonus, i: usize) {
        match kind {
            Bonus::Power => {
                let t = &mut self.tanks[i];
                t.level = (t.level + 1).min(3);
            }
            Bonus::Bomb => {
                // 场上的敌车全炸了，不算分。
                let gone: Vec<Tank> = self.tanks.iter().filter(|t| !t.is_player() && t.active()).cloned().collect();
                self.tanks.retain(|t| t.is_player() || !t.active());
                for t in gone {
                    self.killed[t.kind.slot()] += 1;
                    self.booms.push(Boom { x: t.x + 8, y: t.y + 8, t: 0, big: true });
                }
                tone_wav(&self.boom_big);
            }
            Bonus::Shield => self.tanks[i].shield = 600,
            Bonus::Shovel => {
                self.shovel = 900;
                self.set_ring(true);
            }
            Bonus::Clock => self.freeze = 600,
        }
    }

    // ---------- 画 ----------

    fn draw(&mut self) {
        let f = &mut self.frame;
        f.clear(BG);
        match self.mode {
            Mode::Title => self.draw_title(),
            Mode::Select => self.draw_select(),
            Mode::Play => {
                self.draw_field();
                self.draw_panel();
                if self.paused && (self.tick / 20).is_multiple_of(2) {
                    let f = &mut self.frame;
                    let w = FONT.width("暂停");
                    f.rect(80 - w / 2 - 4, 72, w + 8, 16, INK);
                    f.text(&FONT, 80 - w / 2, 74, "暂停", GOLD);
                }
                if let Some(t) = self.over {
                    // 「游戏结束」从场地下面升到中间。
                    let y = (160 - t as i32).max(72);
                    let s = "游戏结束";
                    let w = FONT.width(s);
                    self.frame.text_outlined(&FONT, 80 - w / 2, y, s, RED, INK);
                }
            }
            Mode::Tally => self.draw_tally(),
            Mode::Over => {
                let big = FONT.sprite("游戏结束", RED).scaled(2);
                let f = &mut self.frame;
                f.blit(&big, (W as i32 - big.w) / 2, 50, false);
                f.text_center(&FONT, 100, &format!("得分 {}", self.score[0]), WHITE);
                f.text_center(&FONT, 116, &format!("最高 {}", self.hi), GOLD);
            }
            Mode::AllClear => {
                let big = FONT.sprite("全部守住了", GOLD).scaled(2);
                let f = &mut self.frame;
                f.blit(&big, (W as i32 - big.w) / 2, 44, false);
                f.text_center(&FONT, 92, "二十关都打完了", WHITE);
                f.text_center(&FONT, 110, &format!("得分 {}", self.score[0]), WHITE);
                f.text_center(&FONT, 126, &format!("最高 {}", self.hi), GOLD);
            }
        }
    }

    /// 画砖墙的一小块（4×4）：取砖的精灵里对应的那个角，墙纹对得上。
    fn brick_mini(f: &mut Frame, s: &Sprite, mx: usize, my: usize) {
        let (sx, sy) = ((mx as i32 % 2) * MINI, (my as i32 % 2) * MINI);
        let (x, y) = (mx as i32 * MINI, my as i32 * MINI);
        for yy in 0..MINI {
            for xx in 0..MINI {
                let c = s.at(sx + xx, sy + yy);
                if c != 0 {
                    f.set(x + xx, y + yy, c);
                }
            }
        }
    }

    fn draw_field(&mut self) {
        let tick = self.tick;
        let f = &mut self.frame;
        let a = &self.art;
        f.rect(0, 0, FIELD, FIELD, BG);
        for cy in 0..CELLS {
            for cx in 0..CELLS {
                let (x, y) = (cx as i32 * CELL, cy as i32 * CELL);
                match self.ground[cy][cx] {
                    Ground::Steel => f.blit(&a.steel, x, y, false),
                    Ground::Water => f.blit(&a.water[((tick / 32) as usize + cx) % 2], x, y, false),
                    Ground::Ice => f.blit(&a.ice, x, y, false),
                    _ => {}
                }
            }
        }
        for my in 0..MINIS {
            for mx in 0..MINIS {
                if self.bricks[my][mx] {
                    Self::brick_mini(f, &a.brick, mx, my);
                }
            }
        }
        f.blit(if self.base_alive { &a.camp } else { &a.ruin }, BASE.0, BASE.1, false);
        for t in &self.tanks {
            if !t.active() {
                // 出车的光：四张来回闪。
                let k = [0, 1, 2, 3, 2, 1][((48 - t.spawn) / 4) as usize % 6];
                f.blit(&a.spark[k], t.x + 4, t.y + 4, false);
                continue;
            }
            let k = match t.kind {
                Kind::Player => 0,
                Kind::Basic => 1,
                Kind::Fast => 2,
                Kind::Power => 3,
                Kind::Armor => 4,
            };
            let s = a.tanks[k][t.dir.turns()][t.anim as usize].recolor(&tank_colors(t, tick));
            f.blit(&s, t.x, t.y, false);
            if t.shield > 0 {
                f.blit(&a.shield[((tick / 2) % 2) as usize], t.x, t.y, false);
            }
        }
        for s in &self.shots {
            f.blit(&a.shot, s.x, s.y, false);
        }
        for cy in 0..CELLS {
            for cx in 0..CELLS {
                if self.ground[cy][cx] == Ground::Grass {
                    f.blit(&a.grass, cx as i32 * CELL, cy as i32 * CELL, false);
                }
            }
        }
        if let Some((kind, x, y)) = self.bonus {
            if !(tick / 12).is_multiple_of(3) {
                let i = BONUSES.iter().position(|b| *b == kind).unwrap();
                f.blit(&a.bonus[i], x, y, false);
            }
        }
        for b in &self.booms {
            if b.big {
                let k = (b.t / 6).min(4) as usize;
                let (s, scale) = match k {
                    0 => (&a.boom[0], 1),
                    1 => (&a.boom[1], 1),
                    2 | 3 => (&a.boom[2], 2),
                    _ => (&a.boom[1], 2),
                };
                if scale == 1 {
                    f.blit(s, b.x - 8, b.y - 8, false);
                } else {
                    f.blit(&s.scaled(2), b.x - 16, b.y - 16, false);
                }
            } else {
                let k = (b.t / 4).min(2) as usize;
                let s = if k == 1 { &a.boom[1] } else { &a.boom[0] };
                f.blit(s, b.x - 8, b.y - 8, false);
            }
        }
        for p in &self.popups {
            f.text_outlined(&FONT, p.x, p.y - (p.t / 6) as i32, &p.text, WHITE, INK);
        }
    }

    /// 右边的记分板：还剩几辆敌车、几条命、火力、第几关、分数。
    fn draw_panel(&mut self) {
        let level = self.player(0).map(|p| p.level).unwrap_or(0);
        let f = &mut self.frame;
        let a = &self.art;
        let x0 = FIELD;
        f.rect(x0, 0, W as i32 - x0, 160, PANEL);
        f.rect(x0, 0, 1, 160, PANEL_LO);
        // 还没出来的敌车，四列。
        for i in 0..self.queue.len() {
            let (cx, cy) = (x0 + 10 + (i as i32 % 4) * 9, 6 + (i as i32 / 4) * 9);
            f.blit(&a.icon, cx, cy, false);
        }
        f.text(&FONT, x0 + 8, 56, "一号车", INK);
        f.blit(&a.life, x0 + 10, 72, false);
        f.text(&FONT, x0 + 22, 69, &format!("×{}", self.lives[0]), INK);
        f.text(&FONT, x0 + 8, 86, "火力", INK);
        for k in 0..3 {
            let c = if k < level { GOLD } else { PANEL_LO };
            f.rect(x0 + 36 + k as i32 * 8, 90, 6, 6, c);
        }
        f.blit(&a.flag, x0 + 10, 106, false);
        f.text(&FONT, x0 + 24, 105, &format!("{}", self.stage + 1), INK);
        f.text(&FONT, x0 + 8, 124, "分", INK);
        f.text(&FONT, x0 + 8, 140, &format!("{:06}", self.score[0]), WHITE);
    }

    fn draw_title(&mut self) {
        let tick = self.tick;
        let big = FONT.sprite("铁壳守营", GOLD).scaled(3);
        let shade = FONT.sprite("铁壳守营", R_LO).scaled(3);
        let f = &mut self.frame;
        let a = &self.art;
        // 底下一排砖，两边一辆一号车、一辆敌车对着。
        for x in (0..W as i32).step_by(8) {
            f.blit(&a.brick, x, 136, false);
            f.blit(&a.brick, x, 144, false);
            f.blit(&a.brick, x, 152, false);
        }
        let x = (W as i32 - big.w) / 2;
        f.blit(&shade, x + 3, 27, false);
        f.blit(&big, x, 24, false);
        f.text_center(&FONT, 70, "守住营地，打掉二十辆敌车", WHITE);
        f.blit(&a.tanks[0][Dir::Right.turns()][(tick / 8 % 2) as usize].recolor(&[(T_BODY, P_MID), (T_HI, P_HI), (T_LO, P_LO), (T_ACC, ORANGE)]), 40, 116, false);
        f.blit(&a.tanks[4][Dir::Left.turns()][(tick / 8 % 2) as usize].recolor(&[(T_BODY, V_MID), (T_HI, V_HI), (T_LO, V_LO), (T_ACC, GOLD)]), 184, 116, false);
        if (tick / 30).is_multiple_of(2) {
            f.text_center(&FONT, 92, "按 START 开始", GOLD);
        }
        f.text_center(&FONT, 110, &format!("最高 {}", self.hi), WHITE);
    }

    fn draw_select(&mut self) {
        let f = &mut self.frame;
        f.rect(0, 0, W as i32, 160, PANEL_LO);
        let title = FONT.sprite(&format!("第 {} 关", self.stage + 1), WHITE).scaled(2);
        f.blit(&title, (W as i32 - title.w) / 2, 14, false);
        // 这一关的地图缩小看：一个小格画成 3 像素。
        let (ox, oy) = (90, 50);
        f.rect(ox - 2, oy - 2, 64, 64, INK);
        let lv = LEVELS[self.stage];
        for (ty, row) in lv.iter().enumerate() {
            for (tx, ch) in row.chars().enumerate() {
                let c = match ch {
                    '#' | '<' | '>' | '^' | 'v' => BRICK,
                    '@' | 'l' | 'r' | 't' | 'b' => STEEL,
                    '~' => WATER,
                    '%' => LEAF,
                    '-' => ICE,
                    _ => BG,
                };
                f.rect(ox + tx as i32 * 6, oy + ty as i32 * 6, 6, 6, c);
            }
        }
        f.rect(ox + 27, oy + 54, 6, 6, CANVAS);
        let hint = if self.unlocked > 0 { "上下选关　A 开始　B 回去" } else { "A 开始　B 回去" };
        f.text_center(&FONT, 124, hint, GOLD);
        f.text_center(&FONT, 140, &format!("开到第 {} 关", self.unlocked + 1), WHITE);
    }

    fn draw_tally(&mut self) {
        let f = &mut self.frame;
        let a = &self.art;
        let shown = self.timer / 30;
        f.text_center(&FONT, 6, &format!("第 {} 关", self.stage + 1), WHITE);
        f.text_center(&FONT, 22, &format!("最高 {}", self.hi), GOLD);
        let names = ["普通", "快车", "炮车", "重甲"];
        let mut total = 0;
        for (r, kind) in Kind::ENEMIES.iter().enumerate() {
            if shown < r as u32 + 1 {
                break;
            }
            let n = self.killed[r];
            total += n;
            let y = 42 + r as i32 * 18;
            f.text(&FONT, 40, y, &format!("{:>5} 分", n * kind.points()), WHITE);
            f.text(&FONT, 104, y, &format!("{:>2} 辆", n), WHITE);
            let s = a.tanks[r + 1][0][0].recolor(&[(T_BODY, E_MID), (T_HI, E_HI), (T_LO, E_LO), (T_ACC, RED)]);
            f.blit(&s, 150, y - 2, false);
            f.text(&FONT, 172, y, names[r], WHITE);
        }
        if shown >= 5 {
            f.rect(40, 116, 160, 1, WHITE);
            f.text(&FONT, 104, 122, &format!("{:>2} 辆", total), WHITE);
            f.text(&FONT, 40, 140, &format!("一号车 {}", self.score[0]), GOLD);
        }
    }
}

static STATE: Mutex<Option<Game>> = Mutex::new(None);

vibegame::main!(|app| {
    app.every(|| {
        let mut s = STATE.lock().unwrap();
        let g = s.get_or_insert_with(Game::default);
        g.keys.read();
        g.step();
        g.frame.show();
    });
});

#[cfg(test)]
mod tests;
