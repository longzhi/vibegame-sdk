//! 小 app《口算星星》：给 6 到 8 岁孩子的口算。
//!
//! 标题上用上下选难度（10 以内、20 以内的加减法），A 开始。一局十道题。
//! 三个答案在左、上、右，按十字键的那个方向回答。答对得一颗星，答错给一会儿看对的答案。
//! 做完看得了几颗星、每一档的最好成绩（存着），A 再来一次。
//! 随机数自己写，种子由「第几局、哪一档、在标题上等了几帧」算出来：同样的按键每次一样，每局的题不一样。

mod art;
pub mod rules;
#[cfg(test)]
mod tests;
mod tune;

use art::*;
use rules::{Level, Question, Rng, ROUND};
use std::sync::Mutex;
use vibeplay::full::{key, lang, Channel, Frame, Keys, Music, Sprite, W};
use vibeplay::FONT;

const SAVE: &str = "progress";
/// 答对以后停几帧再出下一题。
const RIGHT_FRAMES: u32 = 50;
/// 答错以后给几帧看对的答案。
const WRONG_FRAMES: u32 = 120;
/// 三个答案框的位置（左上角）。中间那个高一点：它是「上」。
const BOXES: [(i32, i32); 3] = [(18, 90), (94, 80), (170, 90)];
const BOX_W: i32 = 52;
const BOX_H: i32 = 44;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Title,
    Ask,
    Right,
    Wrong,
    End,
}

pub struct Game {
    pub frame: Frame,
    pub keys: Keys,
    music: Music,
    pub mode: Mode,
    pub level: Level,
    /// 模式里过了几帧。
    pub t: u32,
    /// 一共开过几局（存着，种子用它）。
    pub rounds: u32,
    /// 每一档的最好成绩，没做过是 None。
    pub best: [Option<u8>; 2],
    pub questions: Vec<Question>,
    /// 现在第几道题（0 起）。
    pub n: usize,
    /// 做过的题对不对。
    pub marks: Vec<bool>,
    /// 这道题按了哪一个。
    pub picked: usize,
    /// 这一局破了纪录。
    pub new_best: bool,
    star: Sprite,
    star_empty: Sprite,
    buddy: [Sprite; 2],
}

impl Default for Game {
    fn default() -> Self {
        let mut frame = Frame::default();
        for (i, rgb) in PALETTE {
            frame.palette[i as usize] = rgb;
        }
        let (best, rounds) = load();
        let mut g = Game {
            frame,
            keys: Keys::default(),
            music: Music::default(),
            mode: Mode::Title,
            level: Level::Ten,
            t: 0,
            rounds,
            best,
            questions: Vec::new(),
            n: 0,
            marks: Vec::new(),
            picked: 0,
            new_best: false,
            star: star(),
            star_empty: star_empty(),
            buddy: [buddy(false), buddy(true)],
        };
        g.music.play(tune::title());
        g
    }
}

/// 存档：「10 以内最好,20 以内最好,开过几局」，没做过的档写 -。
fn load() -> ([Option<u8>; 2], u32) {
    let s = vibeplay::load(SAVE).and_then(|b| String::from_utf8(b).ok()).unwrap_or_default();
    let mut it = s.split(',');
    let mut best = [None; 2];
    for b in &mut best {
        *b = it.next().and_then(|v| v.parse::<u8>().ok()).map(|v| v.min(ROUND as u8));
    }
    let rounds = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    (best, rounds)
}

impl Game {
    fn save(&self) {
        let b = |v: Option<u8>| v.map_or("-".to_string(), |v| v.to_string());
        vibeplay::save(SAVE, format!("{},{},{}", b(self.best[0]), b(self.best[1]), self.rounds).as_bytes());
    }

    /// 这一局得了几颗星。
    pub fn stars(&self) -> u8 {
        self.marks.iter().filter(|&&m| m).count() as u8
    }

    pub fn question(&self) -> &Question {
        &self.questions[self.n]
    }

    /// 开一局。`wait` 是在上一个画面停了几帧，掺进种子里。
    pub fn start(&mut self, wait: u32) {
        self.rounds = self.rounds.wrapping_add(1);
        self.save();
        let mut rng = Rng(rules::seed(self.rounds, self.level, wait));
        self.questions = rules::round(&mut rng, self.level);
        self.n = 0;
        self.marks.clear();
        self.new_best = false;
        self.mode = Mode::Ask;
        self.t = 0;
        self.music.stop();
    }

