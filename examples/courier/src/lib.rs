//! 游戏《疾风邮差》：横版跳跃。角色、关卡、画面、音乐都是自己做的。
//!
//! 小邮差送急件：一路往右跑，跳过坑，踩发条兵，顶箱子拿邮票，走进邮局的门过关。六关，最后一关和邮件大盗打一场。
//! 十字键左右走，按住 B 跑，A 跳（按得久跳得高），大个子按下蹲。拿了纸飞机以后 B 扔纸飞机。START 开始、暂停。
//!
//! 手感照那个年代最好的横版跳跃做：起步慢、跑起来快，回头要刹车；跳的高度看按多久、跑多快；踩敌人弹起来，连踩分越来越高。
//! 包裹箱里有：邮票、加急包裹（变大，能顶碎旧木箱，挨一下变回小个子）、纸飞机（能扔）、金邮票（一阵子谁也碰不了你）。
//! 还有看不见的箱子，顶出来一封粉色的信，多一条命。邮票攒到一百张也多一条命。
//!
//! 位置和速度用 1/256 像素的整数（`FX`），同样的按键每次跑出来一样。

mod art;
mod levels;
mod tune;

use art::*;
use vibeplay::FONT;
use levels::LEVELS;
use vibeplay::full::{key, lang, tone_wav, Channel, Frame, Keys, Music, Sprite, H, W};
use std::sync::Mutex;

/// 一个像素是 256 份。
const FX: i32 = 256;
/// 一格 16 像素，地图 10 行。
const T: i32 = 16;
const ROWS: usize = 10;

// 主角的手感（1/256 像素每帧）。
const WALK_MAX: i32 = 384;
const RUN_MAX: i32 = 640;
const ACC_WALK: i32 = 10;
const ACC_RUN: i32 = 14;
const DECEL: i32 = 13;
const SKID: i32 = 26;
const JUMP_V: i32 = 1024;
const JUMP_RUN_V: i32 = 1152;
const G_HOLD: i32 = 32;
const G_FALL: i32 = 128;
const MAX_FALL: i32 = 1152;
/// 一关多少时间（一格是 24 帧）。
const TIME: u32 = 300;
const TIME_FRAMES: u32 = 24;
/// 无敌多久、挨打以后闪多久。
const STAR_TIME: u32 = 600;
const HURT_TIME: u32 = 120;
/// 存档：开到第几关、最高分。
const SAVE: &str = "progress";

// ---------- 地图块 ----------
const EMPTY: u8 = 0;
const GROUND: u8 = 1;
const WALL: u8 = 2;
const STONEB: u8 = 3;
const CRATE: u8 = 4;
const MAIL: u8 = 5;
const MAIL_POWER: u8 = 6;
const MAIL_GOLD: u8 = 7;
const MULTI: u8 = 8;
const HIDDEN: u8 = 9;
const USED: u8 = 10;
const CH_TL: u8 = 11;
const CH_TR: u8 = 12;
const CH_BL: u8 = 13;
const CH_BR: u8 = 14;
const LIQUID: u8 = 15;
const VENT: u8 = 16;
const STAMPT: u8 = 17;

fn solid(t: u8) -> bool {
    matches!(t, GROUND | WALL | STONEB | CRATE | MAIL | MAIL_POWER | MAIL_GOLD | MULTI | USED | CH_TL | CH_TR | CH_BL | CH_BR | VENT)
}

/// 连踩、连撞的分：踩到第十个以后每个多一条命。
const COMBO: [u32; 10] = [100, 200, 400, 500, 800, 1000, 2000, 4000, 5000, 8000];
const KICK_COMBO: [u32; 7] = [500, 800, 1000, 2000, 4000, 5000, 8000];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Walker,
    CanBug,
    /// 罐头虫缩进了罐头。`vx` 不是 0 就是被踢着滚。
    Can,
    Crow,
    Spiky,
    Soot,
    Boss,
    Parcel,
    PlaneItem,
    Gold,
    Love,
    /// 主角扔的纸飞机。
    Shot,
    /// 邮件大盗扔的信。
    Letter,
}

impl Kind {
    fn enemy(self) -> bool {
        matches!(self, Kind::Walker | Kind::CanBug | Kind::Can | Kind::Crow | Kind::Spiky | Kind::Soot | Kind::Boss)
    }
    fn item(self) -> bool {
        matches!(self, Kind::Parcel | Kind::PlaneItem | Kind::Gold | Kind::Love)
    }
    fn points(self) -> u32 {
        match self {
            Kind::Crow | Kind::Spiky | Kind::Soot => 200,
            Kind::Boss => 5000,
            _ => 100,
        }
    }
}

/// 会动的一样东西：敌人、道具、子弹。x、y 是左上角（1/256 像素），w、h 是像素。
#[derive(Debug, Clone)]
pub struct Mob {
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub w: i32,
    pub h: i32,
    pub ground: bool,
    /// 被打倒了：翻过来掉下去，还剩几帧。
    pub dying: u32,
    /// 被踩扁了：扁着还剩几帧。
    pub flat: u32,
    pub hp: i32,
    /// 通用的计时：罐头里醒来、煤球冒头、大盗扔信。
    pub timer: u32,
    /// 从箱子里冒出来还剩几帧（这时不动、不能拿）。
    pub rise: u32,
    /// 出生的地方（像素）：乌鸦绕着它飞，煤球在这个烟囱里。
    pub home: (i32, i32),
    /// 被踢的罐头连着撞倒了几个。
    pub combo: usize,
    /// 刚被踢出去：几帧内不伤主角。
    pub grace: u32,
    /// 大盗被踩了以后闪的几帧。
    pub hurt: u32,
}

impl Mob {
    fn new(kind: Kind, px: i32, py: i32) -> Mob {
        let (w, h) = match kind {
            Kind::Boss => (24, 28),
            Kind::Shot => (8, 6),
            Kind::Letter => (8, 6),
            Kind::Crow => (14, 10),
            _ => (14, 15),
        };
        let vx = match kind {
            Kind::Walker | Kind::CanBug | Kind::Spiky => -128,
            Kind::Crow => -192,
            _ => 0,
        };
        Mob { kind, x: px * FX, y: py * FX, vx, vy: 0, w, h, ground: false, dying: 0, flat: 0, hp: if kind == Kind::Boss { 6 } else { 1 }, timer: 0, rise: 0, home: (px, py), combo: 0, grace: 0, hurt: 0 }
    }
    fn px(&self) -> (i32, i32) {
        (self.x.div_euclid(FX), self.y.div_euclid(FX))
    }
    fn rect(&self) -> (i32, i32, i32, i32) {
        let (x, y) = self.px();
        (x, y, self.w, self.h)
    }
    fn live(&self) -> bool {
        self.dying == 0 && self.flat == 0
    }
}

/// 会动的踏板：横着或竖着来回走。
#[derive(Debug, Clone)]
pub struct Lift {
    pub x: i32,
    pub y: i32,
    pub vertical: bool,
    origin: (i32, i32),
    t: i32,
    /// 这一帧挪了多少像素：站在上面的跟着挪。
    pub dx: i32,
    pub dy: i32,
}

const LIFT_W: i32 = 48;

impl Lift {
    fn step(&mut self) {
        self.t += 1;
        // 来回走：一个三角波。横着 ±96 像素、一帧一像素；竖着 ±40 像素、两帧一像素。
        let (range, slow) = if self.vertical { (40, 2) } else { (96, 1) };
        let period = range * 4 * slow;
        let p = self.t.rem_euclid(period) / slow;
        let off = if p < range { p } else if p < range * 3 { range * 2 - p } else { p - range * 4 };
        let (nx, ny) = if self.vertical { (self.origin.0, self.origin.1 + off) } else { (self.origin.0 + off, self.origin.1) };
        self.dx = nx - self.x;
        self.dy = ny - self.y;
        self.x = nx;
        self.y = ny;
    }
}

/// 主角。x、y 是碰撞框的左上角（1/256 像素）。
#[derive(Debug, Clone)]
pub struct Hero {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub big: bool,
    pub plane: bool,
    pub ground: bool,
    pub dir: i32,
    pub crouch: bool,
    /// 挨打以后闪着、碰不着：还剩几帧。
    pub hurt: u32,
    /// 无敌还剩几帧。
    pub star: u32,
    /// 变大、变小的那一下：还剩几帧，这时全场停住。
    pub morph: u32,
    morph_to_big: bool,
    pub dead: Option<u32>,
    jump_held: bool,
    skid: bool,
    anim: u32,
    walk: usize,
    /// 这一跳里连踩了几个。
    pub combo: usize,
    /// 无敌时撞倒了几个。
    star_combo: usize,
    /// 站在第几个踏板上。
    on_lift: Option<usize>,
    /// 刚离开地面几帧内还能起跳（跑过坑边时手慢一点也行）。
    coyote: u32,
}

impl Hero {
    fn new(px: i32, py: i32) -> Hero {
        Hero { x: px * FX, y: py * FX, vx: 0, vy: 0, big: false, plane: false, ground: true, dir: 1, crouch: false, hurt: 0, star: 0, morph: 0, morph_to_big: false, dead: None, jump_held: false, skid: false, anim: 0, walk: 0, combo: 0, star_combo: 0, on_lift: None, coyote: 0 }
    }
    pub fn w(&self) -> i32 {
        12
    }
    pub fn h(&self) -> i32 {
        if self.big && !self.crouch {
            30
        } else {
            15
        }
    }
    pub fn px(&self) -> (i32, i32) {
        (self.x.div_euclid(FX), self.y.div_euclid(FX))
    }
    fn rect(&self) -> (i32, i32, i32, i32) {
        let (x, y) = self.px();
        (x, y, self.w(), self.h())
    }
}

#[derive(Debug, Clone)]
struct Popup {
    x: i32,
    y: i32,
    /// 空的是「多一条命」：出字时才按系统语言选。
    text: String,
    t: u32,
}

/// 顶碎的木箱飞出去的碎块。
#[derive(Debug, Clone)]
struct Debris {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
}

