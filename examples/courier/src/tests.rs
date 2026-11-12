use super::*;
use std::collections::{HashSet, VecDeque};

/// 开第 `i` 关，直接到玩的那一步。
fn play(i: usize) -> Game {
    let mut g = Game { level: i, ..Default::default() };
    g.new_game();
    g.start_level();
    g
}

/// 按着 `bits` 走 `n` 帧。
fn run(g: &mut Game, bits: u32, n: u32) {
    for _ in 0..n {
        g.keys.set(bits);
        g.step();
    }
}

/// 一块平地上的空关：主角站在 (32, 112)，地面在第 8 行。
fn flat() -> Game {
    let mut g = play(0);
    g.map = vec![vec![EMPTY; 60]; ROWS];
    for x in 0..60 {
        g.map[8][x] = GROUND;
        g.map[9][x] = GROUND;
    }
    g.width = 60;
    g.spawns.clear();
    g.mobs.clear();
    g.lifts.clear();
    g.door = (5000, 0);
    g.hero = Hero::new(32, 113);
    g.cam = 0;
    run(&mut g, 0, 2);
    g
}

#[test]
fn every_level_is_well_formed() {
    for l in &LEVELS {
        for (k, part) in l.parts.iter().enumerate() {
            let w = part[0].len();
            assert!(part.iter().all(|r| r.len() == w), "{} 第 {} 段不一样宽：{:?}", l.name, k + 1, part.iter().map(|r| r.len()).collect::<Vec<_>>());
        }
        let rows = levels::rows(l);
        let all: String = rows.concat();
        assert_eq!(all.matches('@').count(), 1, "{} 要一个起点", l.name);
        assert_eq!(all.matches('D').count(), 1, "{} 要一个门", l.name);
    }
}

/// 从起点到门走得到：站得住的格之间，往上 4 格内、横着 4 格内跳得过去（跑起来上 3 格能跳 5 格），往下随便掉。
/// 踏板走过的格都算站得住。
#[test]
fn every_level_can_be_finished() {
    for (li, l) in LEVELS.iter().enumerate() {
        let mut g = Game::default();
        g.load(li);
        let rows = levels::rows(l);
        let w = g.width as i32;
        let solid_at = |x: i32, y: i32| -> bool { solid(g.tile(x, y)) || g.tile(x, y) == HIDDEN };
        let mut stand: HashSet<(i32, i32)> = HashSet::new();
        for x in 0..w {
            for y in 0..9 {
                if !solid_at(x, y) && g.tile(x, y) != LIQUID && (solid_at(x, y + 1)) {
                    stand.insert((x, y));
                }
            }
        }
        for lift in &g.lifts {
            let (tx, ty) = (lift.origin.0 / T, lift.origin.1 / T);
            if lift.vertical {
                for dy in -2..=2 {
                    for dx in 0..3 {
                        stand.insert((tx + dx, ty + dy - 1));
                    }
                }
            } else {
                for dx in -6..=8 {
                    stand.insert((tx + dx, ty - 1));
                }
            }
        }
        let start = rows.iter().enumerate().find_map(|(y, r)| r.find('@').map(|x| (x as i32, y as i32))).unwrap();
        let goal = rows.iter().enumerate().find_map(|(y, r)| r.find('D').map(|x| (x as i32, y as i32))).unwrap();
        let mut seen = HashSet::from([start]);
        let mut q = VecDeque::from([start]);
        while let Some((x, y)) = q.pop_front() {
            for &(nx, ny) in &stand {
                if seen.contains(&(nx, ny)) {
                    continue;
                }
                let (dx, up) = ((nx - x).abs(), y - ny);
                let ok = (up <= 4 && dx <= 4) || (up <= 3 && dx <= 5) || (up <= 0 && dx <= 6) || (up < 0 && dx <= 7);
                if ok {
                    seen.insert((nx, ny));
                    q.push_back((nx, ny));
                }
            }
        }
        let far = seen.iter().map(|p| p.0).max().unwrap();
        assert!(seen.contains(&goal), "{} 从起点走不到门：最远到第 {} 列", l.name, far);
    }
}

