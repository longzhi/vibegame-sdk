//! 四关：地图（按格子生成）和敌人出场的脚本。
//!
//! 画面往上飞，地图从最下面一行往上卷。脚本一行一条：「卷过几像素 敌人 横坐标 几架」。
//! 卷到顶就停，大怪出来。

use crate::art::*;
use vibeplay::full::{lang, TileMap};

/// 地图多少行（一行 16 像素）。卷完要 (行数 × 16 − 160) × 2 帧，约 85 秒。
pub const ROWS: i32 = 170;
pub const COLS: i32 = 15;

pub struct StageInfo {
    pub name: &'static str,
    /// 英文的关名。
    pub name_en: &'static str,
    /// 地图块后面露出来的底色。
    pub base: u8,
    pub script: &'static str,
}

pub const STAGES: [StageInfo; 4] = [
    StageInfo { name: "海岸线", name_en: "Coastline", base: SEA, script: COAST },
    StageInfo { name: "沙海", name_en: "Dune Sea", base: DUNE, script: DESERT },
    StageInfo { name: "夜云", name_en: "Night Clouds", base: NIGHT, script: CLOUDS },
    StageInfo { name: "天穹要塞", name_en: "Sky Fortress", base: METAL_LO, script: FORTRESS },
];

impl StageInfo {
    /// 按系统语言出的关名。
    pub fn label(&self) -> &'static str {
        lang().pick(self.name_en, self.name)
    }
}

/// 一种敌人。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Dart,
    SwoopL,
    SwoopR,
    Bomber,
    Carrier,
    Mine,
    Turret,
}

/// 脚本的一条：卷过 `at` 像素时，在横坐标 `x` 出 `n` 架。飞的一架接一架，隔 16 帧；地面炮台一起出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wave {
    pub at: i32,
    pub kind: Kind,
    pub x: i32,
    pub n: u32,
}

pub fn parse(script: &str) -> Result<Vec<Wave>, String> {
    let mut out = Vec::new();
    for (i, line) in script.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let w: Vec<&str> = line.split_whitespace().collect();
        let bad = || format!("第 {} 行：{line}", i + 1);
        if w.len() != 4 {
            return Err(bad());
        }
        let kind = match w[1] {
            "dart" => Kind::Dart,
            "swoopl" => Kind::SwoopL,
            "swoopr" => Kind::SwoopR,
            "bomber" => Kind::Bomber,
            "carrier" => Kind::Carrier,
            "mine" => Kind::Mine,
            "turret" => Kind::Turret,
            _ => return Err(bad()),
        };
        let num = |s: &str| s.parse::<i32>().map_err(|_| bad());
        out.push(Wave { at: num(w[0])?, kind, x: num(w[2])?, n: num(w[3])? as u32 });
    }
    out.sort_by_key(|w| w.at);
    Ok(out)
}

/// 0..1000 的平滑噪声：格子 (x, y) 附近的几个随机数插起来。同样的数每次一样。
fn noise(stage: usize, x: i32, y: i32, scale: i32) -> i32 {
    let h = |gx: i32, gy: i32| -> i32 {
        let mut v = (gx as u32).wrapping_mul(374761393) ^ (gy as u32).wrapping_mul(668265263) ^ (stage as u32).wrapping_mul(2246822519);
        v = (v ^ (v >> 13)).wrapping_mul(1274126177);
        ((v ^ (v >> 16)) % 1000) as i32
    };
    let (gx, gy) = (x.div_euclid(scale), y.div_euclid(scale));
    let (fx, fy) = (x.rem_euclid(scale), y.rem_euclid(scale));
    let top = h(gx, gy) * (scale - fx) + h(gx + 1, gy) * fx;
    let bot = h(gx, gy + 1) * (scale - fx) + h(gx + 1, gy + 1) * fx;
    (top * (scale - fy) + bot * fy) / (scale * scale)
}

/// 生成一关的地图。格子的值是 `art::tiles(stage)` 的下标。
pub fn map(stage: usize) -> TileMap {
    let mut m = TileMap::new(COLS, ROWS, 16);
    for y in 0..ROWS {
        for x in 0..COLS {
            let n = noise(stage, x, y, 4);
            let fine = noise(stage, x, y, 1);
            let v = match stage {
                // 海：零星的浪，大小不一的岛，岛边一圈沙，礁石。开头和大怪那块是海。
                0 => {
                    let open = !(12..ROWS - 14).contains(&y);
                    if !open && n > 640 {
                        3
                    } else if !open && n > 570 {
                        2
                    } else if fine > 960 {
                        4
                    } else if fine % 7 == 0 {
                        1
                    } else {
                        0
                    }
                }
                // 沙漠：沙丘、石头，一条弯弯的路，偶尔一片废墟。
                1 => {
                    let road = 7 + (((y as f32) / 9.0).sin() * 3.0) as i32;
                    if x == road {
                        3
                    } else if n > 720 && fine > 500 {
                        4
                    } else if n < 230 {
                        2
                    } else if fine % 3 == 0 {
                        1
                    } else {
                        0
                    }
                }
                // 夜里的云：一团一团，亮的在中间。
                2 => {
                    if n > 640 {
                        2
                    } else if n > 520 {
                        1
                    } else {
                        0
                    }
                }
                // 要塞：甲板铺满，两根管子，通风口和灯排成一行一行。
                _ => {
                    if x == 2 || x == 12 {
                        4
                    } else if y % 6 == 0 && x % 4 == 1 {
                        3
                    } else if (y + x) % 9 == 0 || fine > 920 {
                        2
                    } else if y % 12 >= 10 {
                        0
                    } else {
                        1
                    }
                }
            };
            m.set(x, y, v);
        }
    }
    m
}

