//! 小 app《节拍盒》：八步的音序器。四行：底鼓、军鼓、踩镲、低音，一行八格。
//!
//! 十字键选格，A 开关一格（低音那行按 A 换音：五声音阶的六个音，再按一下关掉），B 清掉这一格。
//! START 放或停。L1、R1 调速度（每拍几下，一步是半拍）。SELECT 换几种现成的节奏，第一种是空的。
//! 每改一次就存：格子和速度。
//!
//! 时间全用帧数：每秒 60 帧，一步 = 3600 / (BPM × 2) 帧。不是整数时攒着余数，平均下来正好。

#[cfg(test)]
mod tests;

use std::sync::Mutex;
use vibeplay::full::{hz, key, lang, tone, Channel, Frame, Keys, Sprite};
use vibeplay::FONT;

pub const STEPS: usize = 8;
pub const ROWS: usize = 4;
pub const KICK: usize = 0;
pub const SNARE: usize = 1;
pub const HAT: usize = 2;
pub const BASS: usize = 3;

pub const BPM_MIN: u32 = 60;
pub const BPM_MAX: u32 = 200;
pub const BPM_STEP: u32 = 5;
const BPM_START: u32 = 120;
/// 一分钟的帧数。一步是半拍，所以每帧攒 `bpm * 2`，攒到这么多走一步。
const FRAMES_PER_MIN: u32 = 3600;
const SAVE: &str = "pattern";

/// 低音那行的音：C 大调五声音阶，C3 到 C4。格里存 1..=6，0 是关。
const SCALE: [u8; 6] = [48, 50, 52, 55, 57, 60];
const NOTE_NAMES: [&str; 6] = ["C", "D", "E", "G", "A", "c"];

// 调色板的格子。
const BG: u8 = 0;
const PANEL: u8 = 1;
const CELL: u8 = 2;
const CELL_HEAD: u8 = 3;
const WHITE: u8 = 4;
const GREY: u8 = 5;
/// 每行的颜色，后面一格是放到它时的亮色。
const ROW_COLOR: [u8; ROWS] = [6, 8, 10, 12];
const INK: u8 = 14;
const BEAT: u8 = 15;

const PALETTE: [(u8, [u8; 3]); 16] = [
    (BG, [18, 16, 34]),
    (PANEL, [34, 30, 60]),
    (CELL, [50, 48, 84]),
    (CELL_HEAD, [84, 80, 128]),
    (WHITE, [248, 248, 240]),
    (GREY, [150, 146, 184]),
    (6, [232, 84, 72]),
    (7, [255, 170, 150]),
    (8, [244, 196, 56]),
    (9, [255, 236, 150]),
    (10, [80, 200, 232]),
    (11, [180, 240, 255]),
    (12, [112, 216, 112]),
    (13, [196, 255, 180]),
    (INK, [18, 16, 34]),
    (BEAT, [120, 116, 168]),
];

// 格子在屏上的位置。
const GRID_X: i32 = 44;
const GRID_Y: i32 = 24;
const PITCH: i32 = 24;
const CELL_W: i32 = 22;