#[test]
fn holding_a_jumps_higher_and_running_jumps_farther() {
    let peak = |hold: u32, run_first: bool| {
        let mut g = flat();
        if run_first {
            run(&mut g, key::RIGHT | key::B, 60);
        }
        let y0 = g.hero.px().1;
        let x0 = g.hero.px().0;
        let mut top = y0;
        let base = if run_first { key::RIGHT | key::B } else { 0 };
        for f in 0..90 {
            g.keys.set(base | if f < hold { key::A } else { 0 });
            g.step();
            top = top.min(g.hero.px().1);
            if f > 3 && g.hero.ground {
                break;
            }
        }
        (y0 - top, g.hero.px().0 - x0)
    };
    let (tap, _) = peak(1, false);
    let (hold, _) = peak(60, false);
    let (run_h, run_dx) = peak(60, true);
    assert!(tap <= 20, "点一下跳得低：{tap}");
    assert!((56..=72).contains(&hold), "按住跳四格左右：{hold}");
    assert!(run_h > hold, "跑起来跳得更高：{run_h} > {hold}");
    assert!(run_dx > 90, "跑起来跳得远：{run_dx}");
}

#[test]
fn turning_around_skids() {
    let mut g = flat();
    run(&mut g, key::RIGHT | key::B, 50);
    assert!(g.hero.vx >= RUN_MAX - ACC_RUN);
    g.keys.set(key::LEFT);
    g.step();
    assert!(g.hero.skid, "跑着回头先刹车");
    assert!(g.hero.vx > 0);
}

#[test]
fn stomping_a_walker_flattens_it_and_combos_score_more() {
    let mut g = flat();
    for x in [64, 84] {
        let mut m = Mob::new(Kind::Walker, x, 113);
        m.vx = 0;
        g.mobs.push(m);
    }
    // 从第一个头上落下来。
    g.hero.x = 64 * FX;
    g.hero.y = 90 * FX;
    g.hero.vy = 512;
    g.hero.ground = false;
    g.hero.coyote = 0;
    run(&mut g, key::A, 6);
    assert!(g.mobs.iter().any(|m| m.flat > 0), "踩扁了");
    assert!(g.hero.vy < 0, "弹起来");
    assert_eq!(g.score, 100);
    // 弹起来落到第二个头上：200 分。
    g.hero.x = 84 * FX;
    for _ in 0..80 {
        g.keys.set(0);
        g.step();
        if g.score > 100 {
            break;
        }
    }
    assert_eq!(g.score, 300, "连踩第二个 200 分");
}

#[test]
fn bumping_crates() {
    let mut g = flat();
    g.map[5][3] = MAIL;
    g.map[5][4] = CRATE;
    // 小个子顶包裹箱：一张邮票，箱子空了。
    g.hero.x = 50 * FX;
    run(&mut g, key::A, 30);
    assert_eq!(g.stamps, 1);
    assert_eq!(g.map[5][3], USED);
    // 小个子顶木箱：顶不碎。
    let mut g = flat();
    g.map[5][4] = CRATE;
    g.hero.x = 66 * FX;
    run(&mut g, key::A, 30);
    assert_eq!(g.map[5][4], CRATE);
    // 大个子顶得碎。
    let mut g = flat();
    g.map[5][4] = CRATE;
    g.hero.big = true;
    g.hero.y -= 15 * FX;
    g.hero.x = 66 * FX;
    run(&mut g, key::A, 30);
    assert_eq!(g.map[5][4], EMPTY);
    assert!(!g.debris.is_empty());
}

