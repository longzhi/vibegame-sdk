use super::*;
use std::collections::{HashSet, VecDeque};

/// 开第 `i` 关，直接到打的那一步。
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

/// 一块平地上的空关：主角站在 (32, 90)，地面在第 7 行。
fn flat() -> Game {
    let mut g = play(0);
    g.map = vec![vec![EMPTY; 60]; 10];
    for x in 0..60 {
        for y in 7..10 {
            g.map[y][x] = ROCK;
        }
    }
    g.width = 60;
    g.height = 10;
    g.spawns.clear();
    g.mobs.clear();
    g.bridges.clear();
    g.boss_at = (5000, 96);
    g.hero = Hero::new(32, 90);
    g.cam = (0, 0);
    run(&mut g, 0, 2);
    g
}

fn add(g: &mut Game, k: Kind, x: i32, y: i32) -> usize {
    let id = g.id();
    g.mobs.push(Mob::new(id, k, x, y));
    g.mobs.len() - 1
}

#[test]
fn every_level_is_well_formed() {
    for l in &LEVELS {
        let rows = levels::rows(l);
        let w = rows[0].len();
        assert!(rows.iter().all(|r| r.len() == w), "{} 每行要一样宽", l.name);
        if l.vertical {
            assert_eq!(w, 15, "{} 竖着的关一屏宽", l.name);
        } else {
            assert_eq!(rows.len(), 10, "{}", l.name);
        }
        let count = |c: char| rows.iter().map(|r| r.matches(c).count()).sum::<usize>();
        assert_eq!(count('@'), 1, "{} 一个起点", l.name);
        assert_eq!(count('X'), 1, "{} 一个大怪", l.name);
        assert!(count('C') + count('P') >= 4, "{} 至少四个掉勋章的地方", l.name);
    }
}

/// 照主角跳得多高多远，从起点一格一格找，走得到大怪的场地。桥当它已经炸了：下面的水接得住。
#[test]
fn every_level_can_reach_the_boss() {
    for (li, l) in LEVELS.iter().enumerate() {
        let mut g = Game::default();
        g.load(li);
        let (w, h) = (g.width as i32, g.height as i32);
        let mut stand: HashSet<(i32, i32)> = HashSet::new();
        for x in 0..w {
            for y in 0..h {
                let t = g.tile(x, y + 1);
                let here = g.tile(x, y);
                let ok = (solid(t) || t == LEDGE || t == POOL) && !solid(here) && here != POOL && !solid(g.tile(x, y - 1));
                if ok {
                    stand.insert((x, y));
                }
            }
        }
        let rows = levels::rows(l);
        let start = rows.iter().enumerate().find_map(|(y, r)| r.find('@').map(|x| (x as i32, y as i32))).unwrap();
        let mut seen = HashSet::from([start]);
        let mut q = VecDeque::from([start]);
        while let Some((x, y)) = q.pop_front() {
            for &(nx, ny) in &stand {
                if seen.contains(&(nx, ny)) {
                    continue;
                }
                let (dx, up) = ((nx - x).abs(), y - ny);
                let ok = (up <= 2 && dx <= 2) || (up <= 0 && dx <= 3) || (up < -1 && dx <= 4);
                if ok {
                    seen.insert((nx, ny));
                    q.push_back((nx, ny));
                }
            }
        }
        let (bx, by) = (g.boss_at.0 / T, g.boss_at.1 / T);
        let reached = if l.vertical { seen.iter().any(|p| p.1 <= 8) } else { seen.iter().any(|p| p.0 >= bx - 4) };
        let far = if l.vertical { seen.iter().map(|p| p.1).min().unwrap() } else { seen.iter().map(|p| p.0).max().unwrap() };
        assert!(reached, "{} 从起点走不到大怪（{bx},{by}）：最远到 {far}", l.name);
    }
}

#[test]
fn jumping_is_a_fixed_arc_with_a_spin() {
    let mut g = flat();
    let (_, y0) = g.hero.px();
    g.keys.set(key::A);
    g.step();
    assert!(g.hero.spin, "跳起来翻跟头");
    let mut top = y0;
    for _ in 0..60 {
        g.keys.set(0);
        g.step();
        top = top.min(g.hero.px().1);
    }
    let rise = y0 - top;
    assert!((36..=44).contains(&rise), "跳得高 2 格多一点：{rise}");
    assert!(g.hero.ground && !g.hero.spin && g.hero.px().1 == y0, "落回地上");
}

