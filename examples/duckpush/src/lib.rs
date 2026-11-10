//! 样板游戏《鸭鸭搬面包》：推箱子。
//!
//! 一格一格走，把面包推到盘子上。二十关，过了一关存一关（`data_write("progress")`）。
//! 选关：十字键选，A 开始。关里：十字键走，B 撤回一步，SELECT 重来，START 回选关。
//! 当模板抄：存档、选关的菜单、撤回（存每一步之前的样子）、按住方向键连着走。

mod levels;

use vibegame::FONT;
use levels::LEVELS;
use vibegame::full::{key, Channel, Frame, Keys, Music, Song, Sprite, Track, H, W};
use std::sync::Mutex;

/// 一格多少像素，上面一条字占多高。
const T: i32 = 16;
const BAR: i32 = 16;
/// 存档的名字：20 个字，第 n 个是 '1' 就是第 n 关过了。
pub const SAVE: &str = "progress";

// 调色板的格。精灵里 0 是透明。
const BG: u8 = 0;
const FLOOR: u8 = 1;
const FLOOR2: u8 = 2;
const WALL: u8 = 3;
const WALL_HI: u8 = 4;
const WALL_LO: u8 = 5;
const PLATE: u8 = 6;
const PLATE_LO: u8 = 7;
const CRUST: u8 = 8;
const CRUMB: u8 = 9;
const CRUST_LO: u8 = 10;
const DUCK: u8 = 11;
const DUCK_LO: u8 = 12;
const BEAK: u8 = 13;
const INK: u8 = 14;
const WHITE: u8 = 15;
const GOLD: u8 = 16;
const GRAY: u8 = 17;
const GREEN: u8 = 18;
const PANEL: u8 = 20;

const PALETTE: [(u8, [u8; 3]); 20] = [
    (BG, [0x1c, 0x18, 0x30]),
    (FLOOR, [0xe8, 0xd8, 0xb0]),
    (FLOOR2, [0xdc, 0xc8, 0x9c]),
    (WALL, [0x6a, 0x5a, 0x7c]),
    (WALL_HI, [0x8c, 0x7c, 0xa0]),
    (WALL_LO, [0x40, 0x34, 0x50]),
    (PLATE, [0xf8, 0xf8, 0xf0]),
    (PLATE_LO, [0xa8, 0xb4, 0xc4]),
    (CRUST, [0xb8, 0x6a, 0x2c]),
    (CRUMB, [0xec, 0xb0, 0x5c]),
    (CRUST_LO, [0x78, 0x40, 0x18]),
    (DUCK, [0xff, 0xd8, 0x20]),
    (DUCK_LO, [0xe0, 0xa0, 0x00]),
    (BEAK, [0xff, 0x80, 0x20]),
    (INK, [0x10, 0x10, 0x18]),
    (WHITE, [0xff, 0xff, 0xff]),
    (GOLD, [0xff, 0xe0, 0x70]),
    (GRAY, [0x80, 0x80, 0x98]),
    (GREEN, [0x50, 0xb8, 0x60]),
    (PANEL, [0x2c, 0x24, 0x40]),
];

/// 小黄鸭，朝右。两张：脚换着迈。
const DUCK_ART: [&str; 16] = [
    "................",
    "......yyyy......",
    ".....yyyyyy.....",
    ".....yyykyyy....",
    ".....yyyyyyooo..",
    ".....yyyyyyoo...",
    "......yyyyy.....",
    ".......yyy......",
    ".y....yyyyyy....",
    ".yyyyyyyyyyyy...",
    ".yyyddyyyyyyyy..",
    "..yyyddddyyyy...",
    "..yyyyyyyyyyy...",
    "...ddddddddd....",
    ".....o...o......",
    "....oo..oo......",
];
const DUCK_COLORS: [(char, u8); 4] = [('y', DUCK), ('d', DUCK_LO), ('o', BEAK), ('k', INK)];

