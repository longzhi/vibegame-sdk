//! 小 app《记分板》：聚会、打牌时记分。二到六个人，每人一个颜色、一个小图标。
//!
//! 上下给现在这个人加减一分，按住连着加，越按越快。左右选人，A 换下一个人，B 撤销上一次改的分。
//! SELECT 掷骰子，START 开菜单（新的一局、几个人）。分一变就存，合盖、没电也不丢。
//! 随机数自己写：同样的按键每次跑出来一样。

mod art;
#[cfg(test)]
mod tests;

use art::*;
use std::sync::Mutex;
use vibeplay::full::{key, lang, tone, Channel, Frame, Keys, Sprite, W};
use vibeplay::FONT;

const SAVE: &str = "scores";
/// 分的上下限：牌局里能扣成负的。
const MIN_SCORE: i32 = -999;
const MAX_SCORE: i32 = 9999;
/// 撤销最多记多少步。
const UNDO_MAX: usize = 100;
/// 骰子转多少帧。转完以后结果留多久（帧）。
const ROLL: u32 = 48;
const SHOW: u32 = 180;
/// 骰子转的时候在第几帧换一面：越往后越慢。
const ROLL_TICKS: [u32; 12] = [0, 3, 6, 9, 12, 15, 19, 23, 28, 33, 40, 47];
/// 主画面上放人的那块：底下一行留给按键提示。
const AREA_Y: i32 = 2;
const AREA_H: i32 = 142;
const HINT_Y: i32 = 146;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// 第一次打开：选几个人。
    Setup,
    Main,
    Menu,
    /// 「新的一局」要再按一次 A。
    Confirm,
}

/// 菜单里的三项。
pub const MENU_NEW: usize = 0;
pub const MENU_PLAYERS: usize = 1;
pub const MENU_BACK: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Die {
    /// 掷了多少帧。
    pub t: u32,
    /// 现在朝上的一面。
    pub face: u8,
    /// 最后停在哪一面。
    pub result: u8,
}

/// 撤销用的一步：哪个人，改之前是多少分。按住连加算一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change {
    pub player: usize,
    pub before: i32,
}

pub struct Game {
    pub frame: Frame,
    pub keys: Keys,
    pub mode: Mode,
    /// 几个人：2 到 6。
    pub players: usize,
    pub scores: [i32; MAX_PLAYERS],
    /// 现在轮到谁（0 起）。
    pub cur: usize,
    pub undo: Vec<Change>,
    /// 上或下按了几帧，0 是没按。
    pub hold: u32,
    /// 按着的是上（1）还是下（-1）。
    hold_dir: i32,
    /// 分变了还没存。
    pub dirty: bool,
    pub die: Option<Die>,
    pub menu: usize,
    /// 从打开起走了几帧。
    pub t: u32,
    seed: u32,
    icons: [Sprite; MAX_PLAYERS],
    crown: Sprite,
}

impl Default for Game {
    fn default() -> Self {
        let mut frame = Frame::default();
        palette(&mut frame);
        let mut g = Game {
            frame,
            keys: Keys::default(),
            mode: Mode::Setup,
            players: 4,
            scores: [0; MAX_PLAYERS],
            cur: 0,
            undo: Vec::new(),
            hold: 0,
            hold_dir: 0,
            dirty: false,
            die: None,
            menu: 0,
            t: 0,
            seed: 20261114,
            icons: icons(),
            crown: crown(),
        };
        if g.load() {
            g.mode = Mode::Main;
        }
        g
    }
}

/// 按住上或下的第 `held` 帧（第一帧是 1）要不要改一次分。
/// 刚按下改一次；按住 24 帧以后每 8 帧一次，一秒半以后每 4 帧，三秒以后每 2 帧。
pub fn repeat_fires(held: u32) -> bool {
    const DELAY: u32 = 25;
    match held {
        0 => false,
        1 => true,
        h if h < DELAY => false,
        h => {
            let every = if h < 90 {
                8
            } else if h < 180 {
                4
            } else {
                2
            };
            (h - DELAY).is_multiple_of(every)
        }
    }
}