#[test]
fn down_lies_flat_and_dodges_high_shots() {
    let mut g = flat();
    run(&mut g, key::DOWN, 2);
    assert!(g.hero.prone);
    let b = g.hero.hurt_box();
    assert!(b.1 >= 104, "趴下以后身子很矮：{b:?}");
    // 齐胸高飞过来的子弹打不着。
    let (hx, hy) = g.hero.px();
    g.eshots.push(EShot { x: (hx + 30) * FX, y: (hy + 8) * FX, vx: -512, vy: 0, kind: EKind::Bullet, bounces: 0, gone: false });
    run(&mut g, key::DOWN, 30);
    assert_eq!(g.hero.dead, 0, "趴着躲过去了");
    // 趴着开枪，子弹贴着地飞。
    g.keys.set(key::DOWN | key::B);
    g.step();
    let s = g.shots.last().unwrap();
    assert!(px(s.y) > hy + 14, "子弹低：{}", px(s.y));
}

#[test]
fn down_and_a_drops_through_a_ledge() {
    let mut g = flat();
    for x in 0..10 {
        g.map[5][x] = LEDGE;
    }
    g.hero = Hero::new(32, 80 - HH);
    run(&mut g, 0, 3);
    assert!(g.hero.ground && g.hero.px().1 == 80 - HH, "站在台子上");
    g.keys.set(key::DOWN | key::A);
    g.step();
    run(&mut g, 0, 40);
    assert_eq!(g.hero.px().1, 112 - HH, "掉到下面的地上");
    // 从下面跳上去。
    run(&mut g, key::A, 1);
    run(&mut g, 0, 30);
    assert!(g.hero.ground && g.hero.px().1 == 80 - HH, "从下面跳得上台子：{}", g.hero.px().1);
}

#[test]
fn eight_ways_to_aim() {
    let mut g = flat();
    let cases = [(0, 0), (key::UP, 48), (key::UP | key::RIGHT, 56), (key::DOWN | key::RIGHT, 8), (key::LEFT, 32), (key::UP | key::LEFT, 40), (key::DOWN | key::LEFT, 24)];
    for (bits, want) in cases {
        g.hero.face = 1;
        g.shots.clear();
        g.hero.cool = 0;
        g.keys.set(bits);
        g.step();
        g.keys.set(bits | key::B);
        g.step();
        let s = g.shots.last().expect("打出去一发");
        assert_eq!(angle_to(s.vx, s.vy, 1), want, "{bits:b}");
    }
    // 在空中按下：往正下打。
    g.shots.clear();
    run(&mut g, key::A, 1);
    run(&mut g, 0, 4);
    g.keys.set(key::DOWN);
    g.step();
    g.keys.set(key::DOWN | key::B);
    g.step();
    let s = g.shots.last().unwrap();
    assert_eq!(angle_to(s.vx, s.vy, 1), 16);
}

#[test]
fn each_gun_fires_its_own_way() {
    let mut g = flat();
    g.hero.weapon = Weapon::Spread;
    run(&mut g, key::B, 1);
    assert_eq!(g.shots.len(), 5, "散弹一次五发");
    // 按住 B 一秒，数打出去几发。
    let fired = |g: &mut Game, w: Weapon| {
        g.shots.clear();
        g.hero.weapon = w;
        run(g, 0, 30);
        let mut n = 0;
        for _ in 0..60 {
            let before = g.shots.len();
            run(g, key::B, 1);
            n += g.shots.len().saturating_sub(before);
        }
        n
    };
    let machine = fired(&mut g, Weapon::Machine);
    let normal = fired(&mut g, Weapon::Normal);
    assert!(machine >= 9, "机枪按住连发：{machine}");
    assert!((4..=5).contains(&normal), "普通枪按住慢慢连发：{normal}");
    // 激光穿过一排兵。
    g.shots.clear();
    g.hero.weapon = Weapon::Laser;
    let (hx, hy) = g.hero.px();
    for k in 0..3 {
        let i = add(&mut g, Kind::Rifle, hx + 40 + k * 16, hy);
        g.mobs[i].hp = 3;
    }
    run(&mut g, 0, 30);
    run(&mut g, key::B, 1);
    run(&mut g, 0, 40);
    assert!(g.mobs.iter().all(|m| m.hp <= 0), "一发激光打倒三个：{:?}", g.mobs.iter().map(|m| m.hp).collect::<Vec<_>>());
}