const BREAD_ART: [&str; 16] = [
    "................",
    "................",
    "................",
    "....cccccccc....",
    "..ccmmmmmmmmcc..",
    ".cmmcmmcmmcmmmc.",
    ".cmmmcmmcmmcmmc.",
    "cmmmmmmmmmmmmmmc",
    "cmmmmmmmmmmmmmmc",
    "cmmmmmmmmmmmmmmc",
    "ccmmmmmmmmmmmmcc",
    ".cccccccccccccc.",
    "..llllllllllll..",
    "................",
    "................",
    "................",
];
const BREAD_COLORS: [(char, u8); 3] = [('c', CRUST), ('m', CRUMB), ('l', CRUST_LO)];

const PLATE_ART: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "....pppppppp....",
    "..ppwwwwwwwwpp..",
    ".pwwwppppppwwwp.",
    ".pwwpwwwwwwpwwp.",
    ".pwwwppppppwwwp.",
    "..ppwwwwwwwwpp..",
    "....pppppppp....",
    "................",
    "................",
    "................",
];
const PLATE_COLORS: [(char, u8); 2] = [('w', PLATE), ('p', PLATE_LO)];

const WALL_ART: [&str; 16] = [
    "hhhhhhhlhhhhhhhl",
    "hwwwwwwlhwwwwwwl",
    "hwwwwwwlhwwwwwwl",
    "hwwwwwwlhwwwwwwl",
    "hwwwwwwlhwwwwwwl",
    "hwwwwwwlhwwwwwwl",
    "hwwwwwwlhwwwwwwl",
    "llllllllllllllll",
    "hhhlhhhhhhhlhhhh",
    "wwwlhwwwwwwlhwww",
    "wwwlhwwwwwwlhwww",
    "wwwlhwwwwwwlhwww",
    "wwwlhwwwwwwlhwww",
    "wwwlhwwwwwwlhwww",
    "wwwlhwwwwwwlhwww",
    "llllllllllllllll",
];
const WALL_COLORS: [(char, u8); 3] = [('w', WALL), ('h', WALL_HI), ('l', WALL_LO)];

/// 原创的小曲：C 大调，32 格一遍。
fn tune() -> Song {
    let lead = "E5 G5 C6:2 B5 G5 A5:2 G5 E5 F5 D5 E5:4 C5 E5 G5:2 A5 G5 F5:2 E5 D5 C5 D5 C5:4";
    let bass = "C3:2 G3:2 C3:2 G3:2 F2:2 C3:2 G2:2 B2:2 A2:2 E3:2 F2:2 C3:2 G2:2 G3:2 C3:4";
    Song::new(9, true, vec![Track::parse(Channel::Square1, 18, lead).unwrap(), Track::parse(Channel::Triangle, 40, bass).unwrap()])
}

