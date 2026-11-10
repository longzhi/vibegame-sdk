//! 游戏《雷霆突击》：横版射击。角色、关卡、画面、音乐都是自己做的。
//!
//! 两个突击队员（雷、霆，开局选一个）一路往前打：丛林、瀑布（竖着往上爬）、雪原、要塞，每关最后一个大怪。
//! 十字键走和瞄（八个方向），B 开枪，A 跳（跳起来翻跟头）。在地上按下趴下，按下加 A 从台子上跳下去。
//! 在水里按下潜水，这时打不着你，你也开不了枪。START 开始、暂停。
//!
//! 挨一下就倒，倒了丢掉手里的枪。打下飞过去的补给舱、打坏一开一关的地堡，掉勋章：
//! S 散弹（一次五发，扇形）、M 机枪（按住连发）、L 激光（穿过敌人）、F 火球（转着圈往前）、R 子弹快、B 护盾（一阵子不怕打）。
//!
//! 位置和速度用 1/256 像素的整数（`FX`），随机数是自己的整数随机，同样的按键每次跑出来一样。

mod art;
mod levels;
mod tune;

use art::*;
use vibegame::FONT;
use levels::LEVELS;
use vibegame::full::{key, tone_wav, Channel, Frame, Keys, Music, Sprite, H, W};
use std::sync::Mutex;

/// 一个像素是 256 份。
const FX: i32 = 256;
/// 一格 16 像素。
const T: i32 = 16;

// 主角的手感（1/256 像素每帧）。
const WALK: i32 = 288;
const JUMP_V: i32 = 1152;
const GRAV: i32 = 64;
const MAX_FALL: i32 = 1280;
/// 主角的身子：10×22。
const HW: i32 = 10;
const HH: i32 = 22;
/// 倒下以后多久接着来、接着来以后多久不怕打。
const DEAD_TIME: u32 = 100;
const SAFE_TIME: u32 = 120;
const BARRIER_TIME: u32 = 1200;
/// 一开始几条命（不算正在用的这条）、能接着打几次。
const LIVES: u32 = 2;
const CONTINUES: u32 = 3;
/// 存档：开到第几关、最高分。
const SAVE: &str = "progress";

// ---------- 地图块 ----------
pub const EMPTY: u8 = 0;
pub const ROCK: u8 = 1;
pub const LEDGE: u8 = 2;
pub const BRIDGE: u8 = 3;
pub const POOL: u8 = 4;
pub const NOZZLE: u8 = 5;
/// 炮台、地堡占着的格：实心，打坏了变回石头或空地。
pub const TURRET_T: u8 = 6;
pub const PILL_T: u8 = 7;

pub fn solid(t: u8) -> bool {
    matches!(t, ROCK | NOZZLE | TURRET_T | PILL_T)
}
/// 只从上面挡的格：台子、桥。
pub fn floor(t: u8) -> bool {
    matches!(t, LEDGE | BRIDGE)
}

/// 64 个方向里第 0 到 16 个的正弦（乘 256）：0 是右，16 是下，32 是左，48 是上。
const SIN: [i32; 17] = [0, 25, 50, 74, 98, 121, 142, 162, 181, 198, 213, 226, 237, 245, 251, 255, 256];

fn sin64(a: i32) -> i32 {
    let a = a.rem_euclid(64);
    match a {
        0..=16 => SIN[a as usize],
        17..=32 => SIN[(32 - a) as usize],
        33..=48 => -SIN[(a - 32) as usize],
        _ => -SIN[(64 - a) as usize],
    }
}

/// 第 `a` 个方向、速度 `v`：(vx, vy)。
fn dir(a: i32, v: i32) -> (i32, i32) {
    (sin64(a + 16) * v / 256, sin64(a) * v / 256)
}

/// 从 (dx, dy) 看过去最近的方向（64 个里挑，`step` 个一档）。
fn angle_to(dx: i32, dy: i32, step: i32) -> i32 {
    let mut best = (i64::MIN, 0);
    let mut a = 0;
    while a < 64 {
        let d = dx as i64 * sin64(a + 16) as i64 + dy as i64 * sin64(a) as i64;
        if d > best.0 {
            best = (d, a);
        }
        a += step;
    }
    best.1
}

fn overlap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

fn px(v: i32) -> i32 {
    v.div_euclid(FX)
}

// ---------- 枪 ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weapon {
    Normal,
    Machine,
    Spread,
    Laser,
    Fire,
}

impl Weapon {
    fn letter(self) -> char {
        match self {
            Weapon::Normal => ' ',
            Weapon::Machine => 'M',
            Weapon::Spread => 'S',
            Weapon::Laser => 'L',
            Weapon::Fire => 'F',
        }
    }
    /// 按住多少帧打一发、屏上最多几发。
    fn rate(self) -> (u32, usize) {
        match self {
            Weapon::Normal => (14, 4),
            Weapon::Machine => (6, 10),
            Weapon::Spread => (16, 10),
            Weapon::Laser => (22, 1),
            Weapon::Fire => (14, 4),
        }
    }
    fn damage(self) -> i32 {
        match self {
            Weapon::Laser => 3,
            Weapon::Fire => 2,
            _ => 1,
        }
    }
}

/// 主角打出去的一发。x、y 是中心（1/256 像素）。
#[derive(Debug, Clone)]
pub struct Shot {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub kind: Weapon,
    /// 火球转圈的中心和转到第几格。
    base: (i32, i32),
    phase: i32,
    /// 激光已经打过的敌人：一发只打一个人一次。
    hit: Vec<u32>,
    pub gone: bool,
}

/// 敌人打出来的东西。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EKind {
    /// 普通子弹：直着飞。
    Bullet,
    /// 炮弹、手雷：有重力，落地炸开。
    Shell,
    /// 大怪吐的火球：大一点。
    Ball,
    /// 机器人扔的炸弹：弹三下再炸。
    Bomb,
}

#[derive(Debug, Clone)]
pub struct EShot {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub kind: EKind,
    pub bounces: u32,
    pub gone: bool,
}

// ---------- 敌人 ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// 从画面边上跑出来的兵。
    Runner,
    /// 端枪站着的兵。
    Rifle,
    /// 躲在沙袋后面，一会儿站起来打一枪。
    Hider,
    /// 蹲在沙袋后面扔手雷。
    Lobber,
    Turret,
    Pillbox,
    /// 飞过去的补给舱。
    Pod,
    Medal,
    /// 滚石头的地方（看不见）。
    RockSpot,
    Rock,
    Tank,
    // 大怪和它的几块。
    GateCannon,
    GateCore,
    Totem,
    Hand,
    Crawler,
    Robot,
}

impl Kind {
    fn points(self) -> u32 {
        match self {
            Kind::Runner => 100,
            Kind::Rifle | Kind::Hider | Kind::Lobber => 200,
            Kind::Rock => 300,
            Kind::Turret | Kind::Pod => 500,
            Kind::Pillbox | Kind::GateCannon | Kind::Hand => 1000,
            Kind::Tank => 2000,
            Kind::GateCore | Kind::Totem | Kind::Crawler | Kind::Robot => 10000,
            Kind::Medal | Kind::RockSpot => 0,
        }
    }
    fn boss(self) -> bool {
        matches!(self, Kind::GateCore | Kind::Totem | Kind::Crawler | Kind::Robot)
    }
    /// 碰到主角，主角就倒。
    fn deadly(self) -> bool {
        matches!(self, Kind::Runner | Kind::Rifle | Kind::Rock | Kind::Tank | Kind::Crawler | Kind::Robot | Kind::Hand | Kind::Totem)
    }
}

/// 一个敌人、一块大怪、一个勋章。x、y 是左上角（1/256 像素），w、h 是像素。
#[derive(Debug, Clone)]
pub struct Mob {
    pub id: u32,
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub w: i32,
    pub h: i32,
    pub hp: i32,
    pub timer: u32,
    /// 朝哪边：1 右，-1 左。
    pub face: i32,
    pub ground: bool,
    /// 被打倒了：还剩几帧（往后飞、炸开）。
    pub dying: u32,
    /// 挨了一下，闪几帧。
    pub flash: u32,
    /// 瞄的方向（64 个里第几个）。
    pub aim: i32,
    /// 掉的勋章、勋章上的字母。
    pub letter: char,
    /// 出生的地方（像素）。
    pub home: (i32, i32),
    /// 状态机走到哪一步（大怪用）。
    pub state: u32,
    /// 几帧内不管台子，往下掉（石头弹过以后、跳下台子的兵）。
    pub through: u32,
}

impl Mob {
    fn new(id: u32, kind: Kind, x: i32, y: i32) -> Mob {
        let (w, h, hp) = match kind {
            Kind::Runner | Kind::Rifle | Kind::Hider => (10, 22, 1),
            Kind::Lobber => (12, 14, 1),
            Kind::Turret => (16, 16, 8),
            Kind::Pillbox => (16, 16, 6),
            Kind::Pod => (16, 10, 1),
            Kind::Medal => (14, 12, 1),
            Kind::RockSpot => (16, 16, 1),
            Kind::Rock => (14, 14, 4),
            Kind::Tank => (34, 18, 20),
            Kind::GateCannon => (14, 10, 12),
            Kind::GateCore => (24, 26, 32),
            Kind::Totem => (40, 32, 48),
            Kind::Hand => (12, 12, 16),
            Kind::Crawler => (64, 30, 60),
            Kind::Robot => (40, 56, 90),
        };
        Mob {
            id,
            kind,
            x: x * FX,
            y: y * FX,
            vx: 0,
            vy: 0,
            w,
            h,
            hp,
            timer: 0,
            face: -1,
            ground: false,
            dying: 0,
            flash: 0,
            aim: 32,
            letter: ' ',
            home: (x, y),
            state: 0,
            through: 0,
        }
    }
    pub fn px(&self) -> (i32, i32) {
        (px(self.x), px(self.y))
    }
    pub fn rect(&self) -> (i32, i32, i32, i32) {
        let (x, y) = self.px();
        (x, y, self.w, self.h)
    }
    pub fn live(&self) -> bool {
        self.dying == 0
    }
    /// 打得着的那块。没有就是这时打不着。
    fn weak(&self) -> Option<(i32, i32, i32, i32)> {
        let (x, y) = self.px();
        match self.kind {
            Kind::Hider if self.state == 0 => None,
            Kind::Pillbox if self.state == 0 => None,
            Kind::Medal | Kind::RockSpot => None,
            Kind::Lobber => Some((x, y, self.w, 8)),
            Kind::Crawler => Some((x + if self.face < 0 { 38 } else { 10 }, y, 16, 12)),
            Kind::Robot => Some((x + 14, y + 30, 12, 12)),
            _ => Some(self.rect()),
        }
    }
    /// 挡子弹的那块（打不坏的地方打上去叮一声）。
    fn armor(&self) -> Option<(i32, i32, i32, i32)> {
        let (x, y) = self.px();
        match self.kind {
            Kind::Crawler => Some((x, y + 10, self.w, self.h - 10)),
            Kind::Robot => Some(self.rect()),
            Kind::Pillbox => Some(self.rect()),
            _ => None,
        }
    }
}

/// 主角。x、y 是身子的左上角（1/256 像素）。
#[derive(Debug, Clone)]
pub struct Hero {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub face: i32,
    pub ground: bool,
    /// 在空中翻跟头。
    pub spin: bool,
    pub prone: bool,
    pub in_water: bool,
    pub diving: bool,
    pub aim: Aim,
    /// 在空中按着下：往正下打。
    pub aim_down: bool,
    pub weapon: Weapon,
    pub rapid: bool,
    /// 倒下了：还剩几帧。
    pub dead: u32,
    pub safe: u32,
    pub barrier: u32,
    /// 几帧内不管台子（按下加 A 跳下去）。
    pub through: u32,
    cool: u32,
    walk: u32,
}

