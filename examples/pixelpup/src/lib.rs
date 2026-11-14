//! 小 app《像素小狗》：一只随真的时间饿的电子宠物。
//!
//! A 喂饭（吃饭的动画和声音），B 遛狗（走一小段，开心涨）。夜里（本地 22:00 到 7:00）小狗睡觉，屋里暗，冒 Zzz。
//! 睡着时按 A、B，小狗哼一声接着睡，什么都不变。
//! 饱和开心随真的时间往下掉。存档记下算到了哪一刻；下次打开时把没开的时候补上，最多补 12 小时。
//! 规则都在 `pet.rs`，收 `now`（Unix 秒），不读钟。读钟只在 `step()` 一个地方。

mod art;
pub mod pet;
#[cfg(test)]
mod tests;

use art::*;
use pet::{is_night, Mood, Pet};
use std::sync::Mutex;
use vibeplay::full::{key, lang, tone, Channel, Frame, Keys, Sprite, W};
use vibeplay::{Time, FONT};

const SAVE: &str = "pup";
/// 吃饭的动画几帧。
const EAT: u32 = 150;
/// 遛狗几帧：五秒。
const WALK: u32 = 300;
/// 睡着被叫醒，哼一声几帧。
const GRUMPY: u32 = 120;
/// 「你离开了多久」挂几帧。
const AWAY: u32 = 240;
/// 屋里的小狗放大几倍，左上角在哪。
const K: i32 = 3;
const DOG_X: i32 = 84;
const DOG_Y: i32 = 54;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Home,
    Eat,
    Walk,
    /// 睡着被叫，哼一声。
    Grumpy,
}

pub struct Game {
    pub frame: Frame,
    pub keys: Keys,
    pub pet: Pet,
    pub mode: Mode,
    /// 这个模式开始后过了几帧。
    pub t: u32,
    /// 开了以后过了几帧：动画用。
    tick: u32,
    /// 这一帧的钟。
    pub now: Time,
    /// 打开时离开了多少秒，和还要挂几帧。
    pub away: (i64, u32),
}

impl Default for Game {
    fn default() -> Self {
        Game::new(vibeplay::now())
    }
}

impl Game {
    /// 读存档，补上没开的时候过的时间。没有存档就领养一只新的。
    pub fn new(now: Time) -> Game {
        let mut frame = Frame::default();
        for (i, rgb) in PALETTE {
            frame.palette[i as usize] = rgb;
        }
        let saved = vibeplay::load(SAVE).and_then(|b| String::from_utf8(b).ok()).and_then(|s| Pet::decode(&s));
        let mut pet = saved.unwrap_or_else(|| Pet::new(now.unix));
        let gap = pet.catch_up(now.unix);
        let g = Game {
            frame,
            keys: Keys::default(),
            pet,
            mode: Mode::Home,
            t: 0,
            tick: 0,
            now,
            away: (gap, if gap >= 3600 { AWAY } else { 0 }),
        };
        g.save();
        g
    }

    fn save(&self) {
        vibeplay::save(SAVE, self.pet.encode().as_bytes());
    }

    pub fn step(&mut self) {
        self.step_at(vibeplay::now());
    }