/// 从箱子里跳出来又落回去的邮票（只是画）。
#[derive(Debug, Clone)]
struct PopStamp {
    x: i32,
    y: i32,
    t: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Title,
    /// 关卡开头：关名、还有几条命。
    Intro,
    Play,
    /// 走进邮局的门，剩下的时间换成分。
    Clear,
    Over,
    /// 六关都送到了。
    Ending,
}

struct Art {
    small: Vec<Sprite>,
    big: Vec<Sprite>,
    die: Sprite,
    walker: [Sprite; 2],
    walker_flat: Sprite,
    canbug: [Sprite; 2],
    can: [Sprite; 2],
    crow: [Sprite; 2],
    spiky: [Sprite; 2],
    soot: [Sprite; 2],
    boss: [Sprite; 2],
    parcel: Sprite,
    plane_item: Sprite,
    plane_shot: Sprite,
    gold: Sprite,
    love: Sprite,
    stamp: [Sprite; 3],
    letter: Sprite,
    top: Sprite,
    fill: Sprite,
    crate_: Sprite,
    mail: [Sprite; 3],
    used: Sprite,
    stone: Sprite,
    chimney: [Sprite; 4],
    brick: Sprite,
    lift: Sprite,
    vent: Sprite,
    stamp_icon: Sprite,
}

impl Art {
    fn new() -> Art {
        let two = |a: &[[&str; 16]; 2], k: &[(char, u8)]| [Sprite::art(&a[0], k), Sprite::art(&a[1], k)];
        let stamp = |i: usize| Sprite::art(&STAMP[i], &STAMP_KEYS);
        let small = hero_small();
        Art {
            small,
            big: hero_big(),
            die: Sprite::art(&HERO_DIE, &HERO_KEYS),
            walker: two(&WALKER, &WALKER_KEYS),
            walker_flat: Sprite::art(&WALKER_FLAT, &WALKER_KEYS),
            canbug: two(&CANBUG, &CAN_KEYS),
            can: two(&CAN, &CAN_KEYS),
            crow: two(&CROW, &CROW_KEYS),
            spiky: two(&SPIKY, &SPIKY_KEYS),
            soot: two(&SOOT, &SOOT_KEYS),
            boss: [Sprite::art(&BOSS[0], &BOSS_KEYS), Sprite::art(&BOSS[1], &BOSS_KEYS)],
            parcel: Sprite::art(&PARCEL, &PARCEL_KEYS),
            plane_item: Sprite::art(&PLANE_ITEM, &PLANE_KEYS),
            plane_shot: Sprite::art(&PLANE_SHOT, &PLANE_KEYS),
            gold: Sprite::art(&GOLD_STAMP, &GOLD_KEYS),
            love: Sprite::art(&LOVE, &LOVE_KEYS),
            stamp: [stamp(0), stamp(1), stamp(2)],
            letter: Sprite::art(&LETTER, &[('k', INK), ('w', WHITE)]),
            top: Sprite::art(&T_TOP, &TILE_KEYS),
            fill: Sprite::art(&T_FILL, &TILE_KEYS),
            crate_: Sprite::art(&T_CRATE, &TILE_KEYS),
            mail: [Sprite::art(&T_MAIL[0], &TILE_KEYS), Sprite::art(&T_MAIL[1], &TILE_KEYS), Sprite::art(&T_MAIL[2], &TILE_KEYS)],
            used: Sprite::art(&T_USED, &TILE_KEYS),
            stone: Sprite::art(&T_STONE, &TILE_KEYS),
            chimney: [0, 1, 2, 3].map(|i| Sprite::art(&T_CHIMNEY[i], &TILE_KEYS)),
            brick: Sprite::art(&T_BRICK, &TILE_KEYS),
            lift: Sprite::art(&T_LIFT, &TILE_KEYS),
            vent: Sprite::art(&T_VENT, &TILE_KEYS),
            stamp_icon: stamp(0),
        }
    }
}

fn overlap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

/// 一段自己生成的噪声，慢慢变小：木箱碎了、大盗倒下。
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
        low = (low + noise) / 2;
        let fade = (n - i) as i32 * 1000 / n as i32;
        let v = (low * fade / 1000).clamp(-32768, 32767) as i16;
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

pub struct Game {
    pub keys: Keys,
    pub frame: Frame,
    music: Music,
    art: Art,
    pub mode: Mode,
    timer: u32,
    tick: u32,
    /// 第几关（从 0 数）。
    pub level: usize,
    pub unlocked: usize,
    pub hi: u32,
    pub score: u32,
    pub stamps: u32,
    pub lives: u32,
    pub time: u32,
    time_frames: u32,
    pub map: Vec<Vec<u8>>,
    pub width: usize,
    pub hero: Hero,
    pub mobs: Vec<Mob>,
    pub lifts: Vec<Lift>,
    /// 还没出场的敌人：种类、格子。画面走到了才放出来。
    spawns: Vec<(Kind, i32, i32)>,
    debris: Vec<Debris>,
    popups: Vec<Popup>,
    pops: Vec<PopStamp>,
    /// 被顶起来的格：格子、还剩几帧。
    bumps: Vec<(usize, usize, u32)>,
    /// 连着顶邮票的木箱：格子、还能顶几张、到第几帧为止。
    multi: Vec<(usize, usize, u32, u32)>,
    /// 画面左边（像素）。只往右走。
    pub cam: i32,
    start: (i32, i32),
    checkpoint: Option<(i32, i32)>,
    pub reached_check: bool,
    pub door: (i32, i32),
    pub boss_dead: bool,
    paused: bool,
    hurry: bool,
    /// 排着的短音：第几帧响、声道、频率、毫秒、音量。
    jingle: Vec<(u32, Channel, u32, u32, u32)>,
    boom: Vec<u8>,
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
            level: 0,
            unlocked: 0,
            hi: 10000,
            score: 0,
            stamps: 0,
            lives: 3,
            time: TIME,
            time_frames: 0,
            map: vec![vec![EMPTY; 1]; ROWS],
            width: 1,
            hero: Hero::new(32, 112),
            mobs: Vec::new(),
            lifts: Vec::new(),
            spawns: Vec::new(),
            debris: Vec::new(),
            popups: Vec::new(),
            pops: Vec::new(),
            bumps: Vec::new(),
            multi: Vec::new(),
            cam: 0,
            start: (32, 112),
            checkpoint: None,
            reached_check: false,
            door: (0, 0),
            boss_dead: false,
            paused: false,
            hurry: false,
            jingle: Vec::new(),
            boom: boom_wav(4, 0x2468_1357),
        };
        if let Some(b) = vibeplay::load(SAVE) {
            let s = String::from_utf8_lossy(&b);
            let mut it = s.split(',');
            if let Some(u) = it.next().and_then(|v| v.trim().parse::<usize>().ok()) {
                g.unlocked = u.min(LEVELS.len() - 1);
            }
            if let Some(h) = it.next().and_then(|v| v.trim().parse::<u32>().ok()) {
                g.hi = g.hi.max(h);
            }
        }
        g.level = g.unlocked;
        g.load(g.level);
        g
    }
}

impl Game {
    fn save(&self) {
        let _ = vibeplay::save(SAVE, format!("{},{}", self.unlocked, self.hi).as_bytes());
    }

    // ---------- 声音 ----------

    fn sfx(&mut self, ch: Channel, hz: u32, ms: u32, vol: u32) {
        self.music.sfx(ch, hz, ms, vol);
    }

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

    /// 放这一关的曲子：无敌时换成快的，大盗在场换成打斗的，时间不多了放快一点。
    fn play_music(&mut self) {
        let theme = LEVELS[self.level].theme;
        let boss = !self.boss_dead && self.mobs.iter().any(|m| m.kind == Kind::Boss && m.live());
        let song = if self.hero.star > 0 {
            tune::star()
        } else if boss {
            tune::boss(self.hurry)
        } else {
            tune::level(theme, self.hurry)
        };
        self.music.play(song);
    }

    // ---------- 地图 ----------

