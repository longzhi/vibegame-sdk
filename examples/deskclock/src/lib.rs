//! 小 app《桌面时钟》：竖在键盘边上看的大字时钟，底下一个像素鱼缸。
//!
//! 上面是点阵的大字「时:分」，冒号每秒闪一次，旁边小字是秒。最上面一行是日期和星期。
//! 下面是鱼缸：鱼游、气泡往上冒、水草摆。缸里的灯跟着钟走：白天蓝，傍晚暖，夜里暗，鱼睡觉。
//! A 换 12 和 24 小时制，上下换颜色，两样都存着。别的时候不用按。
//! 鱼和气泡只看帧数，不用随机数：`vibeplay shot --time` 每次截出来一样。

#[cfg(test)]
mod tests;

use std::sync::Mutex;
use vibeplay::full::{key, lang, Frame, Keys, Sprite, W};
use vibeplay::{Time, FONT};

const SAVE: &str = "settings";

// 调色板：时钟这一半跟着颜色主题，鱼缸这一半跟着时辰。每帧都重填。
const BG: u8 = 0;
const DIGIT: u8 = 1;
const DOT_OFF: u8 = 2;
const TEXT: u8 = 3;
const DIM: u8 = 4;
const GLASS: u8 = 5;
const LAMP: u8 = 6;
const BOX: u8 = 7;
const BOX_TEXT: u8 = 8;
const WATER: [u8; 4] = [10, 11, 12, 13];
const RAY: u8 = 14;
const SAND: u8 = 15;
const SAND_DARK: u8 = 16;
const PEBBLE: u8 = 17;
const LEAF: u8 = 18;
const LEAF_DARK: u8 = 19;
const ROCK: u8 = 20;
const ROCK_HI: u8 = 21;
const BUBBLE: u8 = 22;
const EYE: u8 = 30;
const ZZZ: u8 = 31;
/// 每条鱼三格：身子、鳍、花纹。
const FISH0: u8 = 40;

/// 鱼缸里的东西白天的颜色。傍晚、夜里在这上面调。
const TANK: [(u8, [u8; 3]); 13] = [
    (GLASS, [70, 84, 96]),
    (SAND, [214, 186, 128]),
    (SAND_DARK, [176, 148, 96]),
    (PEBBLE, [130, 116, 104]),
    (LEAF, [72, 176, 88]),
    (LEAF_DARK, [36, 120, 64]),
    (ROCK, [104, 104, 120]),
    (ROCK_HI, [150, 150, 166]),
    (BUBBLE, [220, 244, 255]),
    (EYE, [16, 16, 24]),
    (ZZZ, [200, 210, 255]),
    (BOX, [16, 20, 28]),
    (BOX_TEXT, [240, 240, 232]),
];

/// 鱼：身子、鳍、花纹的颜色，游的高度（像素），速度（1/16 像素每帧），起点。
struct FishKind {
    colors: [[u8; 3]; 3],
    y: i32,
    speed: i32,
    start: i32,
}

const FISH: [FishKind; 4] = [
    FishKind { colors: [[255, 128, 40], [255, 220, 200], [255, 255, 255]], y: 74, speed: 9, start: 40 },
    FishKind { colors: [[255, 212, 64], [232, 120, 40], [40, 40, 56]], y: 94, speed: 6, start: 300 },
    FishKind { colors: [[80, 160, 255], [255, 220, 64], [24, 56, 160]], y: 110, speed: 11, start: 170 },
    FishKind { colors: [[240, 88, 120], [255, 180, 200], [255, 240, 240]], y: 124, speed: 7, start: 410 },
];

/// 颜色主题：背景、亮的点、暗的点、日期的字、小字。
struct Theme {
    en: &'static str,
    zh: &'static str,
    colors: [[u8; 3]; 5],
}