    /// 回答：0 左、1 上、2 右。
    pub fn answer(&mut self, i: usize) {
        self.picked = i;
        let ok = i == self.question().right;
        self.marks.push(ok);
        self.mode = if ok { Mode::Right } else { Mode::Wrong };
        self.t = 0;
        self.music.play(if ok { tune::right() } else { tune::wrong() });
    }

    fn next(&mut self) {
        self.n += 1;
        self.t = 0;
        if self.n < ROUND {
            self.mode = Mode::Ask;
            return;
        }
        let stars = self.stars();
        let i = self.level.index();
        self.new_best = self.best[i].is_none_or(|b| stars > b);
        self.best[i] = rules::better(self.best[i], stars);
        self.save();
        self.mode = Mode::End;
        self.music.play(tune::done(stars));
    }

    fn back_to_title(&mut self) {
        self.mode = Mode::Title;
        self.t = 0;
        self.music.play(tune::title());
    }

    /// 走一帧。
    pub fn step(&mut self) {
        self.t += 1;
        let k = &self.keys;
        match self.mode {
            Mode::Title => {
                let pick = if k.pressed(key::UP) {
                    Some(Level::Ten)
                } else if k.pressed(key::DOWN) {
                    Some(Level::Twenty)
                } else {
                    None
                };
                if let Some(l) = pick {
                    if l != self.level {
                        self.level = l;
                        self.music.sfx(Channel::Square2, 880, 40, 24);
                    }
                }
                if k.pressed(key::A) || k.pressed(key::START) {
                    self.start(self.t);
                }
            }
            Mode::Ask => {
                if k.pressed(key::LEFT) {
                    self.answer(0);
                } else if k.pressed(key::UP) {
                    self.answer(1);
                } else if k.pressed(key::RIGHT) {
                    self.answer(2);
                } else if k.pressed(key::B) {
                    self.back_to_title();
                }
            }
            Mode::Right => {
                if self.t >= RIGHT_FRAMES || (self.t > 15 && k.pressed(key::A)) {
                    self.next();
                }
            }
            Mode::Wrong => {
                if self.t >= WRONG_FRAMES || (self.t > 30 && k.pressed(key::A)) {
                    self.next();
                }
            }
            Mode::End => {
                if self.t > 30 && (k.pressed(key::A) || k.pressed(key::START)) {
                    self.start(self.t);
                } else if k.pressed(key::B) {
                    self.back_to_title();
                }
            }
        }
        self.music.tick();
    }

    // ---------- 画 ----------

    pub fn draw(&mut self) {
        self.draw_sky();
        match self.mode {
            Mode::Title => self.draw_title(),
            Mode::Ask | Mode::Right | Mode::Wrong => self.draw_question(),
            Mode::End => self.draw_end(),
        }
    }

    /// 背景：夜空、一闪一闪的小星、底下一道小山。
    fn draw_sky(&mut self) {
        let f = &mut self.frame;
        f.clear(SKY);
        for y in 0..40 {
            if y % 4 == 0 {
                f.rect(0, y, W as i32, 1, SKY_HI);
            }
        }
        for i in 0..28u32 {
            let x = (i.wrapping_mul(83) % W as u32) as i32;
            let y = (i.wrapping_mul(47) % 140) as i32;
            let on = !(self.t / 20 + i).is_multiple_of(5);
            f.set(x, y, if on && i % 3 == 0 { YELLOW_HI } else if on { GRAY } else { SKY });
        }
        for x in 0..W as i32 {
            let h = 6 + ((x as f32 / 23.0).sin() * 3.0) as i32;
            f.rect(x, 160 - h, 1, h, HILL);
            f.set(x, 160 - h, HILL_HI);
        }
    }

