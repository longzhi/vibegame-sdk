//! 游戏《烈空战机》：竖版的飞机射击。开「烈风」往上飞，四关，每关最后一个大怪。
//!
//! A 开火（按住连发），B 放炸弹（清掉一屏子弹，炸一屏敌人）。打运输机掉 P（火力加一级）和 B（多一颗炸弹）。
//! 挨一下就掉一条命，掉一级火力。没命了能接着打三次。过一关存一关，最高分也存着。
//! 位置都用 1/16 像素的整数，随机数自己写：同样的按键每次跑出来一样。

mod art;
mod stages;
mod tune;
#[cfg(test)]
mod tests;

use art::*;
use stages::{Kind, Wave, STAGES};
use std::sync::Mutex;
use vibeplay::full::{key, lang, tone_wav, Camera, Channel, Frame, Keys, Music, Sprite, TileMap, H, W};
use vibeplay::FONT;

/// 1 像素是 16 个单位。
const S: i32 = 16;
/// 地图每帧往上卷多少（1/16 像素）：半个像素。
const SCROLL: i32 = 8;
/// 卷到顶要卷多少像素。
const TOP: i32 = stages::ROWS * 16 - H as i32;
const SAVE: &str = "progress";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Title,
    /// 关头的字：「第一关 海岸线」。
    Intro,
    Play,
    /// 大怪炸了，算分。
    Clear,
    /// 没命了：问接不接着打。
    Over,
    /// 四关打完。
    Ending,
}

#[derive(Clone, Copy, Debug)]
struct Shot {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pub x: i32,
    pub y: i32,
    vx: i32,
    vy: i32,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    vx: i32,
    vy: i32,
    pub hp: i32,
    t: u32,
    flash: u8,
    /// 打下来掉的东西：0 是 P，1 是 B，2 是星。
    drop: Option<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub kind: u8,
    pub x: i32,
    pub y: i32,
    t: u32,
}

/// 一团火：画面上的像素，`r` 是最大的半径。
#[derive(Clone, Copy, Debug)]
struct Boom {
    x: i32,
    y: i32,
    r: i32,
    t: u32,
}

/// 大怪的一块：炮或者芯。芯要等别的块都打掉才打得动。
#[derive(Clone, Copy, Debug)]
pub struct Part {
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    pub hp: i32,
    pub core: bool,
    flash: u8,
}

#[derive(Clone, Debug)]
pub struct Boss {
    /// 左上角，像素。
    pub x: i32,
    pub y: i32,
    w: i32,
    h: i32,
    t: u32,
    pub parts: Vec<Part>,
    /// 炸着的帧数，0 是还活着。
    pub dying: u32,
    spin: f32,
}

impl Boss {
    fn open(&self) -> bool {
        self.parts.iter().all(|p| p.core || p.hp <= 0)
    }
    fn core(&self) -> &Part {
        self.parts.iter().find(|p| p.core).unwrap()
    }
}

pub struct Game {
    pub frame: Frame,
    pub keys: Keys,
    music: Music,
    pub mode: Mode,
    pub stage: usize,
    /// 打开了几关（1..=4）。
    pub unlocked: usize,
    pub hi: u32,
    pub score: u32,
    pub lives: i32,
    pub bombs: i32,
    /// 火力 1..=4。
    pub power: i32,
    pub continues: i32,
    /// 主角左上角，1/16 像素。
    pub x: i32,
    pub y: i32,
    /// 无敌还剩几帧（刚出来、放炸弹）。
    pub invuln: u32,
    /// 炸了以后还要等几帧再出来。0 是活着。
    pub dead: u32,
    shots: Vec<Shot>,
    pub bullets: Vec<Bullet>,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    booms: Vec<Boom>,
    pub boss: Option<Boss>,
    /// 卷过多少，1/16 像素。
    pub scroll: i32,
    waves: Vec<Wave>,
    next_wave: usize,
    /// 等着出场的：还差几帧、哪种、横坐标。
    queue: Vec<(u32, Kind, i32)>,
    map: TileMap,
    tiles: Vec<Sprite>,
    /// 模式里过了几帧。
    pub t: u32,
    seed: u32,
    /// 炸弹还要炸几帧。
    pub bomb_t: u32,
    pub paused: bool,
    /// 下一条命在多少分。
    extend: u32,
    fire_t: u32,
    /// 运输机掉的东西轮着来。
    drops: u32,
    pick: usize,
    sprites: Sprites,
    boom_wav: Vec<u8>,
}

struct Sprites {
    hero: [Sprite; 2],
    dart: Sprite,
    swoop: Sprite,
    bomber: Sprite,
    carrier: Sprite,
    mine: [Sprite; 2],
    turret: Sprite,
    items: [Sprite; 3],
    life: Sprite,
}

impl Default for Game {
    fn default() -> Self {
        let mut frame = Frame::default();
        for (i, rgb) in PALETTE {
            frame.palette[i as usize] = rgb;
        }
        let (unlocked, hi) = load();
        let mut g = Game {
            frame,
            keys: Keys::default(),
            music: Music::default(),
            mode: Mode::Title,
            stage: 0,
            unlocked,
            hi,
            score: 0,
            lives: 3,
            bombs: 3,
            power: 1,
            continues: 3,
            x: (W as i32 / 2 - 8) * S,
            y: 120 * S,
            invuln: 0,
            dead: 0,
            shots: Vec::new(),
            bullets: Vec::new(),
            enemies: Vec::new(),
            items: Vec::new(),
            booms: Vec::new(),
            boss: None,
            scroll: 0,
            waves: Vec::new(),
            next_wave: 0,
            queue: Vec::new(),
            map: stages::map(0),
            tiles: tiles(0),
            t: 0,
            seed: 12345,
            bomb_t: 0,
            paused: false,
            extend: 20000,
            fire_t: 0,
            drops: 0,
            pick: 0,
            sprites: Sprites {
                hero: [hero(false), hero(true)],
                dart: dart(),
                swoop: swoop(),
                bomber: bomber(),
                carrier: carrier(),
                mine: [mine(false), mine(true)],
                turret: turret_base(),
                items: [item(0), item(1), item(2)],
                life: Sprite::art(&["..w..", ".wrw.", "wrrrw", "..w.."], &[('w', WHITE), ('r', RED)]),
            },
            boom_wav: boom_wav(),
        };
        g.music.play(tune::title());
        g
    }
}

/// 存档：「打开了几关,最高分」。
fn load() -> (usize, u32) {
    let s = vibeplay::load(SAVE).and_then(|b| String::from_utf8(b).ok()).unwrap_or_default();
    let mut it = s.split(',');
    let u = it.next().and_then(|v| v.parse().ok()).unwrap_or(1usize).clamp(1, 4);
    let hi = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    (u, hi)
}