/// 现成的节奏：四行字，`x` 是开，数字是低音的第几个音，`.` 是关。
struct Preset {
    en: &'static str,
    zh: &'static str,
    rows: [&'static str; ROWS],
}

const PRESETS: [Preset; 4] = [
    Preset { en: "Empty", zh: "空白", rows: ["........", "........", "........", "........"] },
    Preset { en: "Rock", zh: "摇滚", rows: ["x...x.x.", "..x...x.", "xxxxxxxx", "1..1.4.5"] },
    Preset { en: "Disco", zh: "迪斯科", rows: ["x.x.x.x.", "..x...x.", ".x.x.x.x", "1.6.1.6."] },
    Preset { en: "Funk", zh: "放克", rows: ["x..x..x.", "..x..x.x", "xx.xx.xx", "1.3.4.53"] },
];

/// 一格：0 关；鼓的行 1 是开；低音那行 1..=6 是音。
pub type Grid = [[u8; STEPS]; ROWS];

fn preset_grid(i: usize) -> Grid {
    let mut g = [[0; STEPS]; ROWS];
    for (r, row) in PRESETS[i].rows.iter().enumerate() {
        for (s, ch) in row.bytes().enumerate() {
            g[r][s] = match ch {
                b'x' => 1,
                b'1'..=b'6' => ch - b'0',
                _ => 0,
            };
        }
    }
    g
}

/// 一步有几帧（可能带小数），用来算音长：毫秒。
pub fn step_ms(bpm: u32) -> u32 {
    FRAMES_PER_MIN * 1000 / 60 / (bpm * 2)
}

/// 存档：`B`、版本 1、速度、上次选的现成节奏、32 格。
pub fn encode(grid: &Grid, bpm: u32, preset: usize) -> Vec<u8> {
    let mut v = vec![b'B', 1, bpm as u8, preset as u8];
    for row in grid {
        v.extend_from_slice(row);
    }
    v
}

/// 读存档。不对就回 None，用空的。
pub fn decode(b: &[u8]) -> Option<(Grid, u32, usize)> {
    if b.len() != 4 + ROWS * STEPS || b[0] != b'B' || b[1] != 1 {
        return None;
    }
    let bpm = b[2] as u32;
    let preset = b[3] as usize;
    if !(BPM_MIN..=BPM_MAX).contains(&bpm) || preset >= PRESETS.len() {
        return None;
    }
    let mut g = [[0; STEPS]; ROWS];
    for r in 0..ROWS {
        for s in 0..STEPS {
            let v = b[4 + r * STEPS + s];
            let max = if r == BASS { SCALE.len() as u8 } else { 1 };
            if v > max {
                return None;
            }
            g[r][s] = v;
        }
    }
    Some((g, bpm, preset))
}

/// 底鼓：三角波往下滑。一帧换一个频率，四帧滑完。
const KICK_HZ: [u32; 4] = [150, 100, 70, 55];
const KICK_MS: u32 = 110;

pub struct Game {
    pub frame: Frame,
    pub keys: Keys,
    pub grid: Grid,
    pub bpm: u32,
    pub preset: usize,
    pub cur_row: usize,
    pub cur_step: usize,
    pub playing: bool,
    /// 放到第几步。
    pub step: usize,
    /// 攒的帧：每帧加 `bpm * 2`，满 3600 走一步。
    acc: u32,
    /// 底鼓滑到第几帧；`KICK_HZ.len()` 是滑完了。
    kick_t: usize,
    /// 这一帧发的音调：声道、频率、毫秒、音量。测试看这个（系统那份是全进程共用的）。
    pub out: Vec<(Channel, u32, u32, u32)>,
    /// 一共存了几次。测试看。
    pub saves: u32,
    play_icon: Sprite,
    stop_icon: Sprite,
}

impl Default for Game {
    fn default() -> Self {
        let mut frame = Frame::default();
        for (i, rgb) in PALETTE {
            frame.palette[i as usize] = rgb;
        }
        let (grid, bpm, preset) =
            vibeplay::load(SAVE).and_then(|b| decode(&b)).unwrap_or(([[0; STEPS]; ROWS], BPM_START, 0));
        Game {
            frame,
            keys: Keys::default(),
            grid,
            bpm,
            preset,
            cur_row: 0,
            cur_step: 0,
            playing: false,
            step: 0,
            acc: 0,
            kick_t: KICK_HZ.len(),
            out: Vec::new(),
            saves: 0,
            play_icon: Sprite::art(&["w...", "ww..", "www.", "wwww", "www.", "ww..", "w..."], &[('w', WHITE)]),
            stop_icon: Sprite::art(&["wwwwww"; 6], &[('w', GREY)]),
        }
    }
}

impl Game {
    fn emit(&mut self, ch: Channel, hz: u32, ms: u32, vol: u32) {
        tone(ch, hz, ms, vol);
        self.out.push((ch, hz, ms, vol));
    }