    fn draw_title(&mut self) {
        let t = self.t;
        let f = &mut self.frame;
        big(f, 14, lang().pick("MATH STAR", "口算星星"), 3, YELLOW);
        let bob = if (t / 20).is_multiple_of(2) { 0 } else { -2 };
        let blink = t % 150 > 140;
        f.blit(&self.buddy[blink as usize].scaled(2), W as i32 / 2 - 15, 52 + bob, false);
        for (i, l) in Level::ALL.iter().enumerate() {
            let y = 92 + i as i32 * 20;
            let on = *l == self.level;
            let name = lang().pick(format!("WITHIN {}", l.max()), format!("{} 以内", l.max()));
            let best = match self.best[i] {
                Some(b) => lang().pick(format!("BEST {b}"), format!("最好 {b}")),
                None => "".to_string(),
            };
            if on {
                f.rect(44, y - 3, 152, 18, PURPLE);
                f.frame_rect(44, y - 3, 152, 18, YELLOW);
                f.blit(&self.star, 50, y, false);
            }
            f.text_outlined(&FONT, 64, y, &name, WHITE, INK);
            let bw = FONT.width(&best);
            f.text_outlined(&FONT, 188 - bw, y, &best, if on { YELLOW } else { GRAY }, INK);
        }
        hint(f, lang().pick("↑↓ LEVEL    A START", "↑↓ 选难度   A 开始"));
    }

    fn draw_progress(&mut self) {
        let f = &mut self.frame;
        let x0 = (W as i32 - ROUND as i32 * 12) / 2;
        for i in 0..ROUND {
            let x = x0 + i as i32 * 12 + 1;
            let lit = self.marks.get(i) == Some(&true);
            // 答对时那颗星在这一题的格子里蹦一下。
            let pop = lit && i == self.n && self.mode == Mode::Right && self.t < 12;
            f.blit(if lit { &self.star } else { &self.star_empty }, x, if pop { 2 } else { 4 }, false);
            if i == self.n && self.mode == Mode::Ask {
                f.rect(x + 2, 15, 5, 2, WHITE);
            }
        }
    }

    fn draw_question(&mut self) {
        self.draw_progress();
        let q = *self.question();
        let (mode, t, picked) = (self.mode, self.t, self.picked);
        let f = &mut self.frame;
        // 题：「7 + 5 = ?」，答错时问号换成对的答案。
        let left = format!("{} ", q.left());
        let (tail, tail_c) = match mode {
            Mode::Wrong => (q.answer().to_string(), GREEN_HI),
            Mode::Right => (q.answer().to_string(), YELLOW),
            _ => ("?".to_string(), YELLOW),
        };
        let wl = (numbers(&left, INK).w + 1) * 4;
        let x = (W as i32 - wl - numbers(&tail, INK).w * 4) / 2;
        big_num(f, x, 26, &left, 4, WHITE);
        big_num(f, x + wl, 26, &tail, 4, tail_c);
        // 三个答案框。
        let arrows = ["←", "↑", "→"];
        for (i, &(bx, by)) in BOXES.iter().enumerate() {
            let is_right = i == q.right;
            let (fill, ink) = match mode {
                Mode::Right if i == picked => (GREEN, WHITE),
                Mode::Wrong if i == picked => (RED, WHITE),
                // 答错时对的那个一闪一闪。
                Mode::Wrong if is_right && (t / 10).is_multiple_of(2) => (GREEN, WHITE),
                Mode::Right | Mode::Wrong => (SKY_HI, GRAY),
                _ => (WHITE, INK),
            };
            bubble(f, bx, by, BOX_W, BOX_H, fill);
            let s = q.choices[i].to_string();
            let sp = numbers(&s, ink).scaled(3);
            f.blit(&sp, bx + (BOX_W - sp.w) / 2, by + 4, false);
            let a = arrows[i];
            f.text(&FONT, bx + (BOX_W - FONT.width(a)) / 2, by + 30, a, if fill == WHITE { PURPLE } else { ink });
        }
        match mode {
            Mode::Right => {
                // 选对的框上蹦出一颗大星。
                let (bx, by) = BOXES[picked];
                let rise = (t as i32).min(16);
                let sp = self.star.scaled(2);
                f.blit(&sp, bx + (BOX_W - sp.w) / 2, by - 4 - rise, false);
                let s = lang().pick("GREAT!", "答对了！");
                f.text_outlined(&FONT, (W as i32 - FONT.width(s)) / 2, 66, s, YELLOW, INK);
            }
            Mode::Wrong => {
                let s = lang().pick("NICE TRY!", "差一点！");
                f.text_outlined(&FONT, (W as i32 - FONT.width(s)) / 2, 66, s, RED_HI, INK);
            }
            _ => {}
        }
        let h = match mode {
            Mode::Ask => lang().pick("←↑→ ANSWER    B MENU", "←↑→ 选答案   B 回菜单"),
            _ => lang().pick("A NEXT", "A 下一题"),
        };
        hint(f, h);
    }