/// 一段炸开的声音：噪声，越来越小。16 位 PCM，11025 Hz。
fn boom_wav() -> Vec<u8> {
    let rate = 11025u32;
    let n = rate as usize * 6 / 10;
    let mut data = Vec::with_capacity(n * 2);
    let (mut s, mut last) = (0x1234_5678u32, 0f32);
    for i in 0..n {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let white = (s % 2000) as f32 / 1000.0 - 1.0;
        // 低通：上一点和这一点平均，声音闷一点、像爆炸。
        last = last * 0.7 + white * 0.3;
        let fall = 1.0 - i as f32 / n as f32;
        let v = (last * fall * fall * 30000.0) as i16;
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

/// 每种敌人的大小（像素）、血、分。
fn size(k: Kind) -> (i32, i32) {
    match k {
        Kind::Dart => (10, 8),
        Kind::SwoopL | Kind::SwoopR => (14, 8),
        Kind::Bomber => (32, 18),
        Kind::Carrier => (19, 9),
        Kind::Mine => (9, 9),
        Kind::Turret => (16, 14),
    }
}

fn worth(k: Kind) -> (i32, u32) {
    match k {
        Kind::Dart => (1, 100),
        Kind::SwoopL | Kind::SwoopR => (2, 150),
        Kind::Bomber => (28, 1500),
        Kind::Carrier => (6, 300),
        Kind::Mine => (3, 200),
        Kind::Turret => (6, 300),
    }
}

fn overlap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

impl Game {
    fn rand(&mut self, n: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.seed >> 16) % n.max(1)) as i32
    }

    /// 主角中间那一点（像素）。挨打只看这一小块。
    pub fn center(&self) -> (i32, i32) {
        (self.x / S + 8, self.y / S + 8)
    }

    fn save(&self) {
        vibeplay::save(SAVE, format!("{},{}", self.unlocked, self.hi).as_bytes());
    }

    /// 开一局：从第 `stage` 关打起。
    pub fn start(&mut self, stage: usize) {
        self.score = 0;
        self.lives = 3;
        self.bombs = 3;
        self.power = 1;
        self.continues = 3;
        self.extend = 20000;
        self.enter(stage);
    }

    /// 进一关：地图、脚本从头来，主角从下面飞上来。
    pub fn enter(&mut self, stage: usize) {
        self.stage = stage;
        self.map = stages::map(stage);
        self.tiles = tiles(stage);
        self.waves = stages::parse(STAGES[stage].script).expect("关卡脚本写错了");
        self.next_wave = 0;
        self.queue.clear();
        self.scroll = 0;
        self.shots.clear();
        self.bullets.clear();
        self.enemies.clear();
        self.items.clear();
        self.booms.clear();
        self.boss = None;
        self.bomb_t = 0;
        self.dead = 0;
        self.x = (W as i32 / 2 - 8) * S;
        self.y = (H as i32 + 8) * S;
        self.invuln = 150;
        self.mode = Mode::Intro;
        self.t = 0;
        self.music.play(tune::stage(stage));
    }

    fn add_score(&mut self, n: u32) {
        self.score += n;
        if self.score >= self.extend {
            self.lives += 1;
            self.extend += if self.extend == 20000 { 40000 } else { 50000 };
            self.music.sfx(Channel::Square2, 1568, 200, 40);
        }
        self.hi = self.hi.max(self.score);
    }

    /// 走一帧。
    pub fn step(&mut self) {
        self.t += 1;
        match self.mode {
            Mode::Title => self.step_title(),
            Mode::Intro => {
                self.scroll_map();
                // 从下面飞到 120 那一行。
                if self.y > 120 * S {
                    self.y -= S;
                }
                if self.t >= 120 {
                    self.mode = Mode::Play;
                    self.t = 0;
                }
            }
            Mode::Play => {
                if self.keys.pressed(key::START) {
                    self.paused = !self.paused;
                    self.music.muted = self.paused;
                }
                if !self.paused {
                    self.step_play();
                }
            }
            Mode::Clear => {
                self.move_things();
                if self.t == 1 {
                    self.music.play(tune::clear());
                    let bonus = self.bombs as u32 * 1000 + 5000;
                    self.add_score(bonus);
                    self.unlocked = self.unlocked.max((self.stage + 2).min(4));
                    self.save();
                }
                if self.t > 60 && self.y > -24 * S {
                    // 飞出画面上面。
                    self.y -= 3 * S;
                }
                if self.t >= 260 {
                    if self.stage + 1 < STAGES.len() {
                        self.enter(self.stage + 1);
                    } else {
                        self.mode = Mode::Ending;
                        self.t = 0;
                        self.music.play(tune::title());
                    }
                }
            }
            Mode::Over => {
                if self.t == 1 {
                    self.music.play(tune::over());
                    self.save();
                }
                if self.t > 40 {
                    if self.continues > 0 && (self.keys.pressed(key::A) || self.keys.pressed(key::START)) {
                        self.continues -= 1;
                        self.score = 0;
                        self.lives = 3;
                        self.bombs = 3;
                        self.respawn();
                        self.mode = Mode::Play;
                        self.t = 0;
                        self.music.play(if self.boss.is_some() { tune::boss() } else { tune::stage(self.stage) });
                    } else if self.keys.pressed(key::B) || (self.continues == 0 && self.keys.pressed(key::START)) {
                        self.back_to_title();
                    }
                }
            }
            Mode::Ending => {
                if self.t > 120 && (self.keys.pressed(key::START) || self.keys.pressed(key::A)) {
                    self.back_to_title();
                }
            }
        }
        self.music.tick();
    }

    fn back_to_title(&mut self) {
        self.mode = Mode::Title;
        self.t = 0;
        self.boss = None;
        self.enemies.clear();
        self.bullets.clear();
        self.music.play(tune::title());
    }

    fn step_title(&mut self) {
        if self.keys.pressed(key::LEFT) && self.pick > 0 {
            self.pick -= 1;
            self.music.sfx(Channel::Square2, 880, 40, 30);
        }
        if self.keys.pressed(key::RIGHT) && self.pick + 1 < self.unlocked {
            self.pick += 1;
            self.music.sfx(Channel::Square2, 880, 40, 30);
        }
        if self.keys.pressed(key::START) || self.keys.pressed(key::A) {
            self.start(self.pick);
        }
    }

    fn scroll_map(&mut self) {
        if self.scroll < TOP * S {
            self.scroll += SCROLL;
        }
    }

    fn step_play(&mut self) {
        let before = self.scroll;
        self.scroll_map();
        let moved = self.scroll - before;
        // 地面上的东西跟着地图往下走。
        for e in self.enemies.iter_mut().filter(|e| e.kind == Kind::Turret) {
            e.y += moved;
        }
        self.spawn();
        self.step_hero();
        self.move_things();
        self.step_enemies();
        if self.boss.is_some() {
            self.step_boss();
        }
        self.hits();
    }

    /// 照脚本出敌人。卷到顶出大怪。
    fn spawn(&mut self) {
        let px = self.scroll / S;
        while let Some(w) = self.waves.get(self.next_wave).copied() {
            if w.at > px {
                break;
            }
            self.next_wave += 1;
            let gap = if w.kind == Kind::Turret { 0 } else { 16 };
            for i in 0..w.n {
                self.queue.push((i * gap, w.kind, w.x));
            }
        }
        let mut ready = Vec::new();
        self.queue.retain_mut(|q| {
            if q.0 == 0 {
                ready.push((q.1, q.2));
                false
            } else {
                q.0 -= 1;
                true
            }
        });
        for (k, x) in ready {
            self.add_enemy(k, x);
        }
        if self.scroll >= TOP * S && self.boss.is_none() && self.enemies.iter().all(|e| e.kind == Kind::Turret) {
            self.boss = Some(make_boss(self.stage));
            self.music.play(tune::boss());
        }
    }

    pub fn add_enemy(&mut self, kind: Kind, x: i32) {
        let (_, h) = size(kind);
        let (hp, _) = worth(kind);
        let mut e = Enemy { kind, x: x * S, y: -h * S, vx: 0, vy: 0, hp, t: 0, flash: 0, drop: None };
        match kind {
            Kind::Dart => e.vy = 24,
            Kind::SwoopL | Kind::SwoopR => e.vy = 22,
            Kind::Bomber => e.vy = 8,
            Kind::Carrier => {
                e.vy = 12;
                e.vx = 16;
                e.drop = Some(if self.drops % 3 == 2 { 1 } else { 0 });
                self.drops += 1;
            }
            Kind::Mine => {
                e.vy = 10;
                e.vx = if x < W as i32 / 2 { 4 } else { -4 };
            }
            Kind::Turret => {
                e.y = -h * S;
                // 每三个地面炮台掉一颗星。
                self.drops += 1;
                if self.drops.is_multiple_of(3) {
                    e.drop = Some(2);
                }
            }
        }
        if kind == Kind::Bomber {
            e.drop = Some(0);
        }
        self.enemies.push(e);
    }

    fn step_hero(&mut self) {
        if self.dead > 0 {
            self.dead -= 1;
            if self.dead == 0 {
                if self.lives <= 0 {
                    self.mode = Mode::Over;
                    self.t = 0;
                } else {
                    self.respawn();
                }
            }
            return;
        }
        let k = &self.keys;
        let dx = k.held(key::RIGHT) as i32 - k.held(key::LEFT) as i32;
        let dy = k.held(key::DOWN) as i32 - k.held(key::UP) as i32;
        // 斜着走不更快。
        let sp = if dx != 0 && dy != 0 { 23 } else { 32 };
        self.x = (self.x + dx * sp).clamp(-4 * S, (W as i32 - 12) * S);
        self.y = (self.y + dy * sp).clamp(12 * S, (H as i32 - 16) * S);
        self.invuln = self.invuln.saturating_sub(1);
        // 按住 A 连发，五帧一排。
        self.fire_t = self.fire_t.saturating_sub(1);
        if self.keys.held(key::A) && self.fire_t == 0 {
            self.fire();
            self.fire_t = 5;
        }
        if self.keys.pressed(key::B) && self.bombs > 0 && self.bomb_t == 0 {
            self.bombs -= 1;
            self.bomb_t = 70;
            self.invuln = self.invuln.max(90);
            tone_wav(&self.boom_wav);
        }
    }

    fn fire(&mut self) {
        let (cx, cy) = (self.x + 8 * S, self.y);
        let mut add = |dx: i32, vx: i32| self.shots.push(Shot { x: cx + dx * S, y: cy, vx, vy: -96 });
        add(-3, 0);
        add(3, 0);
        if self.power >= 2 {
            add(-6, -14);
            add(6, 14);
        }
        if self.power >= 3 {
            add(0, 0);
            add(-8, -26);
            add(8, 26);
        }
        if self.power >= 4 {
            add(-9, -44);
            add(9, 44);
        }
        if self.t.is_multiple_of(2) {
            self.music.sfx(Channel::Square2, 1760, 16, 8);
        }
    }

    /// 炸了以后再出来：从下面中间，无敌一会儿。
    fn respawn(&mut self) {
        self.x = (W as i32 / 2 - 8) * S;
        self.y = 130 * S;
        self.invuln = 150;
        self.dead = 0;
        self.bullets.clear();
    }

    /// 子弹、东西、火团往前走。
    fn move_things(&mut self) {
        for s in &mut self.shots {
            s.x += s.vx;
            s.y += s.vy;
        }
        self.shots.retain(|s| s.y > -8 * S && s.x > -8 * S && s.x < (W as i32 + 8) * S);
        for b in &mut self.bullets {
            b.x += b.vx;
            b.y += b.vy;
        }
        self.bullets.retain(|b| b.x > -8 * S && b.x < (W as i32 + 8) * S && b.y > -8 * S && b.y < (H as i32 + 8) * S);
        if self.bomb_t > 0 {
            self.bomb_t -= 1;
            self.bullets.clear();
        }
        for it in &mut self.items {
            it.t += 1;
            it.y += 6;
            it.x += if (it.t / 30) % 2 == 0 { 4 } else { -4 };
        }
        self.items.retain(|it| it.y < (H as i32 + 8) * S);
        for b in &mut self.booms {
            b.t += 1;
        }
        self.booms.retain(|b| b.t < 24);
    }

    /// 从一点朝主角打一颗子弹。
    fn aimed(&self, x: i32, y: i32, speed: i32) -> Bullet {
        let (cx, cy) = self.center();
        let (dx, dy) = ((cx * S - x) as f32, (cy * S - y) as f32);
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        Bullet { x, y, vx: (dx / len * speed as f32) as i32, vy: (dy / len * speed as f32) as i32 }
    }

    /// 朝一个角度打（弧度，0 朝右，往下是正）。
    fn angled(x: i32, y: i32, a: f32, speed: i32) -> Bullet {
        Bullet { x, y, vx: (a.cos() * speed as f32) as i32, vy: (a.sin() * speed as f32) as i32 }
    }

    fn bullet_speed(&self) -> i32 {
        26 + 3 * self.stage as i32
    }

    fn step_enemies(&mut self) {
        let sp = self.bullet_speed();
        let mut new = Vec::new();
        let mut bursts = Vec::new();
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies[i];
            e.t += 1;
            e.flash = e.flash.saturating_sub(1);
            let (w, h) = size(e.kind);
            let (mx, my) = (e.x + w * S / 2, e.y + h * S);
            match e.kind {
                Kind::Dart => {
                    // 左右晃着往下。
                    e.vx = ((e.t as f32 / 10.0).sin() * 12.0) as i32;
                }
                Kind::SwoopL | Kind::SwoopR => {
                    let side = if e.kind == Kind::SwoopL { 1 } else { -1 };
                    if e.y > 50 * S {
                        // 拐弯，往主角那边兜一圈再往上走。
                        e.vx = (e.vx + side * 2).clamp(-40, 40);
                        e.vy -= 1;
                    }
                    if e.t == 30 {
                        new.push((mx, my, 0));
                    }
                }
                Kind::Bomber => {
                    if e.y > 24 * S && e.t < 420 {
                        e.vy = 0;
                        e.vx = if (e.t / 120).is_multiple_of(2) { 6 } else { -6 };
                    } else if e.t >= 420 {
                        e.vy = 16;
                        e.vx = 0;
                    }
                    if e.t % 55 == 40 && e.y > 0 {
                        new.push((mx, my, 3));
                    }
                }
                Kind::Carrier => {
                    if e.t.is_multiple_of(40) {
                        e.vx = -e.vx;
                    }
                }
                Kind::Mine => {
                    if e.t >= 300 {
                        e.hp = 0;
                        bursts.push((mx, e.y + h * S / 2));
                    }
                }
                Kind::Turret => {
                    if e.t % 80 == 50 && e.y > 0 && e.y < 110 * S {
                        new.push((mx, e.y + 7 * S, 0));
                    }
                }
            }
            e.x += e.vx;
            e.y += e.vy;
        }
        for (x, y, n) in new {
            if n == 0 {
                let b = self.aimed(x, y, sp);
                self.bullets.push(b);
            } else {
                let base = self.aimed(x, y, sp);
                let a = (base.vy as f32).atan2(base.vx as f32);
                for k in 0..n {
                    let off = (k as f32 - (n - 1) as f32 / 2.0) * 0.28;
                    self.bullets.push(Self::angled(x, y, a + off, sp));
                }
            }
        }
        for (x, y) in bursts {
            self.burst(x, y, 8, sp - 6);
        }
        // 出了画面的不要了（往上走的俯冲机、到底的）。
        self.enemies.retain(|e| e.hp > 0 && e.y < (H as i32 + 24) * S && e.y > -60 * S && e.x > -48 * S && e.x < (W as i32 + 48) * S);
    }

    /// 一圈子弹。
    fn burst(&mut self, x: i32, y: i32, n: u32, sp: i32) {
        for k in 0..n {
            let a = k as f32 * std::f32::consts::TAU / n as f32;
            self.bullets.push(Self::angled(x, y, a, sp));
        }
        self.booms.push(Boom { x: x / S, y: y / S, r: 8, t: 0 });
    }

    /// 敌人被打掉：火团、分、掉东西。
    fn kill(&mut self, i: usize) {
        let e = self.enemies[i].clone();
        let (w, h) = size(e.kind);
        let (_, pts) = worth(e.kind);
        self.add_score(pts);
        let (cx, cy) = (e.x / S + w / 2, e.y / S + h / 2);
        let big = e.kind == Kind::Bomber;
        self.booms.push(Boom { x: cx, y: cy, r: if big { 18 } else { 9 }, t: 0 });
        if big {
            tone_wav(&self.boom_wav);
            self.booms.push(Boom { x: cx - 10, y: cy + 4, r: 10, t: 0 });
            self.booms.push(Boom { x: cx + 10, y: cy - 4, r: 10, t: 0 });
        } else {
            self.music.sfx(Channel::Noise, 900, 120, 35);
        }
        if e.kind == Kind::Mine {
            let sp = self.bullet_speed() - 6;
            self.burst(cx * S, cy * S, 8, sp);
        }
        if let Some(d) = e.drop {
            self.items.push(Item { kind: d, x: (cx - 5) * S, y: (cy - 5) * S, t: 0 });
        }
        self.enemies[i].hp = 0;
    }

    fn step_boss(&mut self) {
        let sp = self.bullet_speed();
        let stage = self.stage;
        let (cx, _) = self.center();
        let Some(mut b) = self.boss.take() else { return };
        b.t += 1;
        for p in &mut b.parts {
            p.flash = p.flash.saturating_sub(1);
        }
        if b.dying > 0 {
            b.dying += 1;
            if b.dying % 6 == 0 {
                let (x, y) = (b.x + self.rand(b.w as u32), b.y + self.rand(b.h as u32));
                let r = 10 + self.rand(10);
                self.booms.push(Boom { x, y, r, t: 0 });
                self.music.sfx(Channel::Noise, 600, 150, 40);
            }
            if b.dying == 30 || b.dying == 80 {
                tone_wav(&self.boom_wav);
            }
            if b.dying >= 140 {
                self.mode = Mode::Clear;
                self.t = 0;
                self.bullets.clear();
                return;
            }
            self.boss = Some(b);
            return;
        }
        // 先从上面开进来。
        // 停在分数那一行下面。
        let settle = 24;
        if b.y < settle {
            b.y += 1;
            self.boss = Some(b);
            return;
        }
        let t = b.t as f32;
        match stage {
            0 => b.x = (W as i32 - b.w) / 2 + ((t / 90.0).sin() * 40.0) as i32,
            1 => {
                // 沙蝎朝主角那边挪。
                let mid = b.x + b.w / 2;
                b.x += (cx - mid).signum() * if b.t % 2 == 0 { 1 } else { 0 };
                b.x = b.x.clamp(-8, W as i32 - b.w + 8);
            }
            2 => {
                b.x = (W as i32 - b.w) / 2 + ((t / 70.0).sin() * 50.0) as i32;
                b.y = settle + ((t / 45.0).sin() * 10.0) as i32 + 6;
            }
            _ => b.x = (W as i32 - b.w) / 2 + ((t / 150.0).sin() * 16.0) as i32,
        }
        let open = b.open();
        let mut out: Vec<Bullet> = Vec::new();
        let core_hurt = b.core().hp * 2 < core_hp(stage);
        for p in b.parts.iter().filter(|p| p.hp > 0) {
            let (x, y) = ((b.x + p.dx + p.w / 2) * S, (b.y + p.dy + p.h) * S);
            if !p.core {
                let every = [70, 60, 50, 45][stage];
                if b.t % every == (p.dx as u32 % 20) {
                    match stage {
                        0 => {
                            for k in 0..3 {
                                let mut a = self.aimed(x, y, sp);
                                a.vx += (k - 1) * 3;
                                out.push(a);
                            }
                        }
                        1 => {
                            let base = self.aimed(x, y, sp);
                            let a = (base.vy as f32).atan2(base.vx as f32);
                            for k in -1..=1 {
                                out.push(Self::angled(x, y, a + k as f32 * 0.3, sp));
                            }
                        }
                        _ => out.push(self.aimed(x, y, sp + 4)),
                    }
                }
            } else if !open {
                if b.t % 90 == 45 {
                    out.push(self.aimed(x, y, sp));
                }
            } else {
                match stage {
                    0 => {
                        if b.t % 50 == 0 {
                            for k in 0..7 {
                                out.push(Self::angled(x, y, 0.6 + k as f32 * 0.32, sp));
                            }
                        }
                    }
                    1 => {
                        if b.t % 6 == 0 {
                            b.spin += 0.37;
                            out.push(Self::angled(x, y, b.spin, sp - 4));
                            out.push(Self::angled(x, y, b.spin + std::f32::consts::PI, sp - 4));
                        }
                    }
                    2 => {
                        if b.t % 70 == 0 {
                            for k in 0..12 {
                                out.push(Self::angled(x, y, b.spin + k as f32 * std::f32::consts::TAU / 12.0, sp - 6));
                            }
                            b.spin += 0.13;
                        }
                        if b.t % 70 == 35 {
                            out.push(self.aimed(x, y, sp + 8));
                        }
                    }
                    _ => {
                        if b.t % 5 == 0 {
                            b.spin += if core_hurt { 0.29 } else { 0.21 };
                            for k in 0..if core_hurt { 3 } else { 2 } {
                                let a = b.spin + k as f32 * std::f32::consts::TAU / if core_hurt { 3.0 } else { 2.0 };
                                out.push(Self::angled(x, y, a, sp - 6));
                            }
                        }
                        if core_hurt && b.t % 80 == 0 {
                            for k in 0..16 {
                                out.push(Self::angled(x, y, k as f32 * std::f32::consts::TAU / 16.0, sp - 10));
                            }
                        }
                    }
                }
            }
        }
        self.bullets.extend(out);
        self.boss = Some(b);
    }

    /// 打中、挨打、捡东西。
    fn hits(&mut self) {
        // 主角的子弹打敌人。
        let mut i = 0;
        while i < self.shots.len() {
            let s = self.shots[i];
            let r = (s.x / S - 1, s.y / S - 4, 3, 8);
            let mut used = false;
            for j in 0..self.enemies.len() {
                let e = &self.enemies[j];
                if e.hp <= 0 || e.y < -4 * S {
                    continue;
                }
                let (w, h) = size(e.kind);
                if overlap(r, (e.x / S, e.y / S, w, h)) {
                    self.enemies[j].hp -= 1;
                    self.enemies[j].flash = 3;
                    if self.enemies[j].hp <= 0 {
                        self.kill(j);
                    }
                    used = true;
                    break;
                }
            }
            if !used {
                used = self.hit_boss(r, 1);
            }
            if used {
                self.shots.swap_remove(i);
            } else {
                i += 1;
            }
        }
        // 炸弹：一屏敌人一起挨。
        if self.bomb_t > 0 && self.bomb_t.is_multiple_of(4) {
            for j in 0..self.enemies.len() {
                if self.enemies[j].hp > 0 && self.enemies[j].y > -8 * S {
                    self.enemies[j].hp -= 3;
                    self.enemies[j].flash = 3;
                    if self.enemies[j].hp <= 0 {
                        self.kill(j);
                    }
                }
            }
            self.hit_boss((0, 0, W as i32, H as i32), 2);
        }
        self.enemies.retain(|e| e.hp > 0);
        if self.dead > 0 {
            return;
        }
        let (cx, cy) = self.center();
        // 捡东西：比挨打的那一小块大。
        let me = (cx - 7, cy - 7, 14, 14);
        let mut got = Vec::new();
        self.items.retain(|it| {
            let hit = overlap(me, (it.x / S, it.y / S, 10, 10));
            if hit {
                got.push(it.kind);
            }
            !hit
        });
        for k in got {
            match k {
                0 if self.power < 4 => self.power += 1,
                1 if self.bombs < 5 => self.bombs += 1,
                _ => self.add_score(1000),
            }
            self.music.sfx(Channel::Square2, 1319, 90, 40);
        }
        if self.invuln > 0 {
            return;
        }
        let core = (cx - 2, cy - 2, 4, 4);
        let shot = self.bullets.iter().any(|b| overlap(core, (b.x / S - 1, b.y / S - 1, 3, 3)));
        let rammed = self.enemies.iter().any(|e| e.kind != Kind::Turret && overlap(core, {
            let (w, h) = size(e.kind);
            (e.x / S + 1, e.y / S + 1, w - 2, h - 2)
        }));
        let boss = self.boss.as_ref().is_some_and(|b| b.dying == 0 && b.parts.iter().any(|p| p.hp > 0 && overlap(core, (b.x + p.dx, b.y + p.dy, p.w, p.h))));
        if shot || rammed || boss {
            self.die();
        }
    }

    /// 打大怪：打中一块就算用掉这颗子弹。芯没露出来时打不动，子弹也用掉。
    fn hit_boss(&mut self, r: (i32, i32, i32, i32), dmg: i32) -> bool {
        let Some(b) = self.boss.as_mut() else { return false };
        if b.dying > 0 || b.y < 0 {
            return false;
        }
        let open = b.open();
        let mut used = false;
        let mut killed = None;
        for (k, p) in b.parts.iter_mut().enumerate() {
            if p.hp <= 0 || !overlap(r, (b.x + p.dx, b.y + p.dy, p.w, p.h)) {
                continue;
            }
            used = true;
            if p.core && !open {
                continue;
            }
            p.hp -= dmg;
            p.flash = 3;
            if p.hp <= 0 {
                killed = Some(k);
            }
            if dmg == 1 {
                break;
            }
        }
        if let Some(k) = killed {
            let p = b.parts[k];
            let (x, y) = (b.x + p.dx + p.w / 2, b.y + p.dy + p.h / 2);
            if p.core {
                b.dying = 1;
                self.add_score(10000 * (self.stage as u32 + 1));
                self.bullets.clear();
            } else {
                self.add_score(1000);
            }
            self.booms.push(Boom { x, y, r: 14, t: 0 });
            tone_wav(&self.boom_wav);
        }
        used
    }

    pub fn die(&mut self) {
        let (cx, cy) = self.center();
        self.booms.push(Boom { x: cx, y: cy, r: 16, t: 0 });
        self.booms.push(Boom { x: cx - 6, y: cy + 6, r: 9, t: 0 });
        tone_wav(&self.boom_wav);
        self.lives -= 1;
        self.power = (self.power - 1).max(1);
        self.bombs = self.bombs.max(2);
        self.dead = 90;
        self.shots.clear();
    }

    // ---------- 画 ----------

    pub fn draw(&mut self) {
        match self.mode {
            Mode::Title => self.draw_title(),
            Mode::Ending => self.draw_ending(),
            _ => {
                self.draw_world();
                self.draw_hud();
                self.draw_banner();
            }
        }
    }

    fn cam(&self) -> Camera {
        Camera { x: 0, y: TOP - self.scroll / S }
    }

    fn draw_world(&mut self) {
        let base = STAGES[self.stage].base;
        let cam = self.cam();
        let f = &mut self.frame;
        f.clear(base);
        if self.stage == 2 {
            // 夜里：星星比云卷得慢。
            for i in 0..40u32 {
                let x = (i.wrapping_mul(97) % W as u32) as i32;
                let y = ((i.wrapping_mul(53) as i32 + self.scroll / S / 3) % H as i32 + H as i32) % H as i32;
                f.set(x, y, if i % 5 == 0 { YELLOW_HI } else { GRAY });
            }
        }
        self.map.draw(f, &self.tiles, &cam);
        // 地面炮台在下面，飞的在上面。
        for e in self.enemies.iter().filter(|e| e.kind == Kind::Turret) {
            let (x, y) = (e.x / S, e.y / S);
            let s = if e.flash > 0 { self.sprites.turret.recolor(&[(STEEL, FLASH), (STEEL_LO, FLASH)]) } else { self.sprites.turret.clone() };
            f.blit(&s, x, y, false);
            // 炮管朝着主角。
            let (cx, cy) = (self.x / S + 8, self.y / S + 8);
            let (dx, dy) = ((cx - x - 7) as f32, (cy - y - 7) as f32);
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            for k in 2..9 {
                let (px, py) = (x + 7 + (dx / len * k as f32) as i32, y + 7 + (dy / len * k as f32) as i32);
                f.rect(px, py, 2, 2, if k > 6 { INK } else { STEEL_HI });
            }
        }
        for it in &self.items {
            if (it.t / 4) % 2 == 0 || it.t < 200 {
                let (x, y) = (it.x / S, it.y / S);
                f.blit(&self.sprites.items[it.kind as usize], x, y, false);
                let label = ["P", "B", "*"][it.kind as usize];
                f.text(&FONT, x + 2, y - 1, label, WHITE);
            }
        }
        for e in self.enemies.iter().filter(|e| e.kind != Kind::Turret) {
            let sp = &self.sprites;
            let s = match e.kind {
                Kind::Dart => &sp.dart,
                Kind::SwoopL | Kind::SwoopR => &sp.swoop,
                Kind::Bomber => &sp.bomber,
                Kind::Carrier => &sp.carrier,
                Kind::Mine => &sp.mine[(e.t > 200 && (e.t / 6) % 2 == 0) as usize],
                Kind::Turret => unreachable!(),
            };
            if e.flash > 0 {
                let white: Vec<(u8, u8)> = (1..41).map(|c| (c, FLASH)).collect();
                f.blit(&s.recolor(&white), e.x / S, e.y / S, false);
            } else {
                f.blit(s, e.x / S, e.y / S, false);
            }
        }
        if let Some(b) = &self.boss {
            draw_boss(f, b, self.stage);
        }
        for s in &self.shots {
            let (x, y) = (s.x / S, s.y / S);
            f.rect(x - 1, y - 4, 3, 7, YELLOW);
            f.rect(x, y - 4, 1, 7, YELLOW_HI);
        }
        if self.dead == 0 && (self.invuln == 0 || (self.invuln / 3).is_multiple_of(2)) && self.y < (H as i32 + 4) * S {
            let flame = (self.t / 3).is_multiple_of(2);
            f.blit(&self.sprites.hero[flame as usize], self.x / S, self.y / S, false);
        }
        for b in &self.bullets {
            let (x, y) = (b.x / S, b.y / S);
            f.rect(x - 2, y - 1, 5, 3, PINK);
            f.rect(x - 1, y - 2, 3, 5, PINK);
            f.rect(x - 1, y - 1, 3, 3, PINK_HI);
        }
        for b in &self.booms {
            draw_boom(f, b);
        }
        if self.bomb_t > 0 {
            // 炸弹：一圈白光从主角那里散开。
            let r = (70 - self.bomb_t as i32) * 4;
            let (cx, cy) = (self.x / S + 8, self.y / S + 8);
            ring(f, cx, cy, r, if self.bomb_t % 4 < 2 { WHITE } else { YELLOW_HI });
            ring(f, cx, cy, r - 6, ORANGE);
        }
    }

    fn draw_hud(&mut self) {
        let f = &mut self.frame;
        f.text_outlined(&FONT, 3, 1, &format!("{:07}", self.score), WHITE, INK);
        f.text_outlined(&FONT, 92, 1, &lang().pick(format!("HI {:07}", self.hi), format!("最高 {:07}", self.hi)), YELLOW, INK);
        for i in 0..self.lives.clamp(0, 6) {
            f.blit(&self.sprites.life, 3 + i * 7, 15, false);
        }
        for i in 0..self.bombs.clamp(0, 5) {
            let x = W as i32 - 12 - i * 11;
            f.blit(&self.sprites.items[1], x, 2, false);
            f.text(&FONT, x + 2, 1, "B", WHITE);
        }
        // 火力：四格。
        for i in 0..4 {
            f.rect(W as i32 - 46 + i * 11, 15, 9, 3, if i < self.power { ORANGE } else { INK });
        }
        if let Some(b) = &self.boss {
            if b.dying == 0 && b.y >= 0 {
                let total: i32 = b.parts.iter().map(|p| p.hp.max(0)).sum();
                let full = boss_hp(self.stage);
                let w = 120 * total / full.max(1);
                // 大怪的血放在最下面，不挡大怪。
                f.rect(59, H as i32 - 7, 122, 5, INK);
                f.rect(60, H as i32 - 6, w, 3, if b.open() { RED_HI } else { RED });
            }
        }
    }

    /// 画面中间的字：关头、暂停、过关、接着打。
    fn draw_banner(&mut self) {
        let stage = self.stage;
        let f = &mut self.frame;
        let big = |f: &mut Frame, y: i32, s: &str, c: u8| {
            let sp = FONT.sprite(s, c).scaled(2);
            let edge = FONT.sprite(s, INK).scaled(2);
            let x = (W as i32 - sp.w) / 2;
            for (ox, oy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (1, 1)] {
                f.blit(&edge, x + ox, y + oy, false);
            }
            f.blit(&sp, x, y, false);
        };
        match self.mode {
            Mode::Intro => {
                big(f, 52, &lang().pick(format!("STAGE {}", stage + 1), format!("第{}关", ["一", "二", "三", "四"][stage])), YELLOW);
                let name = STAGES[stage].label();
                f.text_outlined(&FONT, (W as i32 - FONT.width(name)) / 2, 82, name, WHITE, INK);
            }
            Mode::Play if self.paused => big(f, 64, lang().pick("PAUSE", "暂停"), WHITE),
            Mode::Clear if self.t > 30 => {
                big(f, 48, lang().pick("STAGE CLEAR", "过关"), YELLOW);
                let bonus = self.bombs * 1000 + 5000;
                let line = lang().pick(format!("BOMB BONUS {bonus}"), format!("炸弹奖励 {bonus}"));
                f.text_outlined(&FONT, (W as i32 - FONT.width(&line)) / 2, 80, &line, WHITE, INK);
            }
            Mode::Over => {
                big(f, 44, lang().pick("SHOT DOWN", "被击落了"), RED_HI);
                let (a, b) = if self.continues > 0 {
                    let n = self.continues;
                    (
                        lang().pick(format!("CONTINUE? {n} LEFT"), format!("接着打吗？还能接 {n} 次")),
                        lang().pick("A YES   B NO", "A 接着打  B 不打了").to_string(),
                    )
                } else {
                    (
                        lang().pick("NO CONTINUES LEFT", "没有接着打的机会了").to_string(),
                        lang().pick("PRESS START", "按 START 回标题").to_string(),
                    )
                };
                if self.t > 40 {
                    f.text_outlined(&FONT, (W as i32 - FONT.width(&a)) / 2, 80, &a, WHITE, INK);
                    f.text_outlined(&FONT, (W as i32 - FONT.width(&b)) / 2, 100, &b, YELLOW, INK);
                }
            }
            _ => {}
        }
    }

    fn draw_title(&mut self) {
        let f = &mut self.frame;
        f.clear(NAVY);
        // 底下一片云，往下飘。
        for i in 0..60u32 {
            let x = (i.wrapping_mul(71) % W as u32) as i32;
            let y = ((i.wrapping_mul(37) + self.t / 2) % H as u32) as i32;
            f.set(x, y, if i % 4 == 0 { WHITE } else { BLUE_HI });
        }
        let name = lang().pick("SKYFIRE", "烈空战机");
        let title = FONT.sprite(name, RED_HI).scaled(3);
        let edge = FONT.sprite(name, INK).scaled(3);
        let x = (W as i32 - title.w) / 2;
        for (ox, oy) in [(-2, 0), (2, 0), (0, -2), (0, 2), (2, 2)] {
            f.blit(&edge, x + ox, 18 + oy, false);
        }
        f.blit(&title, x, 18, false);
        let bob = ((self.t as f32 / 20.0).sin() * 3.0) as i32;
        f.blit(&self.sprites.hero[((self.t / 3) % 2) as usize].scaled(2), W as i32 / 2 - 16, 64 + bob, false);
        if self.unlocked > 1 {
            let (n, name) = (self.pick, STAGES[self.pick].label());
            let s = lang().pick(format!("← STAGE {} {name} →", n + 1), format!("← 第{}关 {name} →", ["一", "二", "三", "四"][n]));
            f.text_outlined(&FONT, (W as i32 - FONT.width(&s)) / 2, 104, &s, WHITE, INK);
        }
        if (self.t / 30).is_multiple_of(2) {
            let s = lang().pick("PRESS START", "按 START 出击");
            f.text_outlined(&FONT, (W as i32 - FONT.width(s)) / 2, 122, s, YELLOW, INK);
        }
        let hi = lang().pick(format!("HI {:07}", self.hi), format!("最高 {:07}", self.hi));
        f.text_outlined(&FONT, (W as i32 - FONT.width(&hi)) / 2, 142, &hi, GRAY, INK);
    }

    fn draw_ending(&mut self) {
        let f = &mut self.frame;
        f.clear(SKY);
        for i in 0..50u32 {
            let x = (i.wrapping_mul(89) % W as u32) as i32;
            let y = ((i.wrapping_mul(41) + self.t / 3) % H as u32) as i32;
            f.set(x, y, if i % 3 == 0 { YELLOW_HI } else { GRAY });
        }
        let lines = lang().pick(
            ["The Sky Fortress has fallen", "Dawn breaks again", "", "Blaze flies home to the coast"],
            ["天穹要塞打下来了", "天又亮了", "", "烈风回到海岸线上的家"],
        );
        for (i, l) in lines.iter().enumerate() {
            if self.t > 30 * i as u32 {
                f.text_outlined(&FONT, (W as i32 - FONT.width(l)) / 2, 30 + i as i32 * 16, l, WHITE, INK);
            }
        }
        let s = lang().pick(format!("SCORE {:07}", self.score), format!("得分 {:07}", self.score));
        f.text_outlined(&FONT, (W as i32 - FONT.width(&s)) / 2, 104, &s, YELLOW, INK);
        if self.t > 120 {
            let s = lang().pick("THANKS FOR PLAYING", "谢谢你玩");
            f.text_outlined(&FONT, (W as i32 - FONT.width(s)) / 2, 128, s, RED_HI, INK);
        }
    }
}