/// 过关的一小段，放一遍。
fn fanfare() -> Song {
    Song::new(6, false, vec![
        Track::parse(Channel::Square1, 30, "C5 E5 G5 C6:3 r G5 C6:4").unwrap(),
        Track::parse(Channel::Triangle, 45, "C3:3 G3:3 C4:6").unwrap(),
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Menu,
    Play,
    Clear,
    AllDone,
}

/// 一关的地形：墙、盘子、哪些地画出来（墙里面的）。
#[derive(Debug, Clone, Default)]
pub struct Board {
    pub w: i32,
    pub h: i32,
    wall: Vec<bool>,
    plate: Vec<bool>,
    inside: Vec<bool>,
}

impl Board {
    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        ((0..self.w).contains(&x) && (0..self.h).contains(&y)).then(|| (y * self.w + x) as usize)
    }
    pub fn wall(&self, x: i32, y: i32) -> bool {
        self.idx(x, y).is_none_or(|i| self.wall[i])
    }
    pub fn plate(&self, x: i32, y: i32) -> bool {
        self.idx(x, y).is_some_and(|i| self.plate[i])
    }
}

type Pos = (i32, i32);

/// 读一关：地形、鸭子在哪、面包在哪。
pub fn load(rows: &[&str]) -> (Board, Pos, Vec<Pos>) {
    let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
    let h = rows.len() as i32;
    let n = (w * h) as usize;
    let mut b = Board { w, h, wall: vec![false; n], plate: vec![false; n], inside: vec![false; n] };
    let (mut duck, mut breads) = ((0, 0), Vec::new());
    for (y, r) in rows.iter().enumerate() {
        for (x, c) in r.chars().enumerate() {
            let (x, y) = (x as i32, y as i32);
            let i = (y * w + x) as usize;
            b.wall[i] = c == '#';
            b.plate[i] = matches!(c, '.' | '*' | '+');
            if matches!(c, '$' | '*') {
                breads.push((x, y));
            }
            if matches!(c, '@' | '+') {
                duck = (x, y);
            }
        }
    }
    // 从鸭子那里能走到的地才画：墙外面的空格不是地。
    let mut todo = vec![duck];
    while let Some((x, y)) = todo.pop() {
        let Some(i) = b.idx(x, y) else { continue };
        if b.wall[i] || b.inside[i] {
            continue;
        }
        b.inside[i] = true;
        todo.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
    }
    (b, duck, breads)
}

struct Art {
    duck: [Sprite; 2],
    bread: Sprite,
    plate: Sprite,
    wall: Sprite,
}

impl Art {
    fn new() -> Art {
        let duck = Sprite::art(&DUCK_ART, &DUCK_COLORS);
        // 第二张：脚迈到另一边。
        let mut duck2 = duck.clone();
        for x in 0..16 {
            duck2.px[14 * 16 + x] = 0;
            duck2.px[15 * 16 + x] = 0;
        }
        for (x, y) in [(6, 14), (10, 14), (6, 15), (7, 15), (10, 15), (11, 15)] {
            duck2.px[y * 16 + x] = BEAK;
        }
        Art {
            duck: [duck, duck2],
            bread: Sprite::art(&BREAD_ART, &BREAD_COLORS),
            plate: Sprite::art(&PLATE_ART, &PLATE_COLORS),
            wall: Sprite::art(&WALL_ART, &WALL_COLORS),
        }
    }
}

pub struct Game {
    pub frame: Frame,
    pub keys: Keys,
    pub music: Music,
    art: Art,
    pub mode: Mode,
    /// 第几关过了（从 0 数）。
    pub cleared: [bool; 20],
    /// 选关的光标、在玩第几关（从 0 数）。
    pub sel: usize,
    pub level: usize,
    pub board: Board,
    pub duck: Pos,
    pub breads: Vec<Pos>,
    /// 每一步之前的样子：撤回就退回去。
    history: Vec<(Pos, Vec<Pos>)>,
    pub moves: u32,
    left: bool,
    /// 方向键按住了几帧（连着走用）、过关后过了几帧、一共走了几帧。
    held_for: u32,
    timer: u32,
    tick: u32,
}

impl Default for Game {
    fn default() -> Self {
        let mut frame = Frame::default();
        for (i, rgb) in PALETTE {
            frame.palette[i as usize] = rgb;
        }
        let mut g = Game {
            frame,
            keys: Keys::default(),
            music: Music::default(),
            art: Art::new(),
            mode: Mode::Menu,
            cleared: [false; 20],
            sel: 0,
            level: 0,
            board: Board::default(),
            duck: (0, 0),
            breads: Vec::new(),
            history: Vec::new(),
            moves: 0,
            left: false,
            held_for: 0,
            timer: 0,
            tick: 0,
        };
        if let Some(b) = vibegame::load(SAVE) {
            for (i, c) in b.iter().take(20).enumerate() {
                g.cleared[i] = *c == b'1';
            }
        }
        // 光标停在第一个没过的关上。
        g.sel = (0..20).find(|&i| !g.cleared[i]).unwrap_or(19);
        g.music.play(tune());
        g
    }
}

impl Game {
    /// 第 i 关开了没有：第一关，或者前一关过了，或者自己过了。
    pub fn open(&self, i: usize) -> bool {
        i == 0 || self.cleared[i - 1] || self.cleared[i]
    }

    fn start(&mut self, i: usize) {
        self.level = i;
        let (b, d, br) = load(LEVELS[i]);
        (self.board, self.duck, self.breads) = (b, d, br);
        self.history.clear();
        self.moves = 0;
        self.mode = Mode::Play;
    }

    fn save(&self) {
        let s: Vec<u8> = self.cleared.iter().map(|&c| if c { b'1' } else { b'0' }).collect();
        // 存不了（开发机上没给目录）也接着玩，只是下次从头。
        let _ = vibegame::save(SAVE, &s);
    }

    pub fn solved(&self) -> bool {
        self.breads.iter().all(|&(x, y)| self.board.plate(x, y))
    }

    /// 这一帧该往哪走：刚按下走一步；按住 14 帧以后每 6 帧走一步。
    fn dir(&mut self) -> Option<Pos> {
        let dirs = [(key::UP, (0, -1)), (key::DOWN, (0, 1)), (key::LEFT, (-1, 0)), (key::RIGHT, (1, 0))];
        if let Some(&(_, d)) = dirs.iter().find(|(k, _)| self.keys.pressed(*k)) {
            self.held_for = 0;
            return Some(d);
        }
        let held = dirs.iter().find(|(k, _)| self.keys.held(*k))?;
        self.held_for += 1;
        (self.held_for >= 14 && (self.held_for - 14).is_multiple_of(6)).then_some(held.1)
    }

    /// 往 d 走一步，前面是面包就推。推不动、撞墙不算一步。
    pub fn walk(&mut self, d: Pos) {
        let to = (self.duck.0 + d.0, self.duck.1 + d.1);
        if d.0 != 0 {
            self.left = d.0 < 0;
        }
        if self.board.wall(to.0, to.1) {
            return;
        }
        let pushed = self.breads.iter().position(|&b| b == to);
        if let Some(i) = pushed {
            let past = (to.0 + d.0, to.1 + d.1);
            if self.board.wall(past.0, past.1) || self.breads.contains(&past) {
                return;
            }
            self.history.push((self.duck, self.breads.clone()));
            self.breads[i] = past;
            if self.board.plate(past.0, past.1) {
                self.music.sfx(Channel::Square2, 1320, 90, 40);
            } else {
                self.music.sfx(Channel::Square2, 196, 60, 40);
            }
        } else {
            self.history.push((self.duck, self.breads.clone()));
            self.music.sfx(Channel::Noise, 3000, 25, 15);
        }
        self.duck = to;
        self.moves += 1;
        if self.solved() {
            self.cleared[self.level] = true;
            self.save();
            self.mode = Mode::Clear;
            self.timer = 0;
            self.music.play(fanfare());
        }
    }

    /// 撤回一步。
    pub fn undo(&mut self) {
        if let Some((d, b)) = self.history.pop() {
            self.duck = d;
            self.breads = b;
            self.moves -= 1;
            self.music.sfx(Channel::Square2, 523, 50, 30);
        }
    }

    /// 走一帧：按键已经读进 `keys`。
    pub fn step(&mut self) {
        self.tick += 1;
        match self.mode {
            Mode::Menu => {
                let k = self.keys;
                let (col, row) = ((self.sel % 5) as i32, (self.sel / 5) as i32);
                let (col, row) = (
                    (col + k.pressed(key::RIGHT) as i32 - k.pressed(key::LEFT) as i32).rem_euclid(5),
                    (row + k.pressed(key::DOWN) as i32 - k.pressed(key::UP) as i32).rem_euclid(4),
                );
                let sel = (row * 5 + col) as usize;
                if sel != self.sel {
                    self.sel = sel;
                    self.music.sfx(Channel::Square2, 880, 30, 25);
                }
                if (k.pressed(key::A) || k.pressed(key::START)) && self.open(self.sel) {
                    self.music.sfx(Channel::Square2, 1046, 60, 35);
                    self.start(self.sel);
                }
            }
            Mode::Play => {
                if self.keys.pressed(key::START) {
                    self.mode = Mode::Menu;
                    self.sel = self.level;
                } else if self.keys.pressed(key::SELECT) {
                    self.start(self.level);
                } else if self.keys.pressed(key::B) {
                    self.undo();
                } else if let Some(d) = self.dir() {
                    self.walk(d);
                }
            }
            Mode::Clear => {
                self.timer += 1;
                if self.timer > 30 && (self.keys.pressed(key::A) || self.timer >= 150) {
                    if self.level + 1 < 20 {
                        self.start(self.level + 1);
                        self.music.play(tune());
                    } else {
                        self.mode = Mode::AllDone;
                    }
                }
            }
            Mode::AllDone => {
                if self.keys.pressed(key::A) || self.keys.pressed(key::START) {
                    self.mode = Mode::Menu;
                    self.music.play(tune());
                }
            }
        }
        self.music.tick();
        self.draw();
    }

    fn draw(&mut self) {
        let f = &mut self.frame;
        f.clear(BG);
        match self.mode {
            Mode::Menu => {
                f.text_center(&FONT, 4, "鸭鸭搬面包", GOLD);
                f.text_center(&FONT, 20, "把面包推到盘子上", GRAY);
                for i in 0..20 {
                    let (x, y) = (8 + (i % 5) as i32 * 46, 38 + (i / 5) as i32 * 26);
                    let (fill, ink) = if self.cleared[i] {
                        (GREEN, WHITE)
                    } else if i == 0 || self.cleared[i - 1] {
                        (WALL, WHITE)
                    } else {
                        (PANEL, GRAY)
                    };
                    f.rect(x, y, 40, 22, fill);
                    let n = format!("{}", i + 1);
                    let tx = x + (40 - FONT.width(&n)) / 2;
                    f.text(&FONT, tx, y + 5, &n, ink);
                    if i == self.sel {
                        f.frame_rect(x - 2, y - 2, 44, 26, GOLD);
                        f.frame_rect(x - 1, y - 1, 42, 24, GOLD);
                    }
                }
                f.text_center(&FONT, 145, "B 撤回  SELECT 重来  START 选关", GRAY);
            }
            Mode::Play | Mode::Clear | Mode::AllDone => {
                let b = &self.board;
                let (ox, oy) = ((W as i32 - b.w * T) / 2, BAR + (H as i32 - BAR - b.h * T) / 2);
                for y in 0..b.h {
                    for x in 0..b.w {
                        let (px, py) = (ox + x * T, oy + y * T);
                        let i = (y * b.w + x) as usize;
                        if b.wall[i] {
                            f.blit(&self.art.wall, px, py, false);
                        } else if b.inside[i] {
                            f.rect(px, py, T, T, if (x + y) % 2 == 0 { FLOOR } else { FLOOR2 });
                            if b.plate[i] {
                                f.blit(&self.art.plate, px, py, false);
                            }
                        }
                    }
                }
                for &(x, y) in &self.breads {
                    let (px, py) = (ox + x * T, oy + y * T);
                    f.blit(&self.art.bread, px, py, false);
                    if b.plate(x, y) {
                        // 放好的面包下面一条金边。
                        f.rect(px + 3, py + 13, 10, 1, GOLD);
                    }
                }
                let dk = &self.art.duck[(self.moves % 2) as usize];
                f.blit(dk, ox + self.duck.0 * T, oy + self.duck.1 * T, self.left);
                f.rect(0, 0, W as i32, BAR, PANEL);
                f.text(&FONT, 4, 2, &format!("第 {} 关", self.level + 1), WHITE);
                let m = format!("步数 {}", self.moves);
                f.text(&FONT, W as i32 - 4 - FONT.width(&m), 2, &m, WHITE);
                if self.mode == Mode::Clear {
                    f.rect(60, 62, 120, 40, PANEL);
                    f.frame_rect(60, 62, 120, 40, GOLD);
                    f.text_center(&FONT, 68, "过关！", GOLD);
                    f.text_center(&FONT, 84, "A 下一关", WHITE);
                }
                if self.mode == Mode::AllDone {
                    f.rect(30, 62, 180, 40, PANEL);
                    f.frame_rect(30, 62, 180, 40, GOLD);
                    f.text_center(&FONT, 68, "二十关全过了！", GOLD);
                    f.text_center(&FONT, 84, "A 回选关", WHITE);
                }
            }
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
mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// 宽度优先找一条最短的解，回走法（U D L R）。找不到回 None。
    /// 一个样子压成一个整数：每个位置是一格的编号（7 位），鸭子在最低，面包排好序跟在后面。
    fn solve(rows: &[&str]) -> Option<String> {
        let (b, duck, breads) = load(rows);
        let cell = |(x, y): Pos| (y * b.w + x) as u64;
        let pos = |c: u64| ((c as i32) % b.w, (c as i32) / b.w);
        let pack = |d: Pos, bs: &mut Vec<Pos>| {
            bs.sort();
            bs.iter().fold(cell(d), |k, &p| k << 7 | cell(p)) | (bs.len() as u64) << 60
        };
        let unpack = |k: u64| {
            let n = (k >> 60) as usize;
            let mut bs: Vec<Pos> = (0..n).map(|i| pos(k >> (7 * i) & 0x7f)).collect();
            bs.reverse();
            (pos(k >> (7 * n) & 0x7f), bs)
        };
        // 推进不是盘子的墙角就再也推不出来了，不往那里推。
        let dead = |(x, y): Pos| !b.plate(x, y) && (b.wall(x - 1, y) || b.wall(x + 1, y)) && (b.wall(x, y - 1) || b.wall(x, y + 1));
        let start = pack(duck, &mut breads.clone());
        let mut from = std::collections::HashMap::from([(start, (start, ' '))]);
        let mut q = VecDeque::from([start]);
        while let Some(k) = q.pop_front() {
            let (d, bs) = unpack(k);
            if bs.iter().all(|&(x, y)| b.plate(x, y)) {
                let mut path = Vec::new();
                let mut at = k;
                while at != start {
                    let (prev, m) = from[&at];
                    path.push(m);
                    at = prev;
                }
                return Some(path.iter().rev().collect());
            }
            for (m, dx, dy) in [('U', 0, -1), ('D', 0, 1), ('L', -1, 0), ('R', 1, 0)] {
                let to = (d.0 + dx, d.1 + dy);
                if b.wall(to.0, to.1) {
                    continue;
                }
                let mut nb = bs.clone();
                if let Some(i) = nb.iter().position(|&p| p == to) {
                    let past = (to.0 + dx, to.1 + dy);
                    if b.wall(past.0, past.1) || nb.contains(&past) || dead(past) {
                        continue;
                    }
                    nb[i] = past;
                }
                let nk = pack(to, &mut nb);
                if let std::collections::hash_map::Entry::Vacant(e) = from.entry(nk) {
                    e.insert((k, m));
                    q.push_back(nk);
                }
            }
        }
        None
    }

    #[test]
    fn every_level_is_well_formed_fits_and_can_be_solved() {
        for (i, rows) in LEVELS.iter().enumerate() {
            let (b, _, breads) = load(rows);
            let plates = b.plate.iter().filter(|&&p| p).count();
            assert_eq!(plates, breads.len(), "第 {} 关盘子和面包不一样多", i + 1);
            assert!(b.w * T <= W as i32 && b.h * T <= H as i32 - BAR, "第 {} 关太大", i + 1);
            assert!(solve(rows).is_some(), "第 {} 关过不了", i + 1);
        }
        assert_eq!(solve(LEVELS[0]).as_deref(), Some("RRR"));
    }

    #[test]
    fn push_undo_clear_and_save() {
        assert!(vibegame::erase(SAVE));
        let mut g = Game::default();
        assert_eq!((g.mode, g.sel), (Mode::Menu, 0));
        // 第二关没开：按 A 不进。
        g.keys.set(key::RIGHT);
        g.step();
        g.keys.set(key::A);
        g.step();
        assert_eq!(g.mode, Mode::Menu);
        g.keys.set(key::LEFT);
        g.step();
        g.keys.set(key::A);
        g.step();
        assert_eq!((g.mode, g.level), (Mode::Play, 0));
        let start = g.breads.clone();
        g.walk((1, 0));
        g.walk((1, 0));
        assert_ne!(g.breads, start);
        g.undo();
        assert_eq!((g.breads.clone(), g.moves), (start, 1));
        g.walk((1, 0));
        g.walk((1, 0));
        assert_eq!(g.mode, Mode::Clear);
        assert_eq!(vibegame::load(SAVE).unwrap(), b"10000000000000000000");
        // 读档：新开一次，第一关是过了的，光标在第二关。
        let g2 = Game::default();
        assert!(g2.cleared[0] && g2.open(1) && !g2.open(2));
        assert_eq!(g2.sel, 1);
    }
}