    /// 走一帧。钟从参数进来，测试里能给任何时间。
    pub fn step_at(&mut self, now: Time) {
        self.now = now;
        self.tick += 1;
        self.t += 1;
        // 每过一整分钟规则往前走一步，存一次。
        let last = self.pet.last;
        self.pet.catch_up(now.unix);
        if self.pet.last != last {
            self.save();
        }
        let any = self.keys.pressed(key::A) || self.keys.pressed(key::B);
        if self.away.1 > 0 {
            self.away.1 -= 1;
            if any {
                // 先按一下把「离开了多久」关掉，这一下不算喂饭遛狗。
                self.away.1 = 0;
                return;
            }
        }
        let night = is_night(now.hour);
        match self.mode {
            Mode::Home => {
                if any && night {
                    self.enter(Mode::Grumpy);
                } else if self.keys.pressed(key::A) {
                    self.pet.feed(now.unix);
                    self.save();
                    self.enter(Mode::Eat);
                } else if self.keys.pressed(key::B) {
                    self.pet.walk(now.unix);
                    self.save();
                    self.enter(Mode::Walk);
                }
            }
            Mode::Eat => {
                let t = self.t;
                if t < 120 && t % 18 == 1 {
                    tone(Channel::Noise, 1800, 50, 30);
                }
                if t == 125 {
                    tone(Channel::Square2, 1046, 80, 35);
                }
                if t == 131 {
                    tone(Channel::Square2, 1318, 140, 35);
                }
                if t >= EAT {
                    self.enter(Mode::Home);
                }
            }
            Mode::Walk => {
                let t = self.t;
                if t == 1 {
                    tone(Channel::Square2, 700, 60, 35);
                }
                if t == 8 {
                    tone(Channel::Square2, 900, 90, 35);
                }
                if t < 240 && t.is_multiple_of(16) {
                    tone(Channel::Triangle, 196, 40, 25);
                }
                if t == 240 {
                    tone(Channel::Square2, 1175, 120, 35);
                }
                if t >= WALK || (t > 10 && self.keys.pressed(key::B)) {
                    self.enter(Mode::Home);
                }
            }
            Mode::Grumpy => {
                if self.t == 1 {
                    tone(Channel::Square1, 160, 250, 30);
                }
                if self.t == 20 {
                    tone(Channel::Square1, 130, 300, 30);
                }
                if self.t >= GRUMPY {
                    self.enter(Mode::Home);
                }
            }
        }
    }

    fn enter(&mut self, m: Mode) {
        self.mode = m;
        self.t = 0;
    }

    pub fn draw(&mut self) {
        let night = is_night(self.now.hour);
        match self.mode {
            Mode::Walk => self.draw_walk(),
            _ => self.draw_room(night),
        }
        self.draw_top();
        self.draw_meters(night);
        if self.away.1 > 0 {
            self.draw_away();
        }
    }

    fn draw_top(&mut self) {
        let (days, now) = (self.pet.age_days(self.now.unix), self.now);
        let f = &mut self.frame;
        f.rect(0, 0, W as i32, 14, PANEL);
        let age = match (lang().pick(0, 1), days) {
            (0, 1) => "1 day old".to_string(),
            (0, d) => format!("{d} days old"),
            (_, d) => format!("{d} 天大"),
        };
        f.text(&FONT, 4, 1, &age, WHITE);
        let clock = format!("{:02}:{:02}", now.hour, now.minute);
        f.text(&FONT, W as i32 - 4 - FONT.width(&clock), 1, &clock, WHITE);
    }

    fn draw_meters(&mut self, night: bool) {
        let (food, fun, mood) = (self.pet.food, self.pet.fun, self.pet.mood());
        let blink = (self.tick / 20).is_multiple_of(2);
        let mode = self.mode;
        let f = &mut self.frame;
        f.rect(0, 118, W as i32, 42, PANEL);
        let bar = |f: &mut Frame, y: i32, label: &str, v: i64, c: u8| {
            f.text(&FONT, 6, y, label, WHITE);
            f.rect(44, y + 3, 102, 8, INK);
            f.rect(45, y + 4, 100, 6, BAR_BG);
            let w = (v * 100 / pet::MAX) as i32;
            // 低了一闪一闪：橙、粉和红换着。
            let c = if v < pet::LOW && blink { RED } else { c };
            f.rect(45, y + 4, w.max(1), 6, c);
        };
        bar(f, 120, lang().pick("FOOD", "饱"), food, FOOD);
        bar(f, 133, lang().pick("FUN", "乐"), fun, FUN);
        let (text, c) = if night {
            (lang().pick("Asleep", "睡着了"), ZZZ)
        } else {
            match mood {
                Mood::Hungry => (lang().pick("Hungry!", "饿了！"), FOOD),
                Mood::Bored => (lang().pick("Bored", "好无聊"), FUN),
                Mood::Happy => (lang().pick("Happy", "开心"), WHITE),
            }
        };
        f.text(&FONT, 156, 126, text, c);
        let hint = match mode {
            Mode::Eat => lang().pick("Munch munch...", "吧唧吧唧……"),
            Mode::Walk => lang().pick("Walking  B: go home", "遛弯中  B 回家"),
            Mode::Grumpy => lang().pick("Shh... it is night", "嘘……现在是夜里"),
            Mode::Home if night => lang().pick("Sleeping until 7:00", "睡到早上 7 点"),
            Mode::Home => lang().pick("A: feed   B: walk", "A 喂饭   B 遛狗"),
        };
        f.text_center(&FONT, 147, hint, GRAY);
    }