#[test]
fn a_shot_pod_drops_a_medal_that_changes_the_gun() {
    let mut g = flat();
    let (hx, hy) = g.hero.px();
    let i = add(&mut g, Kind::Pod, hx + 60, hy - 20);
    g.mobs[i].letter = 'S';
    g.mobs[i].vx = 0;
    g.mobs[i].home.1 = hy - 20;
    g.kill_mob(i);
    let m = g.mobs.iter().find(|m| m.kind == Kind::Medal).expect("掉了勋章");
    assert_eq!(m.letter, 'S');
    run(&mut g, 0, 60);
    let m = g.mobs.iter().find(|m| m.kind == Kind::Medal).expect("勋章落到地上");
    assert!(m.ground);
    let mx = m.px().0;
    g.hero.x = (mx - 2) * FX;
    run(&mut g, 0, 2);
    assert_eq!(g.hero.weapon, Weapon::Spread, "拿到散弹");
    assert!(g.mobs.iter().all(|m| m.kind != Kind::Medal));
}

#[test]
fn pillbox_only_takes_hits_while_open() {
    let mut g = flat();
    let i = add(&mut g, Kind::Pillbox, 120, 96);
    g.map[6][7] = PILL_T;
    g.mobs[i].timer = 0;
    run(&mut g, 0, 1);
    assert_eq!(g.mobs[i].state, 0);
    assert!(g.mobs[i].weak().is_none(), "关着打不着");
    g.mobs[i].timer = 115;
    run(&mut g, 0, 1);
    assert_eq!(g.mobs[i].state, 1, "开了");
    assert!(g.mobs[i].weak().is_some());
}

#[test]
fn turret_turns_toward_the_hero() {
    let mut g = flat();
    let i = add(&mut g, Kind::Turret, 150, 40);
    g.mobs[i].aim = 0;
    run(&mut g, 0, 200);
    let want = angle_to(37 - 158, 100 - 48, 4);
    let a = g.mobs.iter().find(|m| m.kind == Kind::Turret).unwrap().aim;
    assert!((a - want).rem_euclid(64) <= 4 || (want - a).rem_euclid(64) <= 4, "炮台转过来了：{a} 要 {want}");
}

#[test]
fn one_hit_kills_drops_the_gun_and_costs_a_life() {
    let mut g = flat();
    g.hero.weapon = Weapon::Laser;
    g.hero.safe = 0;
    let lives = g.lives;
    let (hx, hy) = g.hero.px();
    g.eshots.push(EShot { x: (hx + 5) * FX, y: (hy + 10) * FX, vx: 0, vy: 0, kind: EKind::Bullet, bounces: 0, gone: false });
    run(&mut g, 0, 1);
    assert!(g.hero.dead > 0, "挨一下就倒");
    assert_eq!(g.hero.weapon, Weapon::Normal, "枪丢了");
    run(&mut g, 0, DEAD_TIME + 2);
    assert_eq!(g.hero.dead, 0);
    assert_eq!(g.lives, lives - 1);
    assert!(g.hero.safe > 0, "接着来的时候一阵子不怕打");
}

#[test]
fn no_lives_left_asks_to_continue() {
    let mut g = flat();
    g.lives = 0;
    g.hero.safe = 0;
    g.kill_hero();
    run(&mut g, 0, DEAD_TIME + 2);
    assert_eq!(g.mode, Mode::Over);
    let c = g.continues;
    run(&mut g, 0, 40);
    run(&mut g, key::A, 1);
    assert_eq!(g.mode, Mode::Intro, "接着打");
    assert_eq!(g.continues, c - 1);
    assert_eq!(g.lives, LIVES);
    assert_eq!(g.score, 0);
}

#[test]
fn diving_hides_from_bullets() {
    let mut g = flat();
    for x in 0..10 {
        g.map[7][x] = POOL;
        g.map[8][x] = POOL;
        g.map[9][x] = POOL;
    }
    g.hero = Hero::new(32, 90);
    run(&mut g, 0, 30);
    assert!(g.hero.in_water, "站在水里");
    assert_eq!(g.hero.px().1 + HH, 112 + 10, "水没到腰");
    run(&mut g, key::DOWN, 2);
    assert!(g.hero.diving);
    let (hx, hy) = g.hero.px();
    g.eshots.push(EShot { x: (hx + 5) * FX, y: (hy + 10) * FX, vx: 0, vy: 0, kind: EKind::Bullet, bounces: 0, gone: false });
    run(&mut g, key::DOWN, 2);
    assert_eq!(g.hero.dead, 0, "潜着打不着");
    // 潜着不能开枪。
    run(&mut g, key::DOWN | key::B, 1);
    assert!(g.shots.is_empty());
}