// 横坐标是画面上的像素（0..240）。地面炮台的横坐标是它的左边。

const COAST: &str = "
# 一开始几队小飞镖，认识一下
60 dart 60 4
160 dart 170 4
260 dart 110 5
340 carrier 40 1
420 swoopl 30 3
470 swoopr 200 3
560 turret 64 1
580 turret 160 1
640 dart 40 4
660 dart 190 4
760 mine 120 1
820 swoopl 30 4
900 bomber 104 1
1040 dart 80 6
1100 carrier 180 1
1180 turret 32 1
1190 turret 112 1
1200 turret 192 1
1300 swoopr 210 4
1340 swoopl 20 4
1440 mine 60 1
1450 mine 180 1
1540 dart 120 6
1620 bomber 40 1
1640 bomber 170 1
1780 carrier 110 1
1860 swoopl 30 5
1900 swoopr 200 5
1980 turret 48 1
2000 turret 176 1
2100 dart 60 5
2120 dart 170 5
2200 mine 120 1
2280 swoopl 30 3
2300 swoopr 200 3
";

const DESERT: &str = "
60 dart 120 5
140 turret 40 1
150 turret 184 1
240 swoopl 30 4
300 swoopr 200 4
380 carrier 80 1
460 mine 60 1
470 mine 170 1
560 bomber 104 1
700 dart 40 5
720 dart 190 5
800 turret 96 1
810 turret 128 1
900 swoopr 210 5
960 swoopl 20 5
1060 carrier 170 1
1120 mine 120 1
1140 dart 60 6
1240 bomber 40 1
1260 bomber 170 1
1400 turret 24 1
1410 turret 88 1
1420 turret 152 1
1430 turret 208 1
1520 swoopl 30 5
1540 swoopr 200 5
1640 mine 40 1
1650 mine 120 1
1660 mine 200 1
1760 carrier 60 1
1820 dart 120 7
1920 bomber 104 1
2040 swoopl 20 6
2080 swoopr 210 6
2180 turret 64 1
2190 turret 160 1
2260 dart 80 5
2280 dart 150 5
";

const CLOUDS: &str = "
60 swoopl 30 4
120 swoopr 200 4
200 dart 60 6
220 dart 170 6
320 mine 80 1
330 mine 160 1
420 carrier 120 1
500 bomber 40 1
520 bomber 170 1
660 swoopl 20 6
700 swoopr 210 6
800 dart 120 8
900 mine 40 1
910 mine 120 1
920 mine 200 1
1000 carrier 60 1
1080 bomber 104 1
1200 swoopl 30 5
1210 swoopr 200 5
1300 dart 40 6
1320 dart 190 6
1420 bomber 40 1
1430 bomber 104 1
1440 bomber 170 1
1600 carrier 180 1
1660 swoopl 20 6
1690 swoopr 210 6
1780 mine 60 1
1790 mine 180 1
1880 dart 120 8
1980 bomber 60 1
1990 bomber 150 1
2120 swoopl 30 6
2140 swoopr 200 6
2240 dart 80 6
2250 dart 150 6
";

const FORTRESS: &str = "
60 turret 32 1
70 turret 192 1
140 dart 120 6
220 swoopl 30 5
260 swoopr 200 5
340 turret 64 1
350 turret 112 1
360 turret 160 1
440 carrier 120 1
500 bomber 40 1
510 bomber 170 1
640 mine 60 1
650 mine 120 1
660 mine 180 1
760 swoopl 20 6
780 swoopr 210 6
860 turret 32 1
870 turret 96 1
880 turret 144 1
890 turret 192 1
980 dart 60 7
1000 dart 170 7
1100 carrier 60 1
1160 bomber 104 1
1300 swoopl 30 6
1320 swoopr 200 6
1420 turret 48 1
1430 turret 176 1
1500 mine 40 1
1510 mine 120 1
1520 mine 200 1
1600 bomber 40 1
1610 bomber 104 1
1620 bomber 170 1
1780 carrier 180 1
1840 dart 120 8
1940 swoopl 20 6
1960 swoopr 210 6
2060 turret 32 1
2070 turret 96 1
2080 turret 144 1
2090 turret 192 1
2180 dart 60 6
2190 dart 170 6
2260 bomber 104 1
";
