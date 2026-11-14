use super::*;
use vibeplay::full::native::set_lang;
use vibeplay::full::Lang;

/// 按一下键再松开，走 n 帧。
fn press(g: &mut Game, k: u32) {
    g.keys.set(k);
    g.step();
    g.keys.set(0);
    g.step();
}

fn run(g: &mut Game, n: u32) {
    for _ in 0..n {
        g.keys.set(0);
        g.step();
    }
}

/// 一个开了一局的：10 以内，第一题。
fn asking(level: Level) -> Game {
    let mut g = Game { level, ..Default::default() };
    g.start(0);
    g
}

/// 按对的那个方向回答这道题。
fn dir(i: usize) -> u32 {
    [key::LEFT, key::UP, key::RIGHT][i]
}

#[test]
fn questions_stay_in_range_with_no_negatives() {
    for level in Level::ALL {
        let mut rng = Rng(7);
        for _ in 0..2000 {
            let q = rules::question(&mut rng, level);
            let max = level.max();
            assert!(q.a <= max && q.b <= max && q.b >= 1, "{q:?}");
            if q.plus {
                assert!(q.a + q.b <= max, "{q:?}");
            } else {
                assert!(q.b <= q.a, "减出负数：{q:?}");
            }
            if level == Level::Twenty {
                assert!(q.a.max(q.answer()) > 10, "20 以内的题要过 10：{q:?}");
            }
        }
    }
}

#[test]
fn three_different_choices_one_right_and_wrong_ones_are_near() {
    for level in Level::ALL {
        let mut rng = Rng(99);
        for _ in 0..2000 {
            let q = rules::question(&mut rng, level);
            let ans = q.answer();
            assert_eq!(q.choices[q.right], ans);
            assert_eq!(q.choices.iter().filter(|&&c| c == ans).count(), 1, "{q:?}");
            assert!(q.choices[0] != q.choices[1] && q.choices[1] != q.choices[2] && q.choices[0] != q.choices[2]);
            for c in q.choices {
                assert!(c <= level.max() && c.abs_diff(ans) <= 3, "{q:?}");
            }
        }
    }
}

#[test]
fn the_right_answer_is_not_always_in_the_same_place() {
    let mut rng = Rng(1);
    let mut seen = [0; 3];
    for _ in 0..300 {
        seen[rules::question(&mut rng, Level::Ten).right] += 1;
    }
    assert!(seen.iter().all(|&n| n > 60), "{seen:?}");
}

#[test]
fn a_round_has_ten_different_questions_and_rounds_differ() {
    let a = rules::round(&mut Rng(rules::seed(1, Level::Ten, 30)), Level::Ten);
    let b = rules::round(&mut Rng(rules::seed(2, Level::Ten, 30)), Level::Ten);
    assert_eq!(a.len(), ROUND);
    for (i, q) in a.iter().enumerate() {
        assert!(a[i + 1..].iter().all(|p| (p.a, p.b, p.plus) != (q.a, q.b, q.plus)));
    }
    assert_ne!(a, b, "下一局要换题");
    // 同一个种子出同样的题：截图和测试才稳。
    assert_eq!(a, rules::round(&mut Rng(rules::seed(1, Level::Ten, 30)), Level::Ten));
}

#[test]
fn up_down_picks_the_level_and_a_starts() {
    let mut g = Game::default();
    assert_eq!(g.mode, Mode::Title);
    press(&mut g, key::DOWN);
    assert_eq!(g.level, Level::Twenty);
    press(&mut g, key::UP);
    assert_eq!(g.level, Level::Ten);
    press(&mut g, key::DOWN);
    press(&mut g, key::A);
    assert_eq!(g.mode, Mode::Ask);
    assert_eq!(g.questions.len(), ROUND);
    assert!(g.questions.iter().all(|q| q.a.max(q.answer()) > 10));
}

#[test]
fn a_right_answer_earns_a_star_and_moves_on() {
    let mut g = asking(Level::Ten);
    let r = g.question().right;
    press(&mut g, dir(r));
    assert_eq!(g.mode, Mode::Right);
    assert_eq!(g.stars(), 1);
    run(&mut g, RIGHT_FRAMES);
    assert_eq!((g.mode, g.n), (Mode::Ask, 1));
}

#[test]
fn a_wrong_answer_shows_the_right_one_then_moves_on() {
    let mut g = asking(Level::Ten);
    let r = g.question().right;
    press(&mut g, dir((r + 1) % 3));
    assert_eq!(g.mode, Mode::Wrong);
    assert_eq!(g.stars(), 0);
    run(&mut g, 20);
    assert_eq!(g.mode, Mode::Wrong, "要给时间看对的答案");
    run(&mut g, WRONG_FRAMES);
    assert_eq!((g.mode, g.n), (Mode::Ask, 1));
}

#[test]
fn b_goes_back_to_the_title() {
    let mut g = asking(Level::Ten);
    press(&mut g, key::B);
    assert_eq!(g.mode, Mode::Title);
}

#[test]
fn ten_questions_end_the_round_and_the_best_is_kept_per_level() {
    vibeplay::erase(SAVE);
    let mut g = asking(Level::Ten);
    // 对 7 道，错 3 道。
    for i in 0..ROUND {
        let r = g.question().right;
        press(&mut g, dir(if i < 7 { r } else { (r + 2) % 3 }));
        run(&mut g, WRONG_FRAMES + 1);
    }
    assert_eq!(g.mode, Mode::End);
    assert_eq!(g.stars(), 7);
    assert_eq!(g.best, [Some(7), None]);
    assert!(g.new_best);
    // 再来一次，这次只对 5 道：最好成绩还是 7。
    let first = g.questions.clone();
    run(&mut g, 40);
    press(&mut g, key::A);
    assert_eq!(g.mode, Mode::Ask);
    assert_ne!(g.questions, first);
    for i in 0..ROUND {
        let r = g.question().right;
        press(&mut g, dir(if i < 5 { r } else { (r + 1) % 3 }));
        run(&mut g, WRONG_FRAMES + 1);
    }
    assert_eq!(g.stars(), 5);
    assert_eq!(g.best, [Some(7), None]);
    assert!(!g.new_best);
    // 下次打开还在。
    let g = Game::default();
    assert_eq!(g.best, [Some(7), None]);
    assert_eq!(g.rounds, 2);
    assert_eq!(rules::better(Some(7), 9), Some(9));
}

#[test]
fn every_screen_fits_in_both_languages() {
    for l in [Lang::En, Lang::Zh] {
        set_lang(l);
        let mut g = asking(Level::Twenty);
        g.best = [Some(10), Some(10)];
        for s in [
            lang().pick("←↑→ ANSWER    B MENU", "←↑→ 选答案   B 回菜单"),
            lang().pick("A PLAY AGAIN    B MENU", "A 再来一次   B 回菜单"),
            lang().pick("KEEP PRACTICING!", "多练练就会了！"),
        ] {
            assert!(FONT.width(s) <= W as i32 - 8, "{s}");
        }
        assert!(FONT.width(lang().pick("KEEP PRACTICING!", "多练练就会了！")) * 2 <= W as i32);
        // 最长的题：两位数减两位数。
        let q = Question { a: 20, b: 19, plus: false, choices: [0; 3], right: 0 };
        assert!((numbers(&format!("{} ", q.left()), INK).w + 1 + numbers("20", INK).w) * 4 <= W as i32 - 8);
        g.draw();
    }
    set_lang(Lang::En);
}