fn disc(f: &mut Frame, cx: i32, cy: i32, r: i32, c: u8) {
    for dy in -r..=r {
        let half = ((r * r - dy * dy) as f32).sqrt() as i32;
        f.rect(cx - half, cy + dy, half * 2 + 1, 1, c);
    }
}

fn ring(f: &mut Frame, cx: i32, cy: i32, r: i32, c: u8) {
    if r <= 0 {
        return;
    }
    let n = (r * 7).max(12);
    for k in 0..n {
        let a = k as f32 * std::f32::consts::TAU / n as f32;
        f.rect(cx + (a.cos() * r as f32) as i32, cy + (a.sin() * r as f32) as i32, 2, 2, c);
    }
}

/// 一团火：先白，再黄、橙，最后一圈红慢慢散。
fn draw_boom(f: &mut Frame, b: &Boom) {
    let t = b.t as i32;
    let r = (b.r * (t + 4) / 12).min(b.r);
    match t {
        0..=3 => disc(f, b.x, b.y, r, WHITE),
        4..=8 => {
            disc(f, b.x, b.y, r, YELLOW);
            disc(f, b.x, b.y, r / 2, YELLOW_HI);
        }
        9..=14 => {
            disc(f, b.x, b.y, r, ORANGE);
            disc(f, b.x, b.y, r / 2, YELLOW);
        }
        _ => {
            ring(f, b.x, b.y, r, RED);
            ring(f, b.x, b.y, r - 3, RED_LO);
        }
    }
}