impl Hero {
    fn new(x: i32, y: i32) -> Hero {
        Hero {
            x: x * FX,
            y: y * FX,
            vx: 0,
            vy: 0,
            face: 1,
            ground: false,
            spin: false,
            prone: false,
            in_water: false,
            diving: false,
            aim: Aim::Fwd,
            aim_down: false,
            weapon: Weapon::Normal,
            rapid: false,
            dead: 0,
            safe: 0,
            barrier: 0,
            through: 0,
            cool: 0,
            walk: 0,
        }
    }
    pub fn px(&self) -> (i32, i32) {
        (px(self.x), px(self.y))
    }
    /// 挨打的那块：站着、趴着、翻跟头不一样大。
    pub fn hurt_box(&self) -> (i32, i32, i32, i32) {
        let (x, y) = self.px();
        if self.prone {
            (x - 4, y + 15, 18, 7)
        } else if self.spin {
            (x, y + 5, HW, 12)
        } else if self.in_water {
            (x + 1, y + 2, 8, 12)
        } else {
            (x + 1, y + 2, 8, 20)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Title,
    /// 关开头：写着第几关。
    Intro,
    Play,
    /// 打完大怪。
    Clear,
    /// 没命了：接着打还是不打了。
    Over,
    Ending,
}

/// 炸开的一团。
#[derive(Debug, Clone)]
struct Boom {
    x: i32,
    y: i32,
    t: u32,
    size: i32,
    /// 会伤主角（手雷炸开）。
    hurts: bool,
}

/// 一座桥：哪一行、从第几列到第几列、炸到第几列、下一节什么时候炸。
#[derive(Debug, Clone)]
pub struct Bridge {
    pub row: usize,
    pub from: usize,
    pub to: usize,
    pub next: usize,
    pub timer: u32,
    pub lit: bool,
}

struct Art {
    hero: [Vec<Vec<Sprite>>; 2],
    prone: [Sprite; 2],
    spin: [[Sprite; 4]; 2],
    downed: [Sprite; 2],
    soldier: Vec<Sprite>,
    lobber: [Sprite; 2],
    sandbag: Sprite,
    turret: Sprite,
    pillbox: [Sprite; 2],
    pod: [Sprite; 2],
    medal: Sprite,
    rock: Sprite,
    tank: Sprite,
    ledge: Sprite,
    fill: Sprite,
    bridge: Sprite,
    life: Sprite,
}

impl Art {
    fn new() -> Art {
        let lei = hero_poses();
        let ting: Vec<Vec<Sprite>> = lei.iter().map(|v| v.iter().map(|s| s.recolor(&TING)).collect()).collect();
        let prone = Sprite::art(&PRONE, &HERO_KEYS);
        let spin0 = Sprite::art(&SPIN, &HERO_KEYS);
        let spins = |s: &Sprite| {
            let a = rotate(s);
            let b = rotate(&a);
            let c = rotate(&b);
            [s.clone(), a, b, c]
        };
        let downed = Sprite::art(&DOWNED, &HERO_KEYS);
        Art {
            hero: [lei, ting],
            prone: [prone.clone(), prone.recolor(&TING)],
            spin: [spins(&spin0), spins(&spin0.recolor(&TING))],
            downed: [downed.clone(), downed.recolor(&TING)],
            soldier: soldier_poses(),
            lobber: [Sprite::art(&LOBBER[0], &HERO_KEYS).recolor(&ENEMY), Sprite::art(&LOBBER[1], &HERO_KEYS).recolor(&ENEMY)],
            sandbag: Sprite::art(&SANDBAG, &SANDBAG_KEYS),
            turret: Sprite::art(&TURRET, &TURRET_KEYS),
            pillbox: [Sprite::art(&PILLBOX[0], &TURRET_KEYS), Sprite::art(&PILLBOX[1], &TURRET_KEYS)],
            pod: [Sprite::art(&POD[0], &TURRET_KEYS), Sprite::art(&POD[1], &TURRET_KEYS)],
            medal: Sprite::art(&MEDAL, &MEDAL_KEYS),
            rock: Sprite::art(&BOULDER, &ROCK_KEYS),
            tank: Sprite::art(&TANK, &TANK_KEYS),
            ledge: Sprite::art(&T_LEDGE, &TILE_KEYS),
            fill: Sprite::art(&T_FILL, &TILE_KEYS),
            bridge: Sprite::art(&T_BRIDGE, &TILE_KEYS),
            life: Sprite::art(&[".kkk.", "kyYyk", "kYyyk", "kyyyk", ".kok.", "..k.."], &MEDAL_KEYS),
        }
    }
}

/// 一段自己生成的噪声，慢慢变小：炸开的声音。
fn boom_wav(tenths: u32, seed: u32) -> Vec<u8> {
    let rate = 8000u32;
    let n = (rate * tenths / 10) as usize;
    let mut s = seed;
    let mut low = 0i32;
    let mut data = Vec::with_capacity(n * 2);
    for i in 0..n {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let noise = (s >> 16) as i32 % 0x8000 - 0x4000;
        low = (low * 3 + noise) / 4;
        let fade = (n - i) as i32 * 1000 / n as i32;
        let v = (low * 2 * fade / 1000).clamp(-32768, 32767) as i16;
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
    seed: u32,
    /// 第几关（从 0 数）。
    pub level: usize,
    pub unlocked: usize,
    /// 选的队员：0 雷、1 霆。
    pub who: usize,
    pub hi: u32,
    pub score: u32,
    next_life: u32,
    pub lives: u32,
    pub continues: u32,
    pub map: Vec<Vec<u8>>,
    pub width: usize,
    pub height: usize,
    pub vertical: bool,
    pub hero: Hero,
    pub mobs: Vec<Mob>,
    pub shots: Vec<Shot>,
    pub eshots: Vec<EShot>,
    booms: Vec<Boom>,
    pub bridges: Vec<Bridge>,
    /// 还没出场的：种类、像素位置、掉的勋章。
    spawns: Vec<(Kind, i32, i32, char)>,
    next_id: u32,
    runner_timer: u32,
    /// 画面左上角（像素）。横着的关只往右走，竖着的只往上走。
    pub cam: (i32, i32),
    start: (i32, i32),
    boss_at: (i32, i32),
    pub boss_on: bool,
    pub boss_dead: bool,
    /// 大怪倒了以后放炸开的那几帧。
    boss_fall: u32,
    paused: bool,
    menu: usize,
    shake: u32,
    jingle: Vec<(u32, Channel, u32, u32, u32)>,
    boom: Vec<u8>,
    big_boom: Vec<u8>,
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
            seed: 0x1357_9bdf,
            level: 0,
            unlocked: 0,
            who: 0,
            hi: 20000,
            score: 0,
            next_life: 20000,
            lives: LIVES,
            continues: CONTINUES,
            map: vec![vec![EMPTY; 1]; 10],
            width: 1,
            height: 10,
            vertical: false,
            hero: Hero::new(32, 90),
            mobs: Vec::new(),
            shots: Vec::new(),
            eshots: Vec::new(),
            booms: Vec::new(),
            bridges: Vec::new(),
            spawns: Vec::new(),
            next_id: 1,
            runner_timer: 0,
            cam: (0, 0),
            start: (32, 90),
            boss_at: (0, 0),
            boss_on: false,
            boss_dead: false,
            boss_fall: 0,
            paused: false,
            menu: 0,
            shake: 0,
            jingle: Vec::new(),
            boom: boom_wav(3, 0x2468_1357),
            big_boom: boom_wav(9, 0x1122_3344),
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
        g.level = g.unlocked;
        g.load(g.level);
        g.music.play(tune::title());
        g
    }
}

impl Game {
    fn save(&self) {
        let _ = vibegame::save(SAVE, format!("{},{}", self.unlocked, self.hi).as_bytes());
    }

    fn rand(&mut self, n: u32) -> u32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed >> 8) % n.max(1)
    }

    fn id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
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

    fn bang(&mut self, big: bool) {
        let _ = tone_wav(if big { &self.big_boom } else { &self.boom });
    }

    // ---------- 地图 ----------

