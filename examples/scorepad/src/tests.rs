use super::*;
use vibeplay::full::{native, Lang};

/// 存档清掉，刚打开的。
fn fresh() -> Game {
    vibeplay::erase(SAVE);
    Game::default()
}

/// 已经开局、4 个人的。
fn playing() -> Game {
    let mut g = fresh();
    press(&mut g, key::A);
    assert_eq!(g.mode, Mode::Main);
    g
}

/// 按一下：按着一帧，再松开一帧。
fn press(g: &mut Game, k: u32) {
    g.keys.set(k);
    g.step();
    g.keys.set(0);
    g.step();
}

/// 按着 `k` 走 n 帧，再松开一帧。
fn hold(g: &mut Game, k: u32, n: u32) {
    for _ in 0..n {
        g.keys.set(k);
        g.step();
    }
    g.keys.set(0);
    g.step();
}

fn saved() -> String {
    String::from_utf8(vibeplay::load(SAVE).unwrap()).unwrap()
}

#[test]
fn first_open_asks_how_many_players() {
    let mut g = fresh();
    assert_eq!(g.mode, Mode::Setup);
    assert_eq!(g.players, 4);
    press(&mut g, key::RIGHT);
    press(&mut g, key::RIGHT);
    press(&mut g, key::RIGHT);
    assert_eq!(g.players, 6, "最多 6 个");
    for _ in 0..6 {
        press(&mut g, key::LEFT);
    }
    assert_eq!(g.players, 2, "最少 2 个");
    press(&mut g, key::A);
    assert_eq!(g.mode, Mode::Main);
    assert_eq!(saved(), "v1,2,0,0,0,0,0,0,0");
}

#[test]
fn hold_to_repeat_speeds_up() {
    // 刚按下一次，然后等 24 帧，再越来越快。
    let fires: Vec<u32> = (1..=200).filter(|&h| repeat_fires(h)).collect();
    assert_eq!(&fires[..4], &[1, 25, 33, 41]);
    assert!(!repeat_fires(0));
    assert!((2..25).all(|h| !repeat_fires(h)), "刚按下以后先停一下");
    let gaps = |from: u32, to: u32| {
        let f: Vec<u32> = fires.iter().copied().filter(|&h| h >= from && h < to).collect();
        f.windows(2).map(|w| w[1] - w[0]).max().unwrap()
    };
    assert_eq!(gaps(25, 90), 8);
    assert_eq!(gaps(90, 180), 4);
    assert_eq!(gaps(180, 201), 2);

    // 在 app 里：点一下是 1 分；按住一秒（60 帧）是 1 + 5 分。
    let mut g = playing();
    press(&mut g, key::UP);
    assert_eq!(g.scores[0], 1);
    hold(&mut g, key::UP, 60);
    assert_eq!(g.scores[0], 1 + fires.iter().filter(|&&h| h <= 60).count() as i32);
    hold(&mut g, key::DOWN, 1);
    assert_eq!(g.scores[0], 6);
}

#[test]
fn scores_stop_at_the_limits() {
    let mut g = playing();
    g.scores[0] = MIN_SCORE;
    press(&mut g, key::DOWN);
    assert_eq!(g.scores[0], MIN_SCORE);
    g.scores[0] = MAX_SCORE;
    press(&mut g, key::UP);
    assert_eq!(g.scores[0], MAX_SCORE);
}

#[test]
fn a_goes_to_the_next_player_and_wraps() {
    let mut g = playing();
    for want in [1, 2, 3, 0, 1] {
        press(&mut g, key::A);
        assert_eq!(g.cur, want);
    }
    press(&mut g, key::LEFT);
    press(&mut g, key::LEFT);
    assert_eq!(g.cur, 3, "左边从第一个绕到最后一个");
    press(&mut g, key::RIGHT);
    assert_eq!(g.cur, 0);
}

#[test]
fn b_undoes_the_last_change_and_goes_back_to_that_player() {
    let mut g = playing();
    press(&mut g, key::UP);
    press(&mut g, key::UP);
    press(&mut g, key::A);
    hold(&mut g, key::UP, 60); // 按住连加算一步
    press(&mut g, key::A);
    assert_eq!((g.scores[0], g.scores[1], g.cur), (2, 6, 2));

    press(&mut g, key::B);
    assert_eq!((g.scores[1], g.cur), (0, 1), "一整段连加一次撤掉，并且选回那个人");
    press(&mut g, key::B);
    assert_eq!((g.scores[0], g.cur), (1, 0));
    press(&mut g, key::B);
    assert_eq!(g.scores[0], 0);
    press(&mut g, key::B);
    assert_eq!(g.scores[..4], [0, 0, 0, 0], "没得撤就不动");
    assert!(saved().starts_with("v1,4,0,0,0"), "撤销也存");
}