    fn save(&mut self) {
        vibeplay::save(SAVE, &encode(&self.grid, self.bpm, self.preset));
        self.saves += 1;
    }

    /// 一格的声音。放到这一步时响，停着时按 A 也响一下给人听。
    fn sound(&mut self, row: usize, v: u8) {
        if v == 0 {
            return;
        }
        let ms = step_ms(self.bpm);
        match row {
            KICK => {
                self.kick_t = 0;
                self.kick_frame();
            }
            SNARE => self.emit(Channel::Noise, 6000, 90.min(ms), 60),
            HAT => self.emit(Channel::Noise, 16000, 25, 35),
            _ => self.emit(Channel::Square1, hz(SCALE[v as usize - 1]), ms * 4 / 5, 35),
        }
    }

    /// 底鼓滑下去的一帧。
    fn kick_frame(&mut self) {
        if self.kick_t < KICK_HZ.len() {
            let left = KICK_MS - (self.kick_t as u32 * 1000 / 60);
            self.emit(Channel::Triangle, KICK_HZ[self.kick_t], left, 100);
            self.kick_t += 1;
        }
    }

    /// 放到一步：这一列亮着的格都响。军鼓和踩镲都在噪声上，同一步时只响军鼓。
    fn hit(&mut self) {
        let s = self.step;
        let col: [u8; ROWS] = std::array::from_fn(|r| self.grid[r][s]);
        self.sound(KICK, col[KICK]);
        if col[SNARE] != 0 {
            self.sound(SNARE, 1);
        } else {
            self.sound(HAT, col[HAT]);
        }
        self.sound(BASS, col[BASS]);
    }

    fn start(&mut self) {
        self.playing = true;
        self.step = 0;
        self.acc = 0;
        self.hit();
    }

    fn stop(&mut self) {
        self.playing = false;
        self.kick_t = KICK_HZ.len();
        for ch in [Channel::Square1, Channel::Triangle, Channel::Noise] {
            self.emit(ch, 0, 0, 0);
        }
    }

    /// 走一帧：读键、走节拍、发声。
    pub fn step(&mut self) {
        self.out.clear();
        let k = &self.keys;
        let (up, down, left, right) = (k.pressed(key::UP), k.pressed(key::DOWN), k.pressed(key::LEFT), k.pressed(key::RIGHT));
        let (a, b, start, select, l1, r1) = (
            k.pressed(key::A),
            k.pressed(key::B),
            k.pressed(key::START),
            k.pressed(key::SELECT),
            k.pressed(key::L1),
            k.pressed(key::R1),
        );
        // 底鼓先滑完上一帧开的那一下，这一帧再开新的。
        self.kick_frame();
        if up {
            self.cur_row = (self.cur_row + ROWS - 1) % ROWS;
        }
        if down {
            self.cur_row = (self.cur_row + 1) % ROWS;
        }
        if left {
            self.cur_step = (self.cur_step + STEPS - 1) % STEPS;
        }
        if right {
            self.cur_step = (self.cur_step + 1) % STEPS;
        }
        if a {
            let (r, s) = (self.cur_row, self.cur_step);
            let max = if r == BASS { SCALE.len() as u8 } else { 1 };
            let v = (self.grid[r][s] + 1) % (max + 1);
            self.grid[r][s] = v;
            if !self.playing {
                self.sound(r, v);
            }
            self.save();
        }
        if b && self.grid[self.cur_row][self.cur_step] != 0 {
            self.grid[self.cur_row][self.cur_step] = 0;
            self.save();
        }
        if l1 || r1 {
            let bpm = if l1 { self.bpm.saturating_sub(BPM_STEP) } else { self.bpm + BPM_STEP };
            let bpm = bpm.clamp(BPM_MIN, BPM_MAX);
            if bpm != self.bpm {
                self.bpm = bpm;
                self.save();
            }
        }
        if select {
            self.preset = (self.preset + 1) % PRESETS.len();
            self.grid = preset_grid(self.preset);
            self.save();
        }
        if start {
            if self.playing {
                self.stop();
            } else {
                self.start();
                return;
            }
        }
        if self.playing {
            self.acc += self.bpm * 2;
            if self.acc >= FRAMES_PER_MIN {
                self.acc -= FRAMES_PER_MIN;
                self.step = (self.step + 1) % STEPS;
                self.hit();
            }
        }
    }