fn core_hp(stage: usize) -> i32 {
    [110, 150, 180, 260][stage]
}

fn boss_hp(stage: usize) -> i32 {
    make_boss(stage).parts.iter().map(|p| p.hp).sum()
}

/// 每关的大怪：多大，几块，芯在哪。
pub fn make_boss(stage: usize) -> Boss {
    let part = |dx, dy, w, h, hp, core| Part { dx, dy, w, h, hp, core, flash: 0 };
    let (w, h, parts) = match stage {
        // 铁鲸：一艘大战舰，两座炮塔，中间舰桥是芯。
        0 => (120, 52, vec![part(16, 24, 16, 14, 40, false), part(88, 24, 16, 14, 40, false), part(52, 8, 16, 22, core_hp(0), true)]),
        // 沙蝎：两只钳子，背上的炮是芯。
        1 => (112, 48, vec![part(0, 28, 22, 18, 55, false), part(90, 28, 22, 18, 55, false), part(48, 10, 16, 20, core_hp(1), true)]),
        // 夜鸮：一只大飞翼，两个发动机，中间的眼睛是芯。
        2 => (128, 36, vec![part(12, 16, 18, 14, 60, false), part(98, 16, 18, 14, 60, false), part(56, 6, 16, 16, core_hp(2), true)]),
        // 天穹要塞：四门炮，中间的反应堆是芯。
        _ => (
            144,
            60,
            vec![
                part(6, 34, 16, 14, 45, false),
                part(38, 44, 16, 14, 45, false),
                part(90, 44, 16, 14, 45, false),
                part(122, 34, 16, 14, 45, false),
                part(64, 12, 16, 22, core_hp(3), true),
            ],
        ),
    };
    Boss { x: (W as i32 - w) / 2, y: -h, w, h, t: 0, parts, dying: 0, spin: 0.0 }
}