#[test]
fn every_change_is_saved_and_comes_back() {
    let mut g = playing();
    press(&mut g, key::UP);
    assert_eq!(saved(), "v1,4,0,1,0,0,0,0,0", "点一下马上存");
    press(&mut g, key::A);
    press(&mut g, key::DOWN);
    press(&mut g, key::DOWN);
    assert_eq!(saved(), "v1,4,1,1,-2,0,0,0,0");

    // 按住连加时半秒存一次，不等松手。
    for _ in 0..100 {
        g.keys.set(key::UP);
        g.step();
    }
    let mid: Vec<i32> = saved().split(',').map(|v| v.parse().unwrap_or(0)).collect();
    assert!(mid[4] > 0 && mid[4] <= g.scores[1], "按着时也存过：{mid:?}");

    // 合上盖再打开：重新起一个 Game，分和轮到谁都在，不再问几个人。
    g.keys.set(0);
    g.step();
    let back = Game::default();
    assert_eq!(back.mode, Mode::Main);
    assert_eq!((back.players, back.cur, back.scores), (4, 1, g.scores));
}

#[test]
fn broken_save_starts_over() {
    fresh();
    vibeplay::save(SAVE, b"v1,9,7,1,2,x");
    assert_eq!(Game::default().mode, Mode::Setup);
    vibeplay::save(SAVE, b"v1,9,7,1,2,3,4,5,6");
    let g = Game::default();
    assert_eq!((g.players, g.cur), (6, 5), "人数和轮到谁夹在范围里");
}

#[test]
fn crown_goes_to_the_leaders() {
    let mut g = playing();
    assert!(g.leaders().is_empty(), "都是 0 分时没有人领先");
    g.scores[..4].copy_from_slice(&[3, 7, 7, -1]);
    assert_eq!(g.leaders(), vec![1, 2]);
    g.scores[4] = 99; // 不在局里的人不算
    assert_eq!(g.leaders(), vec![1, 2]);
}

#[test]
fn select_rolls_a_die_that_lands_on_one_to_six() {
    let mut g = playing();
    let mut seen = [false; 7];
    for i in 0..60 {
        for _ in 0..i % 7 {
            g.keys.set(0);
            g.step();
        }
        press(&mut g, key::SELECT);
        let d = g.die.expect("掷了");
        // 转的时候按上下不加分。
        hold(&mut g, key::UP, 10);
        assert_eq!(g.scores[0], 0);
        for _ in 0..ROLL {
            g.keys.set(0);
            g.step();
        }
        let done = g.die.expect("转完了还在");
        assert_eq!(done.face, d.result);
        seen[d.result as usize] = true;
        press(&mut g, key::B);
        assert!(g.die.is_none(), "按一下收起来");
        assert!(g.undo.is_empty(), "收骰子的那一下不算撤销");
    }
    assert!(seen[1..].iter().all(|&s| s), "六面都掷到过：{seen:?}");
}

#[test]
fn menu_new_game_asks_first() {
    let mut g = playing();
    press(&mut g, key::UP);
    press(&mut g, key::START);
    assert_eq!(g.mode, Mode::Menu);
    press(&mut g, key::A);
    assert_eq!(g.mode, Mode::Confirm);
    press(&mut g, key::B);
    assert_eq!((g.mode, g.scores[0]), (Mode::Menu, 1), "不清就回菜单，分还在");
    press(&mut g, key::A);
    press(&mut g, key::A);
    assert_eq!((g.mode, g.scores[0]), (Mode::Main, 0));
    assert_eq!(saved(), "v1,4,0,0,0,0,0,0,0");
}

#[test]
fn menu_changes_the_number_of_players() {
    let mut g = playing();
    g.scores[..4].copy_from_slice(&[5, 6, 7, 8]);
    g.cur = 3;
    press(&mut g, key::START);
    press(&mut g, key::DOWN);
    assert_eq!(g.menu, MENU_PLAYERS);
    press(&mut g, key::LEFT);
    assert_eq!((g.players, g.cur), (3, 2), "少了人，轮到的人退回最后一个");
    press(&mut g, key::RIGHT);
    press(&mut g, key::RIGHT);
    assert_eq!(g.players, 5);
    assert_eq!(g.scores[..5], [5, 6, 7, 0, 0], "留下的人分不变，新来的从 0 起");
    assert!(saved().starts_with("v1,5,"));
    press(&mut g, key::START);
    assert_eq!(g.mode, Mode::Main);
}

#[test]
fn words_fit_on_the_screen_in_both_languages() {
    let mut g = playing();
    for l in [Lang::En, Lang::Zh] {
        native::set_lang(l);
        for p in 0..MAX_PLAYERS {
            assert!(FONT.width(color_name(p)) <= 48);
        }
        for s in [
            lang().pick("↑↓ SCORE A NEXT B UNDO SEL DIE", "↑↓分 A下一个 B撤销 SEL骰子"),
            lang().pick("↑↓ PICK  ←→ CHANGE  B BACK", "↑↓选  ←→改  B返回"),
        ] {
            assert!(FONT.width(s) <= W as i32 - 4, "{s} 太宽");
        }
        g.draw();
    }
    native::set_lang(Lang::En);
}