const THEMES: [Theme; 5] = [
    Theme { en: "Mint", zh: "薄荷", colors: [[14, 22, 26], [132, 240, 188], [24, 42, 38], [200, 220, 212], [110, 150, 136]] },
    Theme { en: "Amber", zh: "琥珀", colors: [[24, 16, 8], [255, 176, 64], [56, 36, 14], [236, 212, 176], [168, 128, 80]] },
    Theme { en: "Ice", zh: "冰蓝", colors: [[12, 18, 36], [168, 216, 255], [28, 44, 76], [200, 212, 236], [112, 136, 176]] },
    Theme { en: "Rose", zh: "玫瑰", colors: [[28, 14, 22], [255, 140, 176], [62, 28, 44], [236, 208, 220], [170, 110, 136]] },
    Theme { en: "Paper", zh: "白纸", colors: [[232, 224, 206], [44, 44, 60], [214, 204, 182], [70, 62, 54], [130, 120, 104]] },
];

/// 缸里的灯：跟着几点换。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Light {
    Day,
    Evening,
    Night,
}

/// 几点用哪种灯：6 点到 16 点白天，17 到 19 点傍晚，别的是夜里。
pub fn light(hour: u32) -> Light {
    match hour {
        6..=16 => Light::Day,
        17..=19 => Light::Evening,
        _ => Light::Night,
    }
}

/// 上午还是下午（12 小时制用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    Am,
    Pm,
}

/// 大字的四位：时的两位、分的两位。None 是空着不画（12 小时制的 1 到 9 点）。
/// 12 小时制：0 点是 12 AM，12 点是 12 PM，13 点是 1 PM。
pub fn clock(hour: u32, minute: u32, h12: bool) -> ([Option<u8>; 4], Option<Half>) {
    let (h, half) = if h12 {
        let half = if hour < 12 { Half::Am } else { Half::Pm };
        let h = match hour % 12 {
            0 => 12,
            h => h,
        };
        (h, Some(half))
    } else {
        (hour, None)
    };
    let tens = if h12 && h < 10 { None } else { Some((h / 10) as u8) };
    (
        [tens, Some((h % 10) as u8), Some((minute / 10) as u8), Some((minute % 10) as u8)],
        half,
    )
}

const WEEK_EN: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const WEEK_ZH: [&str; 7] = ["一", "二", "三", "四", "五", "六", "日"];
const MONTH_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// 日期那一行：「Saturday, Nov 14」或「11月14日 星期六」。
pub fn date_line(t: &Time) -> String {
    let wd = t.weekday as usize % 7;
    lang().pick(
        format!("{}, {} {}", WEEK_EN[wd], MONTH_EN[(t.month as usize + 11) % 12], t.day),
        format!("{}月{}日 星期{}", t.month, t.day, WEEK_ZH[wd]),
    )
}

