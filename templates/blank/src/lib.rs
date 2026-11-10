//! 一个最小的游戏：十字键走，捡星星，三十秒看能捡几颗。最高分存档。
//! 从这里改起：画面在 `draw`，规则在 `step`，颜色在 `PALETTE`。

use std::sync::Mutex;
use vibegame::full::{key, lang, Channel, Frame, Keys, Music, Song, Sprite, Track, H, W};
use vibegame::FONT;

// 调色板：格子编号起名字，画的时候只写名字。第 0 格是背景，精灵里的 0 是透明。
const SKY: u8 = 0;
const INK: u8 = 1;
const WHITE: u8 = 2;
const GOLD: u8 = 3;
const SKIN: u8 = 4;
const RED: u8 = 5;
const PALETTE: [(u8, [u8; 3]); 6] =
    [(SKY, [24, 28, 64]), (INK, [12, 12, 20]), (WHITE, [240, 240, 232]), (GOLD, [255, 208, 64]), (SKIN, [248, 184, 136]), (RED, [216, 56, 72])];

/// 一局多少帧：每秒 60 帧，三十秒。
const ROUND: u32 = 30 * 60;
const SAVE: &str = "best";

fn hero() -> Sprite {
    Sprite::art(
        &["..rrrr..", ".rrrrrr.", ".ssssss.", ".sisssi.", ".ssssss.", "..rrrr..", ".rr..rr.", ".i....i."],
        &[('r', RED), ('s', SKIN), ('i', INK)],
    )
    .scaled(2)
}

fn star() -> Sprite {
    Sprite::art(&["...g...", "..ggg..", "ggggggg", ".ggggg.", ".gg.gg.", "g.....g"], &[('g', GOLD)])
}

fn song() -> Song {
    let melody = Track::parse(Channel::Square1, 30, "C5 E5 G5 E5 F5 A5 G5 r D5 F5 E5 C5 D5:2 r:2").unwrap();
    let bass = Track::parse(Channel::Triangle, 50, "C3:4 F3:4 G3:4 G3:2 r:2").unwrap();
    Song::new(10, true, vec![melody, bass])
}

#[derive(PartialEq)]
enum Mode {
    Title,
    Play,
    Over,
}

struct Game {
    frame: Frame,
    keys: Keys,
    music: Music,
    mode: Mode,
    /// 主角的位置，像素。
    x: i32,
    y: i32,
    star: (i32, i32),
    score: u32,
    best: u32,
    left: u32,
    /// 自己的随机数：同样的按键每次跑出来一样。
    seed: u32,
    hero: Sprite,
    star_art: Sprite,
}

impl Default for Game {
    fn default() -> Self {
        let mut frame = Frame::default();
        for (i, rgb) in PALETTE {
            frame.palette[i as usize] = rgb;
        }
        let best = vibegame::load(SAVE).and_then(|b| String::from_utf8(b).ok()?.parse().ok()).unwrap_or(0);
        Game {
            frame,
            keys: Keys::default(),
            music: Music::default(),
            mode: Mode::Title,
            x: 112,
            y: 72,
            star: (60, 40),
            score: 0,
            best,
            left: ROUND,
            seed: 7,
            hero: hero(),
            star_art: star(),
        }
    }
}

impl Game {
    fn rand(&mut self, n: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.seed >> 16) % n) as i32
    }

    /// 走一帧。
    fn step(&mut self) {
        match self.mode {
            Mode::Title | Mode::Over => {
                if self.keys.pressed(key::START) || self.keys.pressed(key::A) {
                    self.mode = Mode::Play;
                    self.score = 0;
                    self.left = ROUND;
                    self.music.play(song());
                }
            }
            Mode::Play => {
                let k = &self.keys;
                let (dx, dy) = ((k.held(key::RIGHT) as i32) - (k.held(key::LEFT) as i32), (k.held(key::DOWN) as i32) - (k.held(key::UP) as i32));
                self.x = (self.x + dx * 2).clamp(0, W as i32 - 16);
                self.y = (self.y + dy * 2).clamp(14, H as i32 - 16);
                let (sx, sy) = self.star;
                if (self.x - sx).abs() < 12 && (self.y - sy).abs() < 12 {
                    self.score += 1;
                    self.music.sfx(Channel::Square2, 1320, 60, 40);
                    self.star = (self.rand(W as u32 - 8), 16 + self.rand(H as u32 - 24));
                }
                self.left -= 1;
                if self.left == 0 {
                    self.mode = Mode::Over;
                    self.music.stop();
                    if self.score > self.best {
                        self.best = self.score;
                        vibegame::save(SAVE, self.best.to_string().as_bytes());
                    }
                }
            }
        }
        self.music.tick();
    }

    fn draw(&mut self) {
        let f = &mut self.frame;
        f.clear(SKY);
        match self.mode {
            Mode::Title => {
                // 出字时问系统语言：人在暂停时换了，马上跟着变。
                let name = lang().pick("STAR CATCH", "捡星星");
                f.blit(&FONT.sprite(name, GOLD).scaled(3), 120 - FONT.width(name) * 3 / 2, 40, false);
                f.blit(&self.hero, 112, 92, false);
                f.text_center(&FONT, 128, lang().pick("PRESS START", "按 START 开始"), WHITE);
            }
            Mode::Play => {
                f.blit(&self.star_art, self.star.0, self.star.1, false);
                f.blit(&self.hero, self.x, self.y, false);
                f.rect(0, 0, W as i32, 13, INK);
                let (score, sec) = (self.score, self.left.div_ceil(60));
                f.text(&FONT, 4, 0, &lang().pick(format!("STARS {score}"), format!("星星 {score}")), GOLD);
                f.text(&FONT, 190, 0, &lang().pick(format!("TIME {sec}"), format!("{sec} 秒")), WHITE);
            }
            Mode::Over => {
                let (score, best) = (self.score, self.best);
                f.text_center(&FONT, 56, &lang().pick(format!("YOU GOT {score}"), format!("捡到 {score} 颗")), GOLD);
                f.text_center(&FONT, 76, &lang().pick(format!("BEST {best}"), format!("最多 {best} 颗")), WHITE);
                f.text_center(&FONT, 112, lang().pick("PRESS START TO PLAY AGAIN", "按 START 再来"), WHITE);
            }
        }
    }
}

static GAME: Mutex<Option<Game>> = Mutex::new(None);

vibegame::main!(|app| {
    app.every(|| {
        let mut g = GAME.lock().unwrap();
        let g = g.get_or_insert_with(Game::default);
        g.keys.read();
        g.step();
        g.draw();
        g.frame.show();
    });
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walking_onto_a_star_scores() {
        let mut g = Game::default();
        g.keys.set(key::START);
        g.step();
        assert!(g.mode == Mode::Play);
        g.star = (g.x + 4, g.y);
        g.keys.set(0);
        g.step();
        assert_eq!(g.score, 1);
    }

    #[test]
    fn title_follows_the_system_language() {
        use vibegame::full::{native::set_lang, Lang};
        let mut g = Game::default();
        g.draw();
        let en = g.frame.pixels.clone();
        set_lang(Lang::Zh);
        g.draw();
        let zh = g.frame.pixels.clone();
        set_lang(Lang::En);
        assert_ne!(en, zh);
    }
}