#[test]
fn parcel_grows_and_a_hit_shrinks() {
    let mut g = flat();
    g.map[5][3] = MAIL_POWER;
    g.hero.x = 50 * FX;
    run(&mut g, key::A, 20);
    assert!(g.mobs.iter().any(|m| m.kind == Kind::Parcel), "顶出加急包裹");
    // 包裹往右滑，主角追上去拿。
    for _ in 0..200 {
        g.keys.set(key::RIGHT);
        g.step();
        if g.hero.big {
            break;
        }
    }
    assert!(g.hero.big, "拿到变大");
    run(&mut g, 0, 50);
    // 大了再顶：出纸飞机。
    g.map[5][10] = MAIL_POWER;
    g.hero.x = 162 * FX;
    g.hero.vx = 0;
    run(&mut g, key::A, 20);
    assert!(g.mobs.iter().any(|m| m.kind == Kind::PlaneItem), "大个子顶出纸飞机");
    // 挨一下变小，闪着时再挨不算，闪完再挨就倒下。
    g.hurt_hero();
    assert!(!g.hero.big);
    run(&mut g, 0, 41);
    g.hurt_hero();
    assert!(g.hero.dead.is_none());
    g.hero.hurt = 0;
    g.hurt_hero();
    assert!(g.hero.dead.is_some());
}

#[test]
fn plane_power_throws_planes_that_knock_enemies() {
    let mut g = flat();
    g.hero.big = true;
    g.hero.plane = true;
    g.hero.y -= 15 * FX;
    let mut m = Mob::new(Kind::Spiky, 120, 113);
    m.vx = 0;
    g.mobs.push(m);
    run(&mut g, key::B, 1);
    assert_eq!(g.mobs.iter().filter(|m| m.kind == Kind::Shot).count(), 1);
    run(&mut g, 0, 40);
    assert!(g.mobs.iter().all(|m| m.kind != Kind::Spiky || m.dying > 0), "纸飞机打倒刺海胆");
    assert_eq!(g.score, 200);
}

#[test]
fn can_bug_retreats_and_a_kicked_can_knocks_others() {
    let mut g = flat();
    let mut bug = Mob::new(Kind::CanBug, 64, 113);
    bug.vx = 0;
    g.mobs.push(bug);
    let mut w = Mob::new(Kind::Walker, 200, 113);
    w.vx = 0;
    g.mobs.push(w);
    g.hero.x = 64 * FX;
    g.hero.y = 90 * FX;
    g.hero.vy = 512;
    g.hero.ground = false;
    g.hero.coyote = 0;
    run(&mut g, 0, 6);
    assert_eq!(g.mobs[0].kind, Kind::Can, "踩一下缩进罐头");
    assert_eq!(g.mobs[0].vx, 0);
    // 落地以后从左边碰它：往右踢出去。
    run(&mut g, 0, 40);
    g.hero.x = 48 * FX;
    g.hero.y = 113 * FX;
    run(&mut g, key::RIGHT, 6);
    let can = g.mobs.iter().find(|m| m.kind == Kind::Can).unwrap();
    assert!(can.vx > 0, "踢出去往右滚");
    run(&mut g, 0, 80);
    assert!(g.mobs.iter().all(|m| m.kind != Kind::Walker || m.dying > 0), "滚着的罐头撞倒发条兵");
}

#[test]
fn gold_stamp_makes_enemies_fall_on_touch() {
    let mut g = flat();
    g.hero.star = STAR_TIME;
    let mut m = Mob::new(Kind::Spiky, 60, 113);
    m.vx = 0;
    g.mobs.push(m);
    run(&mut g, key::RIGHT, 30);
    assert!(g.mobs.iter().all(|m| m.dying > 0));
    assert!(g.hero.dead.is_none());
}

#[test]
fn hidden_crate_shows_up_only_from_below() {
    let mut g = flat();
    g.map[5][3] = HIDDEN;
    // 从旁边走过去没事，跳起来在下面顶出来。
    g.hero.x = 50 * FX;
    run(&mut g, key::A, 20);
    assert_eq!(g.map[5][3], USED);
    assert!(g.mobs.iter().any(|m| m.kind == Kind::Love));
}

