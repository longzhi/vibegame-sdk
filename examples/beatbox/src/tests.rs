use super::*;

/// 按一下键：这一帧按着，下一帧松开。
fn press(g: &mut Game, k: u32) {
    g.keys.set(k);
    g.step();
    g.keys.set(0);
    g.step();
}

/// 一个空的：先清掉存档。
fn empty() -> Game {
    vibeplay::erase(SAVE);
    Game::default()
}

/// 放起来，记下每一步开头在第几帧（从按 START 那一帧算 0）。
fn step_frames(bpm: u32, frames: u32) -> Vec<(u32, usize)> {
    let mut g = empty();
    g.bpm = bpm;
    g.keys.set(key::START);
    g.step();
    let mut at = vec![(0, g.step)];
    for n in 1..frames {
        g.keys.set(0);
        let before = g.step;
        g.step();
        if g.step != before {
            at.push((n, g.step));
        }
    }
    at
}

#[test]
fn steps_at_120_bpm_are_15_frames() {
    let at = step_frames(120, 121);
    let frames: Vec<u32> = at.iter().map(|a| a.0).collect();
    assert_eq!(frames, [0, 15, 30, 45, 60, 75, 90, 105, 120]);
    let steps: Vec<usize> = at.iter().map(|a| a.1).collect();
    assert_eq!(steps, [0, 1, 2, 3, 4, 5, 6, 7, 0], "八步以后回到头");
}

#[test]
fn steps_at_90_and_150_bpm() {
    let f90: Vec<u32> = step_frames(90, 61).iter().map(|a| a.0).collect();
    assert_eq!(f90, [0, 20, 40, 60]);
    let f150: Vec<u32> = step_frames(150, 49).iter().map(|a| a.0).collect();
    assert_eq!(f150, [0, 12, 24, 36, 48]);
}

#[test]
fn uneven_tempo_averages_out() {
    // 140 BPM：一步 12.857 帧。一圈八步要 102.86 帧，七步正好 90 帧。
    let at = step_frames(140, 200);
    assert_eq!(at[7].0, 90);
    for w in at.windows(2) {
        let d = w[1].0 - w[0].0;
        assert!(d == 12 || d == 13, "隔 {d} 帧");
    }
}

#[test]
fn a_toggles_drums_and_cycles_bass_notes() {
    let mut g = empty();
    press(&mut g, key::A);
    assert_eq!(g.grid[KICK][0], 1);
    press(&mut g, key::A);
    assert_eq!(g.grid[KICK][0], 0);
    // 上一格绕到最下面一行：低音。
    press(&mut g, key::UP);
    assert_eq!(g.cur_row, BASS);
    let mut seen = vec![];
    for _ in 0..7 {
        press(&mut g, key::A);
        seen.push(g.grid[BASS][0]);
    }
    assert_eq!(seen, [1, 2, 3, 4, 5, 6, 0]);
    press(&mut g, key::A);
    press(&mut g, key::B);
    assert_eq!(g.grid[BASS][0], 0, "B 清掉");
}

#[test]
fn toggling_previews_only_when_stopped() {
    let mut g = empty();
    g.keys.set(key::DOWN);
    g.step();
    g.keys.set(key::A);
    g.step();
    assert_eq!(g.out, [(Channel::Noise, 6000, 90, 60)]);
}

#[test]
fn playing_sounds_the_right_channels() {
    let mut g = empty();
    g.grid = preset_grid(1);
    g.keys.set(key::START);
    g.step();
    // 第 0 步：底鼓、踩镲、低音 C3。
    assert_eq!(g.out, [(Channel::Triangle, 150, 110, 100), (Channel::Noise, 16000, 25, 35), (Channel::Square1, 131, 200, 35)]);
    g.keys.set(0);
    g.step();
    assert_eq!(g.out, [(Channel::Triangle, 100, 94, 100)], "底鼓往下滑");
    // 走到第 2 步：军鼓和踩镲同一步，只响军鼓。
    for _ in 0..29 {
        g.step();
    }
    assert_eq!(g.step, 2);
    assert_eq!(g.out, [(Channel::Noise, 6000, 90, 60)]);
}

#[test]
fn stop_silences() {
    let mut g = empty();
    press(&mut g, key::START);
    g.keys.set(key::START);
    g.step();
    assert!(!g.playing);
    assert!(g.out.iter().all(|t| t.1 == 0 && t.2 == 0));
}

#[test]
fn tempo_is_clamped_and_saved() {
    let mut g = empty();
    let s = g.saves;
    press(&mut g, key::R1);
    assert_eq!(g.bpm, 125);
    assert_eq!(g.saves, s + 1);
    for _ in 0..40 {
        press(&mut g, key::L1);
    }
    assert_eq!(g.bpm, BPM_MIN);
    let s = g.saves;
    press(&mut g, key::L1);
    assert_eq!(g.saves, s, "到头了不再存");
}

#[test]
fn select_cycles_presets_starting_from_empty() {
    let mut g = empty();
    press(&mut g, key::SELECT);
    assert_eq!(g.preset, 1);
    assert_eq!(g.grid, preset_grid(1));
    for _ in 0..3 {
        press(&mut g, key::SELECT);
    }
    assert_eq!(g.preset, 0);
    assert_eq!(g.grid, [[0; STEPS]; ROWS]);
}

#[test]
fn each_change_saves_and_reloads() {
    let mut g = empty();
    press(&mut g, key::SELECT);
    press(&mut g, key::RIGHT);
    press(&mut g, key::A);
    press(&mut g, key::R1);
    let (grid, bpm, preset) = decode(&vibeplay::load(SAVE).unwrap()).unwrap();
    assert_eq!((grid, bpm, preset), (g.grid, 125, 1));
    let h = Game::default();
    assert_eq!(h.grid, g.grid);
    assert_eq!(h.bpm, 125);
}

#[test]
fn bad_saves_are_ignored() {
    assert!(decode(b"").is_none());
    let mut b = encode(&preset_grid(3), 120, 3);
    assert_eq!(decode(&b), Some((preset_grid(3), 120, 3)));
    b[2] = 250;
    assert!(decode(&b).is_none(), "速度出界");
    let mut b = encode(&preset_grid(0), 120, 0);
    b[4] = 2;
    assert!(decode(&b).is_none(), "鼓的格只能是 0 或 1");
}

#[test]
fn both_languages_fit() {
    use vibeplay::full::native::set_lang;
    use vibeplay::full::Lang;
    for l in [Lang::En, Lang::Zh] {
        set_lang(l);
        let mut g = empty();
        g.draw();
        set_lang(Lang::En);
    }
    assert!(FONT.width("START 放  L1/R1 速度  SELECT 换") <= 232);
    assert!(FONT.width("START play  L1/R1 tempo  SELECT next") <= 232);
}
