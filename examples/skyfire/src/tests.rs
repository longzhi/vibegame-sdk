use super::*;

fn playing(stage: usize) -> Game {
    let mut g = Game::default();
    g.start(stage);
    g.mode = Mode::Play;
    g.t = 0;
    g.y = 120 * S;
    g
}

/// 走 n 帧，什么键都不按。
fn run(g: &mut Game, n: u32) {
    for _ in 0..n {
        g.keys.set(0);
        g.step();
    }
}

#[test]
fn scripts_parse() {
    for s in &STAGES {
        let w = stages::parse(s.script).unwrap();
        assert!(w.len() > 20, "{}", s.name);
        assert!(w.iter().all(|w| w.at < TOP), "{} 有一条出不来", s.name);
    }
    assert!(stages::parse("10 blimp 3 1").is_err());
}

#[test]
fn a_shot_kills_a_dart() {
    let mut g = playing(0);
    g.waves.clear();
    g.add_enemy(Kind::Dart, 115);
    g.enemies[0].y = 40 * S;
    g.x = 112 * S;
    g.y = 120 * S;
    g.invuln = 999;
    for _ in 0..40 {
        g.keys.set(key::A);
        g.step();
    }
    assert!(g.enemies.is_empty());
    assert_eq!(g.score, 100);
}

#[test]
fn a_hit_costs_a_life_and_a_power_level() {
    let mut g = playing(0);
    g.power = 3;
    g.invuln = 0;
    let (cx, cy) = g.center();
    g.bullets.push(Bullet { x: cx * S, y: cy * S, vx: 0, vy: 0 });
    g.step();
    assert_eq!(g.lives, 2);
    assert_eq!(g.power, 2);
    assert!(g.dead > 0);
    run(&mut g, 100);
    assert_eq!(g.dead, 0);
    assert!(g.invuln > 0, "出来以后一会儿打不着");
}

#[test]
fn no_lives_left_asks_to_continue() {
    let mut g = playing(1);
    g.lives = 1;
    g.invuln = 0;
    g.die();
    run(&mut g, 100);
    assert_eq!(g.mode, Mode::Over);
    run(&mut g, 50);
    g.keys.set(key::A);
    g.step();
    assert_eq!(g.mode, Mode::Play);
    assert_eq!((g.lives, g.continues, g.stage), (3, 2, 1));
}

#[test]
fn a_bomb_clears_bullets() {
    let mut g = playing(0);
    g.invuln = 0;
    g.bullets.push(Bullet { x: 30 * S, y: 30 * S, vx: 0, vy: 0 });
    g.keys.set(key::B);
    g.step();
    assert_eq!(g.bombs, 2);
    assert!(g.bullets.is_empty());
    assert!(g.invuln > 0);
}

#[test]
fn p_raises_power_and_b_adds_a_bomb() {
    let mut g = playing(0);
    let (cx, cy) = g.center();
    g.items.push(Item { kind: 0, x: (cx - 5) * S, y: (cy - 5) * S, t: 0 });
    g.items.push(Item { kind: 1, x: (cx - 5) * S, y: (cy - 5) * S, t: 0 });
    g.step();
    assert_eq!((g.power, g.bombs), (2, 4));
}

#[test]
fn the_scroll_reaches_the_boss() {
    let mut g = playing(0);
    g.invuln = u32::MAX / 2;
    g.lives = 99;
    let mut n = 0;
    while g.boss.is_none() {
        g.keys.set(0);
        g.step();
        n += 1;
        assert!(n < 7000, "大怪一直不出来");
    }
    // 一关大约八十多秒。
    assert!(n > 60 * 60, "{n}");
}

#[test]
fn each_boss_can_be_beaten_and_progress_is_saved() {
    for stage in 0..4 {
        let mut g = playing(stage);
        g.waves.clear();
        g.scroll = TOP * S;
        g.invuln = u32::MAX / 2;
        g.lives = 99;
        g.power = 4;
        run(&mut g, 1);
        assert!(g.boss.is_some());
        let mut n = 0;
        while g.mode == Mode::Play {
            // 主角对着还没打掉的一块（最后是芯），按住 A。
            if let Some(b) = &g.boss {
                let p = b.parts.iter().find(|p| !p.core && p.hp > 0).unwrap_or_else(|| b.core());
                g.x = (b.x + p.dx + p.w / 2 - 8) * S;
            }
            g.y = 120 * S;
            g.invuln = 999;
            g.keys.set(key::A);
            g.step();
            n += 1;
            assert!(n < 60 * 120, "第 {stage} 关大怪两分钟没打下来");
        }
        assert_eq!(g.mode, Mode::Clear);
        run(&mut g, 2);
        assert_eq!(g.unlocked, (stage + 2).min(4));
        assert_eq!(load().0, g.unlocked);
    }
}

#[test]
fn clearing_the_last_stage_shows_the_ending() {
    let mut g = playing(3);
    g.mode = Mode::Clear;
    g.t = 0;
    run(&mut g, 270);
    assert_eq!(g.mode, Mode::Ending);
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