    fn draw_room(&mut self, night: bool) {
        let (mode, t, tick, food) = (self.mode, self.t, self.tick, self.pet.food);
        let f = &mut self.frame;
        let (wall, wall2, floor, floor2) = if night { (WALL_N, WALL2_N, FLOOR_N, FLOOR2_N) } else { (WALL, WALL2, FLOOR, FLOOR2) };
        // 墙：竖条纹。
        f.rect(0, 14, W as i32, 78, wall);
        for x in (0..W as i32).step_by(16) {
            f.rect(x, 14, 8, 78, wall2);
        }
        // 地板：一条一条的木板。
        f.rect(0, 92, W as i32, 26, floor);
        for y in [99, 107] {
            f.rect(0, y, W as i32, 1, floor2);
        }
        for (i, y) in [92, 100, 108].into_iter().enumerate() {
            for x in (i as i32 * 23..W as i32).step_by(61) {
                f.rect(x, y, 1, 7, floor2);
            }
        }
        f.rect(0, 91, W as i32, 1, INK);
        // 窗：白天蓝天白云，夜里月亮星星。
        f.rect(16, 24, 48, 40, FRAME);
        f.rect(19, 27, 42, 34, if night { NIGHT } else { SKY });
        if night {
            f.blit(&moon(), 48, 30, false);
            for (x, y) in [(24, 33), (34, 30), (28, 50), (40, 44), (55, 54)] {
                if (tick / 30 + x as u32).is_multiple_of(3) {
                    continue;
                }
                f.set(x, y, WHITE);
            }
        } else {
            let cx = 19 + ((tick / 20) as i32 % 50) - 8;
            for (dx, dy, w) in [(0, 2, 12), (3, 0, 6)] {
                for x in cx + dx..cx + dx + w {
                    if (19..61).contains(&x) {
                        f.rect(x, 37 + dy, 1, 3, WHITE);
                    }
                }
            }
        }
        f.rect(39, 27, 2, 34, FRAME);
        f.rect(19, 43, 42, 2, FRAME);
        // 窗下的垫子：夜里小狗在上面睡。
        f.rect(70, 100, 92, 10, if night { CUSHION_N } else { CUSHION });
        f.rect(72, 98, 88, 2, if night { CUSHION_N } else { CUSHION });

        if night || mode == Mode::Grumpy {
            let dim = |s: Sprite| s.recolor(&DIM).scaled(K);
            let (x, y) = (DOG_X - 6, 84);
            f.blit(&dim(lying()), x, y, false);
            if mode == Mode::Grumpy {
                // 抬起头，睁一只眼，哼一声。
                f.blit(&dim(head(true)), x + 13 * K, y - 4 * K, false);
                bubble(f, x + 72, 52, lang().pick("Grr...", "哼……"));
            } else {
                f.blit(&dim(head(false)), x + 13 * K, y - K, false);
                // Zzz 从头上往上飘。
                for i in 0..3u32 {
                    let p = (tick + i * 40) % 120;
                    let zy = 80 - p as i32 / 2;
                    let zx = x + 66 + i as i32 * 4 + (p as i32 / 20) % 2;
                    let s = if i == 1 { "Z" } else { "z" };
                    f.text(&FONT, zx, zy, s, ZZZ);
                }
            }
            return;
        }

        // 饭碗：剩的饭跟着饱不饱，吃饭时一口一口变少。
        let left = if mode == Mode::Eat { 3 - (t * 4 / EAT).min(3) } else { (food / 34_000).min(3) as u32 };
        let bowl = bowl(left).scaled(2);
        if mode == Mode::Eat {
            f.blit(&bowl, DOG_X + 54, 88, false);
            let bob = if (t / 9).is_multiple_of(2) { 0 } else { 1 };
            f.blit(&body_eat().scaled(K), DOG_X, DOG_Y + 6 * K, false);
            f.blit(&head(true).scaled(K), DOG_X + 14 * K, DOG_Y + (7 + bob) * K, false);
            if t > 125 {
                f.blit(&heart().scaled(2), DOG_X + 30, DOG_Y + 2 - (t as i32 - 125) / 4, false);
            }
        } else {
            f.blit(&bowl, 176, 88, false);
            let wag = (tick / 10).is_multiple_of(2);
            let blinking = tick % 180 < 8;
            f.blit(&body(wag).scaled(K), DOG_X, DOG_Y + 6 * K, false);
            f.blit(&head(!blinking).scaled(K), DOG_X + 8 * K, DOG_Y, false);
        }
    }