/// 点阵大字：一个字 5×8 个点。
const DIGITS: [[&str; 8]; 10] = [
    [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
    [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
    [".###.", "#...#", "....#", "..##.", "....#", "....#", "#...#", ".###."],
    ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#.", "...#."],
    ["#####", "#....", "####.", "....#", "....#", "....#", "#...#", ".###."],
    ["..##.", ".#...", "#....", "####.", "#...#", "#...#", "#...#", ".###."],
    ["#####", "....#", "...#.", "...#.", "..#..", "..#..", "..#..", "..#.."],
    [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", "#...#", ".###."],
    [".###.", "#...#", "#...#", "#...#", ".####", "....#", "...#.", ".##.."],
];
/// 一个点占几个像素（4 个亮，1 个空）。
const PITCH: i32 = 5;
const DOT: i32 = 4;
/// 大字的位置：四个字的左边，和上边。
const DIGIT_X: [i32; 4] = [56, 86, 126, 156];
const COLON_X: i32 = 116;
const CLOCK_Y: i32 = 20;

/// 鱼缸：里面的水从 TANK_TOP 到 SAND_TOP，沙子到 TANK_BOTTOM。
const TANK_L: i32 = 6;
const TANK_R: i32 = 234;
const TANK_TOP: i32 = 68;
const SAND_TOP: i32 = 144;
const TANK_BOTTOM: i32 = 156;

/// 一圈 16 格的正弦，-8 到 8。摆、晃都用它。
const SIN: [i32; 16] = [0, 3, 6, 7, 8, 7, 6, 3, 0, -3, -6, -7, -8, -7, -6, -3];
fn sin(i: i32) -> i32 {
    SIN[i.rem_euclid(16) as usize]
}

/// 4×4 的有序抖动表：两种颜色之间一格一格交错。
const BAYER: [[i32; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// 刚按了键时屏幕中间出的一行字。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toast {
    Help,
    Hours,
    Theme,
}

/// 开机后、按了键后，那行字留多少帧。
const TOAST_FRAMES: u32 = 150;
const HELP_FRAMES: u32 = 300;

pub struct Game {
    pub frame: Frame,
    pub keys: Keys,
    /// 开了多少帧。鱼、气泡、水草都只看它。
    pub t: u32,
    pub h12: bool,
    pub theme: usize,
    pub toast: Option<(Toast, u32)>,
    /// 上一帧的秒，和那一秒从第几帧开始：冒号前半秒亮，后半秒灭。
    last_sec: u32,
    sec_start: u32,
    fish: [Sprite; 2],
    zzz: Sprite,
}

impl Default for Game {
    fn default() -> Self {
        let (h12, theme) = match vibeplay::load(SAVE).as_deref() {
            Some(&[h, th]) => (h == 1, th as usize % THEMES.len()),
            _ => (false, 0),
        };
        let art = |rows: &[&str]| Sprite::art(rows, &[('b', 1), ('f', 2), ('s', 3), ('e', EYE)]).scaled(2);
        Game {
            frame: Frame::default(),
            keys: Keys::default(),
            t: 0,
            h12,
            theme,
            toast: Some((Toast::Help, HELP_FRAMES)),
            last_sec: u32::MAX,
            sec_start: 0,
            // 鱼朝右。两张换着画，尾巴一摆一摆。1、2、3 是身子、鳍、花纹，画的时候换成每条鱼的颜色。
            fish: [
                art(&["....ff.....", "ff.fbbsb...", "fbfbbbsbbb.", "fbbbbbsbbeb", "fbfbbbsbbb.", "ff..bbsb...", "....ff....."]),
                art(&["....ff.....", ".f.fbbsb...", ".ffbbbsbbb.", ".fbbbbsbbeb", ".ffbbbsbbb.", ".f..bbsb...", "....ff....."]),
            ],
            zzz: Sprite::art(&["zzz", "..z", ".z.", "z..", "zzz"], &[('z', ZZZ)]),
        }
    }
}

impl Game {
    fn save(&self) {
        vibeplay::save(SAVE, &[self.h12 as u8, self.theme as u8]);
    }

    /// 走一帧：读过的按键、现在的时间。不在这里面调 `vibeplay::now()`，测试才能给任何时间。
    pub fn step(&mut self, now: &Time) {
        self.t += 1;
        if now.second != self.last_sec {
            self.last_sec = now.second;
            self.sec_start = self.t;
        }
        if self.keys.pressed(key::A) {
            self.h12 = !self.h12;
            self.save();
            self.toast = Some((Toast::Hours, TOAST_FRAMES));
        }
        let n = THEMES.len();
        if self.keys.pressed(key::UP) {
            self.theme = (self.theme + 1) % n;
            self.save();
            self.toast = Some((Toast::Theme, TOAST_FRAMES));
        }
        if self.keys.pressed(key::DOWN) {
            self.theme = (self.theme + n - 1) % n;
            self.save();
            self.toast = Some((Toast::Theme, TOAST_FRAMES));
        }
        if let Some((what, left)) = self.toast {
            self.toast = (left > 1).then(|| (what, left - 1));
        }
    }

    /// 冒号这一帧亮不亮：每一秒的前半秒亮。
    pub fn colon_on(&self) -> bool {
        self.t.wrapping_sub(self.sec_start) < 30
    }

    pub fn draw(&mut self, now: &Time) {
        let lit = light(now.hour);
        self.paint_palette(lit);
        let colon = if self.colon_on() { DIGIT } else { DOT_OFF };
        let f = &mut self.frame;
        f.clear(BG);

        // 日期
        f.text_center(&FONT, 4, &date_line(now), TEXT);

        // 大字
        let (digits, half) = clock(now.hour, now.minute, self.h12);
        for (d, x) in digits.iter().zip(DIGIT_X) {
            draw_digit(f, x, CLOCK_Y, *d);
        }
        f.rect(COLON_X, CLOCK_Y + 2 * PITCH, DOT, DOT, colon);
        f.rect(COLON_X, CLOCK_Y + 5 * PITCH, DOT, DOT, colon);
        // 右边：上午下午在上，秒在下。
        let right = DIGIT_X[3] + 5 * PITCH + 3;
        if let Some(h) = half {
            let s = match h {
                Half::Am => lang().pick("AM", "上午"),
                Half::Pm => lang().pick("PM", "下午"),
            };
            f.text(&FONT, right, CLOCK_Y - 1, s, TEXT);
        }
        f.text(&FONT, right, CLOCK_Y + 8 * PITCH - 12, &format!("{:02}", now.second), DIM);

        self.draw_tank(lit);
        self.draw_toast();
    }

    /// 时钟这一半照主题，鱼缸这一半照灯。
    fn paint_palette(&mut self, lit: Light) {
        let p = &mut self.frame.palette;
        for (i, rgb) in THEMES[self.theme].colors.iter().enumerate() {
            p[i] = *rgb;
        }
        let water: [[u8; 3]; 4] = match lit {
            Light::Day => [[120, 200, 240], [72, 160, 224], [40, 120, 200], [28, 88, 168]],
            Light::Evening => [[240, 176, 120], [200, 128, 112], [136, 88, 112], [80, 56, 96]],
            Light::Night => [[20, 32, 64], [16, 26, 54], [12, 20, 44], [8, 14, 34]],
        };
        for (i, rgb) in WATER.iter().zip(water) {
            p[*i as usize] = rgb;
        }
        p[RAY as usize] = match lit {
            Light::Day => [176, 230, 252],
            Light::Evening => [255, 210, 150],
            Light::Night => [30, 44, 80],
        };
        p[LAMP as usize] = match lit {
            Light::Day => [250, 250, 220],
            Light::Evening => [255, 180, 96],
            Light::Night => [40, 44, 60],
        };
        for (i, rgb) in TANK {
            p[i as usize] = tint(rgb, lit);
        }
        // 字的框不跟着灯变。
        p[BOX as usize] = [16, 20, 28];
        p[BOX_TEXT as usize] = [240, 240, 232];
        p[ZZZ as usize] = [200, 210, 255];
        for (k, fish) in FISH.iter().enumerate() {
            for (j, rgb) in fish.colors.iter().enumerate() {
                p[(FISH0 + k as u8 * 3) as usize + j] = tint(*rgb, lit);
            }
        }
    }

    fn draw_tank(&mut self, lit: Light) {
        let t = self.t as i32;
        let f = &mut self.frame;
        // 玻璃和灯
        f.rect(TANK_L - 2, TANK_TOP - 4, TANK_R - TANK_L + 4, TANK_BOTTOM - TANK_TOP + 6, GLASS);
        f.rect(TANK_L - 2, TANK_TOP - 4, TANK_R - TANK_L + 4, 3, LAMP);
        // 水：上浅下深，抖动过渡。白天、傍晚有几道斜着的光慢慢走。
        let h = SAND_TOP - TANK_TOP;
        for y in TANK_TOP..SAND_TOP {
            let v = (y - TANK_TOP) * 3 * 16 / h;
            let (band, frac) = ((v / 16) as usize, v % 16);
            for x in TANK_L..TANK_R {
                let b = BAYER[(y & 3) as usize][(x & 3) as usize];
                let mut c = if frac > b { WATER[band + 1] } else { WATER[band] };
                if lit != Light::Night && y < TANK_TOP + 44 {
                    let r = (x + (y - TANK_TOP) / 2 - t / 6).rem_euclid(64);
                    if r < 5 && (x + y) % 2 == 0 {
                        c = RAY;
                    }
                }
                f.set(x, y, c);
            }
        }
        // 沙子和小石子
        for y in SAND_TOP..TANK_BOTTOM {
            for x in TANK_L..TANK_R {
                let b = BAYER[(y & 3) as usize][(x & 3) as usize];
                let c = if (y - SAND_TOP) * 2 > b + 6 { SAND_DARK } else { SAND };
                f.set(x, y, c);
            }
        }
        for i in 0..17 {
            let x = TANK_L + 5 + i * 13 + (i * 7) % 5;
            let y = SAND_TOP + 3 + (i * 5) % 8;
            f.rect(x, y, 2, 1, PEBBLE);
        }
        // 石头
        let rock = Sprite::art(
            &["....hhhh....", "..hhrrrrhh..", ".hrrrrrrrrr.", "hrrrrrr..rrr", "rrrrrr....rr", "rrrrrrr..rrr"],
            &[('h', ROCK_HI), ('r', ROCK)],
        )
        .scaled(2);
        f.blit(&rock, 112, SAND_TOP - rock.h + 2, false);
        // 左边的气泡石
        f.rect(46, SAND_TOP - 1, 6, 3, ROCK);
        f.rect(47, SAND_TOP - 2, 4, 1, ROCK_HI);

        // 水草：越往上摆得越多。
        for (x, height, phase) in [(22, 52, 0), (34, 36, 5), (70, 44, 9), (178, 58, 3), (196, 40, 11), (220, 48, 7)] {
            draw_plant(f, x, height, t / 10 + phase);
        }

        // 气泡：从石头和左边冒出来。夜里只有一串。
        let streams: &[(i32, i32)] = if lit == Light::Night { &[(126, 5)] } else { &[(126, 9), (49, 7)] };
        for &(bx, speed) in streams {
            for k in 0..4 {
                let from = if bx < 100 { SAND_TOP - 4 } else { SAND_TOP - 8 };
                let span = from - TANK_TOP;
                let rise = (t * speed / 16 + k * 19).rem_euclid(span);
                let y = from - rise;
                let x = bx + sin(y / 5) / 4;
                if rise < 20 {
                    f.rect(x, y, 2, 2, BUBBLE);
                } else {
                    f.frame_rect(x - 1, y - 1, 4, 4, BUBBLE);
                }
            }
        }

        // 鱼
        for (k, kind) in FISH.iter().enumerate() {
            let base = FISH0 + k as u8 * 3;
            let colors = [(1, base), (2, base + 1), (3, base + 2)];
            let ki = k as i32;
            if lit == Light::Night {
                // 睡觉：停在沙子上面，慢慢浮一点，闭着眼。
                let spr = self.fish[0].recolor(&colors).recolor(&[(EYE, base)]);
                let x = [28, 84, 150, 190][k];
                let y = SAND_TOP - 16 - ki % 2 * 8 + sin(t / 40 + ki * 4) / 4;
                f.blit(&spr, x, y, k % 2 == 1);
                if k % 2 == 0 {
                    let zy = (t / 3 + ki * 20).rem_euclid(40);
                    f.blit(&self.zzz, x + 18 + zy / 8, y - 4 - zy, false);
                }
                continue;
            }
            let speed = if lit == Light::Evening { kind.speed * 2 / 3 } else { kind.speed };
            let spr = self.fish[((t / 10 + ki) % 2) as usize].recolor(&colors);
            let span = TANK_R - TANK_L - spr.w;
            let pos = (t * speed / 16 + kind.start).rem_euclid(2 * span);
            let (x, left) = if pos < span { (TANK_L + pos, false) } else { (TANK_L + 2 * span - pos, true) };
            let y = kind.y + sin(t / 12 + ki * 5) / 4;
            f.blit(&spr, x, y, left);
        }
    }

    fn draw_toast(&mut self) {
        let Some((what, _)) = self.toast else { return };
        let s = match what {
            Toast::Help => lang().pick("A: 12/24h   Up/Down: color", "A：12/24 小时  上下：颜色").to_string(),
            Toast::Hours => match self.h12 {
                true => lang().pick("12-hour clock", "12 小时制").to_string(),
                false => lang().pick("24-hour clock", "24 小时制").to_string(),
            },
            Toast::Theme => {
                let th = &THEMES[self.theme];
                lang().pick(format!("Color: {}", th.en), format!("颜色：{}", th.zh))
            }
        };
        let w = FONT.width(&s) + 12;
        let x = (W as i32 - w) / 2;
        let f = &mut self.frame;
        f.rect(x, TANK_TOP + 6, w, 18, BOX);
        f.frame_rect(x, TANK_TOP + 6, w, 18, BOX_TEXT);
        f.text(&FONT, x + 6, TANK_TOP + 9, &s, BOX_TEXT);
    }
}

/// 画一个点阵大字。None 只画暗的点。
fn draw_digit(f: &mut Frame, x: i32, y: i32, d: Option<u8>) {
    let blank = [""; 8];
    let rows = d.map_or(&blank, |d| &DIGITS[d as usize]);
    for (row, bits) in rows.iter().enumerate() {
        for col in 0..5 {
            let on = bits.as_bytes().get(col) == Some(&b'#');
            let c = if on { DIGIT } else { DOT_OFF };
            f.rect(x + col as i32 * PITCH, y + row as i32 * PITCH, DOT, DOT, c);
        }
    }
}

/// 一棵水草：两个像素宽的茎，隔几个像素左右各一片叶子。`phase` 走一圈是摆一下。
fn draw_plant(f: &mut Frame, x: i32, height: i32, phase: i32) {
    for h in 0..height {
        let dx = sin(phase - h / 8) * h / height / 2;
        let y = SAND_TOP + 2 - h;
        f.rect(x + dx, y, 2, 1, LEAF_DARK);
        if h % 7 == 3 && h > 4 {
            let side = if (h / 7) % 2 == 0 { -1 } else { 1 };
            for i in 1..4 {
                let lx = if side < 0 { x + dx - i } else { x + dx + 1 + i };
                f.set(lx, y - i / 2, LEAF);
                f.set(lx, y - i / 2 + 1, LEAF_DARK);
            }
        } else {
            f.set(x + dx, y, LEAF);
        }
    }
}

/// 鱼缸里的颜色跟着灯调：傍晚往暖色靠，夜里暗下来偏蓝。
fn tint(rgb: [u8; 3], lit: Light) -> [u8; 3] {
    let [r, g, b] = rgb.map(|c| c as u32);
    match lit {
        Light::Day => rgb,
        Light::Evening => [(r * 3 + 255) / 4, (g * 3 + 140) / 4 * 9 / 10, (b * 3 + 60) / 4 * 8 / 10].map(|c| c.min(255) as u8),
        Light::Night => [r * 30 / 100, g * 32 / 100, (b * 40 / 100 + 18).min(255)].map(|c| c as u8),
    }
}

static GAME: Mutex<Option<Game>> = Mutex::new(None);

vibeplay::main!(|app| {
    app.every(|| {
        let mut g = GAME.lock().unwrap();
        let g = g.get_or_insert_with(Game::default);
        let now = vibeplay::now();
        g.keys.read();
        g.step(&now);
        g.draw(&now);
        g.frame.show();
    });
});