    /// 读第 `i` 关：地图块、敌人、起点、大怪的位置。
    pub fn load(&mut self, i: usize) {
        let l = &LEVELS[i];
        let rows = levels::rows(l);
        self.vertical = l.vertical;
        self.width = rows.iter().map(|r| r.len()).max().unwrap_or(1);
        self.height = rows.len();
        self.map = vec![vec![EMPTY; self.width]; self.height];
        self.spawns.clear();
        self.bridges.clear();
        let mut drops: Vec<(i32, i32, usize)> = Vec::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let (px, py) = (x as i32 * T, y as i32 * T);
                self.map[y][x] = match ch {
                    '#' => ROCK,
                    '-' => LEDGE,
                    'b' => BRIDGE,
                    '~' => POOL,
                    'f' => NOZZLE,
                    't' => TURRET_T,
                    'P' => PILL_T,
                    _ => EMPTY,
                };
                // 兵写在站的那格上面一格：脚踩在下一格的顶上。
                let stand = py + T;
                match ch {
                    '@' => self.start = (px + 3, stand - HH),
                    'g' => self.spawns.push((Kind::Rifle, px + 3, stand - 22, ' ')),
                    'h' => self.spawns.push((Kind::Hider, px + 3, stand - 22, ' ')),
                    'n' => self.spawns.push((Kind::Lobber, px + 2, stand - 14, ' ')),
                    't' => self.spawns.push((Kind::Turret, px, py, ' ')),
                    'P' => {
                        drops.push((px, py, self.spawns.len()));
                        self.spawns.push((Kind::Pillbox, px, py, ' '));
                    }
                    'C' => {
                        drops.push((px, py, self.spawns.len()));
                        self.spawns.push((Kind::Pod, px, py, ' '));
                    }
                    'k' => self.spawns.push((Kind::RockSpot, px, py, ' ')),
                    'T' => self.spawns.push((Kind::Tank, px - 16, stand - 22, ' ')),
                    'X' => self.boss_at = (px, py),
                    _ => {}
                }
            }
        }
        // 勋章按出场的先后给：横着的关从左到右，竖着的关从下往上。
        if self.vertical {
            drops.sort_by_key(|d| -d.1);
        } else {
            drops.sort_by_key(|d| d.0);
        }
        let letters: Vec<char> = l.drops.chars().collect();
        for (n, d) in drops.iter().enumerate() {
            self.spawns[d.2].3 = letters[n % letters.len()];
        }
        for (y, row) in self.map.iter().enumerate() {
            let mut x = 0;
            while x < self.width {
                if row[x] == BRIDGE {
                    let from = x;
                    while x < self.width && row[x] == BRIDGE {
                        x += 1;
                    }
                    self.bridges.push(Bridge { row: y, from, to: x - 1, next: from, timer: 0, lit: false });
                } else {
                    x += 1;
                }
            }
        }
    }

    pub fn tile(&self, tx: i32, ty: i32) -> u8 {
        if tx < 0 || tx >= self.width as i32 {
            return ROCK;
        }
        if ty < 0 || ty >= self.height as i32 {
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

    /// 从 `old` 掉到 `new`（脚底的像素）的路上踩到的顶：回顶的像素、那一格。
    /// `through` 时不管台子；`water` 时水面也接得住（主角站在水里只露上半身）。
    fn landing(&self, x: i32, w: i32, old: i32, new: i32, through: bool, water: bool) -> Option<(i32, u8)> {
        let mut best: Option<(i32, u8)> = None;
        for ty in old.div_euclid(T)..=new.div_euclid(T) + 1 {
            for tx in x.div_euclid(T)..=(x + w - 1).div_euclid(T) {
                let t = self.tile(tx, ty);
                let top = ty * T + if t == POOL { 10 } else { 0 };
                let stands = solid(t) || (floor(t) && !through) || (t == POOL && water);
                let above_free = !solid(self.tile(tx, ty - 1)) || t == POOL;
                if stands && above_free && old <= top && new >= top && best.is_none_or(|b| top < b.0) {
                    best = Some((top, t));
                }
            }
        }
        best
    }

    fn max_cam(&self) -> (i32, i32) {
        ((self.width as i32 * T - W as i32).max(0), (self.height as i32 * T - H as i32).max(0))
    }

    // ---------- 开局 ----------

    fn new_game(&mut self) {
        self.score = 0;
        self.next_life = 20000;
        self.lives = LIVES;
        self.continues = CONTINUES;
        self.hero = Hero::new(0, 0);
        self.begin_intro();
    }

    fn begin_intro(&mut self) {
        self.mode = Mode::Intro;
        self.timer = 0;
        self.music.play(tune::intro());
    }

    /// 从头开始这一关。手里的枪留着。
    pub fn start_level(&mut self) {
        let (weapon, rapid) = (self.hero.weapon, self.hero.rapid);
        self.load(self.level);
        self.hero = Hero::new(self.start.0, self.start.1);
        self.hero.weapon = weapon;
        self.hero.rapid = rapid;
        self.hero.safe = 60;
        self.mobs.clear();
        self.shots.clear();
        self.eshots.clear();
        self.booms.clear();
        let (mx, my) = self.max_cam();
        self.cam = if self.vertical { (0, my) } else { ((self.start.0 - 100).clamp(0, mx), 0) };
        self.boss_on = false;
        self.boss_dead = false;
        self.boss_fall = 0;
        self.paused = false;
        self.runner_timer = 60;
        self.seed = 0x1357_9bdf ^ (self.level as u32 * 7919);
        self.mode = Mode::Play;
        self.timer = 0;
        self.music.play(tune::level(LEVELS[self.level].theme));
    }

    // ---------- 一帧 ----------

    pub fn step(&mut self) {
        self.tick += 1;
        self.timer += 1;
        self.music.tick();
        self.run_jingle();
        match self.mode {
            Mode::Title => self.title_step(),
            Mode::Intro => {
                if self.timer > 150 || (self.timer > 30 && self.keys.pressed(key::START)) {
                    self.start_level();
                }
            }
            Mode::Play => self.play(),
            Mode::Clear => self.clear_step(),
            Mode::Over => self.over_step(),
            Mode::Ending => {
                if self.timer > 120 && (self.keys.pressed(key::START) || self.keys.pressed(key::A)) || self.timer > 900 {
                    self.back_to_title();
                }
            }
        }
        self.draw();
    }

    fn back_to_title(&mut self) {
        self.mode = Mode::Title;
        self.timer = 0;
        self.level = self.unlocked;
        self.load(self.level);
        self.music.play(tune::title());
    }

    fn title_step(&mut self) {
        if self.keys.pressed(key::LEFT) || self.keys.pressed(key::RIGHT) {
            self.who = 1 - self.who;
            self.sfx(Channel::Square2, 660, 30, 30);
        }
        if self.keys.pressed(key::DOWN) && self.level < self.unlocked {
            self.level += 1;
            self.sfx(Channel::Square2, 520, 30, 30);
        }
        if self.keys.pressed(key::UP) && self.level > 0 {
            self.level -= 1;
            self.sfx(Channel::Square2, 520, 30, 30);
        }
        if self.keys.pressed(key::START) || self.keys.pressed(key::A) {
            self.sfx(Channel::Square2, 880, 60, 40);
            self.new_game();
        }
    }

    fn over_step(&mut self) {
        if self.continues == 0 {
            if self.timer > 120 && (self.keys.pressed(key::START) || self.keys.pressed(key::A)) || self.timer > 400 {
                self.back_to_title();
            }
            return;
        }
        if self.keys.pressed(key::UP) || self.keys.pressed(key::DOWN) {
            self.menu = 1 - self.menu;
            self.sfx(Channel::Square2, 660, 30, 30);
        }
        if self.timer > 30 && (self.keys.pressed(key::START) || self.keys.pressed(key::A)) {
            if self.menu == 0 {
                self.continues -= 1;
                self.score = 0;
                self.next_life = 20000;
                self.lives = LIVES;
                self.hero = Hero::new(0, 0);
                self.begin_intro();
            } else {
                self.back_to_title();
            }
        }
    }

    fn clear_step(&mut self) {
        if self.timer == 1 {
            self.music.play(tune::clear());
        }
        if self.timer > 240 {
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

    fn add_score(&mut self, n: u32) {
        self.score += n;
        if self.score > self.hi {
            self.hi = self.score;
        }
        while self.score >= self.next_life {
            self.lives += 1;
            self.next_life += if self.next_life < 60000 { 40000 } else { 60000 };
            self.arpeggio(Channel::Square2, &[784, 988, 1175, 1568], 4, 40);
        }
    }

    fn play(&mut self) {
        if self.keys.pressed(key::START) {
            self.paused = !self.paused;
            self.sfx(Channel::Square2, 440, 60, 30);
        }
        if self.paused {
            return;
        }
        self.shake = self.shake.saturating_sub(1);
        self.move_hero();
        self.update_cam();
        self.spawn();
        self.runners();
        self.move_mobs();
        self.move_shots();
        self.move_eshots();
        self.hit_mobs();
        self.bridges_step();
        self.booms.retain_mut(|b| {
            b.t += 1;
            b.t < 20
        });
        self.hurt_hero();
        self.boss_step();
    }

    // ---------- 主角 ----------

    fn move_hero(&mut self) {
        let k = &self.keys;
        let (left, right, up, down) = (k.held(key::LEFT), k.held(key::RIGHT), k.held(key::UP), k.held(key::DOWN));
        let (jump, fire_press, fire_held) = (k.pressed(key::A), k.pressed(key::B), k.held(key::B));
        let mut h = self.hero.clone();
        if h.dead > 0 {
            self.dead_step(h);
            return;
        }
        h.safe = h.safe.saturating_sub(1);
        h.barrier = h.barrier.saturating_sub(1);
        h.through = h.through.saturating_sub(1);
        h.cool = h.cool.saturating_sub(1);
        let lr = right as i32 - left as i32;
        if lr != 0 {
            h.face = lr;
        }
        h.diving = h.in_water && down && h.ground;
        h.prone = h.ground && !h.in_water && down && lr == 0;
        h.vx = if h.prone || h.diving { 0 } else { lr * WALK };
        if h.ground && lr != 0 && !h.prone {
            h.walk += 1;
        } else if h.ground {
            h.walk = 0;
        }
        // 往哪打。
        h.aim_down = false;
        h.aim = if up {
            if lr != 0 { Aim::DiagUp } else { Aim::Up }
        } else if down {
            if !h.ground {
                if lr != 0 {
                    Aim::DiagDown
                } else {
                    h.aim_down = true;
                    Aim::Fwd
                }
            } else if lr != 0 && !h.in_water {
                Aim::DiagDown
            } else {
                Aim::Fwd
            }
        } else {
            Aim::Fwd
        };
        // 跳：在台子上按下加 A 跳下去，别的时候跳起来翻跟头。
        if jump && h.ground && !h.diving {
            let (x, y) = h.px();
            let under = self.tile((x + HW / 2).div_euclid(T), (y + HH).div_euclid(T));
            if down && floor(under) && !solid(self.tile((x + HW / 2).div_euclid(T), (y + HH).div_euclid(T) + 1)) {
                h.through = 10;
                h.ground = false;
                h.prone = false;
            } else {
                h.vy = -JUMP_V;
                h.ground = false;
                h.spin = true;
                h.prone = false;
            }
        }
        // 左右走，碰墙停。不出画面。
        let (_, y) = h.px();
        let nx = px(h.x + h.vx);
        let (cx, cy) = self.cam;
        let min_x = if self.vertical { T * 2 - 1 } else { cx };
        let max_x = if self.vertical { self.width as i32 * T - T * 2 - HW + 1 } else { cx + W as i32 - HW };
        let gate = self.gate_rect();
        let blocked = self.hits(nx, y, HW, HH) || gate.is_some_and(|g| overlap((nx, y, HW, HH), g));
        if !blocked && nx >= min_x && nx <= max_x {
            h.x += h.vx;
        }
        // 上下：重力，踩台子、水面。
        h.vy = (h.vy + GRAV).min(MAX_FALL);
        let (x, _) = h.px();
        let old = y + HH;
        let new = px(h.y + h.vy) + HH;
        if h.vy >= 0 {
            match self.landing(x, HW, old, new, h.through > 0, true) {
                Some((top, t)) => {
                    h.y = (top - HH) * FX;
                    h.vy = 0;
                    h.ground = true;
                    h.spin = false;
                    h.in_water = t == POOL;
                }
                None => {
                    h.y += h.vy;
                    h.ground = false;
                    h.in_water = false;
                }
            }
        } else if self.hits(x, px(h.y + h.vy), HW, 4) {
            h.vy = 0;
        } else {
            h.y += h.vy;
            h.ground = false;
            h.in_water = false;
        }
        if h.ground {
            h.spin = false;
        }
        let _ = cy;
        self.hero = h;
        // 掉出画面就没了。
        let (_, hy) = self.hero.px();
        let bottom = if self.vertical { self.cam.1 + H as i32 } else { self.height as i32 * T };
        if hy > bottom + 8 {
            self.kill_hero();
            return;
        }
        // 开枪。
        if !self.hero.diving {
            let (rate, max) = self.hero.weapon.rate();
            let rate = if self.hero.rapid { rate * 3 / 4 } else { rate };
            let mine = self.shots.len();
            let want = fire_press || (fire_held && self.hero.cool == 0);
            if want && (mine < max || self.hero.weapon == Weapon::Laser) {
                self.fire();
                self.hero.cool = rate;
            }
        }
    }

    /// 枪口在哪（像素）、往哪个方向。
    pub fn muzzle(&self) -> (i32, i32, i32) {
        let h = &self.hero;
        let (x, y) = h.px();
        let f = h.face;
        let side = |a: i32| if f > 0 { a } else { 32 - a };
        if h.spin {
            let a = if h.aim_down {
                16
            } else {
                match h.aim {
                    Aim::Fwd => side(0),
                    Aim::Up => 48,
                    Aim::DiagUp => side(56),
                    Aim::DiagDown => side(8),
                }
            };
            return (x + HW / 2, y + 11, a);
        }
        if h.prone {
            return (if f > 0 { x + 24 } else { x - 14 }, y + 18, side(0));
        }
        // 精灵 24 宽：朝右时身子在第 3 到 12 列，朝左翻过来。
        let sx = |c: i32| if f > 0 { x - 3 + c } else { x - 11 + (23 - c) };
        let (c, r, a) = match h.aim {
            Aim::Fwd => (23, 9, side(0)),
            Aim::Up => (12, -1, 48),
            Aim::DiagUp => (22, 0, side(56)),
            Aim::DiagDown => (18, 22, side(8)),
        };
        (sx(c), y - 2 + r, a)
    }

    fn fire(&mut self) {
        let (mx, my, a) = self.muzzle();
        let w = self.hero.weapon;
        let fast = if self.hero.rapid { 3 } else { 2 };
        let mk = |a: i32, v: i32| {
            let (vx, vy) = dir(a, v);
            Shot { x: mx * FX, y: my * FX, vx, vy, kind: w, base: (mx * FX, my * FX), phase: 0, hit: Vec::new(), gone: false }
        };
        match w {
            Weapon::Normal | Weapon::Machine => {
                self.shots.push(mk(a, 512 * fast));
                self.sfx(Channel::Noise, if w == Weapon::Machine { 2400 } else { 2000 }, 20, 28);
            }
            Weapon::Spread => {
                for d in [-4, -2, 0, 2, 4] {
                    self.shots.push(mk(a + d, 448 * fast));
                }
                self.sfx(Channel::Noise, 1400, 40, 34);
            }
            Weapon::Laser => {
                self.shots.retain(|s| s.kind != Weapon::Laser);
                self.shots.push(mk(a, 768 * fast));
                self.arpeggio(Channel::Square2, &[1800, 1400, 1000], 1, 28);
            }
            Weapon::Fire => {
                self.shots.push(mk(a, 320 * fast));
                self.sfx(Channel::Square2, 300, 40, 30);
            }
        }
    }

    fn dead_step(&mut self, mut h: Hero) {
        h.dead -= 1;
        // 往后飞一下，掉到地上躺着。
        if h.dead > DEAD_TIME - 40 {
            h.vy = (h.vy + GRAV).min(MAX_FALL);
            let (x, y) = h.px();
            if !self.hits(px(h.x + h.vx), y, HW, HH) {
                h.x += h.vx;
            }
            match self.landing(x, HW, y + HH, px(h.y + h.vy) + HH, false, true) {
                Some((top, _)) if h.vy > 0 => {
                    h.y = (top - HH) * FX;
                    h.vy = 0;
                    h.vx = 0;
                }
                _ => h.y += h.vy,
            }
        }
        self.hero = h;
        if self.hero.dead == 0 {
            if self.lives == 0 {
                self.mode = Mode::Over;
                self.timer = 0;
                self.menu = 0;
                self.music.play(tune::game_over());
                self.save();
                return;
            }
            self.lives -= 1;
            self.respawn();
        }
    }

    /// 倒下以后从画面上面掉下来，接着打。
    fn respawn(&mut self) {
        let (cx, cy) = self.cam;
        // 挑画面左边一列下面接得住的地方。
        let catches = |tx: i32| (0..self.height as i32).any(|ty| {
            let t = self.tile(tx, ty);
            solid(t) || floor(t) || t == POOL
        });
        let x = if self.vertical {
            112
        } else {
            (0..8).map(|c| (cx + 24) / T + c).find(|&tx| catches(tx)).map_or(cx + 40, |tx| tx * T + 3)
        };
        let mut h = Hero::new(x, cy - 8);
        h.safe = SAFE_TIME;
        h.face = 1;
        h.spin = true;
        self.hero = h;
    }

    fn kill_hero(&mut self) {
        if self.hero.dead > 0 {
            return;
        }
        let f = self.hero.face;
        let h = &mut self.hero;
        h.dead = DEAD_TIME;
        h.vx = -f * 160;
        h.vy = -800;
        h.weapon = Weapon::Normal;
        h.rapid = false;
        h.barrier = 0;
        h.prone = false;
        h.spin = false;
        h.diving = false;
        self.arpeggio(Channel::Square2, &[440, 415, 392, 370, 349, 294], 5, 40);
    }

    /// 碰到敌人、子弹、火、手雷炸开就倒。
    fn hurt_hero(&mut self) {
        let h = &self.hero;
        if h.dead > 0 || h.safe > 0 || h.diving {
            return;
        }
        let b = h.hurt_box();
        let shield = h.barrier > 0;
        let mut die = false;
        for s in &self.eshots {
            let r = if s.kind == EKind::Ball { 4 } else { 2 };
            if !s.gone && overlap(b, (px(s.x) - r, px(s.y) - r, r * 2, r * 2)) {
                die = true;
            }
        }
        for bm in &self.booms {
            if bm.hurts && bm.t < 12 && overlap(b, (bm.x - bm.size, bm.y - bm.size, bm.size * 2, bm.size * 2)) {
                die = true;
            }
        }
        let mut crushed: Vec<usize> = Vec::new();
        for (i, m) in self.mobs.iter().enumerate() {
            if !m.live() || !m.kind.deadly() {
                continue;
            }
            if overlap(b, m.rect()) {
                if shield && matches!(m.kind, Kind::Runner | Kind::Rifle | Kind::Rock) {
                    crushed.push(i);
                } else {
                    die = true;
                }
            }
            if m.kind == Kind::Hand {
                for p in self.arm_balls(m) {
                    if overlap(b, (p.0 - 4, p.1 - 4, 8, 8)) {
                        die = true;
                    }
                }
            }
            if m.kind == Kind::Robot && m.state == 2 && (12..36).contains(&m.timer) {
                let (rx, ry) = m.px();
                let y = ry + 34;
                let beam = if m.face < 0 { (self.cam.0 - 8, y, rx - self.cam.0 + 8, 6) } else { (rx + m.w, y, 400, 6) };
                if overlap(b, beam) {
                    die = true;
                }
            }
        }
        for (x, y) in self.flames() {
            if overlap(b, (x + 3, y, 10, 40)) {
                die = true;
            }
        }
        for i in crushed {
            self.kill_mob(i);
        }
        if die && !shield {
            self.kill_hero();
        }
        // 护盾时子弹碰上就没了。
        if shield {
            self.eshots.retain(|s| !overlap(b, (px(s.x) - 3, px(s.y) - 3, 6, 6)));
        }
    }

    // ---------- 画面 ----------

    fn update_cam(&mut self) {
        let (hx, hy) = self.hero.px();
        let (mx, my) = self.max_cam();
        if self.vertical {
            let want = (hy - 64).clamp(0, my);
            if want < self.cam.1 && !self.boss_on {
                self.cam.1 = want;
            }
        } else {
            let want = (hx - 104).clamp(0, mx);
            if want > self.cam.0 && !self.boss_on {
                self.cam.0 = want;
            }
        }
        if !self.boss_on && ((self.vertical && self.cam.1 == 0) || (!self.vertical && self.cam.0 == mx)) {
            self.boss_on = true;
            self.spawn_boss();
            self.music.play(tune::boss());
        }
    }

    /// 画面到了，把地图上写的敌人放出来。
    fn spawn(&mut self) {
        let (cx, cy) = self.cam;
        let mut i = 0;
        while i < self.spawns.len() {
            let (kind, x, y, letter) = self.spawns[i];
            let seen = if self.vertical { y + 32 > cy } else { x < cx + W as i32 + 8 };
            if !seen {
                i += 1;
                continue;
            }
            self.spawns.swap_remove(i);
            let id = self.id();
            let mut m = Mob::new(id, kind, x, y);
            m.letter = letter;
            match kind {
                Kind::Pod => {
                    if self.vertical {
                        m.x = -16 * FX;
                    } else {
                        m.x = (cx - 16) * FX;
                    }
                    m.vx = 384;
                    m.face = 1;
                }
                Kind::Tank => {
                    m.face = -1;
                }
                Kind::Pillbox => m.timer = self.rand(60),
                _ => {}
            }
            self.mobs.push(m);
        }
    }

    /// 兵从画面边上一个一个跑出来。
    fn runners(&mut self) {
        if self.boss_on || self.vertical || self.hero.dead > 0 {
            return;
        }
        let edge = (self.cam.0 + W as i32) / T;
        let on = LEVELS[self.level].runners.iter().any(|&(a, b)| edge >= a && edge <= b);
        if !on {
            return;
        }
        if self.runner_timer > 0 {
            self.runner_timer -= 1;
            return;
        }
        self.runner_timer = 50 + self.rand(60);
        if self.mobs.iter().filter(|m| m.kind == Kind::Runner).count() >= 4 {
            return;
        }
        let from_left = self.rand(5) == 0 && self.cam.0 > 0;
        let x = if from_left { self.cam.0 - 12 } else { self.cam.0 + W as i32 + 2 };
        let col = (x + 5).div_euclid(T);
        // 那一列能站的地方，挑一个。
        let mut rows = Vec::new();
        for ty in 1..self.height as i32 {
            let t = self.tile(col, ty);
            if (solid(t) || floor(t)) && !solid(self.tile(col, ty - 1)) && !solid(self.tile(col, ty - 2)) && self.tile(col, ty - 1) != POOL {
                rows.push(ty);
            }
        }
        if rows.is_empty() {
            return;
        }
        let ty = rows[self.rand(rows.len() as u32) as usize];
        let id = self.id();
        let mut m = Mob::new(id, Kind::Runner, x, ty * T - 22);
        m.face = if from_left { 1 } else { -1 };
        m.ground = true;
        self.mobs.push(m);
    }

    /// 一个会掉的东西走一帧：碰墙回 true。
    fn fall(&self, m: &mut Mob) -> bool {
        let (_, y) = m.px();
        let mut wall = false;
        if m.vx != 0 {
            let nx = px(m.x + m.vx);
            if self.hits(nx, y, m.w, m.h - 2) {
                wall = true;
            } else {
                m.x += m.vx;
            }
        }
        m.through = m.through.saturating_sub(1);
        m.vy = (m.vy + GRAV).min(MAX_FALL);
        let (x, _) = m.px();
        if m.vy >= 0 {
            match self.landing(x, m.w, y + m.h, px(m.y + m.vy) + m.h, m.through > 0, false) {
                Some((top, _)) => {
                    m.y = (top - m.h) * FX;
                    m.vy = 0;
                    m.ground = true;
                }
                None => {
                    m.y += m.vy;
                    m.ground = false;
                }
            }
        } else {
            m.y += m.vy;
            m.ground = false;
        }
        wall
    }

    fn move_mobs(&mut self) {
        let (hx, hy) = self.hero.px();
        let (hcx, hcy) = (hx + HW / 2, hy + 8);
        let (cx, cy) = self.cam;
        let mut out: Vec<EShot> = Vec::new();
        let mut new_mobs: Vec<Mob> = Vec::new();
        let mut sounds: Vec<(Channel, u32, u32, u32)> = Vec::new();
        let mut mobs = std::mem::take(&mut self.mobs);
        let hero_alive = self.hero.dead == 0;
        for m in mobs.iter_mut() {
            m.timer += 1;
            m.flash = m.flash.saturating_sub(1);
            if m.dying > 0 {
                m.dying -= 1;
                if matches!(m.kind, Kind::Runner | Kind::Rifle | Kind::Hider | Kind::Lobber) {
                    m.x += m.vx;
                    m.y += m.vy;
                    m.vy += GRAV / 2;
                }
                continue;
            }
            let (x, y) = m.px();
            let on_screen = x + m.w > cx && x < cx + W as i32 && y + m.h > cy && y < cy + H as i32;
            let (mcx, mcy) = (x + m.w / 2, y + m.h / 2);
            match m.kind {
                Kind::Runner => {
                    m.vx = m.face * 320;
                    let was_ground = m.ground;
                    if self.fall(m) {
                        m.face = -m.face;
                    }
                    // 走到台边：有时候跳一下，有时候直接掉下去。
                    if was_ground && !m.ground && m.vy > 0 && m.timer % 3 == 0 {
                        m.vy = -768;
                    }
                    let (_, ny) = m.px();
                    if self.tile((x + 5).div_euclid(T), (ny + m.h - 4).div_euclid(T)) == POOL {
                        m.dying = 1;
                        m.hp = 0;
                    }
                }
                Kind::Rifle | Kind::Hider => {
                    m.face = if hcx < mcx { -1 } else { 1 };
                    if m.kind == Kind::Hider {
                        // 躲 80 帧，站起来 60 帧。
                        let t = (m.timer + m.home.0 as u32) % 140;
                        m.state = (t >= 80) as u32;
                        if t != 100 {
                            continue;
                        }
                    } else if !(on_screen && m.timer % 100 == 50) {
                        continue;
                    }
                    if !on_screen || !hero_alive {
                        continue;
                    }
                    let a = angle_to(hcx - mcx, hcy - (y + 9), 4);
                    let (vx, vy) = dir(a, 384);
                    out.push(EShot { x: (mcx + m.face * 10) * FX, y: (y + 9) * FX, vx, vy, kind: EKind::Bullet, bounces: 0, gone: false });
                    m.aim = a;
                }
                Kind::Lobber => {
                    m.face = if hcx < mcx { -1 } else { 1 };
                    m.state = (m.timer % 120 > 100) as u32;
                    if on_screen && hero_alive && m.timer % 120 == 110 {
                        let dx = (hcx - mcx).clamp(-140, 140);
                        out.push(EShot { x: mcx * FX, y: y * FX, vx: dx * FX / 48, vy: -900, kind: EKind::Shell, bounces: 0, gone: false });
                    }
                }
                Kind::Turret => {
                    if !on_screen || !hero_alive {
                        continue;
                    }
                    let want = angle_to(hcx - mcx, hcy - mcy, 4);
                    if m.timer % 10 == 0 && want != m.aim {
                        let d = (want - m.aim).rem_euclid(64);
                        m.aim = (m.aim + if d < 32 { 4 } else { -4 }).rem_euclid(64);
                    }
                    if m.timer % 70 == 0 && (want - m.aim).rem_euclid(64) <= 4 {
                        let (vx, vy) = dir(m.aim, 384);
                        let (ox, oy) = dir(m.aim, 10 * FX);
                        out.push(EShot { x: mcx * FX + ox, y: mcy * FX + oy, vx, vy, kind: EKind::Bullet, bounces: 0, gone: false });
                    }
                }
                Kind::Pillbox => {
                    // 关着 110 帧，开着 70 帧。
                    m.state = (m.timer % 180 >= 110) as u32;
                }
                Kind::Pod => {
                    m.x += m.vx;
                    let t = m.timer as i32;
                    m.y = (m.home.1 + sin64(t) * 14 / 256) * FX;
                    if m.timer > 400 {
                        m.dying = 1;
                        m.hp = 0;
                    }
                }
                Kind::Medal => {
                    if !m.ground {
                        self.fall(m);
                        let (_, my) = m.px();
                        if self.tile((x + 7).div_euclid(T), (my + m.h).div_euclid(T)) == POOL {
                            m.dying = 1;
                            m.hp = 0;
                        }
                    } else {
                        m.vx = 0;
                    }
                    if m.timer > 900 {
                        m.dying = 1;
                        m.hp = 0;
                    }
                }
                Kind::RockSpot => {
                    if on_screen && m.timer % 150 == 0 && hy > y {
                        new_mobs.push(Mob::new(0, Kind::Rock, x + 1, y));
                    }
                }
                Kind::Rock => {
                    let was = m.ground;
                    self.fall(m);
                    if m.ground && !was {
                        // 落地弹一下，接着往下掉。
                        m.vy = -640;
                        m.vx = if hcx < mcx { -160 } else { 160 };
                        m.through = 14;
                        m.ground = false;
                        sounds.push((Channel::Noise, 300, 40, 30));
                    }
                }
                Kind::Tank => {
                    if !on_screen {
                        continue;
                    }
                    m.vx = if x > cx + 130 { -128 } else { 0 };
                    self.fall(m);
                    if hero_alive && m.timer % 100 == 60 {
                        let a = angle_to(hcx - x, hcy - (y + 6), 2);
                        let (vx, vy) = dir(a, 512);
                        out.push(EShot { x: x * FX, y: (y + 6) * FX, vx, vy, kind: EKind::Ball, bounces: 0, gone: false });
                        sounds.push((Channel::Noise, 400, 80, 40));
                    }
                }
                Kind::GateCannon => {
                    if m.timer % 90 == (m.home.1 as u32 % 90) && hero_alive {
                        let vx = -(256 + (m.timer as i32 * 37 % 3) * 200);
                        out.push(EShot { x: x * FX, y: (y + 4) * FX, vx, vy: -500, kind: EKind::Shell, bounces: 0, gone: false });
                        sounds.push((Channel::Noise, 500, 60, 36));
                    }
                }
                Kind::GateCore => {}
                Kind::Totem => {
                    if hero_alive && m.timer % 150 == 100 {
                        let a = angle_to(hcx - mcx, hcy - (y + 26), 1);
                        for d in [-5, 0, 5] {
                            let (vx, vy) = dir(a + d, 320);
                            out.push(EShot { x: mcx * FX, y: (y + 26) * FX, vx, vy, kind: EKind::Ball, bounces: 0, gone: false });
                        }
                        sounds.push((Channel::Square2, 180, 120, 40));
                    }
                }
                Kind::Hand => {
                    // 手跟着胳膊甩：肩膀在 home，胳膊长 52 像素。
                    let a = self.arm_angle(m);
                    let (dx, dy) = dir(a, 52 * FX);
                    m.x = m.home.0 * FX + dx - 6 * FX;
                    m.y = m.home.1 * FX + dy - 6 * FX;
                    if hero_alive && m.timer % 80 == (m.state * 40) {
                        let (x, y) = m.px();
                        let a = angle_to(hcx - x - 6, hcy - y - 6, 2);
                        let (vx, vy) = dir(a, 384);
                        out.push(EShot { x: (x + 6) * FX, y: (y + 6) * FX, vx, vy, kind: EKind::Bullet, bounces: 0, gone: false });
                    }
                }
                Kind::Crawler => self.crawler(m, hcx, &mut out, &mut sounds),
                Kind::Robot => self.robot(m, hcx, &mut out, &mut sounds),
            }
        }
        // 出了画面的去掉（大怪不去）。
        let (cx, cy) = self.cam;
        mobs.retain(|m| {
            let (x, y) = m.px();
            let gone = if self.vertical { y > cy + H as i32 + 32 } else { x + m.w < cx - 32 || (m.kind == Kind::Runner && x > cx + W as i32 + 40) || y > H as i32 + 32 };
            !(gone && !m.kind.boss()) && !(m.hp <= 0 && m.dying == 0)
        });
        for mut m in new_mobs {
            m.id = self.id();
            mobs.push(m);
        }
        self.mobs = mobs;
        self.eshots.extend(out);
        for (ch, hz, ms, vol) in sounds {
            self.sfx(ch, hz, ms, vol);
        }
        // 碰到勋章就拿。
        if self.hero.dead == 0 {
            let b = self.hero.hurt_box();
            let got: Vec<usize> = self.mobs.iter().enumerate().filter(|(_, m)| m.kind == Kind::Medal && m.live() && overlap(b, m.rect())).map(|(i, _)| i).collect();
            for i in got.into_iter().rev() {
                let l = self.mobs[i].letter;
                self.mobs.remove(i);
                self.take(l);
            }
        }
    }

    /// 拿到一个勋章。
    fn take(&mut self, l: char) {
        let h = &mut self.hero;
        match l {
            'S' => h.weapon = Weapon::Spread,
            'M' => h.weapon = Weapon::Machine,
            'L' => h.weapon = Weapon::Laser,
            'F' => h.weapon = Weapon::Fire,
            'R' => h.rapid = true,
            'B' => h.barrier = BARRIER_TIME,
            _ => {}
        }
        self.add_score(1000);
        self.arpeggio(Channel::Square2, &[523, 659, 784, 1047], 3, 40);
    }

    /// 胳膊甩到哪个方向：左胳膊朝左下，右胳膊朝右下，来回甩。
    fn arm_angle(&self, m: &Mob) -> i32 {
        let t = (m.timer as i32 + m.state as i32 * 45) % 180;
        let swing = if t < 90 { t - 45 } else { 135 - t } / 4;
        if m.state == 0 {
            24 + swing
        } else {
            8 - swing
        }
    }

    /// 胳膊上的四个铁球（像素中心）。
    fn arm_balls(&self, m: &Mob) -> Vec<(i32, i32)> {
        let a = self.arm_angle(m);
        (1..5)
            .map(|k| {
                let (dx, dy) = dir(a, k * 10 * FX);
                (m.home.0 + px(dx), m.home.1 + px(dy))
            })
            .collect()
    }

    /// 装甲车：来回开，停下往天上打三发，过一阵冲过来。
    fn crawler(&self, m: &mut Mob, hcx: i32, out: &mut Vec<EShot>, sounds: &mut Vec<(Channel, u32, u32, u32)>) {
        let left = self.cam.0 + 12;
        let right = self.cam.0 + W as i32 - m.w - 4;
        let (x, y) = m.px();
        match m.state {
            0 => {
                if m.vx == 0 {
                    m.vx = -192;
                }
                if x <= left + 60 {
                    m.vx = 192;
                }
                if x >= right {
                    m.vx = -192;
                }
                m.face = if m.vx < 0 { -1 } else { 1 };
                if m.timer > 150 {
                    m.timer = 0;
                    m.state = 1;
                    m.vx = 0;
                }
            }
            1 => {
                if m.timer == 12 || m.timer == 30 {
                    for d in [-1, 0, 1] {
                        let tx = hcx + d * 28 - (x + m.w / 2);
                        out.push(EShot { x: (x + m.w / 2) * FX, y: y * FX, vx: tx * FX / 70, vy: -1100, kind: EKind::Shell, bounces: 0, gone: false });
                    }
                    sounds.push((Channel::Noise, 450, 80, 40));
                }
                if m.timer > 50 {
                    // 每三轮冲一次。
                    m.timer = 0;
                    m.home.0 += 1;
                    m.state = if m.home.0 % 3 == 0 { 2 } else { 0 };
                }
            }
            2 => {
                // 冲：往主角那边一直开到墙边，再慢慢退回去。
                if m.timer < 30 {
                    m.vx = 0;
                    if m.timer.is_multiple_of(6) {
                        sounds.push((Channel::Noise, 200, 60, 30));
                    }
                } else if m.timer == 30 {
                    m.face = if hcx < x + m.w / 2 { -1 } else { 1 };
                    m.vx = m.face * 576;
                } else if (m.vx < 0 && x <= left) || (m.vx > 0 && x >= right) {
                    m.vx = 0;
                    m.state = 0;
                    m.timer = 0;
                }
            }
            _ => m.state = 0,
        }
        m.x = (m.x + m.vx).clamp(left * FX, right * FX);
        m.y = (self.boss_at.1 + T - m.h) * FX;
    }

    /// 巨型机器人：走、胸口放激光、跳到另一边扔炸弹。
    fn robot(&self, m: &mut Mob, hcx: i32, out: &mut Vec<EShot>, sounds: &mut Vec<(Channel, u32, u32, u32)>) {
        let left = self.cam.0 + 8;
        let right = self.cam.0 + W as i32 - m.w - 8;
        let floor_y = (self.boss_at.1 + T) - m.h;
        let (x, y) = m.px();
        let angry = m.hp < 45;
        match m.state {
            0 => {
                m.face = if hcx < x + m.w / 2 { -1 } else { 1 };
                m.vx = m.face * if angry { 160 } else { 112 };
                if m.timer > if angry { 70 } else { 100 } {
                    m.timer = 0;
                    m.state = 2;
                    m.vx = 0;
                }
            }
            2 => {
                // 胸口的灯闪 12 帧，激光打 24 帧。
                if m.timer == 12 {
                    sounds.push((Channel::Square2, 120, 400, 40));
                }
                if m.timer > 44 {
                    m.timer = 0;
                    m.state = 3;
                    m.face = if x + m.w / 2 < self.cam.0 + W as i32 / 2 { 1 } else { -1 };
                    m.vx = m.face * 448;
                    m.vy = -1400;
                }
            }
            3 => {
                m.vy += GRAV;
                m.y += m.vy;
                if px(m.y) >= floor_y {
                    m.y = floor_y * FX;
                    m.vy = 0;
                    m.vx = 0;
                    m.state = 0;
                    m.timer = 0;
                    sounds.push((Channel::Noise, 150, 160, 50));
                    for d in [-1, 1] {
                        out.push(EShot { x: (x + m.w / 2) * FX, y: (y + 20) * FX, vx: d * 300, vy: -900, kind: EKind::Bomb, bounces: 0, gone: false });
                    }
                }
            }
            _ => m.state = 0,
        }
        m.x = (m.x + m.vx).clamp(left * FX, right * FX);
        if m.state != 3 {
            m.y = floor_y * FX;
        }
    }

    // ---------- 子弹 ----------

    fn move_shots(&mut self) {
        let (cx, cy) = self.cam;
        for s in self.shots.iter_mut() {
            if s.kind == Weapon::Fire {
                s.base.0 += s.vx;
                s.base.1 += s.vy;
                s.phase += 1;
                let (ox, oy) = dir(s.phase * 5, 9 * FX);
                s.x = s.base.0 + ox;
                s.y = s.base.1 + oy;
            } else {
                s.x += s.vx;
                s.y += s.vy;
            }
            let (x, y) = (px(s.x), px(s.y));
            if x < cx - 24 || x > cx + W as i32 + 24 || y < cy - 24 || y > cy + H as i32 + 24 {
                s.gone = true;
            }
        }
        self.shots.retain(|s| !s.gone);
    }

    fn move_eshots(&mut self) {
        let (cx, cy) = self.cam;
        let mut blasts: Vec<(i32, i32)> = Vec::new();
        let mut eshots = std::mem::take(&mut self.eshots);
        for s in eshots.iter_mut() {
            match s.kind {
                EKind::Bullet | EKind::Ball => {
                    s.x += s.vx;
                    s.y += s.vy;
                }
                EKind::Shell | EKind::Bomb => {
                    s.vy = (s.vy + GRAV / 2).min(MAX_FALL);
                    let (x, y) = (px(s.x), px(s.y));
                    let ny = px(s.y + s.vy);
                    s.x += s.vx;
                    match self.landing(x - 2, 4, y, ny, false, false) {
                        Some((top, _)) => {
                            if s.kind == EKind::Bomb && s.bounces < 3 {
                                s.bounces += 1;
                                s.y = top * FX;
                                s.vy = -700;
                            } else {
                                s.gone = true;
                                blasts.push((x, top - 6));
                            }
                        }
                        None => s.y += s.vy,
                    }
                    if self.tile(px(s.x).div_euclid(T), px(s.y).div_euclid(T)) == POOL {
                        s.gone = true;
                    }
                }
            }
            let (x, y) = (px(s.x), px(s.y));
            if x < cx - 32 || x > cx + W as i32 + 32 || y < cy - 64 || y > cy + H as i32 + 16 {
                s.gone = true;
            }
        }
        eshots.retain(|s| !s.gone);
        self.eshots = eshots;
        for (x, y) in blasts {
            self.booms.push(Boom { x, y, t: 0, size: 10, hurts: true });
            self.sfx(Channel::Noise, 350, 90, 40);
        }
    }

    /// 主角的子弹打中敌人。
    fn hit_mobs(&mut self) {
        let mut kills: Vec<usize> = Vec::new();
        let mut ting = false;
        let mut hurt = false;
        let mut shots = std::mem::take(&mut self.shots);
        for s in shots.iter_mut() {
            let (x, y) = (px(s.x), px(s.y));
            let r = match s.kind {
                Weapon::Spread => 3,
                Weapon::Fire => 4,
                Weapon::Laser => 3,
                _ => 2,
            };
            let sb = (x - r, y - r, r * 2, r * 2);
            for (i, m) in self.mobs.iter_mut().enumerate() {
                if !m.live() || m.hp <= 0 || s.gone {
                    continue;
                }
                if s.kind == Weapon::Laser && s.hit.contains(&m.id) {
                    continue;
                }
                if let Some(wk) = m.weak() {
                    if overlap(sb, wk) {
                        m.hp -= s.kind.damage();
                        m.flash = 4;
                        if s.kind == Weapon::Laser {
                            s.hit.push(m.id);
                        } else {
                            s.gone = true;
                        }
                        if m.hp <= 0 {
                            kills.push(i);
                        } else {
                            hurt = true;
                        }
                        continue;
                    }
                }
                if let Some(ar) = m.armor() {
                    if overlap(sb, ar) {
                        s.gone = true;
                        ting = true;
                    }
                }
            }
        }
        shots.retain(|s| !s.gone);
        self.shots = shots;
        if ting {
            self.sfx(Channel::Square2, 1600, 20, 24);
        }
        if hurt {
            self.sfx(Channel::Square2, 220, 30, 30);
        }
        kills.sort_unstable();
        kills.dedup();
        for i in kills.into_iter().rev() {
            self.kill_mob(i);
        }
    }

    fn kill_mob(&mut self, i: usize) {
        let m = &mut self.mobs[i];
        if m.dying > 0 {
            return;
        }
        let kind = m.kind;
        let (x, y) = m.px();
        let (w, h) = (m.w, m.h);
        let letter = m.letter;
        m.hp = 0;
        match kind {
            Kind::Runner | Kind::Rifle | Kind::Hider | Kind::Lobber => {
                m.dying = 24;
                m.vx = -m.face * 192;
                m.vy = -640;
                self.sfx(Channel::Noise, 900, 70, 36);
            }
            _ => {
                m.dying = 1;
            }
        }
        self.add_score(kind.points());
        match kind {
            Kind::Turret | Kind::Pillbox => {
                let (tx, ty) = (x.div_euclid(T) as usize, y.div_euclid(T) as usize);
                if ty < self.height && tx < self.width {
                    self.map[ty][tx] = if kind == Kind::Turret { ROCK } else { EMPTY };
                }
                self.booms.push(Boom { x: x + 8, y: y + 8, t: 0, size: 14, hurts: false });
                self.bang(false);
            }
            Kind::Pod | Kind::Rock | Kind::Tank | Kind::GateCannon | Kind::Hand => {
                self.booms.push(Boom { x: x + w / 2, y: y + h / 2, t: 0, size: 12 + w / 4, hurts: false });
                self.bang(false);
            }
            _ => {}
        }
        if matches!(kind, Kind::Pod | Kind::Pillbox) && letter != ' ' {
            let id = self.id();
            let mut md = Mob::new(id, Kind::Medal, x, y);
            md.letter = letter;
            md.vy = -900;
            md.vx = 128;
            md.ground = false;
            self.mobs.push(md);
        }
        if kind.boss() {
            self.boss_dead = true;
            self.boss_fall = 1;
            self.eshots.clear();
            self.bang(true);
            // 大怪的别的几块一起炸。
            for m in self.mobs.iter_mut() {
                if matches!(m.kind, Kind::GateCannon | Kind::Hand | Kind::Rifle) && m.live() {
                    m.hp = 0;
                    m.dying = 1;
                }
            }
        }
    }

    // ---------- 地形 ----------

    fn bridges_step(&mut self) {
        let (hx, hy) = self.hero.px();
        let feet_row = (hy + HH).div_euclid(T);
        let mut booms = Vec::new();
        for b in self.bridges.iter_mut() {
            let col = (hx + HW / 2).div_euclid(T);
            if !b.lit && feet_row == b.row as i32 && col >= b.from as i32 && col <= b.to as i32 && self.hero.ground {
                b.lit = true;
                b.timer = 8;
            }
            if b.lit && b.next <= b.to {
                if b.timer == 0 {
                    self.map[b.row][b.next] = EMPTY;
                    booms.push((b.next as i32 * T + 8, b.row as i32 * T + 4));
                    b.next += 1;
                    b.timer = 16;
                } else {
                    b.timer -= 1;
                }
            }
        }
        for (x, y) in booms {
            self.booms.push(Boom { x, y, t: 0, size: 12, hurts: false });
            self.bang(false);
        }
    }

    /// 正在喷的火（火柱左下角的像素）。
    pub fn flames(&self) -> Vec<(i32, i32)> {
        let mut v = Vec::new();
        if LEVELS[self.level].theme != Theme::Fort {
            return v;
        }
        let (cx, _) = self.cam;
        for tx in (cx / T - 1).max(0)..((cx + W as i32) / T + 2).min(self.width as i32) {
            for ty in 0..self.height as i32 {
                if self.tile(tx, ty) == NOZZLE && self.flame_on(tx) {
                    v.push((tx * T, ty * T - 40));
                }
            }
        }
        v
    }

    /// 喷火口 150 帧一轮，喷 60 帧。
    fn flame_on(&self, tx: i32) -> bool {
        (self.tick + tx as u32 * 37) % 150 < 60
    }

    // ---------- 大怪 ----------

    /// 铁门挡着主角的那块。
    fn gate_rect(&self) -> Option<(i32, i32, i32, i32)> {
        if LEVELS[self.level].theme != Theme::Jungle || !self.boss_on || self.boss_fall > 0 {
            return None;
        }
        let x0 = self.boss_at.0 - 8;
        Some((x0, 0, self.width as i32 * T - x0, self.boss_at.1 + T))
    }

    fn spawn_boss(&mut self) {
        let (bx, by) = self.boss_at;
        let theme = LEVELS[self.level].theme;
        let add = |g: &mut Game, k: Kind, x: i32, y: i32| -> usize {
            let id = g.id();
            g.mobs.push(Mob::new(id, k, x, y));
            g.mobs.len() - 1
        };
        match theme {
            Theme::Jungle => {
                // 铁门：上下两门炮，下面一个红芯，门顶上站一个兵。
                let x0 = bx - 8;
                let floor = by + T;
                let i = add(self, Kind::GateCannon, x0 - 12, floor - 70);
                self.mobs[i].home.1 = 0;
                let i = add(self, Kind::GateCannon, x0 - 12, floor - 44);
                self.mobs[i].home.1 = 45;
                add(self, Kind::GateCore, x0 + 6, floor - 26);
                let i = add(self, Kind::Rifle, x0 + 22, floor - 96 - 22);
                self.mobs[i].face = -1;
            }
            Theme::Falls => {
                let i = add(self, Kind::Totem, bx - 12, by + 8);
                self.mobs[i].face = 1;
                for side in 0..2 {
                    let i = add(self, Kind::Hand, 0, 0);
                    let m = &mut self.mobs[i];
                    m.state = side;
                    m.home = (if side == 0 { bx - 14 } else { bx + 30 }, by + 30);
                }
            }
            Theme::Snow => {
                let i = add(self, Kind::Crawler, bx - 8, by + T - 30);
                self.mobs[i].face = -1;
            }
            Theme::Fort => {
                let i = add(self, Kind::Robot, bx - 8, by + T - 56);
                self.mobs[i].face = -1;
            }
        }
    }

    fn boss_step(&mut self) {
        if self.boss_fall == 0 {
            return;
        }
        self.boss_fall += 1;
        let t = self.boss_fall;
        if t < 120 && t.is_multiple_of(8) {
            let (bx, by) = self.boss_at;
            let rx = self.rand(64) as i32 - 24;
            let ry = self.rand(80) as i32 - 50;
            self.booms.push(Boom { x: bx + rx, y: by + ry, t: 0, size: 16, hurts: false });
            self.shake = 6;
            if t.is_multiple_of(24) {
                self.bang(true);
            }
        }
        if t == 60 {
            self.music.stop();
        }
        if t > 150 && self.hero.dead == 0 {
            self.mode = Mode::Clear;
            self.timer = 0;
            self.add_score(5000);
            self.save();
        }
    }

    // ---------- 画 ----------

    fn set_theme(&mut self) {
        for (i, rgb) in LEVELS[self.level].theme.colors() {
            self.frame.palette[i as usize] = rgb;
        }
        // 土最暗的那格每关不一样。
        let lo = match LEVELS[self.level].theme {
            Theme::Jungle => [0x40, 0x2c, 0x20],
            Theme::Falls => [0x48, 0x40, 0x3c],
            Theme::Snow => [0x38, 0x40, 0x58],
            Theme::Fort => [0x2c, 0x28, 0x34],
        };
        self.frame.palette[FILL_LO as usize] = lo;
    }

    fn draw(&mut self) {
        self.set_theme();
        match self.mode {
            Mode::Title => self.draw_title(),
            Mode::Intro => self.draw_intro(),
            Mode::Play | Mode::Clear => {
                self.draw_world();
                self.draw_hud();
                if self.paused && (self.tick / 20).is_multiple_of(2) {
                    let f = &mut self.frame;
                    let w = FONT.width("暂停");
                    f.rect(120 - w / 2 - 4, 70, w + 8, 16, INK);
                    f.text(&FONT, 120 - w / 2, 72, "暂停", YELLOW_HI);
                }
                if self.mode == Mode::Clear {
                    let f = &mut self.frame;
                    f.rect(40, 54, 160, 48, INK);
                    f.frame_rect(41, 55, 158, 46, YELLOW);
                    f.text_center(&FONT, 60, &format!("第 {} 关 打下来了", self.level + 1), YELLOW_HI);
                    f.text_center(&FONT, 80, &format!("得分 {}", self.score), WHITE);
                }
            }
            Mode::Over => self.draw_over(),
            Mode::Ending => self.draw_ending(),
        }
    }

    fn draw_world(&mut self) {
        let real = self.cam;
        if self.shake > 0 {
            self.cam.1 += if self.shake.is_multiple_of(2) { 2 } else { -2 };
        }
        self.draw_background();
        self.draw_tiles();
        self.draw_flames();
        self.draw_mobs();
        self.draw_hero();
        self.draw_shots();
        self.draw_booms();
        self.cam = real;
    }

    fn draw_background(&mut self) {
        let theme = LEVELS[self.level].theme;
        let (cx, cy) = self.cam;
        let tick = self.tick as i32;
        let f = &mut self.frame;
        let hash = |n: i32| -> i32 { (n.wrapping_mul(1_103_515_245).wrapping_add(12345) >> 8) & 0xff };
        match theme {
            Theme::Jungle => {
                // 傍晚的天：三层，抖一排点接起来。
                f.rect(0, 0, W as i32, 40, SKY);
                f.rect(0, 40, W as i32, 30, SKY2);
                f.rect(0, 70, W as i32, 90, SKY3);
                for x in (0..W as i32).step_by(2) {
                    f.set(x, 40, SKY);
                    f.set(x + 1, 70, SKY2);
                }
                // 太阳落在山后面。
                for y in -12..=12 {
                    for x in -12..=12 {
                        if x * x + y * y <= 144 {
                            f.set(170 - cx / 32 + x, 74 + y, LIGHT);
                        }
                    }
                }
                // 远山。
                for x in 0..W as i32 {
                    let wx = x + cx / 6;
                    let hgt = 22 + (sin64(wx / 3) * 10 + sin64(wx / 7 + 9) * 8) / 256;
                    f.rect(x, 96 - hgt, 1, 64 + hgt, BG_FAR);
                }
                // 林子：一排棕榈，近的快。
                for x in 0..W as i32 {
                    let wx = x + cx / 2;
                    let hgt = 18 + (hash(wx / 6) & 7) + (sin64(wx) * 3 / 256);
                    f.rect(x, 112 - hgt, 1, 48 + hgt, BG_MID);
                }
                for k in 0..6 {
                    let tx = (k * 53 - cx * 2 / 3).rem_euclid(W as i32 + 60) - 30;
                    Self::palm(f, tx, 112, BG_NEAR);
                }
            }
            Theme::Falls => {
                f.rect(0, 0, W as i32, H as i32, SKY);
                // 中间的瀑布：往下流的白道道。
                for x in 40..200 {
                    for y in 0..H as i32 {
                        let wy = y + cy;
                        let c = if (wy - tick * 3 + hash(x) / 4).rem_euclid(24) < 3 + (x % 5 == 0) as i32 * 4 {
                            WATER_HI
                        } else if (x + wy / 2) % 7 == 0 {
                            WATER_LO
                        } else {
                            WATER
                        };
                        f.set(x, y, c);
                    }
                }
                // 两边的崖：远的石头，一层一层。
                for y in 0..H as i32 {
                    let wy = y + cy / 2;
                    let edge = 40 + (sin64(wy / 2) * 8 + sin64(wy / 5 + 7) * 6) / 256;
                    f.rect(0, y, edge, 1, BG_FAR);
                    f.rect(W as i32 - edge, y, edge, 1, BG_FAR);
                    if (wy + hash(wy / 9)) % 13 == 0 {
                        f.rect(edge - 10, y, 8, 1, BG_DARK);
                        f.rect(W as i32 - edge + 2, y, 8, 1, BG_DARK);
                    }
                }
            }
            Theme::Snow => {
                f.rect(0, 0, W as i32, 50, SKY);
                f.rect(0, 50, W as i32, 40, SKY2);
                f.rect(0, 90, W as i32, 70, SKY3);
                for x in (0..W as i32).step_by(2) {
                    f.set(x, 50, SKY);
                    f.set(x + 1, 90, SKY2);
                }
                // 星星。
                for k in 0..30 {
                    let sx = (hash(k * 7) - cx / 16).rem_euclid(W as i32);
                    let sy = hash(k * 13 + 5) % 60;
                    if (tick / 20 + k) % 9 != 0 {
                        f.set(sx, sy, LIGHT);
                    }
                }
                // 雪山：带白顶。
                for x in 0..W as i32 {
                    let wx = x + cx / 8;
                    let p = (wx % 120 - 60).abs();
                    let hgt = 70 - p + (hash(wx / 3) & 3);
                    let top = 110 - hgt;
                    f.rect(x, top, 1, H as i32 - top, BG_MID);
                    f.rect(x, top, 1, (hgt / 4).max(2), LIGHT);
                    f.rect(x, top + hgt / 4, 1, 2, BG_FAR);
                }
                for x in 0..W as i32 {
                    let wx = x + cx / 3;
                    let hgt = 16 + (sin64(wx / 2) * 6) / 256 + (hash(wx / 5) & 3);
                    f.rect(x, 116 - hgt, 1, 44 + hgt, BG_NEAR);
                }
                // 下雪。
                for k in 0..40 {
                    let sx = (hash(k * 31) + tick / 3 + sin64(tick + k * 9) * 4 / 256 - cx / 2).rem_euclid(W as i32);
                    let sy = (hash(k * 17 + 3) + tick * (1 + k % 3) / 2).rem_euclid(H as i32);
                    f.set(sx, sy, WHITE);
                    if k % 3 == 0 {
                        f.set(sx + 1, sy, WHITE);
                    }
                }
            }
            Theme::Fort => {
                f.rect(0, 0, W as i32, H as i32, SKY2);
                // 铁墙：一块一块的板，铆钉，管子，红灯。
                for y in 0..H as i32 {
                    for x in 0..W as i32 {
                        let wx = x + cx / 2;
                        if wx % 48 == 0 || y % 40 == 0 {
                            f.set(x, y, BG_DARK);
                        } else if wx % 48 == 1 || y % 40 == 1 {
                            f.set(x, y, BG_FAR);
                        } else if (wx % 48 == 6 || wx % 48 == 42) && (y % 40 == 6 || y % 40 == 34) {
                            f.set(x, y, BG_MID);
                        }
                    }
                }
                for x in 0..W as i32 {
                    let wx = x + cx / 2;
                    if wx % 96 < 8 {
                        f.rect(x, 0, 1, H as i32, if wx % 96 < 2 { BG_FAR } else { BG_NEAR });
                    }
                }
                for k in 0..8 {
                    let lx = (k * 96 + 44 - cx / 2).rem_euclid(W as i32 + 96) - 48;
                    let on = (tick / 30 + k) % 2 == 0;
                    f.rect(lx, 50, 4, 4, if on { LIGHT } else { WATER_LO });
                }
            }
        }
    }

    fn palm(f: &mut Frame, x: i32, base: i32, c: u8) {
        // 弯一点的树干。
        for y in 0..44 {
            let dx = (y * y) / 300;
            f.rect(x + dx, base - y, 3, 1, c);
        }
        let (tx, ty) = (x + 6, base - 44);
        for (a, len) in [(36, 16), (44, 14), (52, 14), (60, 16), (4, 12), (28, 12)] {
            for k in 0..len {
                let (dx, dy) = dir(a, k * 256);
                let droop = k * k / 20;
                f.rect(tx + dx / 256, ty + dy / 256 + droop, 2, 2, c);
            }
        }
    }

    fn draw_tiles(&mut self) {
        let (cx, cy) = self.cam;
        let theme = LEVELS[self.level].theme;
        let tick = self.tick as i32;
        let a = &self.art;
        let f = &mut self.frame;
        for ty in (cy / T).max(0)..((cy + H as i32) / T + 1).min(self.height as i32) {
            for tx in (cx / T).max(0)..((cx + W as i32) / T + 1).min(self.width as i32) {
                let t = self.map[ty as usize][tx as usize];
                let (sx, sy) = (tx * T - cx, ty * T - cy);
                let above = if ty > 0 { self.map[ty as usize - 1][tx as usize] } else { ROCK };
                match t {
                    ROCK | TURRET_T | PILL_T => {
                        if solid(above) || ty == 0 {
                            f.blit(&a.fill, sx, sy, false);
                        } else {
                            f.blit(&a.ledge, sx, sy, false);
                        }
                        if theme == Theme::Fort {
                            f.set(sx + 2, sy + 9, FILL_HI);
                            f.set(sx + 13, sy + 9, FILL_HI);
                            f.rect(sx, sy + 15, 16, 1, FILL_LO);
                            f.rect(sx + 15, sy, 1, 16, FILL_LO);
                        }
                    }
                    LEDGE => {
                        // 浮着的台子：只画上面半格，下面一道影子。
                        for y in 0..8 {
                            for x in 0..16 {
                                let c = a.ledge.at(x, y);
                                if c != 0 {
                                    f.set(sx + x, sy + y, c);
                                }
                            }
                        }
                        f.rect(sx, sy + 8, 16, 1, INK);
                    }
                    BRIDGE => f.blit(&a.bridge, sx, sy, false),
                    POOL => {
                        let surface = above != POOL;
                        let top = if surface { 8 } else { 0 };
                        f.rect(sx, sy + top, 16, 16 - top, WATER);
                        if surface {
                            for x in 0..16 {
                                let wave = ((tx * 16 + x + tick / 4) % 12 < 6) as i32;
                                f.set(sx + x, sy + 8 + wave, WATER_HI);
                            }
                        }
                        for x in (0..16).step_by(5) {
                            let y = (x * 3 + tick / 6 + tx * 7) % 16;
                            if y >= top + 3 {
                                f.set(sx + x, sy + y, WATER_LO);
                            }
                        }
                    }
                    NOZZLE => {
                        f.blit(&a.fill, sx, sy, false);
                        f.rect(sx + 3, sy, 10, 6, STEEL_LO);
                        f.rect(sx + 4, sy, 8, 5, STEEL);
                        f.rect(sx + 5, sy, 6, 2, INK);
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw_flames(&mut self) {
        let (cx, cy) = self.cam;
        let tick = self.tick as i32;
        let flames = self.flames();
        let f = &mut self.frame;
        for (x, y) in flames {
            let (sx, sy) = (x - cx, y - cy);
            for yy in 0..40 {
                let w = 4 + (40 - yy) / 6 + ((tick + yy) % 4 == 0) as i32;
                let c = if yy > 30 { YELLOW_HI } else if yy > 16 { YELLOW } else if yy > 6 { ORANGE } else { RED };
                f.rect(sx + 8 - w / 2 + ((tick / 2 + yy) % 3 - 1), sy + yy, w, 1, c);
            }
        }
        // 快要喷的时候冒火星。
        for tx in (cx / T).max(0)..((cx + W as i32) / T + 1).min(self.width as i32) {
            for ty in 0..self.height as i32 {
                if self.tile(tx, ty) == NOZZLE {
                    let t = (self.tick + tx as u32 * 37) % 150;
                    if t > 130 && (tick / 3) % 2 == 0 {
                        self.frame.rect(tx * T - cx + 6, ty * T - cy - 4, 4, 3, ORANGE);
                    }
                }
            }
        }
    }

    fn draw_mobs(&mut self) {
        let (cx, cy) = self.cam;
        let tick = self.tick;
        let mobs = std::mem::take(&mut self.mobs);
        for m in &mobs {
            let (x, y) = m.px();
            let (sx, sy) = (x - cx, y - cy);
            if sx + m.w < -64 || sx > W as i32 + 64 || sy + m.h < -64 || sy > H as i32 + 64 {
                continue;
            }
            let flip = m.face < 0;
            match m.kind {
                Kind::Runner | Kind::Rifle | Kind::Hider => {
                    let s = if m.kind == Kind::Runner {
                        &self.art.soldier[(tick / 5 % 3) as usize]
                    } else {
                        let a = if m.face < 0 { 32 - m.aim } else { m.aim }.rem_euclid(64);
                        let p = if (40..60).contains(&a) { 4 } else if (4..24).contains(&a) { 5 } else { 3 };
                        &self.art.soldier[p]
                    };
                    let s = if m.flash > 0 { s.recolor(&[(UNI, WHITE), (UNI_LO, WHITE)]) } else { s.clone() };
                    let s = if m.dying > 0 { flip_v(&s) } else { s };
                    let ox = if flip { -11 } else { -3 };
                    if m.kind == Kind::Hider {
                        // 躲着时只露钢盔顶。
                        let up = if m.state == 1 || m.dying > 0 { 0 } else { 14 };
                        let part = Sprite { w: s.w, h: s.h - up, px: s.px[..(s.w * (s.h - up)) as usize].to_vec() };
                        self.frame.blit(&part, sx + ox, sy - 2 + up, flip);
                        self.frame.blit(&self.art.sandbag, sx - 3, sy + 14, false);
                    } else {
                        self.frame.blit(&s, sx + ox, sy - 2, flip);
                    }
                }
                Kind::Lobber => {
                    let s = &self.art.lobber[m.state as usize];
                    let s = if m.dying > 0 { flip_v(s) } else { s.clone() };
                    self.frame.blit(&s, sx - 2, sy - 2, flip);
                    if m.dying == 0 {
                        self.frame.blit(&self.art.sandbag, sx - 2 + if flip { -6 } else { 4 }, sy + 6, false);
                    }
                }
                Kind::Turret => {
                    self.frame.blit(&self.art.turret, sx, sy, false);
                    let (dx, dy) = dir(m.aim, 256);
                    for k in 3..11 {
                        let (bx, by) = (sx + 8 + dx * k / 256, sy + 8 + dy * k / 256);
                        self.frame.rect(bx - 1, by - 1, 3, 3, INK);
                    }
                    for k in 3..10 {
                        self.frame.set(sx + 8 + dx * k / 256, sy + 8 + dy * k / 256, if m.flash > 0 { WHITE } else { STEEL_HI });
                    }
                }
                Kind::Pillbox => {
                    let s = &self.art.pillbox[m.state as usize];
                    let s = if m.flash > 0 { s.recolor(&[(RED, WHITE)]) } else { s.clone() };
                    self.frame.blit(&s, sx, sy, false);
                }
                Kind::Pod => self.frame.blit(&self.art.pod[(tick / 6 % 2) as usize], sx, sy, false),
                Kind::Medal => {
                    if m.timer > 600 && (tick / 4).is_multiple_of(2) {
                        continue;
                    }
                    self.frame.blit(&self.art.medal, sx, sy, false);
                    if let Some((_, rows)) = LETTERS.iter().find(|(c, _)| *c == m.letter) {
                        let s = Sprite::art(rows, &[('x', RED_LO)]);
                        self.frame.blit(&s, sx + 5, sy + 2, false);
                    }
                }
                Kind::RockSpot => {}
                Kind::Rock => {
                    let s = &self.art.rock;
                    let s = match (tick / 6) % 4 {
                        0 => s.clone(),
                        1 => rotate(s),
                        2 => rotate(&rotate(s)),
                        _ => rotate(&rotate(&rotate(s))),
                    };
                    self.frame.blit(&s, sx - 1, sy - 1, false);
                }
                Kind::Tank => {
                    let s = if m.flash > 0 { self.art.tank.recolor(&[(GREEN, WHITE)]) } else { self.art.tank.clone() };
                    self.frame.blit(&s, sx - 3, sy - 3, false);
                }
                Kind::GateCannon | Kind::GateCore => {}
                Kind::Totem | Kind::Hand => {}
                Kind::Crawler => Self::draw_crawler(&mut self.frame, m, sx, sy, tick),
                Kind::Robot => Self::draw_robot(&mut self.frame, m, sx, sy, tick),
            }
        }
        // 大怪画在最后：铁门、图腾的胳膊。
        if LEVELS[self.level].theme == Theme::Jungle && (self.boss_on || self.cam.0 + W as i32 > self.boss_at.0 - 8) {
            self.draw_gate(&mobs);
        }
        for m in &mobs {
            if m.kind == Kind::Totem && m.live() {
                Self::draw_totem(&mut self.frame, m, m.px().0 - cx, m.px().1 - cy, tick);
            }
            if m.kind == Kind::Hand && m.live() {
                for (bx, by) in self.arm_balls(m) {
                    let (sx, sy) = (bx - cx, by - cy);
                    Self::ball(&mut self.frame, sx, sy, 4, STEEL_LO, STEEL, STEEL_HI);
                }
                let (x, y) = m.px();
                Self::ball(&mut self.frame, x - cx + 6, y - cy + 6, 6, RED_LO, if m.flash > 0 { WHITE } else { RED }, RED_HI);
            }
        }
        self.mobs = mobs;
    }

    fn ball(f: &mut Frame, x: i32, y: i32, r: i32, lo: u8, mid: u8, hi: u8) {
        for dy in -r..=r {
            for dx in -r..=r {
                let d = dx * dx + dy * dy;
                if d <= r * r {
                    let c = if d >= (r - 1) * (r - 1) {
                        INK
                    } else if dx + dy < -r / 2 {
                        hi
                    } else if dx + dy > r / 2 {
                        lo
                    } else {
                        mid
                    };
                    f.set(x + dx, y + dy, c);
                }
            }
        }
    }

    /// 铁门：一面高墙，铆钉，两门炮，下面一个红芯。
    fn draw_gate(&mut self, mobs: &[Mob]) {
        let (cx, cy) = self.cam;
        let x0 = self.boss_at.0 - 8 - cx;
        let floor = self.boss_at.1 + T - cy;
        let top = floor - 96;
        let wide = self.width as i32 * T - cx - x0;
        let f = &mut self.frame;
        let broken = self.boss_fall > 0;
        f.rect(x0, top, wide, floor - top, if broken { GRAY_LO } else { STEEL_LO });
        f.rect(x0 + 2, top + 2, wide - 4, floor - top - 4, if broken { GRAY_LO } else { STEEL });
        for y in (top + 6..floor - 4).step_by(14) {
            f.rect(x0 + 2, y, wide - 4, 1, STEEL_LO);
            for x in (x0 + 6..x0 + wide - 2).step_by(10) {
                f.set(x, y + 4, STEEL_HI);
            }
        }
        f.rect(x0, top, wide, 3, INK);
        f.rect(x0, top, 2, floor - top, INK);
        // 顶上的锯齿。
        for x in (x0..x0 + wide).step_by(8) {
            f.rect(x, top - 6, 5, 6, STEEL_LO);
            f.rect(x + 1, top - 5, 3, 4, STEEL);
        }
        for m in mobs {
            if !m.live() {
                continue;
            }
            let (x, y) = m.px();
            let (sx, sy) = (x - cx, y - cy);
            match m.kind {
                Kind::GateCannon => {
                    f.rect(sx + 6, sy - 2, 14, 14, INK);
                    f.rect(sx + 7, sy - 1, 12, 12, if m.flash > 0 { WHITE } else { GRAY });
                    f.rect(sx, sy + 2, 14, 6, INK);
                    f.rect(sx + 1, sy + 3, 13, 4, STEEL_HI);
                }
                Kind::GateCore => {
                    let pulse = (self.tick / 8 % 3) as i32;
                    f.rect(sx - 2, sy - 2, m.w + 4, m.h + 4, INK);
                    f.rect(sx, sy, m.w, m.h, RED_LO);
                    let c = if m.flash > 0 { WHITE } else { [RED, RED_HI, ORANGE][pulse as usize] };
                    Self::ball(f, sx + m.w / 2, sy + m.h / 2, 9, RED_LO, c, YELLOW_HI);
                }
                _ => {}
            }
        }
    }

    /// 崖顶上的炮：铁的脑袋，一只大眼，张嘴吐火球。
    fn draw_totem(f: &mut Frame, m: &Mob, sx: i32, sy: i32, tick: u32) {
        let open = m.timer % 150 > 85 && m.timer % 150 < 115;
        f.rect(sx - 2, sy - 2, 44, 36, INK);
        f.rect(sx, sy, 40, 32, if m.flash > 0 { WHITE } else { STEEL_LO });
        f.rect(sx + 2, sy + 2, 36, 12, STEEL);
        f.rect(sx + 2, sy + 2, 36, 2, STEEL_HI);
        // 角。
        for k in 0..8 {
            f.rect(sx - 4 - k / 2, sy - k, 4, 1, STEEL);
            f.rect(sx + 40 + k / 2, sy - k, 4, 1, STEEL);
        }
        // 眼睛跟着闪。
        let eye = if (tick / 6).is_multiple_of(8) { RED_LO } else { RED };
        Self::ball(f, sx + 20, sy + 12, 7, RED_LO, eye, YELLOW_HI);
        f.rect(sx + 18, sy + 10, 4, 4, INK);
        // 嘴。
        f.rect(sx + 8, sy + 22, 24, if open { 8 } else { 3 }, INK);
        if open {
            f.rect(sx + 10, sy + 24, 20, 4, ORANGE);
        }
        for x in (sx + 8..sx + 32).step_by(4) {
            f.rect(x, sy + 22, 2, 2, WHITE);
        }
    }

    fn draw_crawler(f: &mut Frame, m: &Mob, sx: i32, sy: i32, tick: u32) {
        let flip = m.face > 0;
        let w = m.w;
        let hl = |f: &mut Frame, x: i32, y: i32, ww: i32, h: i32, c: u8| {
            let x0 = if flip { sx + w - x - ww } else { sx + x };
            f.rect(x0, sy + y, ww, h, c);
        };
        // 履带。
        hl(f, 2, 20, 60, 10, INK);
        hl(f, 3, 21, 58, 8, GRAY_LO);
        for k in 0..7 {
            let x = 6 + k * 8 + ((tick as i32 / 2) % 8) * (m.vx.signum());
            hl(f, x.rem_euclid(52) + 4, 22, 4, 6, GRAY);
        }
        // 车身。
        hl(f, 0, 10, 64, 12, INK);
        hl(f, 1, 11, 62, 10, GREEN_LO);
        hl(f, 1, 11, 62, 3, GREEN);
        // 前面的铁铲。
        hl(f, 0, 12, 6, 14, INK);
        hl(f, 1, 13, 4, 12, STEEL);
        // 驾驶舱：打得着的地方。
        hl(f, 36, -1, 18, 13, INK);
        let glass = if m.flash > 0 { WHITE } else if (tick / 10).is_multiple_of(2) { CYAN } else { BLUE_HI };
        hl(f, 37, 0, 16, 11, GREEN);
        hl(f, 40, 2, 10, 6, glass);
        // 炮管朝上。
        hl(f, 20, 2, 4, 9, INK);
        hl(f, 21, 3, 2, 8, STEEL_HI);
    }

    fn draw_robot(f: &mut Frame, m: &Mob, sx: i32, sy: i32, tick: u32) {
        let walk = if m.vx != 0 { (tick / 8 % 2) as i32 * 3 } else { 0 };
        let flash = m.flash > 0;
        let body = if flash { WHITE } else { PURPLE };
        // 腿。
        for (lx, off) in [(8, walk), (24, 3 - walk)] {
            f.rect(sx + lx - 1, sy + 36 - off, 10, 20 + off, INK);
            f.rect(sx + lx, sy + 37 - off, 8, 18 + off, STEEL_LO);
            f.rect(sx + lx - 2, sy + 52, 12, 4, INK);
            f.rect(sx + lx - 1, sy + 52, 10, 3, STEEL);
        }
        // 身子。
        f.rect(sx - 1, sy + 11, 42, 28, INK);
        f.rect(sx, sy + 12, 40, 26, body);
        f.rect(sx, sy + 12, 40, 3, PINK);
        // 胳膊（炮）。
        let ax = if m.face < 0 { sx - 10 } else { sx + 38 };
        f.rect(ax, sy + 18, 12, 8, INK);
        f.rect(ax + 1, sy + 19, 10, 6, STEEL);
        // 头。
        f.rect(sx + 9, sy - 1, 22, 14, INK);
        f.rect(sx + 10, sy, 20, 12, STEEL);
        let ex = if m.face < 0 { sx + 12 } else { sx + 22 };
        f.rect(ex, sy + 4, 6, 3, RED_HI);
        // 胸口的芯：弱点，放激光前闪。
        let charging = m.state == 2 && m.timer < 12 && (tick / 2).is_multiple_of(2);
        let core = if flash { WHITE } else if charging { YELLOW_HI } else { CYAN };
        Self::ball(f, sx + 20, sy + 36, 6, BLUE, core, WHITE);
        // 激光。
        if m.state == 2 && (12..36).contains(&m.timer) {
            let y = sy + 34;
            let (x0, x1) = if m.face < 0 { (0, sx + 14) } else { (sx + 26, W as i32) };
            f.rect(x0, y, x1 - x0, 6, CYAN);
            f.rect(x0, y + 2, x1 - x0, 2, WHITE);
        } else if m.state == 2 && m.timer < 12 {
            let y = sy + 36;
            let (x0, x1) = if m.face < 0 { (0, sx + 14) } else { (sx + 26, W as i32) };
            if (tick / 2).is_multiple_of(2) {
                f.rect(x0, y, x1 - x0, 1, RED_HI);
            }
        }
    }

    fn draw_hero(&mut self) {
        let h = &self.hero;
        if h.safe > 0 && h.dead == 0 && (self.tick / 2).is_multiple_of(2) {
            return;
        }
        let (cx, cy) = self.cam;
        let (x, y) = h.px();
        let (sx, sy) = (x - cx, y - cy);
        let who = self.who;
        let flip = h.face < 0;
        let a = &self.art;
        let mut s = if h.dead > 0 {
            if h.dead > DEAD_TIME - 40 {
                a.spin[who][((DEAD_TIME - h.dead) / 4 % 4) as usize].clone()
            } else {
                a.downed[who].clone()
            }
        } else if h.spin {
            a.spin[who][(self.tick / 3 % 4) as usize].clone()
        } else if h.prone {
            a.prone[who].clone()
        } else {
            let aim = match h.aim {
                Aim::Fwd => 0,
                Aim::Up => 1,
                Aim::DiagUp => 2,
                Aim::DiagDown => 3,
            };
            let leg = if h.walk > 0 { 1 + (h.walk / 6 % 3) as usize } else { 0 };
            a.hero[who][aim][leg].clone()
        };
        if h.barrier > 0 && (self.tick / 3).is_multiple_of(2) {
            s = s.recolor(&[(VEST, WHITE), (PANTS, CYAN), (BAND, YELLOW_HI)]);
        }
        let f = &mut self.frame;
        if h.dead > 0 && h.dead <= DEAD_TIME - 40 {
            f.blit(&s, sx - 7, sy + HH - 6, flip);
        } else if h.spin {
            f.blit(&s, sx - 3, sy + 3, false);
        } else if h.prone {
            f.blit(&s, if flip { sx - 18 } else { sx - 4 }, sy + HH - 8, flip);
        } else if h.diving {
            // 只看得见水面上一圈波纹。
            let t = (self.tick / 6 % 2) as i32;
            f.rect(sx - 2 - t, sy + HH - 4, 14 + t * 2, 1, WATER_HI);
        } else if h.in_water {
            let ox = if flip { -11 } else { -3 };
            let keep = 14;
            let part = Sprite { w: s.w, h: keep, px: s.px[..(s.w * keep) as usize].to_vec() };
            let bob = (self.tick / 16 % 2) as i32;
            f.blit(&part, sx + ox, sy - 2 + bob, flip);
        } else {
            let ox = if flip { -11 } else { -3 };
            f.blit(&s, sx + ox, sy - 2, flip);
        }
    }

    fn draw_shots(&mut self) {
        let (cx, cy) = self.cam;
        let tick = self.tick;
        let f = &mut self.frame;
        for s in &self.shots {
            let (x, y) = (px(s.x) - cx, px(s.y) - cy);
            match s.kind {
                Weapon::Normal | Weapon::Machine => {
                    f.rect(x - 1, y - 1, 3, 3, if s.kind == Weapon::Machine { YELLOW } else { WHITE });
                    f.set(x, y, WHITE);
                }
                Weapon::Spread => {
                    f.rect(x - 2, y - 1, 5, 3, RED);
                    f.rect(x - 1, y - 2, 3, 5, RED);
                    f.rect(x - 1, y - 1, 3, 3, RED_HI);
                    f.set(x, y, WHITE);
                }
                Weapon::Laser => {
                    for k in 0..14 {
                        let lx = x - s.vx * k / (s.vx.abs().max(s.vy.abs()).max(1));
                        let ly = y - s.vy * k / (s.vx.abs().max(s.vy.abs()).max(1));
                        f.rect(lx - 1, ly - 1, 3, 3, CYAN);
                        f.set(lx, ly, WHITE);
                    }
                }
                Weapon::Fire => {
                    let c = if (tick / 2).is_multiple_of(2) { ORANGE } else { YELLOW };
                    f.rect(x - 3, y - 2, 7, 5, RED);
                    f.rect(x - 2, y - 3, 5, 7, RED);
                    f.rect(x - 2, y - 2, 5, 5, c);
                    f.rect(x - 1, y - 1, 2, 2, YELLOW_HI);
                }
            }
        }
        for s in &self.eshots {
            let (x, y) = (px(s.x) - cx, px(s.y) - cy);
            match s.kind {
                EKind::Bullet => {
                    f.rect(x - 2, y - 1, 4, 2, INK);
                    f.rect(x - 1, y - 2, 2, 4, INK);
                    f.rect(x - 1, y - 1, 2, 2, if (tick / 3).is_multiple_of(2) { WHITE } else { RED_HI });
                }
                EKind::Ball => Self::ball(f, x, y, 4, RED_LO, ORANGE, YELLOW_HI),
                EKind::Shell => Self::ball(f, x, y, 3, GRAY_LO, GRAY, WHITE),
                EKind::Bomb => {
                    Self::ball(f, x, y, 4, INK, GRAY_LO, GRAY);
                    if (tick / 4).is_multiple_of(2) {
                        f.set(x + 2, y - 4, RED_HI);
                    }
                }
            }
        }
    }

    fn draw_booms(&mut self) {
        let (cx, cy) = self.cam;
        let f = &mut self.frame;
        for b in &self.booms {
            let (x, y) = (b.x - cx, b.y - cy);
            let r = (b.size * (b.t as i32 + 4) / 16).min(b.size);
            let rings = [(r, if b.t < 10 { RED } else { GRAY_LO }), (r * 3 / 4, ORANGE), (r / 2, YELLOW), (r / 4, YELLOW_HI)];
            for (rr, c) in rings {
                if b.t > 14 && c != GRAY_LO {
                    continue;
                }
                for dy in -rr..=rr {
                    let half = ((rr * rr - dy * dy) as f32).sqrt() as i32;
                    let jag = ((dy + b.t as i32) % 3) - 1;
                    f.rect(x - half + jag, y + dy, half * 2 + 1, 1, c);
                }
            }
        }
    }

    fn draw_hud(&mut self) {
        let f = &mut self.frame;
        let a = &self.art;
        // 左上：几条命、手里的枪。
        for i in 0..self.lives.min(6) as i32 {
            f.blit(&a.life, 4 + i * 7, 4, false);
        }
        if self.lives > 6 {
            f.text_outlined(&FONT, 48, 1, &format!("×{}", self.lives), WHITE, INK);
        }
        let l = self.hero.weapon.letter();
        if l != ' ' || self.hero.rapid {
            let tag = format!("{}{}", l.to_string().trim(), if self.hero.rapid { "+" } else { "" });
            f.text_outlined(&FONT, 4, 12, &tag, YELLOW_HI, INK);
        }
        let s = format!("{:06}", self.score);
        f.text_outlined(&FONT, W as i32 - 4 - FONT.width(&s), 1, &s, WHITE, INK);
    }

    fn draw_intro(&mut self) {
        let f = &mut self.frame;
        f.clear(BLACK);
        let l = &LEVELS[self.level];
        let big = FONT.sprite(&format!("第 {} 关", self.level + 1), WHITE).scaled(2);
        f.blit(&big, (W as i32 - big.w) / 2, 34, false);
        f.text_center(&FONT, 66, l.name, YELLOW_HI);
        f.text_center(&FONT, 96, &format!("队员 {}　还有 {} 条命", ["雷", "霆"][self.who], self.lives), WHITE);
        f.text_center(&FONT, 114, &format!("得分 {}　最高 {}", self.score, self.hi), GRAY);
        let s = &self.art.hero[self.who][0][1 + (self.tick / 6 % 3) as usize];
        f.blit(s, 108, 128, false);
    }

    fn draw_over(&mut self) {
        let f = &mut self.frame;
        f.clear(BLACK);
        let big = FONT.sprite("打不动了", RED_HI).scaled(2);
        f.blit(&big, (W as i32 - big.w) / 2, 30, false);
        f.text_center(&FONT, 64, &format!("得分 {}　最高 {}", self.score, self.hi), WHITE);
        if self.continues > 0 {
            let items = [format!("接着打（还能 {} 次）", self.continues), "不打了".to_string()];
            for (i, t) in items.iter().enumerate() {
                let y = 92 + i as i32 * 18;
                let c = if i == self.menu { YELLOW_HI } else { GRAY };
                f.text_center(&FONT, y, t, c);
                if i == self.menu {
                    let w = FONT.width(t);
                    f.text(&FONT, 120 - w / 2 - 14, y, "▶", YELLOW_HI);
                }
            }
        }
    }

    fn draw_title(&mut self) {
        let f = &mut self.frame;
        let tick = self.tick as i32;
        // 黑夜里的探照灯和远山。
        f.clear(NAVY);
        for y in 0..H as i32 {
            if y > 60 {
                f.rect(0, y, W as i32, 1, if y > 110 { BLACK } else { INK });
            }
        }
        for k in 0..2 {
            let base = if k == 0 { 40 } else { 200 };
            let sweep = sin64(tick / 2 + k * 20) * 30 / 256;
            for yy in 0..100 {
                let cxp = base + sweep * (100 - yy) / 100;
                let w = 2 + (100 - yy) / 6;
                for x in cxp - w..cxp + w {
                    if (x + yy) % 2 == 0 {
                        f.set(x, 110 - yy - 10, BLUE);
                    }
                }
            }
        }
        for x in 0..W as i32 {
            let hgt = 24 + (sin64(x / 2) * 10 + sin64(x / 5 + 11) * 6) / 256;
            f.rect(x, 110 - hgt, 1, hgt, GRAY_LO);
        }
        // 标题：大字带红边。
        let title = FONT.sprite("雷霆突击", YELLOW_HI).scaled(3);
        let edge = FONT.sprite("雷霆突击", RED).scaled(3);
        let tx = (W as i32 - title.w) / 2;
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 2), (2, 2)] {
            f.blit(&edge, tx + dx, 12 + dy, false);
        }
        f.blit(&title, tx, 12, false);
        // 两个队员站在前面，选中的那个在亮处。
        for i in 0..2 {
            let s = &self.art.hero[i][2][0];
            let x = if i == 0 { 72 } else { 144 };
            let s = if i == self.who { s.clone() } else { s.recolor(&[(VEST, GRAY_LO), (VEST_LO, GRAY_LO), (SAND, GRAY_LO), (SAND_LO, GRAY_LO), (PANTS, GRAY_LO), (GREEN_LO, GRAY_LO)]) };
            f.blit(&s, x, 104, i == 1);
            let name = ["雷", "霆"][i];
            f.text_outlined(&FONT, x + 6, 130, name, if i == self.who { YELLOW_HI } else { GRAY }, INK);
        }
        if self.unlocked > 0 {
            let l = format!("▲ 第 {} 关 {} ▼", self.level + 1, LEVELS[self.level].name);
            f.text_outlined(&FONT, (W as i32 - FONT.width(&l)) / 2, 66, &l, WHITE, INK);
        }
        if (tick / 30) % 2 == 0 {
            f.text_outlined(&FONT, (W as i32 - FONT.width("按 START 开始")) / 2, 84, "按 START 开始", YELLOW_HI, INK);
        }
        let hi = format!("最高 {}", self.hi);
        f.text_outlined(&FONT, (W as i32 - FONT.width(&hi)) / 2, 146, &hi, WHITE, INK);
    }

    fn draw_ending(&mut self) {
        let f = &mut self.frame;
        let tick = self.tick as i32;
        f.clear(NAVY);
        // 天亮了：下面一道橙色。
        f.rect(0, 100, W as i32, 60, RED_LO);
        f.rect(0, 112, W as i32, 48, ORANGE);
        f.rect(0, 124, W as i32, 36, BLACK);
        let big = FONT.sprite("任务完成", YELLOW_HI).scaled(2);
        f.blit(&big, (W as i32 - big.w) / 2, 20, false);
        f.text_center(&FONT, 56, "要塞拿下了", WHITE);
        f.text_center(&FONT, 72, &format!("得分 {}", self.score), WHITE);
        f.text_center(&FONT, 88, &format!("最高 {}", self.hi), YELLOW_HI);
        let run = (tick / 2).rem_euclid(W as i32 + 64) - 32;
        let a = &self.art;
        f.blit(&a.hero[0][0][1 + (tick / 6 % 3) as usize], run, 100, false);
        f.blit(&a.hero[1][0][1 + ((tick + 9) / 6 % 3) as usize], run - 28, 100, false);
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