#[test]
fn the_bridge_blows_up_under_your_feet() {
    let mut g = play(0);
    let b = g.bridges[0].clone();
    let x = b.from as i32 * T + 4;
    g.hero = Hero::new(x, b.row as i32 * T - HH);
    g.cam.0 = (x - 100).max(0);
    run(&mut g, 0, 3);
    assert!(g.bridges[0].lit, "走上去就开始炸");
    run(&mut g, 0, 16 * (b.to - b.from + 2) as u32);
    assert!((b.from..=b.to).all(|c| g.map[b.row][c] == EMPTY), "一节一节都炸了");
}

#[test]
fn flames_burn_on_and_off() {
    let mut g = play(3);
    let (tx, ty) = (0..g.width as i32).flat_map(|x| (0..10).map(move |y| (x, y))).find(|&(x, y)| g.tile(x, y) == NOZZLE).unwrap();
    g.cam.0 = (tx * T - 100).max(0);
    let on: Vec<bool> = (0..150).map(|t| {
        g.tick = t;
        g.flames().iter().any(|f| f.0 == tx * T)
    }).collect();
    let n = on.iter().filter(|b| **b).count();
    assert_eq!(n, 60, "150 帧里喷 60 帧");
    // 站在火上就倒。
    g.tick = (0..150).find(|&t| (t + tx as u32 * 37).is_multiple_of(150)).unwrap();
    g.hero = Hero::new(tx * T + 3, ty * T - HH);
    g.hero.safe = 0;
    g.hurt_hero();
    assert!(g.hero.dead > 0);
}

#[test]
fn lobbed_grenades_burst_on_the_ground() {
    let mut g = flat();
    g.eshots.push(EShot { x: 100 * FX, y: 60 * FX, vx: 0, vy: 0, kind: EKind::Shell, bounces: 0, gone: false });
    run(&mut g, 0, 60);
    assert!(g.eshots.is_empty(), "落地炸了");
    assert!(g.booms.iter().any(|b| b.hurts) || g.booms.is_empty(), "炸开的一团伤人");
}

/// 每个大怪都打得死：一直打它的弱点。
#[test]
fn every_boss_can_be_beaten() {
    for (li, l) in LEVELS.iter().enumerate() {
        let mut g = play(li);
        let (mx, _) = g.max_cam();
        if g.vertical {
            g.cam.1 = 0;
        } else {
            g.cam.0 = mx;
        }
        g.hero.safe = 100_000;
        g.update_cam();
        assert!(g.boss_on, "{} 画面到头大怪出来", l.name);
        let boss = g.mobs.iter().position(|m| m.kind.boss()).unwrap();
        let id = g.mobs[boss].id;
        for _ in 0..200 {
            let Some(i) = g.mobs.iter().position(|m| m.id == id) else { break };
            if g.mobs[i].hp <= 0 {
                break;
            }
            let (x, y, w, h) = g.mobs[i].weak().unwrap();
            g.shots.push(Shot { x: (x + w / 2) * FX, y: (y + h / 2) * FX, vx: 0, vy: 0, kind: Weapon::Normal, base: (0, 0), phase: 0, hit: vec![], gone: false });
            g.hero.safe = 100_000;
            run(&mut g, 0, 1);
        }
        assert!(g.boss_dead, "{} 的大怪打倒了", l.name);
        g.hero.safe = 100_000;
        run(&mut g, 0, 200);
        assert_eq!(g.mode, Mode::Clear, "{} 过关", l.name);
    }
}

#[test]
fn the_camera_only_moves_forward() {
    let mut g = play(0);
    run(&mut g, key::RIGHT, 200);
    let c = g.cam.0;
    assert!(c > 0);
    run(&mut g, key::LEFT, 100);
    assert_eq!(g.cam.0, c, "往回走画面不动");
    assert!(g.hero.px().0 >= c, "出不了画面左边");
    // 竖着的关只往上走。
    let g = play(1);
    assert_eq!(g.cam.1, g.max_cam().1, "从最下面开始");
}

#[test]
fn clearing_saves_progress_and_score_gives_lives() {
    let mut g = flat();
    let l = g.lives;
    g.add_score(20000);
    assert_eq!(g.lives, l + 1, "两万分多一条命");
    g.add_score(40000);
    assert_eq!(g.lives, l + 2, "六万分再多一条");
    g.mode = Mode::Clear;
    g.timer = 0;
    run(&mut g, 0, 245);
    assert_eq!(g.level, 1);
    assert_eq!(g.unlocked, 1);
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