    fn draw_walk(&mut self) {
        let (t, tick) = (self.t as i32, self.tick);
        let f = &mut self.frame;
        f.rect(0, 14, W as i32, 104, SKY);
        // 云最慢，树中间，草最快：远的走得慢。
        for i in 0..3 {
            let x = (i * 97 - t / 4).rem_euclid(W as i32 + 40) - 20;
            let y = 24 + i * 9;
            f.rect(x, y, 22, 4, WHITE);
            f.rect(x + 5, y - 3, 10, 3, WHITE);
        }
        let tr = tree().scaled(2);
        for i in 0..3 {
            let x = (i * 90 + 30 - t).rem_euclid(W as i32 + 30) - 26;
            f.blit(&tr, x, 74 - (i % 2) * 6, false);
        }
        f.rect(0, 94, W as i32, 24, GRASS);
        f.rect(0, 102, W as i32, 10, PATH);
        for i in 0..12 {
            let x = (i * 23 - t * 2).rem_euclid(W as i32 + 10) - 5;
            f.rect(x, 96 + (i % 3) * 2, 1, 3, GRASS2);
            f.rect(x + 2, 95 + (i % 3) * 2, 1, 4, GRASS2);
            f.rect(x + 7, 113 + i % 2, 3, 1, GRASS2);
        }
        // 小狗朝右走：两张身子换着，头一点一点。
        let k = K;
        let (x, y) = (80, 110 - 16 * k);
        let step = (tick / 8).is_multiple_of(2);
        let b = if step { body_step() } else { body(true) };
        f.blit(&b.scaled(k), x, y + 6 * k, false);
        f.blit(&head(true).scaled(k), x + 10 * k, y + 3 * k + if step { k } else { 0 }, false);
        if t > 240 {
            f.blit(&heart().scaled(2), x + 50, y - 4 - (t - 240) / 6, false);
        }
    }

    fn draw_away(&mut self) {
        let (gap, mood) = (self.away.0, self.pet.mood());
        let f = &mut self.frame;
        let (h, m, d) = (gap / 3600, gap / 60 % 60, gap / 86_400);
        let line1 = match (lang().pick(0, 1), d >= 2) {
            (0, true) => format!("Away {d} days"),
            (0, false) => format!("Away {h}h {m}m"),
            (_, true) => format!("离开了 {d} 天"),
            (_, false) => format!("离开了 {h} 小时 {m} 分"),
        };
        let line2 = if mood == Mood::Hungry { lang().pick("Pup got hungry!", "小狗饿了！") } else { lang().pick("Pup missed you!", "小狗想你了！") };
        f.rect(40, 18, 160, 30, INK);
        f.frame_rect(40, 18, 160, 30, FRAME);
        f.text_center(&FONT, 21, &line1, WHITE);
        f.text_center(&FONT, 34, line2, FOOD);
    }
}

/// 一个对话框，左下角有个尖。
fn bubble(f: &mut Frame, x: i32, y: i32, s: &str) {
    let w = FONT.width(s) + 8;
    f.rect(x, y, w, 16, WHITE);
    f.frame_rect(x, y, w, 16, INK);
    f.rect(x + 2, y + 16, 3, 2, WHITE);
    f.set(x + 1, y + 18, WHITE);
    f.text(&FONT, x + 4, y + 2, s, INK);
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