/// 存档：「v1,几个人,轮到谁,六个人的分」。
fn encode(players: usize, cur: usize, scores: &[i32; MAX_PLAYERS]) -> String {
    let s: Vec<String> = scores.iter().map(|v| v.to_string()).collect();
    format!("v1,{players},{cur},{}", s.join(","))
}

fn decode(s: &str) -> Option<(usize, usize, [i32; MAX_PLAYERS])> {
    let mut it = s.split(',');
    if it.next()? != "v1" {
        return None;
    }
    let players: usize = it.next()?.parse().ok()?;
    let cur: usize = it.next()?.parse().ok()?;
    let mut scores = [0; MAX_PLAYERS];
    for v in scores.iter_mut() {
        *v = it.next()?.parse::<i32>().ok()?.clamp(MIN_SCORE, MAX_SCORE);
    }
    let players = players.clamp(2, MAX_PLAYERS);
    Some((players, cur.min(players - 1), scores))
}

fn color_name(p: usize) -> &'static str {
    let en = ["RED", "BLUE", "GREEN", "YELLOW", "PURPLE", "ORANGE"];
    let zh = ["红", "蓝", "绿", "黄", "紫", "橙"];
    lang().pick(en[p], zh[p])
}

impl Game {
    fn rand(&mut self, n: u32) -> u32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed >> 16) % n.max(1)
    }

    fn load(&mut self) -> bool {
        let Some((players, cur, scores)) = vibeplay::load(SAVE).and_then(|b| String::from_utf8(b).ok()).and_then(|s| decode(&s)) else {
            return false;
        };
        self.players = players;
        self.cur = cur;
        self.scores = scores;
        true
    }

    pub fn save(&mut self) {
        vibeplay::save(SAVE, encode(self.players, self.cur, &self.scores).as_bytes());
        self.dirty = false;
    }

    /// 领先的人：分最高的，几个人一样高就都算。大家一样多时没有人领先。
    pub fn leaders(&self) -> Vec<usize> {
        let s = &self.scores[..self.players];
        let (hi, lo) = (*s.iter().max().unwrap(), *s.iter().min().unwrap());
        if hi == lo {
            return Vec::new();
        }
        (0..self.players).filter(|&p| s[p] == hi).collect()
    }

    pub fn step(&mut self) {
        self.t += 1;
        match self.mode {
            Mode::Setup => self.step_setup(),
            Mode::Main => self.step_main(),
            Mode::Menu => self.step_menu(),
            Mode::Confirm => self.step_confirm(),
        }
        // 按住连加时不是每一步都存：松手存，按着时半秒存一次。
        if self.dirty && (self.hold == 0 || self.t.is_multiple_of(30)) {
            self.save();
        }
    }

    fn step_setup(&mut self) {
        let k = self.keys;
        if (k.pressed(key::LEFT) || k.pressed(key::DOWN)) && self.players > 2 {
            self.players -= 1;
            blip(660);
        }
        if (k.pressed(key::RIGHT) || k.pressed(key::UP)) && self.players < MAX_PLAYERS {
            self.players += 1;
            blip(880);
        }
        if k.pressed(key::A) || k.pressed(key::START) {
            self.scores = [0; MAX_PLAYERS];
            self.cur = 0;
            self.undo.clear();
            self.mode = Mode::Main;
            self.save();
            chime();
        }
    }

    fn step_main(&mut self) {
        let k = self.keys;
        if let Some(mut d) = self.die {
            d.t += 1;
            if d.t < ROLL {
                // 转着的时候别的键都不管。换一面就嗒一声，最后一下停在结果上。
                if d.t == ROLL_TICKS[ROLL_TICKS.len() - 1] {
                    d.face = d.result;
                    chime();
                } else if ROLL_TICKS.contains(&d.t) {
                    // 加 0 到 4 再绕回来：每次都换成别的一面。
                    d.face = (d.face + self.rand(5) as u8) % 6 + 1;
                    tone(Channel::Noise, 3000 + d.t * 40, 30, 40);
                }
                self.die = Some(d);
                return;
            }
            // 转完了：SELECT 再掷，别的键收起来（这一下不算），过一会儿自己收。
            if !k.pressed(key::SELECT) {
                self.die = if k.pressed(!0) || d.t > ROLL + SHOW { None } else { Some(d) };
                return;
            }
        }
        if k.pressed(key::START) {
            self.mode = Mode::Menu;
            self.menu = 0;
            self.hold = 0;
            blip(660);
            return;
        }
        if k.pressed(key::SELECT) {
            self.roll();
            return;
        }
        // 上下：加减分，按住连着来。
        let dir = if k.held(key::UP) {
            1
        } else if k.held(key::DOWN) {
            -1
        } else {
            0
        };
        if dir != self.hold_dir {
            self.hold = 0;
            self.hold_dir = dir;
        }
        if dir != 0 {
            self.hold += 1;
            if repeat_fires(self.hold) {
                self.bump(dir, self.hold == 1);
            }
            return;
        }
        if k.pressed(key::A) || k.pressed(key::RIGHT) {
            self.select((self.cur + 1) % self.players);
        } else if k.pressed(key::LEFT) {
            self.select((self.cur + self.players - 1) % self.players);
        } else if k.pressed(key::B) {
            self.undo_last();
        }
    }

    /// 给现在这个人改一分。`new_step` 是刚按下（撤销时算新的一步），按住连加的不是。
    pub fn bump(&mut self, d: i32, new_step: bool) {
        let p = self.cur;
        let before = self.scores[p];
        let after = (before + d).clamp(MIN_SCORE, MAX_SCORE);
        if new_step {
            if self.undo.len() == UNDO_MAX {
                self.undo.remove(0);
            }
            self.undo.push(Change { player: p, before });
        }
        if after != before {
            self.scores[p] = after;
            self.dirty = true;
            blip(if d > 0 { 990 } else { 520 });
        }
    }

    pub fn select(&mut self, p: usize) {
        self.cur = p;
        self.dirty = true;
        tone(Channel::Square2, 740, 50, 30);
    }

    /// 撤销上一次改的分，并且选回那个人。没得撤就响一下低的。
    pub fn undo_last(&mut self) {
        while let Some(c) = self.undo.pop() {
            if c.player < self.players {
                self.scores[c.player] = c.before;
                self.cur = c.player;
                self.dirty = true;
                tone(Channel::Triangle, 330, 90, 50);
                return;
            }
        }
        tone(Channel::Triangle, 110, 60, 40);
    }

    pub fn roll(&mut self) {
        // 跟着第几帧掷的变：同样的按键掷出来一样，不同的时候按掷出来不一样。
        self.seed ^= self.t.wrapping_mul(2654435761);
        let result = self.rand(6) as u8 + 1;
        let face = self.rand(6) as u8 + 1;
        self.die = Some(Die { t: 0, face, result });
        self.hold = 0;
    }

    fn step_menu(&mut self) {
        let k = self.keys;
        if k.pressed(key::B) || k.pressed(key::START) {
            self.mode = Mode::Main;
            blip(520);
            return;
        }
        if k.pressed(key::UP) {
            self.menu = (self.menu + 2) % 3;
            blip(660);
        }
        if k.pressed(key::DOWN) {
            self.menu = (self.menu + 1) % 3;
            blip(660);
        }
        if self.menu == MENU_PLAYERS {
            let n = if k.pressed(key::LEFT) {
                self.players.saturating_sub(1).max(2)
            } else if k.pressed(key::RIGHT) || k.pressed(key::A) {
                if k.pressed(key::A) && self.players == MAX_PLAYERS {
                    2
                } else {
                    (self.players + 1).min(MAX_PLAYERS)
                }
            } else {
                self.players
            };
            if n != self.players {
                self.set_players(n);
                blip(880);
            }
            return;
        }
        if k.pressed(key::A) {
            match self.menu {
                MENU_NEW => {
                    self.mode = Mode::Confirm;
                    blip(660);
                }
                _ => {
                    self.mode = Mode::Main;
                    blip(520);
                }
            }
        }
    }

    /// 换几个人：留下的人分不变，新来的从 0 起。
    pub fn set_players(&mut self, n: usize) {
        let n = n.clamp(2, MAX_PLAYERS);
        for p in n..MAX_PLAYERS {
            self.scores[p] = 0;
        }
        self.players = n;
        self.cur = self.cur.min(n - 1);
        self.save();
    }

    fn step_confirm(&mut self) {
        let k = self.keys;
        if k.pressed(key::A) {
            self.new_game();
            self.mode = Mode::Main;
            chime();
        } else if k.pressed(key::B) || k.pressed(key::START) {
            self.mode = Mode::Menu;
            blip(520);
        }
    }

    /// 新的一局：分都清零，从第一个人起。清零撤不回来，所以先问一次。
    pub fn new_game(&mut self) {
        self.scores = [0; MAX_PLAYERS];
        self.cur = 0;
        self.undo.clear();
        self.save();
    }

    // ------------------------------------------------------------ 画

    pub fn draw(&mut self) {
        self.frame.clear(BG);
        match self.mode {
            Mode::Setup => self.draw_setup(),
            Mode::Main => {
                self.draw_players();
                if let Some(d) = self.die {
                    self.draw_die(d);
                }
                let hint = if self.die.is_some() {
                    lang().pick("ANY KEY: CLOSE", "按任何键收起")
                } else {
                    lang().pick("↑↓ SCORE A NEXT B UNDO SEL DIE", "↑↓分 A下一个 B撤销 SEL骰子")
                };
                self.draw_hint(hint);
            }
            Mode::Menu => {
                self.draw_players();
                self.draw_menu();
                self.draw_hint(lang().pick("↑↓ PICK  ←→ CHANGE  B BACK", "↑↓选  ←→改  B返回"));
            }
            Mode::Confirm => {
                self.draw_players();
                self.draw_confirm();
                self.draw_hint(lang().pick("A RESET  B CANCEL", "A清零  B不清"));
            }
        }
    }

    fn draw_hint(&mut self, s: &str) {
        let f = &mut self.frame;
        f.rect(0, HINT_Y - 1, W as i32, 15, PANEL);
        f.text_center(&FONT, HINT_Y + 1, s, GRAY);
    }

    /// 每个人一张卡片：左上图标和颜色名，中间大大的分。
    pub fn cards(&self) -> Vec<(i32, i32, i32, i32)> {
        let n = self.players as i32;
        let (cols, rows) = match n {
            2 => (2, 1),
            3 => (3, 1),
            4 => (2, 2),
            _ => (3, 2),
        };
        let gap = 4;
        let cw = (W as i32 - 4 - (cols - 1) * gap) / cols;
        let ch = (AREA_H - (rows - 1) * gap) / rows;
        (0..n)
            .map(|i| {
                let (r, c) = (i / cols, i % cols);
                // 最后一排不满（5 个人）时放在中间。
                let in_row = (n - r * cols).min(cols);
                let x0 = (W as i32 - (in_row * cw + (in_row - 1) * gap)) / 2;
                (x0 + c * (cw + gap), AREA_Y + r * (ch + gap), cw, ch)
            })
            .collect()
    }

    fn draw_players(&mut self) {
        let leaders = self.leaders();
        let cards = self.cards();
        // 大字放多大：能放下三位数的最大倍数，几张卡一样大。
        let (cw, ch) = (cards[0].2, cards[0].3);
        let k = (2..=5).rev().find(|&k| 18 * k <= cw - 16 && 8 * k <= ch - 26).unwrap_or(2);
        for (p, &(x, y, w, h)) in cards.iter().enumerate() {
            let on = p == self.cur;
            let f = &mut self.frame;
            let (base, light, dark) = (BASE + p as u8, LIGHT + p as u8, DARK + p as u8);
            if on {
                f.rect(x, y, w, h, WHITE);
                f.rect(x + 2, y + 2, w - 4, h - 4, base);
            } else {
                f.rect(x, y, w, h, base);
                f.rect(x + 1, y + 1, w - 2, h - 2, dark);
            }
            // 图标和名字。
            let icon = self.icons[p].recolor(&[(1, if on { WHITE } else { light })]);
            if on {
                f.blit(&icon.recolor(&[(1, INK)]), x + 6, y + 6, false);
            }
            f.blit(&icon, x + 5, y + 5, false);
            let name = color_name(p);
            if on {
                f.text_outlined(&FONT, x + 16, y + 3, name, WHITE, INK);
            } else {
                f.text(&FONT, x + 16, y + 3, name, light);
            }
            if leaders.contains(&p) {
                let cx = x + w - 15;
                f.blit(&self.crown.recolor(&[(GOLD, INK), (GOLD_HI, INK)]), cx + 1, y + 6, false);
                f.blit(&self.crown, cx, y + 5, false);
            }
            // 分：数字的笔画在字格的第 2 到 9 行，放大后上面空着 2k，往上挪回来。
            let s = self.scores[p].to_string();
            let mut kk = k;
            while kk > 2 && FONT.width(&s) * kk > w - 16 {
                kk -= 1;
            }
            let big = FONT.sprite(&s, if on { WHITE } else { light }).scaled(kk);
            let sx = x + (w - big.w) / 2;
            let sy = y + 16 + (h - 16 - 8 * kk) / 2 - 2 * kk;
            if on {
                f.blit(&FONT.sprite(&s, INK).scaled(kk), sx + kk / 2 + 1, sy + kk / 2 + 1, false);
            }
            f.blit(&big, sx, sy, false);
        }
    }

    fn draw_die(&mut self, d: Die) {
        let f = &mut self.frame;
        let (bw, bh) = (96, 84);
        let (bx, by) = ((W as i32 - bw) / 2, AREA_Y + (AREA_H - bh) / 2);
        f.rect(bx - 1, by - 1, bw + 2, bh + 2, INK);
        f.rect(bx, by, bw, bh, PANEL);
        f.frame_rect(bx + 1, by + 1, bw - 2, bh - 2, DIM);
        let s = 36;
        let rolling = d.t < ROLL;
        // 转着的时候上下跳、一边压扁，像在桌上翻。
        let (dy, tilt) = if rolling {
            let i = ROLL_TICKS.iter().filter(|&&t| t <= d.t).count() as i32;
            let since = d.t as i32 - ROLL_TICKS[(i - 1).max(0) as usize] as i32;
            (-((6 - since).max(0)) * (12 - i).max(2) / 4, if since < 2 { (i % 2) * 2 - 1 } else { 0 })
        } else {
            (0, 0)
        };
        draw_die(f, (W as i32 - s) / 2, by + 14 + dy, s, d.face, tilt);
        if !rolling {
            let s = lang().pick(format!("ROLLED {}", d.result), format!("掷出 {} 点", d.result));
            f.text_center(&FONT, by + bh - 18, &s, WHITE);
        } else {
            f.text_center(&FONT, by + bh - 18, lang().pick("ROLLING…", "掷骰子…"), GRAY);
        }
    }

    fn draw_box(&mut self, w: i32, h: i32) -> (i32, i32) {
        let f = &mut self.frame;
        let (x, y) = ((W as i32 - w) / 2, AREA_Y + (AREA_H - h) / 2);
        f.rect(x - 2, y - 2, w + 4, h + 4, INK);
        f.rect(x, y, w, h, PANEL);
        f.frame_rect(x + 1, y + 1, w - 2, h - 2, GRAY);
        (x, y)
    }

    fn draw_menu(&mut self) {
        let (x, y) = self.draw_box(150, 82);
        let n = self.players;
        let items = [
            lang().pick("NEW GAME".to_string(), "新的一局".to_string()),
            lang().pick(format!("PLAYERS  < {n} >"), format!("人数  < {n} >")),
            lang().pick("BACK".to_string(), "返回".to_string()),
        ];
        let f = &mut self.frame;
        f.text_center(&FONT, y + 5, lang().pick("MENU", "菜单"), GOLD);
        for (i, s) in items.iter().enumerate() {
            let iy = y + 22 + i as i32 * 18;
            let on = i == self.menu;
            if on {
                f.rect(x + 8, iy - 2, 134, 16, BASE + 1);
                // 小三角指着选中的一项（字里没有 ▶）。
                for i in 0..4 {
                    f.rect(x + 14 + i, iy + 2 + i, 1, 8 - 2 * i, WHITE);
                }
            }
            f.text(&FONT, x + 28, iy, s, if on { WHITE } else { GRAY });
        }
    }

    fn draw_confirm(&mut self) {
        let (_, y) = self.draw_box(176, 64);
        let f = &mut self.frame;
        f.text_center(&FONT, y + 10, lang().pick("NEW GAME?", "新的一局？"), GOLD);
        f.text_center(&FONT, y + 28, lang().pick("ALL SCORES GO BACK TO 0.", "所有人的分都清零。"), WHITE);
        f.text_center(&FONT, y + 44, lang().pick("A YES   B NO", "A 清零   B 不清"), GRAY);
    }

    fn draw_setup(&mut self) {
        let f = &mut self.frame;
        let title = FONT.sprite(lang().pick("SCORE PAD", "记分板"), GOLD).scaled(2);
        let tx = (W as i32 - title.w) / 2;
        f.blit(&FONT.sprite(lang().pick("SCORE PAD", "记分板"), INK).scaled(2), tx + 2, 8, false);
        f.blit(&title, tx, 6, false);
        f.text_center(&FONT, 38, lang().pick("HOW MANY PLAYERS?", "几个人玩？"), WHITE);
        // 大数字，两边是箭头。
        let n = self.players;
        let big = FONT.sprite(&n.to_string(), WHITE).scaled(4);
        let bx = (W as i32 - big.w) / 2;
        f.blit(&big, bx, 48, false);
        let lc = if n > 2 { WHITE } else { DIM };
        let rc = if n < MAX_PLAYERS { WHITE } else { DIM };
        f.text(&FONT, bx - 26, 66, "←", lc);
        f.text(&FONT, bx + big.w + 14, 66, "→", rc);
        // 每个人一个色块和图标。
        let size = 26;
        let gap = 6;
        let total = n as i32 * size + (n as i32 - 1) * gap;
        let x0 = (W as i32 - total) / 2;
        for p in 0..n {
            let x = x0 + p as i32 * (size + gap);
            let y = 100;
            f.rect(x, y, size, size, BASE + p as u8);
            f.rect(x + 1, y + 1, size - 2, size - 2, DARK + p as u8);
            let icon = self.icons[p].scaled(2).recolor(&[(1, LIGHT + p as u8)]);
            f.blit(&icon, x + 5, y + 5, false);
        }
        self.draw_hint(lang().pick("←→ NUMBER   A START", "←→ 人数   A 开始"));
    }
}

/// 短短一声。
fn blip(hz: u32) {
    tone(Channel::Square1, hz, 40, 30);
}

/// 两个音往上走：开始、掷完。
fn chime() {
    tone(Channel::Square2, 1320, 120, 35);
    tone(Channel::Triangle, 660, 160, 50);
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