/// 画大怪：身子用方块拼，每一块画成炮或者芯。
fn draw_boss(f: &mut Frame, b: &Boss, stage: usize) {
    let (x, y, w, h) = (b.x, b.y, b.w, b.h);
    match stage {
        0 => {
            // 船身：灰，船头朝下尖。
            f.rect(x + 8, y, w - 16, h - 12, STEEL_LO);
            f.rect(x + 10, y + 2, w - 20, h - 16, STEEL);
            for k in 0..12 {
                f.rect(x + 8 + k * 4, y + h - 12 + k, w - 16 - k * 8, 1, STEEL_LO);
            }
            f.rect(x + 14, y + 6, w - 28, 2, STEEL_HI);
            f.rect(x + 2, y + 10, 6, 18, STEEL_LO);
            f.rect(x + w - 8, y + 10, 6, 18, STEEL_LO);
        }
        1 => {
            // 沙蝎：橄榄绿的身子，两条履带，黑边，在沙地上看得清。
            f.rect(x + 9, y + 3, w - 18, h - 10, INK);
            f.rect(x + 10, y + 4, w - 20, h - 12, OLIVE_LO);
            f.rect(x + 14, y + 8, w - 28, h - 20, OLIVE);
            f.rect(x + 16, y + 10, w - 32, 2, OLIVE_HI);
            f.rect(x + 3, y - 1, 12, h - 2, INK);
            f.rect(x + w - 15, y - 1, 12, h - 2, INK);
            f.rect(x + 4, y, 10, h - 4, ROCK_LO);
            f.rect(x + w - 14, y, 10, h - 4, ROCK_LO);
            for k in 0..(h / 4) {
                let c = if (k + b.t as i32 / 4) % 2 == 0 { ROCK } else { ROCK_LO };
                f.rect(x + 5, y + k * 4, 8, 2, c);
                f.rect(x + w - 13, y + k * 4, 8, 2, c);
            }
        }
        2 => {
            // 飞翼：深紫，翅膀往两边斜下去。
            for k in 0..h {
                let half = w / 2 - k * w / (h * 3);
                f.rect(x + w / 2 - half, y + k, half * 2, 1, if k % 6 == 0 { PURPLE } else { NIGHT });
            }
            f.rect(x + w / 2 - 14, y, 28, h, PURPLE);
            f.rect(x + w / 2 - 10, y + 2, 20, h - 6, CLOUD);
        }
        _ => {
            // 要塞：一大块铁，边上一排灯。
            f.rect(x - 1, y + 5, w + 2, h - 4, INK);
            f.rect(x + 19, y - 1, w - 38, 12, INK);
            f.rect(x, y + 6, w, h - 6, STEEL_LO);
            f.rect(x + 4, y + 10, w - 8, h - 16, STEEL);
            f.rect(x + 20, y, w - 40, 10, STEEL_HI);
            for k in 0..8 {
                let on = (b.t / 8 + k).is_multiple_of(8);
                f.rect(x + 10 + k as i32 * 17, y + h - 6, 6, 3, if on { LAMP } else { RED_LO });
            }
        }
    }
    let open = b.open();
    for p in &b.parts {
        let (px, py) = (x + p.dx, y + p.dy);
        if p.hp <= 0 {
            // 打掉了：一个黑坑冒烟。
            f.rect(px + 2, py + 2, p.w - 4, p.h - 4, INK);
            if (b.t / 5).is_multiple_of(3) {
                f.rect(px + p.w / 2 - 1, py - 2, 3, 3, GRAY);
            }
            continue;
        }
        let hot = p.flash > 0;
        if p.core {
            let (c, hi) = if !open { (STEEL_LO, STEEL) } else if (b.t / 6).is_multiple_of(2) { (RED, RED_HI) } else { (RED_LO, RED) };
            f.rect(px, py, p.w, p.h, INK);
            f.rect(px + 1, py + 1, p.w - 2, p.h - 2, if hot { FLASH } else { c });
            f.rect(px + 4, py + 4, p.w - 8, p.h - 8, if hot { FLASH } else { hi });
            if !open {
                // 没露出来：两扇门关着。
                f.rect(px + p.w / 2, py + 1, 1, p.h - 2, INK);
            }
        } else {
            f.rect(px, py, p.w, p.h, INK);
            f.rect(px + 1, py + 1, p.w - 2, p.h - 2, if hot { FLASH } else { STEEL });
            f.rect(px + 3, py + 3, p.w - 6, p.h - 6, if hot { FLASH } else { STEEL_LO });
            f.rect(px + p.w / 2 - 1, py + p.h - 2, 3, 6, INK);
        }
    }
}

static GAME: Mutex<Option<Game>> = Mutex::new(None);

vibeplay::main!(|app| {
    app.every(|| {
        let mut g = GAME.lock().unwrap();
        let g = g.get_or_insert_with(Game::default);
        g.keys.read();
        g.step();
        g.draw();
        g.frame.show();
    });
});