    /// 读第 `i` 关：地图块、敌人、踏板、起点、门。
    pub fn load(&mut self, i: usize) {
        let rows = levels::rows(&LEVELS[i]);
        self.width = rows.iter().map(|r| r.len()).max().unwrap_or(1);
        self.map = vec![vec![EMPTY; self.width]; ROWS];
        self.spawns.clear();
        self.lifts.clear();
        self.multi.clear();
        self.checkpoint = None;
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let (px, py) = (x as i32 * T, y as i32 * T);
                let t = match ch {
                    '#' => GROUND,
                    'R' => WALL,
                    'S' => STONEB,
                    'B' => CRATE,
                    'M' => MAIL,
                    'P' => MAIL_POWER,
                    'I' => MAIL_GOLD,
                    'T' => MULTI,
                    'L' => HIDDEN,
                    '<' => CH_TL,
                    '>' => CH_TR,
                    '(' => CH_BL,
                    ')' => CH_BR,
                    '~' => LIQUID,
                    '^' => VENT,
                    'o' => STAMPT,
                    _ => EMPTY,
                };
                self.map[y][x] = t;
                match ch {
                    '@' => self.start = (px + 2, py + 1),
                    'D' => self.door = (px, py),
                    'K' => self.checkpoint = Some((px + 2, py + 1)),
                    '=' | '|' => self.lifts.push(Lift { x: px, y: py, vertical: ch == '|', origin: (px, py), t: 0, dx: 0, dy: 0 }),
                    'w' => self.spawns.push((Kind::Walker, px + 1, py + 1)),
                    'k' => self.spawns.push((Kind::CanBug, px + 1, py + 1)),
                    'r' => self.spawns.push((Kind::Crow, px + 1, py + 3)),
                    's' => self.spawns.push((Kind::Spiky, px + 1, py + 1)),
                    'c' => self.spawns.push((Kind::Soot, px + 9, py + 16)),
                    'X' => self.spawns.push((Kind::Boss, px, py - 12)),
                    _ => {}
                }
            }
        }
    }

    pub fn tile(&self, tx: i32, ty: i32) -> u8 {
        if tx < 0 || tx >= self.width as i32 {
            return WALL;
        }
        if ty < 0 || ty >= ROWS as i32 {
            return EMPTY;
        }
        self.map[ty as usize][tx as usize]
    }

    /// 这块地方有实心的格。
    pub fn hits(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        for ty in y.div_euclid(T)..=(y + h - 1).div_euclid(T) {
            for tx in x.div_euclid(T)..=(x + w - 1).div_euclid(T) {
                if solid(self.tile(tx, ty)) {
                    return true;
                }
            }
        }
        false
    }

    // ---------- 开局 ----------

    fn new_game(&mut self) {
        self.score = 0;
        self.stamps = 0;
        self.lives = 3;
        self.reached_check = false;
        self.hero = Hero::new(0, 0);
        self.begin_intro();
    }

    fn begin_intro(&mut self) {
        self.mode = Mode::Intro;
        self.timer = 0;
        self.music.stop();
    }

    /// 从起点（过了中途点就从中途点）开始这一关。大小、纸飞机留着。
    pub fn start_level(&mut self) {
        let (big, plane) = (self.hero.big, self.hero.plane);
        self.load(self.level);
        let at = if self.reached_check { self.checkpoint.unwrap_or(self.start) } else { self.start };
        self.hero = Hero::new(at.0, at.1 + if big { -15 } else { 0 });
        self.hero.big = big;
        self.hero.plane = plane;
        self.mobs.clear();
        self.debris.clear();
        self.popups.clear();
        self.pops.clear();
        self.bumps.clear();
        self.cam = (at.0 - 104).clamp(0, self.max_cam());
        self.time = TIME;
        self.time_frames = 0;
        self.boss_dead = false;
        self.paused = false;
        self.hurry = false;
        self.mode = Mode::Play;
        self.timer = 0;
        self.play_music();
    }

    fn max_cam(&self) -> i32 {
        (self.width as i32 * T - W as i32).max(0)
    }

    // ---------- 一帧 ----------

    pub fn step(&mut self) {
        self.tick += 1;
        self.timer += 1;
        self.music.tick();
        self.run_jingle();
        match self.mode {
            Mode::Title => {
                if self.keys.pressed(key::RIGHT) && self.level < self.unlocked {
                    self.level += 1;
                    self.sfx(Channel::Square2, 660, 30, 30);
                }
                if self.keys.pressed(key::LEFT) && self.level > 0 {
                    self.level -= 1;
                    self.sfx(Channel::Square2, 520, 30, 30);
                }
                if self.keys.pressed(key::START) || self.keys.pressed(key::A) {
                    self.sfx(Channel::Square2, 880, 60, 40);
                    self.new_game();
                }
            }
            Mode::Intro => {
                if self.timer > 120 {
                    self.start_level();
                }
            }
            Mode::Play => self.play(),
            Mode::Clear => self.clear_step(),
            Mode::Over | Mode::Ending => {
                if self.timer > 90 && (self.keys.pressed(key::START) || self.keys.pressed(key::A)) || self.timer > 600 {
                    self.mode = Mode::Title;
                    self.timer = 0;
                    self.level = self.unlocked;
                    self.load(self.level);
                }
            }
        }
        self.draw();
    }

    fn play(&mut self) {
        if self.hero.dead.is_none() && self.keys.pressed(key::START) {
            self.paused = !self.paused;
            self.arpeggio(Channel::Square2, &[659, 523, 659], 4, 35);
        }
        if self.paused {
            return;
        }
        if let Some(t) = self.hero.dead {
            self.dying(t);
            return;
        }
        if self.hero.morph > 0 {
            self.hero.morph -= 1;
            return;
        }
        // 时间：一格 24 帧。剩一百时提醒一下，曲子快一点。
        self.time_frames += 1;
        if self.time_frames >= TIME_FRAMES {
            self.time_frames = 0;
            self.time = self.time.saturating_sub(1);
            if self.time == 100 && !self.hurry {
                self.hurry = true;
                self.arpeggio(Channel::Square1, &[1047, 1319, 1568, 1047, 1319, 1568], 5, 40);
                self.play_music();
            }
            if self.time == 0 {
                self.kill_hero();
                return;
            }
        }
        for l in self.lifts.iter_mut() {
            l.step();
        }
        self.drive_hero();
        if self.hero.dead.is_some() {
            return;
        }
        self.spawn_mobs();
        self.move_mobs();
        self.touch_mobs();
        if self.steam_on() && self.steam_rects().iter().any(|r| overlap(self.hero.rect(), *r)) {
            self.hurt_hero();
        }
        self.bumps.retain_mut(|b| {
            b.2 = b.2.saturating_sub(1);
            b.2 > 0
        });
        self.debris.retain_mut(|d| {
            d.x += d.vx;
            d.vy += G_FALL / 2;
            d.y += d.vy;
            d.y < (H as i32 + 16) * FX
        });
        self.pops.retain_mut(|p| {
            p.t += 1;
            p.t < 30
        });
        self.popups.retain_mut(|p| {
            p.t += 1;
            p.t < 45
        });
        // 无敌快完了，换回原来的曲子。
        if self.hero.star > 0 {
            self.hero.star -= 1;
            if self.hero.star == 0 {
                self.play_music();
            }
        }
        self.hero.hurt = self.hero.hurt.saturating_sub(1);
        // 走到门口：进门。钟楼的门要大盗倒下才开。
        let (hx, hy, hw, hh) = self.hero.rect();
        let locked = self.spawns.iter().any(|s| s.0 == Kind::Boss) || self.mobs.iter().any(|m| m.kind == Kind::Boss);
        if !locked && self.hero.ground && overlap((hx, hy, hw, hh), (self.door.0 + 4, self.door.1 - 8, 8, 24)) {
            self.mode = Mode::Clear;
            self.timer = 0;
            self.music.stop();
            self.music.play(tune::clear());
        }
        if let Some(c) = self.checkpoint {
            if hx >= c.0 {
                self.reached_check = true;
            }
        }
    }

    // ---------- 主角 ----------

    fn drive_hero(&mut self) {
        let k = &self.keys;
        let (left, right, down) = (k.held(key::LEFT), k.held(key::RIGHT), k.held(key::DOWN));
        let run = k.held(key::B);
        let jump_press = k.pressed(key::A);
        let jump_held = k.held(key::A);
        let fire = k.pressed(key::B);
        let h = &mut self.hero;
        let input = if left == right { 0 } else if left { -1 } else { 1 };
        // 蹲：大个子在地上按下。蹲着不走，会滑一会儿。站起来时头顶有东西就接着蹲。
        let was_crouch = h.crouch;
        h.crouch = h.big && h.ground && down;
        if was_crouch && !h.crouch {
            let (x, y) = h.px();
            if self.hits(x, y - 15, 12, 30) {
                self.hero.crouch = true;
            } else {
                self.hero.y -= 15 * FX;
            }
        } else if !was_crouch && h.crouch {
            h.y += 15 * FX;
        }
        let h = &mut self.hero;
        let max = if run { RUN_MAX } else { WALK_MAX };
        let acc = if run { ACC_RUN } else { ACC_WALK };
        h.skid = false;
        if h.crouch {
            h.vx -= h.vx.signum() * DECEL.min(h.vx.abs());
        } else if input != 0 {
            if h.ground && h.vx != 0 && h.vx.signum() != input {
                // 回头先刹车。
                h.vx += input * SKID;
                h.skid = true;
                if h.vx.signum() == input {
                    h.vx = 0;
                }
            } else if h.vx * input < max {
                h.vx += input * acc;
                if h.vx * input > max {
                    h.vx = input * max;
                }
            } else if h.ground {
                // 松了 B：从跑的速度慢慢降到走的。
                h.vx -= input * DECEL;
            }
            if h.ground {
                h.dir = input;
            }
        } else if h.ground {
            h.vx -= h.vx.signum() * DECEL.min(h.vx.abs());
        }
        // 起跳：跑得快跳得高。
        if h.ground {
            h.coyote = 4;
        } else {
            h.coyote = h.coyote.saturating_sub(1);
        }
        let mut jumped = false;
        if jump_press && h.coyote > 0 && h.vy >= 0 {
            h.vy = -if h.vx.abs() > 512 { JUMP_RUN_V } else { JUMP_V };
            h.ground = false;
            h.coyote = 0;
            h.on_lift = None;
            jumped = true;
        }
        h.jump_held = jump_held;
        let g = if h.vy < 0 && jump_held { G_HOLD } else { G_FALL };
        h.vy = (h.vy + g).min(MAX_FALL);
        let big = h.big;
        if jumped {
            let notes: &[u32] = if big { &[262, 392, 523] } else { &[392, 523, 784] };
            self.arpeggio(Channel::Square2, notes, 2, 30);
        }
        if fire && self.hero.plane && self.mobs.iter().filter(|m| m.kind == Kind::Shot).count() < 2 {
            let (x, y) = self.hero.px();
            let mut s = Mob::new(Kind::Shot, x + if self.hero.dir > 0 { 10 } else { -6 }, y + 6);
            s.vx = self.hero.dir * 768;
            s.vy = 64;
            s.timer = 90;
            self.mobs.push(s);
            self.arpeggio(Channel::Square2, &[1568, 1175], 2, 30);
        }
        self.move_hero();
        // 动画：走得越快脚换得越快。
        let h = &mut self.hero;
        if h.ground && h.vx != 0 {
            h.anim += (h.vx.abs() / 64 + 2) as u32;
            if h.anim >= 24 {
                h.anim = 0;
                h.walk = h.walk % 3 + 1;
            }
        } else if h.ground {
            h.walk = 0;
            h.anim = 0;
        }
    }

    fn move_hero(&mut self) {
        // 站在踏板上：先跟着踏板挪。
        if let Some(i) = self.hero.on_lift {
            let (dx, dy) = (self.lifts[i].dx, self.lifts[i].dy);
            let (x, y) = self.hero.px();
            let (w, h) = (self.hero.w(), self.hero.h());
            if !self.hits(x + dx, y, w, h) {
                self.hero.x += dx * FX;
            }
            if dy < 0 && self.hits(x, y + dy, w, h) {
                // 被踏板顶到天花板：放它走。
                self.hero.on_lift = None;
            } else {
                self.hero.y += dy * FX;
            }
        }
        // 横着：一个像素一个像素走，撞墙停。不能走出画面左边。
        let (mut x, y) = self.hero.px();
        let (w, h) = (self.hero.w(), self.hero.h());
        let tx = (self.hero.x + self.hero.vx).div_euclid(FX);
        let mut blocked = false;
        while x != tx {
            let s = (tx - x).signum();
            if self.hits(x + s, y, w, h) || x + s < self.cam {
                blocked = true;
                break;
            }
            x += s;
        }
        if blocked {
            self.hero.x = x * FX;
            self.hero.vx = 0;
        } else {
            self.hero.x += self.hero.vx;
        }
        // 竖着。
        let (x, mut y) = self.hero.px();
        let ty = (self.hero.y + self.hero.vy).div_euclid(FX);
        let mut stop = false;
        let mut head: Option<(i32, i32)> = None;
        while y != ty {
            let s = (ty - y).signum();
            if s < 0 {
                if let Some(t) = self.head_hit(x, y - 1, w) {
                    head = Some(t);
                    stop = true;
                    break;
                }
                if self.hits(x, y - 1, w, h) {
                    // 头擦到格子的边：往旁边让一点（四个像素以内）就过去。
                    if let Some(nx) = (1..=4).flat_map(|d| [x + d, x - d]).find(|&nx| !self.hits(nx, y - 1, w, h) && !self.hits(nx, y, w, h)) {
                        self.hero.x = nx * FX;
                        return self.move_hero_rest(nx, y, s);
                    }
                    stop = true;
                    break;
                }
            } else if self.feet_blocked(x, y, w, h) {
                stop = true;
                break;
            }
            y += s;
        }
        self.finish_vertical(x, y, stop, head);
    }

    /// 让过格子边以后接着往上走完这一帧。
    fn move_hero_rest(&mut self, x: i32, y: i32, _s: i32) {
        let ty = (self.hero.y + self.hero.vy).div_euclid(FX);
        let (w, h) = (self.hero.w(), self.hero.h());
        let mut y = y;
        let mut stop = false;
        let mut head = None;
        while y > ty {
            if let Some(t) = self.head_hit(x, y - 1, w) {
                head = Some(t);
                stop = true;
                break;
            }
            if self.hits(x, y - 1, w, h) {
                stop = true;
                break;
            }
            y -= 1;
        }
        self.finish_vertical(x, y, stop, head);
    }

    fn finish_vertical(&mut self, x: i32, y: i32, stop: bool, head: Option<(i32, i32)>) {
        let (w, h) = (self.hero.w(), self.hero.h());
        if stop {
            self.hero.y = y * FX;
            if self.hero.vy < 0 {
                self.hero.vy = 0;
                // 顶到了：顶哪一格看主角中间对着哪一格。
                let t = head.unwrap_or(((x + w / 2).div_euclid(T), (y - 1).div_euclid(T)));
                self.bump(t.0, t.1);
            } else {
                self.hero.vy = 0;
            }
        } else {
            self.hero.y += self.hero.vy;
        }
        let (x, y) = self.hero.px();
        let was = self.hero.ground;
        self.hero.ground = self.hero.vy >= 0 && self.feet_blocked(x, y, w, h);
        self.hero.on_lift = if self.hero.ground { self.lift_under(x, y, w, h) } else { None };
        if self.hero.ground {
            self.hero.vy = 0;
            if let Some(i) = self.hero.on_lift {
                self.hero.y = (self.lifts[i].y - h) * FX;
            } else {
                self.hero.y = y * FX;
            }
            self.hero.combo = 0;
            if !was {
                self.hero.walk = 0;
            }
        }
        // 碰到邮票就拿。
        let (x, y) = self.hero.px();
        for ty in y.div_euclid(T)..=(y + h - 1).div_euclid(T) {
            for tx in x.div_euclid(T)..=(x + w - 1).div_euclid(T) {
                if self.tile(tx, ty) == STAMPT {
                    self.map[ty as usize][tx as usize] = EMPTY;
                    self.add_stamp();
                }
                if self.tile(tx, ty) == LIQUID {
                    self.kill_hero();
                    return;
                }
            }
        }
        if y > H as i32 + 8 {
            self.kill_hero();
        }
    }

    /// 往上走一个像素会不会撞到看不见的箱子（只从下面撞得出来）。
    fn head_hit(&self, x: i32, y: i32, w: i32) -> Option<(i32, i32)> {
        let ty = y.div_euclid(T);
        // 只在头正好碰到格子的下沿时算。
        if (y + 1).rem_euclid(T) != 0 {
            return None;
        }
        let cx = (x + w / 2).div_euclid(T);
        (self.tile(cx, ty) == HIDDEN).then_some((cx, ty))
    }

    /// 脚下一个像素有地（实心的格或者踏板的顶）。
    fn feet_blocked(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        self.hits(x, y + 1, w, h) || self.lift_under(x, y, w, h).is_some()
    }

    fn lift_under(&self, x: i32, y: i32, w: i32, h: i32) -> Option<usize> {
        let bottom = y + h;
        self.lifts.iter().position(|l| x + w > l.x && x < l.x + LIFT_W && bottom >= l.y && bottom <= l.y + 3)
    }

    fn kill_hero(&mut self) {
        if self.hero.dead.is_some() {
            return;
        }
        self.hero.dead = Some(0);
        self.hero.big = false;
        self.hero.plane = false;
        self.hero.star = 0;
        self.hero.vy = -1024;
        self.music.stop();
        self.music.play(tune::death());
    }

    fn dying(&mut self, t: u32) {
        let t = t + 1;
        self.hero.dead = Some(t);
        // 停一下，再跳起来掉出画面。
        if t > 30 {
            self.hero.vy = (self.hero.vy + G_FALL / 2).min(MAX_FALL);
            self.hero.y += self.hero.vy;
        }
        if t > 180 {
            if self.lives > 0 {
                self.lives -= 1;
                self.hero.big = false;
                self.hero.plane = false;
                self.begin_intro();
            } else {
                self.mode = Mode::Over;
                self.timer = 0;
                self.hi = self.hi.max(self.score);
                self.save();
                self.music.play(tune::game_over());
            }
        }
    }

    /// 挨了一下：无敌不算，闪着不算；大个子变小，小个子倒下。
    fn hurt_hero(&mut self) {
        let h = &mut self.hero;
        if h.star > 0 || h.hurt > 0 || h.dead.is_some() {
            return;
        }
        if h.big {
            h.big = false;
            h.plane = false;
            h.crouch = false;
            h.hurt = HURT_TIME;
            h.morph = 40;
            h.morph_to_big = false;
            h.y += 15 * FX;
            self.arpeggio(Channel::Square2, &[784, 587, 440, 330], 3, 40);
        } else {
            self.kill_hero();
        }
    }

    fn add_score(&mut self, n: u32, x: i32, y: i32) {
        self.score += n;
        self.popups.push(Popup { x, y, text: n.to_string(), t: 0 });
    }

    fn one_up(&mut self, x: i32, y: i32) {
        self.lives += 1;
        self.popups.push(Popup { x, y, text: String::new(), t: 0 });
        self.arpeggio(Channel::Square2, &[659, 784, 1319, 1047, 1175, 1568], 4, 40);
    }

    fn add_stamp(&mut self) {
        self.stamps += 1;
        self.score += 200;
        self.arpeggio(Channel::Square2, &[988, 1319], 3, 35);
        if self.stamps >= 100 {
            self.stamps -= 100;
            let (x, y) = self.hero.px();
            self.one_up(x, y - 8);
        }
    }

    /// 从下面顶了一格。
    pub fn bump(&mut self, tx: i32, ty: i32) {
        let t = self.tile(tx, ty);
        if !(0..self.width as i32).contains(&tx) || !(0..ROWS as i32).contains(&ty) {
            return;
        }
        let (ux, uy) = (tx as usize, ty as usize);
        let (px, py) = (tx * T, ty * T);
        let mut bumped = true;
        match t {
            CRATE if self.hero.big => {
                self.map[uy][ux] = EMPTY;
                for (dx, vx, vy) in [(0, -256, -1280), (8, 256, -1280), (0, -192, -768), (8, 192, -768)] {
                    self.debris.push(Debris { x: (px + dx) * FX, y: py * FX, vx, vy });
                }
                self.score += 50;
                self.sfx(Channel::Noise, 1200, 120, 45);
                tone_wav(&self.boom);
            }
            CRATE => self.sfx(Channel::Square2, 110, 60, 40),
            MAIL => {
                self.map[uy][ux] = USED;
                self.pop_stamp(px, py);
            }
            MULTI => {
                let now = self.tick;
                let i = match self.multi.iter().position(|m| (m.0, m.1) == (ux, uy)) {
                    Some(i) => i,
                    None => {
                        self.multi.push((ux, uy, 10, now + 240));
                        self.multi.len() - 1
                    }
                };
                let m = &mut self.multi[i];
                m.2 = m.2.saturating_sub(1);
                if m.2 == 0 || now > m.3 {
                    self.map[uy][ux] = USED;
                }
                self.pop_stamp(px, py);
            }
            MAIL_POWER | MAIL_GOLD | HIDDEN => {
                self.map[uy][ux] = USED;
                let kind = match t {
                    MAIL_GOLD => Kind::Gold,
                    HIDDEN => Kind::Love,
                    _ if self.hero.big => Kind::PlaneItem,
                    _ => Kind::Parcel,
                };
                let mut m = Mob::new(kind, px + 1, py);
                m.rise = 16;
                self.mobs.push(m);
                self.arpeggio(Channel::Square2, &[262, 330, 392, 523, 659], 2, 35);
            }
            _ if solid(t) => {
                bumped = false;
                self.sfx(Channel::Square2, 110, 50, 35);
            }
            _ => bumped = false,
        }
        if !bumped {
            return;
        }
        self.bumps.push((ux, uy, 8));
        // 站在这格上面的敌人翻倒，道具弹一下换方向；上面一格的邮票也拿到。
        let top = (px, py - 4, T, 6);
        let mut kills = Vec::new();
        for (i, m) in self.mobs.iter_mut().enumerate() {
            if !m.live() || !overlap(m.rect(), top) {
                continue;
            }
            if m.kind.enemy() && m.kind != Kind::Boss {
                kills.push(i);
            } else if m.kind.item() && m.rise == 0 {
                m.vy = -768;
                m.vx = if (m.x / FX + m.w / 2) < px + 8 { -256 } else { 256 };
            }
        }
        for i in kills.into_iter().rev() {
            self.knock(i, 0);
        }
        if self.tile(tx, ty - 1) == STAMPT {
            self.map[uy - 1][ux] = EMPTY;
            self.pop_stamp(px, py - T);
        }
    }

    fn pop_stamp(&mut self, px: i32, py: i32) {
        self.pops.push(PopStamp { x: px + 2, y: py - 16, t: 0 });
        self.add_stamp();
    }

    // ---------- 敌人和道具 ----------

    fn spawn_mobs(&mut self) {
        let edge = self.cam + W as i32 + 24;
        let mut i = 0;
        while i < self.spawns.len() {
            let (kind, x, y) = self.spawns[i];
            if x < edge {
                self.spawns.remove(i);
                let mut m = Mob::new(kind, x, y);
                if kind == Kind::Soot {
                    // 煤球先藏在烟囱里。
                    m.timer = 60;
                }
                self.mobs.push(m);
                if kind == Kind::Boss {
                    self.play_music();
                }
            } else {
                i += 1;
            }
        }
    }

    /// 带重力走一帧，撞墙回头。`edge` 是走到台子边会不会回头。回横着撞没撞墙。
    fn walk(&mut self, i: usize, gravity: bool) -> bool {
        let m = &self.mobs[i];
        let (mut x, y) = m.px();
        let (w, h) = (m.w, m.h);
        let tx = (m.x + m.vx).div_euclid(FX);
        let mut wall = false;
        while x != tx {
            let s = (tx - x).signum();
            if self.hits(x + s, y, w, h) {
                wall = true;
                break;
            }
            x += s;
        }
        let m = &mut self.mobs[i];
        if wall {
            m.x = x * FX;
        } else {
            m.x += m.vx;
        }
        if !gravity {
            return wall;
        }
        m.vy = (m.vy + G_FALL / 2).min(MAX_FALL);
        let (x, mut y) = self.mobs[i].px();
        let ty = (self.mobs[i].y + self.mobs[i].vy).div_euclid(FX);
        let mut stop = false;
        while y != ty {
            let s = (ty - y).signum();
            if self.hits(x, y + s, w, h) {
                stop = true;
                break;
            }
            y += s;
        }
        let m = &mut self.mobs[i];
        if stop {
            m.y = y * FX;
            m.vy = 0;
        } else {
            m.y += m.vy;
        }
        let (x, y) = self.mobs[i].px();
        let ground = self.hits(x, y + 1, w, h);
        self.mobs[i].ground = ground;
        wall
    }

    fn move_mobs(&mut self) {
        let hero = self.hero.rect();
        let n = self.mobs.len();
        for i in 0..n {
            let m = &mut self.mobs[i];
            if m.dying > 0 {
                m.dying += 1;
                m.vy = (m.vy + G_FALL / 2).min(MAX_FALL);
                m.y += m.vy;
                m.x += m.vx;
                continue;
            }
            if m.flat > 0 {
                m.flat += 1;
                continue;
            }
            if m.rise > 0 {
                m.rise -= 1;
                m.y -= FX;
                if m.rise == 0 && matches!(m.kind, Kind::Parcel | Kind::Love) {
                    m.vx = 256;
                }
                if m.rise == 0 && m.kind == Kind::Gold {
                    m.vx = 256;
                    m.vy = -896;
                }
                continue;
            }
            m.grace = m.grace.saturating_sub(1);
            m.hurt = m.hurt.saturating_sub(1);
            match m.kind {
                Kind::Walker | Kind::CanBug | Kind::Spiky | Kind::Parcel | Kind::Love => {
                    if self.walk(i, true) {
                        self.mobs[i].vx = -self.mobs[i].vx;
                    }
                }
                Kind::Can => {
                    let rolling = self.mobs[i].vx != 0;
                    if self.walk(i, true) && rolling {
                        self.mobs[i].vx = -self.mobs[i].vx;
                        self.sfx(Channel::Square2, 110, 30, 30);
                    }
                    if !rolling {
                        // 缩着的罐头过一会儿伸出腿来。
                        let m = &mut self.mobs[i];
                        m.timer += 1;
                        if m.timer > 360 {
                            m.kind = Kind::CanBug;
                            m.vx = if hero.0 < m.px().0 { -128 } else { 128 };
                            m.timer = 0;
                        }
                    }
                }
                Kind::Gold => {
                    let wall = self.walk(i, true);
                    let m = &mut self.mobs[i];
                    if wall {
                        m.vx = -m.vx;
                    }
                    if m.ground {
                        m.vy = -896;
                    }
                }
                Kind::PlaneItem => {}
                Kind::Crow => {
                    let m = &mut self.mobs[i];
                    m.timer += 1;
                    m.x += m.vx;
                    let wave = ((m.timer as f32) * std::f32::consts::TAU / 96.0).sin() * 16.0;
                    m.y = (m.home.1 + wave as i32) * FX;
                }
                Kind::Soot => {
                    // 在烟囱里：藏着、冒头、待一会儿、缩回去。主角站在烟囱旁边时不出来。
                    let m = &mut self.mobs[i];
                    m.timer += 1;
                    let near = (hero.0 + hero.2 / 2 - (m.home.0 + 7)).abs() < 24;
                    let cycle = m.timer % 240;
                    let out = match cycle {
                        0..=119 => 0,
                        120..=151 => (cycle - 120) as i32 / 2,
                        152..=211 => 16,
                        _ => 16 - (cycle - 212) as i32 / 2,
                    };
                    if cycle == 119 && near {
                        m.timer -= 1;
                    }
                    m.y = (m.home.1 - out) * FX;
                }
                Kind::Boss => self.move_boss(i),
                Kind::Shot => {
                    let wall = self.walk(i, false);
                    let m = &mut self.mobs[i];
                    m.y += m.vy;
                    m.timer = m.timer.saturating_sub(1);
                    if wall || m.timer == 0 {
                        m.flat = 100;
                    }
                }
                Kind::Letter => {
                    let m = &mut self.mobs[i];
                    m.x += m.vx;
                    m.vy = (m.vy + 32).min(MAX_FALL);
                    m.y += m.vy;
                }
            }
        }
        // 踢着滚的罐头撞别的敌人；纸飞机打敌人。
        for i in 0..self.mobs.len() {
            let (kind, live, rolling) = (self.mobs[i].kind, self.mobs[i].live(), self.mobs[i].vx != 0);
            if !live || !((kind == Kind::Can && rolling) || kind == Kind::Shot) {
                continue;
            }
            let r = self.mobs[i].rect();
            for j in 0..self.mobs.len() {
                if i == j || !self.mobs[j].live() || !self.mobs[j].kind.enemy() || !overlap(r, self.mobs[j].rect()) {
                    continue;
                }
                if self.mobs[j].kind == Kind::Soot && self.soot_hidden(j) {
                    continue;
                }
                if kind == Kind::Shot {
                    self.mobs[i].flat = 100;
                    if self.mobs[j].kind == Kind::Boss {
                        self.hit_boss(j, 1);
                    } else {
                        let p = self.mobs[j].kind.points();
                        self.knock(j, p);
                    }
                    break;
                }
                if self.mobs[j].kind == Kind::Boss {
                    continue;
                }
                let c = self.mobs[i].combo;
                self.mobs[i].combo += 1;
                if c >= KICK_COMBO.len() {
                    let (x, y) = self.mobs[j].px();
                    self.one_up(x, y);
                    self.knock(j, 0);
                } else {
                    self.knock(j, KICK_COMBO[c]);
                }
            }
        }
        // 出了画面太远、掉下去了、扁了够久了：拿掉。
        let cam = self.cam;
        self.mobs.retain(|m| {
            let (x, y) = m.px();
            !(y > H as i32 + 32 || x < cam - 64 || x > cam + W as i32 + 160 || (m.flat > 30) || (m.dying > 120))
        });
    }

    /// 蒸汽口三秒喷一次，喷一秒。
    pub fn steam_on(&self) -> bool {
        (100..160).contains(&(self.tick % 180))
    }

    /// 看得见的蒸汽口上面那一柱（两格高）。
    fn steam_rects(&self) -> Vec<(i32, i32, i32, i32)> {
        let x0 = self.cam.div_euclid(T);
        let mut out = Vec::new();
        for ty in 0..ROWS as i32 {
            for tx in x0..=x0 + W as i32 / T + 1 {
                if self.tile(tx, ty) == VENT {
                    out.push((tx * T + 3, ty * T - 32, 10, 32));
                }
            }
        }
        out
    }

    fn soot_hidden(&self, i: usize) -> bool {
        let m = &self.mobs[i];
        m.px().1 > m.home.1 - 6
    }

    fn move_boss(&mut self, i: usize) {
        let (hx, _, _, _) = self.hero.rect();
        let m = &mut self.mobs[i];
        m.timer += 1;
        // 在台子上来回走，隔一会儿跳一下、扔一封信，血少了更凶。
        let angry = m.hp <= 3;
        if m.vx == 0 {
            m.vx = -160;
        }
        let jump_every = if angry { 90 } else { 140 };
        let wall = self.walk(i, true);
        let m = &mut self.mobs[i];
        if wall {
            m.vx = -m.vx;
        }
        if m.ground && m.timer.is_multiple_of(jump_every) {
            m.vy = -1024;
        }
        let throw_every = if angry { 60 } else { 100 };
        if m.timer % throw_every == 30 {
            let (x, y) = m.px();
            let dir = if hx < x { -1 } else { 1 };
            let mut l = Mob::new(Kind::Letter, x + 8, y + 4);
            l.vx = dir * 384;
            l.vy = -768;
            self.mobs.push(l);
            self.sfx(Channel::Square2, 330, 40, 30);
        }
    }

    fn hit_boss(&mut self, i: usize, dmg: i32) {
        let m = &mut self.mobs[i];
        if m.hurt > 0 {
            return;
        }
        m.hp -= dmg;
        m.hurt = 60;
        self.sfx(Channel::Noise, 300, 120, 45);
        self.arpeggio(Channel::Square2, &[165, 131], 4, 40);
        if self.mobs[i].hp <= 0 {
            self.knock(i, Kind::Boss.points());
            self.boss_dead = true;
            tone_wav(&self.boom);
            self.mobs.retain(|m| m.kind != Kind::Letter);
            self.play_music();
        }
    }

    /// 打倒第 `i` 个敌人：翻过来掉下去，给 `points` 分（0 是不另给）。
    fn knock(&mut self, i: usize, points: u32) {
        let m = &mut self.mobs[i];
        m.dying = 1;
        m.vy = -768;
        m.vx = m.vx.signum() * 64;
        let (x, y) = m.px();
        if points > 0 {
            self.add_score(points, x, y);
        }
        self.sfx(Channel::Noise, 600, 60, 40);
    }

    /// 主角碰到了什么。
    fn touch_mobs(&mut self) {
        if self.hero.dead.is_some() {
            return;
        }
        let hero = self.hero.rect();
        let falling = self.hero.vy > 0;
        let hero_bottom = hero.1 + hero.3;
        for i in 0..self.mobs.len() {
            if i >= self.mobs.len() {
                break;
            }
            let m = &self.mobs[i];
            if !m.live() || m.rise > 0 || !overlap(hero, m.rect()) {
                continue;
            }
            let kind = m.kind;
            if kind == Kind::Soot && self.soot_hidden(i) {
                continue;
            }
            let (mx, my, mw, _) = m.rect();
            match kind {
                Kind::Parcel => {
                    self.mobs[i].flat = 100;
                    self.grow(false, mx, my);
                }
                Kind::PlaneItem => {
                    self.mobs[i].flat = 100;
                    self.grow(true, mx, my);
                }
                Kind::Gold => {
                    self.mobs[i].flat = 100;
                    self.hero.star = STAR_TIME;
                    self.hero.star_combo = 0;
                    self.add_score(1000, mx, my);
                    self.play_music();
                }
                Kind::Love => {
                    self.mobs[i].flat = 100;
                    self.one_up(mx, my);
                }
                Kind::Shot => {}
                Kind::Letter => self.hurt_hero(),
                _ if self.hero.star > 0 => {
                    if kind == Kind::Boss {
                        self.hit_boss(i, 1);
                    } else {
                        let c = self.hero.star_combo.min(COMBO.len() - 1);
                        self.hero.star_combo += 1;
                        self.knock(i, COMBO[c]);
                    }
                }
                Kind::Can if self.mobs[i].vx == 0 => {
                    // 停着的罐头：碰一下就踢出去，往离主角远的那边滚。
                    let dir = if hero.0 + hero.2 / 2 < mx + mw / 2 { 1 } else { -1 };
                    let m = &mut self.mobs[i];
                    m.vx = dir * 768;
                    m.grace = 12;
                    m.combo = 0;
                    m.timer = 0;
                    if falling && hero_bottom - my < 8 {
                        self.hero.vy = -768;
                    }
                    self.sfx(Channel::Square2, 880, 30, 40);
                    self.add_score(400, mx, my);
                }
                _ => {
                    let stompable = !matches!(kind, Kind::Spiky | Kind::Soot);
                    if stompable && falling && hero_bottom - my < 8 {
                        self.stomp(i);
                    } else if self.mobs[i].grace == 0 {
                        self.hurt_hero();
                    }
                }
            }
        }
        self.mobs.retain(|m| m.flat < 100);
    }

    fn stomp(&mut self, i: usize) {
        let held = self.keys.held(key::A);
        self.hero.vy = if held { -1024 } else { -640 };
        self.hero.ground = false;
        let (x, y) = self.mobs[i].px();
        let c = self.hero.combo;
        self.hero.combo += 1;
        self.sfx(Channel::Noise, 500, 50, 40);
        self.sfx(Channel::Square2, 220, 50, 35);
        if self.mobs[i].kind == Kind::Boss {
            self.hit_boss(i, 2);
            return;
        }
        let points = if c >= COMBO.len() {
            self.one_up(x, y);
            0
        } else {
            COMBO[c]
        };
        let m = &mut self.mobs[i];
        match m.kind {
            Kind::Walker => m.flat = 1,
            Kind::CanBug => {
                m.kind = Kind::Can;
                m.vx = 0;
                m.timer = 0;
            }
            Kind::Can => {
                m.vx = 0;
                m.timer = 0;
            }
            _ => {
                m.dying = 1;
                m.vy = 0;
            }
        }
        if points > 0 {
            self.add_score(points, x, y);
        }
    }

    fn grow(&mut self, plane: bool, x: i32, y: i32) {
        self.add_score(1000, x, y);
        let h = &mut self.hero;
        let was_big = h.big;
        if plane {
            h.plane = true;
        }
        if !was_big {
            h.big = true;
            h.y -= 15 * FX;
            // 头顶有东西：先蹲着。
            let (hx, hy) = h.px();
            if self.hits(hx, hy, 12, 30) {
                self.hero.crouch = true;
            }
            self.hero.morph = 40;
            self.hero.morph_to_big = true;
        }
        self.arpeggio(Channel::Square2, &[523, 659, 784, 1047, 1319, 1568], 3, 40);
    }

    // ---------- 过关 ----------

    fn clear_step(&mut self) {
        // 先走进门（缩没了），再把剩下的时间一格一格换成分。
        if self.timer < 60 {
            return;
        }
        if self.time > 0 {
            if self.timer.is_multiple_of(2) {
                let n = self.time.min(2);
                self.time -= n;
                self.score += n * 50;
                if self.timer.is_multiple_of(4) {
                    self.sfx(Channel::Square2, 1760, 20, 25);
                }
            }
            return;
        }
        let done = self.timer > 60 && !self.music.playing();
        if done && self.timer > 200 {
            self.hi = self.hi.max(self.score);
            self.reached_check = false;
            if self.level + 1 >= LEVELS.len() {
                self.mode = Mode::Ending;
                self.timer = 0;
                self.music.play(tune::ending());
            } else {
                self.level += 1;
                self.unlocked = self.unlocked.max(self.level);
                self.begin_intro();
            }
            self.save();
        }
    }

    // ---------- 画 ----------

    fn set_theme(&mut self) {
        for (i, rgb) in LEVELS[self.level].theme.colors() {
            self.frame.palette[i as usize] = rgb;
        }
    }

    fn draw(&mut self) {
        self.set_theme();
        match self.mode {
            Mode::Title => self.draw_title(),
            Mode::Intro => self.draw_intro(),
            Mode::Play | Mode::Clear => {
                self.update_cam();
                self.draw_world();
                self.draw_hud();
                if self.paused && (self.tick / 20).is_multiple_of(2) {
                    let f = &mut self.frame;
                    let s = lang().pick("PAUSE", "暂停");
                    let w = FONT.width(s);
                    f.rect(120 - w / 2 - 4, 70, w + 8, 16, INK);
                    f.text(&FONT, 120 - w / 2, 72, s, GOLD_HI);
                }
                if self.mode == Mode::Clear && self.time == 0 {
                    let s = lang().pick("DELIVERED!", "送达！");
                    self.frame.text_outlined(&FONT, 120 - FONT.width(s) / 2, 60, s, GOLD_HI, INK);
                }
            }
            Mode::Over => {
                let f = &mut self.frame;
                f.clear(INK);
                let big = FONT.sprite(lang().pick("GAME OVER", "送不到了"), RED_HI).scaled(2);
                f.blit(&big, (W as i32 - big.w) / 2, 54, false);
                f.text_center(&FONT, 100, &lang().pick(format!("SCORE {}", self.score), format!("得分 {}", self.score)), WHITE);
                f.text_center(&FONT, 116, &lang().pick(format!("HI {}", self.hi), format!("最高 {}", self.hi)), GOLD_HI);
            }
            Mode::Ending => self.draw_ending(),
        }
    }

    fn update_cam(&mut self) {
        let (hx, _) = self.hero.px();
        let want = (hx - 104).clamp(0, self.max_cam());
        if want > self.cam {
            self.cam = want;
        }
    }

    fn draw_background(&mut self) {
        let theme = LEVELS[self.level].theme;
        let cam = self.cam;
        let tick = self.tick;
        let f = &mut self.frame;
        // 天：三层，上深下浅。
        f.rect(0, 0, W as i32, 50, SKY);
        f.rect(0, 50, W as i32, 40, SKY2);
        f.rect(0, 90, W as i32, 70, SKY3);
        // 抖一排点让两层之间不那么硬。
        for x in (0..W as i32).step_by(2) {
            f.set(x, 50, SKY);
            f.set(x + 1, 90, SKY2);
        }
        let hash = |n: i32| -> i32 { (n.wrapping_mul(1_103_515_245).wrapping_add(12345) >> 8) & 0xff };
        match theme {
            Theme::Street | Theme::Roof => {
                if theme == Theme::Roof {
                    // 落山的太阳。
                    for y in -14..=14 {
                        for x in -14..=14 {
                            if x * x + y * y <= 196 {
                                f.set(180 - cam / 16 + x, 92 + y, BG_LIGHT);
                            }
                        }
                    }
                } else {
                    // 云：飘得慢。
                    for k in 0..4 {
                        let cx = (k * 97 - cam / 4 - (tick / 8) as i32).rem_euclid(W as i32 + 64) - 32;
                        let cy = 18 + (k * 23) % 30;
                        for (dx, dy, r) in [(0, 4, 6), (8, 0, 8), (18, 3, 7), (26, 6, 5)] {
                            for y in -r..=r {
                                for x in -r..=r {
                                    if x * x + y * y <= r * r {
                                        f.set(cx + dx + x, cy + dy + y, if y > r / 2 { CLOUD_LO } else { CLOUD });
                                    }
                                }
                            }
                        }
                    }
                }
                Self::skyline(f, cam / 4, 92, 36, BG_FAR, None, hash);
                Self::skyline(f, cam / 2, 112, 34, BG_NEAR, Some(BG_LIGHT), hash);
            }
            Theme::Harbor | Theme::NightRoof => {
                for k in 0..40 {
                    let (x, y) = ((hash(k) * 7 + k * 31 - cam / 8).rem_euclid(W as i32), hash(k + 99) % 80);
                    if !(tick / 16 + k as u32).is_multiple_of(9) {
                        f.set(x, y, CLOUD);
                    }
                }
                for y in -10..=10 {
                    for x in -10..=10 {
                        if x * x + y * y <= 100 && (x + 4) * (x + 4) + (y - 2) * (y - 2) > 64 {
                            f.set(196 - cam / 32 + x, 30 + y, BG_LIGHT);
                        }
                    }
                }
                Self::skyline(f, cam / 4, 80, 40, BG_FAR, Some(BG_LIGHT), hash);
                Self::skyline(f, cam / 2, 100, 40, BG_NEAR, Some(BG_LIGHT), hash);
            }
            Theme::Sewer => {
                // 一排排拱：远处的墙。
                let off = cam / 2;
                for x in 0..W as i32 {
                    let wx = (x + off).rem_euclid(64);
                    let arch = ((wx - 32) * (wx - 32)) / 24;
                    for y in 30 + arch.min(40)..128 {
                        f.set(x, y, if (y / 8 + (x + off) / 16) % 2 == 0 { BG_FAR } else { BG_NEAR });
                    }
                }
            }
            Theme::Tower => {
                // 墙上的大齿轮慢慢转。
                for (gx, gy, r) in [(60, 60, 30), (180, 90, 40), (300, 50, 26)] {
                    let cx = (gx - cam / 3).rem_euclid(W as i32 + 120) - 60;
                    let a0 = tick as f32 / 120.0 * if r > 30 { -1.0 } else { 1.0 };
                    for y in -r - 6..=r + 6 {
                        for x in -r - 6..=r + 6 {
                            let d = ((x * x + y * y) as f32).sqrt();
                            let ang = (y as f32).atan2(x as f32) + a0;
                            let tooth = (ang * 8.0).sin() > 0.3;
                            if (d <= r as f32 && d > r as f32 - 8.0) || (d > r as f32 && d <= r as f32 + 5.0 && tooth) || d < 6.0 {
                                f.set(cx + x, gy + y, BG_FAR);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 一排房子的剪影，带亮着的窗。`scroll` 是这一层挪了多少。
    fn skyline(f: &mut Frame, scroll: i32, base: i32, tall: i32, c: u8, lit: Option<u8>, hash: impl Fn(i32) -> i32) {
        for x in 0..W as i32 {
            let wx = x + scroll;
            let b = wx.div_euclid(20);
            let h = 10 + hash(b) * tall / 256;
            let top = base - h;
            for y in top..H as i32 {
                f.set(x, y, c);
            }
            if let Some(l) = lit {
                let lx = wx.rem_euclid(20);
                for y in (top + 4..H as i32).step_by(8) {
                    if ((4..7).contains(&lx) || (11..14).contains(&lx))
                        && hash(b * 31 + y + lx / 7) % 3 == 0 {
                            f.set(x, y, l);
                            f.set(x, y + 1, l);
                        }
                }
            }
        }
    }

    fn draw_world(&mut self) {
        self.draw_background();
        let cam = self.cam;
        let tick = self.tick;
        // 煤球先画，烟囱压在它上面（看起来是从烟囱里钻出来）。
        for m in self.mobs.iter().filter(|m| m.kind == Kind::Soot && m.dying == 0) {
            let (x, y) = m.px();
            self.frame.blit(&self.art.soot[(tick / 8 % 2) as usize], x - cam - 1, y - 1, false);
        }
        self.draw_tiles();
        self.draw_door();
        let f = &mut self.frame;
        let a = &self.art;
        for l in &self.lifts {
            for k in 0..3 {
                f.blit(&a.lift, l.x - cam + k * 16, l.y, false);
            }
        }
        for p in &self.pops {
            let y = p.y - [0, 6, 10, 13, 15, 16, 16, 15, 13, 10, 6, 2][(p.t / 3).min(11) as usize];
            f.blit(&a.stamp[((p.t / 3) % 3) as usize], p.x - cam, y, false);
        }
        for d in &self.debris {
            let (x, y) = (d.x / FX - cam, d.y / FX);
            for yy in 0..8 {
                for xx in 0..8 {
                    let c = a.crate_.at(xx + 4, yy + 4);
                    if c != 0 {
                        f.set(x + xx, y + yy, c);
                    }
                }
            }
        }
        for m in &self.mobs {
            if m.kind == Kind::Soot && m.dying == 0 {
                continue;
            }
            let (x, y) = m.px();
            let (sx, sy) = (x - cam, y);
            let anim = (tick / 10 % 2) as usize;
            let right = m.vx > 0;
            let flip_dead = m.dying > 0;
            let (spr, dx, dy): (Sprite, i32, i32) = match m.kind {
                Kind::Walker if m.flat > 0 => (a.walker_flat.clone(), -1, 9),
                Kind::Walker => (a.walker[anim].clone(), -1, -1),
                Kind::CanBug => (a.canbug[anim].clone(), -1, -1),
                Kind::Can => {
                    let peek = m.vx == 0 && m.timer > 300 && (tick / 4).is_multiple_of(2);
                    (a.can[peek as usize].clone(), -1, -1)
                }
                Kind::Crow => (a.crow[(tick / 8 % 2) as usize].clone(), -1, -3),
                Kind::Spiky => (a.spiky[anim].clone(), -1, -1),
                Kind::Soot => (a.soot[0].clone(), -1, -1),
                Kind::Boss => {
                    if m.hurt > 0 && (tick / 3).is_multiple_of(2) {
                        continue;
                    }
                    (a.boss[(!m.ground) as usize].clone(), -4, -4)
                }
                Kind::Parcel => (a.parcel.clone(), -1, -1),
                Kind::PlaneItem => (a.plane_item.clone(), -1, -1),
                Kind::Gold => (a.gold.clone(), -1, -1),
                Kind::Love => (a.love.clone(), -1, -1),
                Kind::Shot => (a.plane_shot.clone(), 0, 0),
                Kind::Letter => (a.letter.clone(), 0, 0),
            };
            let spr = if flip_dead { flip_v(&spr) } else { spr };
            // 道具从箱子里冒出来时，箱子以下的部分不画。
            if m.rise > 0 {
                let cut = m.home.1 - sy - dy;
                for yy in 0..spr.h.min(cut) {
                    for xx in 0..spr.w {
                        let c = spr.at(xx, yy);
                        if c != 0 {
                            f.set(sx + dx + xx, sy + dy + yy, c);
                        }
                    }
                }
                continue;
            }
            let flip = match m.kind {
                Kind::Shot | Kind::Letter => m.vx < 0,
                Kind::Parcel | Kind::PlaneItem | Kind::Gold | Kind::Love => false,
                _ => right,
            };
            f.blit(&spr, sx + dx, sy + dy, flip);
        }
        self.draw_hero();
        if self.steam_on() {
            let phase = (self.tick % 180 - 100) as i32;
            let rects = self.steam_rects();
            let f = &mut self.frame;
            for (x, y, w, h) in rects {
                for k in 0..6 {
                    let r = 3 + (k + phase / 4) % 3;
                    let cy = y + h - k * 6 - (phase % 6);
                    for yy in -r..=r {
                        for xx in -r..=r {
                            if xx * xx + yy * yy <= r * r {
                                f.set(x - cam + w / 2 + xx + if k % 2 == 0 { -1 } else { 1 }, cy + yy, STEAM);
                            }
                        }
                    }
                }
            }
        }
        let f = &mut self.frame;
        for p in &self.popups {
            let s = if p.text.is_empty() { lang().pick("1UP", "多一条命") } else { &p.text };
            f.text_outlined(&FONT, p.x - cam, p.y - 10 - (p.t / 3) as i32, s, WHITE, INK);
        }
    }

    fn draw_tiles(&mut self) {
        let cam = self.cam;
        let tick = self.tick;
        let (map, width, bumps) = (&self.map, self.width as i32, &self.bumps);
        let tile = |tx: i32, ty: i32| -> u8 {
            if tx < 0 || tx >= width {
                WALL
            } else if !(0..ROWS as i32).contains(&ty) {
                EMPTY
            } else {
                map[ty as usize][tx as usize]
            }
        };
        let f = &mut self.frame;
        let a = &self.art;
        let x0 = cam.div_euclid(T);
        for ty in 0..ROWS as i32 {
            for tx in x0..=x0 + W as i32 / T + 1 {
                let t = tile(tx, ty);
                if t == EMPTY || t == HIDDEN || tx >= width {
                    continue;
                }
                let lift = bumps.iter().find(|b| (b.0 as i32, b.1 as i32) == (tx, ty)).map_or(0, |b| [0, 2, 4, 5, 5, 4, 3, 1, 0][(8 - b.2) as usize]);
                let (x, y) = (tx * T - cam, ty * T - lift);
                let spr = match t {
                    GROUND => {
                        if tile(tx, ty - 1) == GROUND {
                            &a.fill
                        } else {
                            &a.top
                        }
                    }
                    WALL => &a.brick,
                    STONEB => &a.stone,
                    CRATE | MULTI => &a.crate_,
                    MAIL | MAIL_POWER | MAIL_GOLD => &a.mail[[0, 0, 0, 1, 2, 1][(tick / 8 % 6) as usize]],
                    USED => &a.used,
                    CH_TL => &a.chimney[0],
                    CH_TR => &a.chimney[1],
                    CH_BL => &a.chimney[2],
                    CH_BR => &a.chimney[3],
                    VENT => &a.vent,
                    STAMPT => {
                        let k = [0, 0, 1, 2, 1][(tick / 6 % 5) as usize];
                        f.blit(&a.stamp[k], x + 2, y + 1, false);
                        continue;
                    }
                    LIQUID => {
                        let top = tile(tx, ty - 1) != LIQUID;
                        for yy in 0..T {
                            for xx in 0..T {
                                let wave = ((xx + tx * T + (tick / 4) as i32) % 16) < 8;
                                let c = if top && yy < 2 { if wave { WATER_HI } else { WATER } } else if top && yy == 2 && wave { WATER_HI } else { WATER };
                                f.set(x + xx, y + yy, c);
                            }
                        }
                        continue;
                    }
                    _ => continue,
                };
                f.blit(spr, x, y, false);
            }
        }
    }

    /// 邮局：门两边是墙和窗，门上一块招牌。钟楼里大盗没倒时门关着。
    fn draw_door(&mut self) {
        let (dx, dy) = (self.door.0 - self.cam, self.door.1);
        if dx < -64 || dx > W as i32 + 64 {
            return;
        }
        let locked = self.spawns.iter().any(|s| s.0 == Kind::Boss) || self.mobs.iter().any(|m| m.kind == Kind::Boss);
        let f = &mut self.frame;
        // 墙：五格宽、四格高。
        f.rect(dx - 32, dy - 48, 80, 64, CREAM);
        f.frame_rect(dx - 32, dy - 48, 80, 64, INK);
        f.rect(dx - 36, dy - 52, 88, 6, RED);
        f.rect(dx - 36, dy - 52, 88, 1, INK);
        f.rect(dx - 36, dy - 47, 88, 1, RED_LO);
        // 两扇窗。
        for wx in [dx - 24, dx + 24] {
            f.rect(wx, dy - 36, 16, 16, NAVY);
            f.frame_rect(wx, dy - 36, 16, 16, INK);
            f.rect(wx + 7, dy - 36, 2, 16, INK);
            f.rect(wx + 2, dy - 34, 4, 4, BLUE_HI);
        }
        // 招牌。
        let sign = lang().pick("POST", "邮局");
        let w = FONT.width(sign);
        f.rect(dx + 8 - w / 2 - 3, dy - 46, w + 6, 13, GREEN_LO);
        f.text(&FONT, dx + 8 - w / 2, dy - 46, sign, GOLD_HI);
        // 门。
        f.rect(dx, dy - 16, 16, 32, if locked { WOOD_LO } else { INK });
        f.frame_rect(dx - 1, dy - 17, 18, 33, WOOD_LO);
        if locked {
            f.rect(dx + 7, dy - 16, 2, 32, INK);
            f.rect(dx + 4, dy, 8, 6, GRAY);
        } else {
            f.rect(dx + 2, dy - 12, 12, 2, GOLD);
        }
        // 门口一个红邮筒。
        f.rect(dx + 22, dy + 2, 10, 14, RED);
        f.rect(dx + 21, dy, 12, 3, RED_LO);
        f.rect(dx + 24, dy + 5, 6, 2, INK);
    }

    fn hero_sprite(&self) -> (Sprite, i32, i32) {
        let h = &self.hero;
        let a = &self.art;
        if h.dead.is_some() {
            return (a.die.clone(), -2, -1);
        }
        let big = if h.morph > 0 {
            // 变的那一下：大小换着画。
            let flip = (h.morph / 4).is_multiple_of(2);
            if h.morph_to_big {
                flip
            } else {
                !flip
            }
        } else {
            h.big
        };
        let pose = if !h.ground {
            4
        } else if h.skid {
            5
        } else if h.vx != 0 {
            h.walk.max(1)
        } else {
            0
        };
        let (mut s, dy) = if big {
            let p = if h.crouch { 6 } else { pose };
            (a.big[p].clone(), if h.big && !h.crouch { -2 } else { -17 })
        } else {
            (a.small[pose].clone(), if h.big && !h.crouch { 14 } else { -1 })
        };
        if h.plane {
            s = s.recolor(&[(CAP, RED), (CAP_HI, RED_HI), (SHIRT, WHITE), (SHIRT_LO, TIN)]);
        }
        if h.star > 0 {
            let sets = [(RED, RED_HI, GOLD), (GOLD, GOLD_HI, WHITE), (GREEN, GREEN_HI, GOLD_HI), (PURPLE, PURPLE_HI, PINK)];
            let speed = if h.star < 120 { 8 } else { 3 };
            let (c, hi, sh) = sets[(self.tick / speed % 4) as usize];
            s = s.recolor(&[(CAP, c), (CAP_HI, hi), (SHIRT, sh), (SHIRT_LO, c), (PANTS, c)]);
        }
        (s, -2, dy)
    }

    fn draw_hero(&mut self) {
        let h = &self.hero;
        if self.mode == Mode::Clear && self.timer > 20 {
            return;
        }
        if h.hurt > 0 && h.morph == 0 && (self.tick / 2).is_multiple_of(2) {
            return;
        }
        let (x, y) = h.px();
        let (s, dx, dy) = self.hero_sprite();
        let flip = h.dir < 0;
        self.frame.blit(&s, x - self.cam + dx, y + dy, flip);
    }

    fn draw_hud(&mut self) {
        let f = &mut self.frame;
        let a = &self.art;
        f.text_outlined(&FONT, 4, 1, &format!("{:06}", self.score), WHITE, INK);
        f.blit(&a.stamp_icon, 50, 1, false);
        f.text_outlined(&FONT, 63, 1, &format!("×{:02}", self.stamps), WHITE, INK);
        let name = LEVELS[self.level].label();
        let short = name.split(' ').next().unwrap_or(name);
        f.text_outlined(&FONT, 112, 1, short, WHITE, INK);
        let tc = if self.time <= 100 && (self.tick / 15).is_multiple_of(2) { RED_HI } else { WHITE };
        f.text_outlined(&FONT, 168, 1, lang().pick("TIME", "时间"), WHITE, INK);
        f.text_outlined(&FONT, 196, 1, &format!("{:03}", self.time), tc, INK);
    }

    fn draw_intro(&mut self) {
        let f = &mut self.frame;
        f.clear(INK);
        let name = LEVELS[self.level].label();
        let big = FONT.sprite(name, WHITE).scaled(2);
        f.blit(&big, (W as i32 - big.w) / 2, 50, false);
        f.blit(&self.art.small[0], 96, 92, false);
        f.text(&FONT, 118, 94, &format!("× {}", self.lives), WHITE);
        f.text_center(&FONT, 130, &lang().pick(format!("SCORE {:06}", self.score), format!("得分 {:06}", self.score)), GRAY);
    }

    fn draw_title(&mut self) {
        self.cam = 0;
        self.draw_background();
        let tick = self.tick;
        let f = &mut self.frame;
        let a = &self.art;
        // 地面一排。
        for x in (0..W as i32).step_by(16) {
            f.blit(&a.top, x, 128, false);
            f.blit(&a.fill, x, 144, false);
        }
        // 标题：大字带阴影。
        let name = lang().pick("GALE COURIER", "疾风邮差");
        let title = FONT.sprite(name, GOLD_HI).scaled(3);
        let shade = FONT.sprite(name, RED_LO).scaled(3);
        let tx = (W as i32 - title.w) / 2;
        f.blit(&shade, tx + 3, 21, false);
        f.blit(&title, tx, 18, false);
        // 小邮差从左跑到右，后面拖着风的线。
        let run = (tick as i32 * 2).rem_euclid(W as i32 + 64) - 32;
        let pose = [1, 2, 3, 2][(tick / 5 % 4) as usize];
        let s = &a.big[pose];
        for k in 0..3 {
            let y = 104 + k * 6;
            f.rect(run - 20 - k * 6, y, 14, 1, WHITE);
        }
        f.blit(s, run, 96, false);
        let name = LEVELS[self.level].label();
        let pick = if self.unlocked > 0 { format!("← {} →", name) } else { name.to_string() };
        f.text_outlined(&FONT, (W as i32 - FONT.width(&pick)) / 2, 62, &pick, WHITE, INK);
        if (tick / 30).is_multiple_of(2) {
            let s = lang().pick("PRESS START", "按 START 开始");
            f.text_outlined(&FONT, (W as i32 - FONT.width(s)) / 2, 78, s, GOLD_HI, INK);
        }
        let hi = lang().pick(format!("HI {}", self.hi), format!("最高 {}", self.hi));
        f.text_outlined(&FONT, (W as i32 - FONT.width(&hi)) / 2, 146, &hi, WHITE, INK);
    }

    fn draw_ending(&mut self) {
        let f = &mut self.frame;
        f.clear(NAVY);
        let big = FONT.sprite(lang().pick("ALL DELIVERED", "全部送达"), GOLD_HI).scaled(2);
        f.blit(&big, (W as i32 - big.w) / 2, 30, false);
        f.text_center(&FONT, 70, lang().pick("The mail thief is caught", "邮件大盗抓到了"), WHITE);
        f.text_center(&FONT, 86, lang().pick("Every letter got through", "今天的急件一封不少"), WHITE);
        f.text_center(&FONT, 110, &lang().pick(format!("SCORE {}", self.score), format!("得分 {}", self.score)), WHITE);
        f.text_center(&FONT, 126, &lang().pick(format!("HI {}", self.hi), format!("最高 {}", self.hi)), GOLD_HI);
        let run = (self.tick as i32).rem_euclid(W as i32 + 32) - 16;
        f.blit(&self.art.small[[1, 2, 3, 2][(self.tick / 5 % 4) as usize]], run, 142, false);
    }
}

/// 上下翻过来（被打倒的敌人）。
fn flip_v(s: &Sprite) -> Sprite {
    let mut px = Vec::with_capacity(s.px.len());
    for y in (0..s.h).rev() {
        for x in 0..s.w {
            px.push(s.at(x, y));
        }
    }
    Sprite { w: s.w, h: s.h, px }
}

static STATE: Mutex<Option<Game>> = Mutex::new(None);

vibeplay::main!(|app| {
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