    pub fn draw(&mut self) {
        let l = lang();
        let f = &mut self.frame;
        f.clear(BG);
        // 上面一条：名字、速度。
        f.rect(0, 0, 240, 16, PANEL);
        f.text(&FONT, 4, 2, l.pick("BEAT BOX", "节拍盒"), WHITE);
        let bpm = format!("{} BPM", self.bpm);
        let w = FONT.width(&bpm);
        f.text(&FONT, 236 - w, 2, &bpm, WHITE);
        if self.playing {
            f.blit(&self.play_icon, 236 - w - 10, 4, false);
        } else {
            f.blit(&self.stop_icon, 236 - w - 11, 5, false);
        }

        // 每拍一个点；放着时一个三角指着这一步。
        for s in 0..STEPS {
            let x = GRID_X + s as i32 * PITCH;
            if s % 2 == 0 {
                f.rect(x + CELL_W / 2 - 1, GRID_Y - 4, 2, 2, BEAT);
            }
        }
        if self.playing {
            let x = GRID_X + self.step as i32 * PITCH + CELL_W / 2;
            for i in 0..3 {
                f.rect(x - 3 + i, GRID_Y - 6 + i, 6 - 2 * i, 1, WHITE);
            }
        }

        let labels = [l.pick("KICK", "底鼓"), l.pick("SNARE", "军鼓"), l.pick("HAT", "踩镲"), l.pick("BASS", "低音")];
        for r in 0..ROWS {
            let y = GRID_Y + r as i32 * PITCH;
            let label_c = if r == self.cur_row { WHITE } else { GREY };
            f.text(&FONT, 4, y + 5, labels[r], label_c);
            for s in 0..STEPS {
                let x = GRID_X + s as i32 * PITCH;
                let head = self.playing && s == self.step;
                let v = self.grid[r][s];
                let c = if v == 0 {
                    if head {
                        CELL_HEAD
                    } else {
                        CELL
                    }
                } else if head {
                    ROW_COLOR[r] + 1
                } else {
                    ROW_COLOR[r]
                };
                f.rect(x, y, CELL_W, CELL_W, c);
                if r == BASS && v != 0 {
                    let name = NOTE_NAMES[v as usize - 1];
                    f.text(&FONT, x + (CELL_W - FONT.width(name)) / 2, y + 5, name, INK);
                }
                if r == self.cur_row && s == self.cur_step {
                    f.frame_rect(x - 1, y - 1, CELL_W + 2, CELL_W + 2, WHITE);
                    f.frame_rect(x, y, CELL_W, CELL_W, INK);
                }
            }
        }

        // 下面：现成节奏的名字，两行按键说明。
        let p = &PRESETS[self.preset];
        let edited = if self.grid == preset_grid(self.preset) { "" } else { l.pick(" (edited)", "（改过）") };
        let pre = format!("{} {}/{}  {}{}", l.pick("PATTERN", "节奏"), self.preset + 1, PRESETS.len(), l.pick(p.en, p.zh), edited);
        f.text(&FONT, 4, 121, &pre, GREY);
        f.rect(0, 134, 240, 26, PANEL);
        f.text(&FONT, 4, 135, l.pick("D-pad move  A note  B clear", "十字键 选格  A 开关  B 清掉"), WHITE);
        f.text(&FONT, 4, 147, l.pick("START play  L1/R1 tempo  SELECT next", "START 放  L1/R1 速度  SELECT 换"), WHITE);
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