    fn draw_end(&mut self) {
        let stars = self.stars();
        let t = self.t;
        let f = &mut self.frame;
        let msg = match stars {
            10 => lang().pick("PERFECT!", "全对！真棒！"),
            7..=9 => lang().pick("GREAT JOB!", "真厉害！"),
            4..=6 => lang().pick("GOOD WORK!", "做得好！"),
            _ => lang().pick("KEEP PRACTICING!", "多练练就会了！"),
        };
        let mw = FONT.width(msg) * 2;
        big_at(f, (W as i32 - mw) / 2, 6, msg, 2, YELLOW);
        // 一排星：得到的一颗一颗亮起来。
        let x0 = (W as i32 - 10 * 22) / 2;
        for i in 0..ROUND {
            let lit = (i as u8) < stars && t > 10 + i as u32 * 6;
            let sp = if lit { self.star.scaled(2) } else { self.star_empty.scaled(2) };
            f.blit(&sp, x0 + i as i32 * 22 + 2, 38, false);
        }
        let score = format!("{stars} / {}", ROUND);
        let sw = numbers(&score, INK).w * 3;
        big_num(f, (W as i32 - sw) / 2, 66, &score, 3, WHITE);
        for (i, l) in Level::ALL.iter().enumerate() {
            let best = self.best[i].map_or("-".to_string(), |b| format!("{b} / {}", ROUND));
            let s = lang().pick(format!("WITHIN {}   BEST {best}", l.max()), format!("{} 以内   最好 {best}", l.max()));
            let c = if *l == self.level { WHITE } else { GRAY };
            f.text_outlined(&FONT, (W as i32 - FONT.width(&s)) / 2, 102 + i as i32 * 14, &s, c, INK);
        }
        if self.new_best && (t / 20).is_multiple_of(2) {
            let s = lang().pick("NEW BEST!", "新纪录！");
            f.text_outlined(&FONT, 176, 76, s, PINK, INK);
        }
        hint(f, lang().pick("A PLAY AGAIN    B MENU", "A 再来一次   B 回菜单"));
    }
}

/// 一行大字，横着放中间，带一圈黑边。
fn big(f: &mut Frame, y: i32, s: &str, k: i32, c: u8) {
    let w = FONT.width(s) * k;
    big_at(f, (W as i32 - w) / 2, y, s, k, c);
}

fn big_at(f: &mut Frame, x: i32, y: i32, s: &str, k: i32, c: u8) {
    let edge = FONT.sprite(s, INK).scaled(k);
    for (ox, oy) in [(-2, 0), (2, 0), (0, -2), (0, 2), (2, 2)] {
        f.blit(&edge, x + ox, y + oy, false);
    }
    f.blit(&FONT.sprite(s, c).scaled(k), x, y, false);
}

/// 一串大数字（自己画的那套），带一圈黑边。
fn big_num(f: &mut Frame, x: i32, y: i32, s: &str, k: i32, c: u8) {
    let edge = numbers(s, INK).scaled(k);
    for (ox, oy) in [(-2, 0), (2, 0), (0, -2), (0, 2), (2, 2)] {
        f.blit(&edge, x + ox, y + oy, false);
    }
    f.blit(&numbers(s, c).scaled(k), x, y, false);
}

/// 圆角的框，黑边。
fn bubble(f: &mut Frame, x: i32, y: i32, w: i32, h: i32, c: u8) {
    f.rect(x + 2, y, w - 4, h, INK);
    f.rect(x, y + 2, w, h - 4, INK);
    f.rect(x + 1, y + 1, w - 2, h - 2, INK);
    f.rect(x + 2, y + 2, w - 4, h - 4, c);
    f.rect(x + 1, y + 3, w - 2, h - 6, c);
    f.rect(x + 3, y + 1, w - 6, h - 2, c);
}

/// 底下一行小字：这一屏能按什么。
fn hint(f: &mut Frame, s: &str) {
    f.text_outlined(&FONT, (W as i32 - FONT.width(s)) / 2, 146, s, WHITE, INK);
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