#[test]
fn time_out_and_falling_kill() {
    let mut g = flat();
    g.time = 1;
    run(&mut g, 0, TIME_FRAMES + 1);
    assert!(g.hero.dead.is_some());
    let mut g = flat();
    g.map[8][3] = EMPTY;
    g.map[9][3] = EMPTY;
    g.map[8][4] = EMPTY;
    g.map[9][4] = EMPTY;
    g.hero.x = 52 * FX;
    run(&mut g, 0, 60);
    assert!(g.hero.dead.is_some(), "掉进坑里");
}

#[test]
fn hundred_stamps_give_a_life() {
    let mut g = flat();
    let lives = g.lives;
    g.stamps = 99;
    g.add_stamp();
    assert_eq!((g.stamps, g.lives), (0, lives + 1));
}

#[test]
fn walking_into_the_door_clears_the_level() {
    let mut g = flat();
    g.door = (96, 112);
    run(&mut g, key::RIGHT, 80);
    assert_eq!(g.mode, Mode::Clear);
    let time = g.time;
    let score = g.score;
    for _ in 0..700 {
        g.keys.set(0);
        g.step();
        if g.mode == Mode::Intro {
            break;
        }
    }
    assert!(g.score >= score + time * 50);
    assert_eq!(g.mode, Mode::Intro);
    assert_eq!(g.level, 1);
    assert_eq!(g.unlocked, 1);
}

#[test]
fn the_boss_takes_three_stomps_and_unlocks_the_door() {
    let mut g = play(5);
    // 直接到最后的台子。
    let door = g.door;
    g.hero.x = (door.0 - 160) * FX;
    g.hero.y = 113 * FX;
    g.cam = (door.0 - 240).max(0);
    run(&mut g, 0, 2);
    g.spawn_mobs();
    let b = g.mobs.iter().position(|m| m.kind == Kind::Boss).expect("大盗出场");
    for _ in 0..3 {
        let b = g.mobs.iter().position(|m| m.kind == Kind::Boss).unwrap();
        g.mobs[b].hurt = 0;
        g.stomp(b);
    }
    let _ = b;
    assert!(g.boss_dead);
    run(&mut g, 0, 200);
    assert!(g.mobs.iter().all(|m| m.kind != Kind::Boss));
}

#[test]
fn soot_hides_while_the_hero_stands_next_to_the_chimney() {
    let mut g = flat();
    let mut s = Mob::new(Kind::Soot, 100, 96);
    s.timer = 110;
    g.mobs.push(s);
    g.hero.x = 96 * FX;
    run(&mut g, 0, 60);
    assert!(g.soot_hidden(0), "主角在旁边，不出来");
    g.hero.x = 20 * FX;
    run(&mut g, 0, 60);
    assert!(!g.soot_hidden(0), "走开了才出来");
}

#[test]
fn lifts_carry_the_hero() {
    let mut g = flat();
    g.lifts.push(Lift { x: 160, y: 100, vertical: false, origin: (160, 100), t: 0, dx: 0, dy: 0 });
    g.hero.x = 170 * FX;
    g.hero.y = 80 * FX;
    run(&mut g, 0, 20);
    assert_eq!(g.hero.on_lift, Some(0));
    let x0 = g.hero.px().0;
    run(&mut g, 0, 30);
    assert!(g.hero.px().0 > x0 + 20, "跟着踏板走");
}

#[test]
fn title_follows_the_system_language() {
    use vibeplay::full::{native::set_lang, Lang};
    let mut g = Game::default();
    g.draw();
    let en = g.frame.pixels.clone();
    set_lang(Lang::Zh);
    g.draw();
    let zh = g.frame.pixels.clone();
    set_lang(Lang::En);
    assert_ne!(en, zh);
}
