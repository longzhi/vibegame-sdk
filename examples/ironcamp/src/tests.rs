use super::*;
use std::collections::VecDeque;

/// 开第 `i` 关，跳过出车的光。
fn play(i: usize) -> Game {
    let mut g = Game { stage: i, ..Default::default() };
    g.new_game();
    for t in g.tanks.iter_mut() {
        t.spawn = 0;
    }
    g
}

fn me(g: &Game) -> &Tank {
    g.tanks.iter().find(|t| t.is_player()).unwrap()
}

fn me_mut(g: &mut Game) -> &mut Tank {
    g.tanks.iter_mut().find(|t| t.is_player()).unwrap()
}

/// 按着 `bits` 走 `n` 帧（按键每帧重设）。
fn run(g: &mut Game, bits: u32, n: u32) {
    for _ in 0..n {
        g.keys.set(bits);
        g.step();
    }
}

/// 不算砖墙（能打掉），车从 `from` 能不能开到 `to`。按 8 像素一格找。
fn reach(g: &Game, from: (i32, i32), to: (i32, i32)) -> bool {
    let solid = |x: i32, y: i32| {
        if x < 0 || y < 0 || x + 16 > FIELD || y + 16 > FIELD {
            return true;
        }
        (y / CELL..=(y + 15) / CELL).any(|cy| (x / CELL..=(x + 15) / CELL).any(|cx| matches!(g.ground[cy as usize][cx as usize], Ground::Steel | Ground::Water)))
    };
    let mut seen = vec![false; 400];
    let mut q = VecDeque::from([from]);
    while let Some((x, y)) = q.pop_front() {
        if (x, y) == to {
            return true;
        }
        for (dx, dy) in [(8, 0), (-8, 0), (0, 8), (0, -8)] {
            let (nx, ny) = (x + dx, y + dy);
            if solid(nx, ny) {
                continue;
            }
            let k = (ny / 8 * 20 + nx / 8) as usize;
            if !seen[k] {
                seen[k] = true;
                q.push_back((nx, ny));
            }
        }
    }
    false
}

#[test]
fn every_level_loads_and_is_open() {
    for i in 0..LEVELS.len() {
        let mut g = Game::default();
        g.load_map(i);
        for row in LEVELS[i] {
            assert_eq!(row.chars().count(), 10, "第 {} 关有一行不是 10 个字", i + 1);
        }
        assert_eq!(MIXES[i].iter().map(|&n| n as usize).sum::<usize>(), PER_STAGE, "第 {} 关不是二十辆", i + 1);
        for &at in GATES.iter().chain(HOMES.iter()) {
            assert!(!g.blocked(at.0, at.1, 16, 16), "第 {} 关 {:?} 被挡住", i + 1, at);
        }
        // 每个出口都开得到营地外面（营地上面那格），一号车也开得到。
        for &at in GATES.iter().chain(HOMES.iter()) {
            assert!(reach(&g, at, (72, 120)), "第 {} 关从 {:?} 到不了营地", i + 1, at);
        }
    }
}

#[test]
fn a_shot_takes_a_strip_off_a_brick() {
    let mut g = play(0);
    g.tanks.retain(|t| t.is_player());
    // 第一关左数第二列从第 1 行到第 4 行是砖：像素 x 16..32、y 16..80。车放在下面朝上打。
    let t = me_mut(&mut g);
    (t.x, t.y, t.dir) = (16, 80, Dir::Up);
    assert!(g.bricks[19][4] && g.bricks[18][4]);
    g.fire(0);
    for _ in 0..30 {
        g.move_shots();
    }
    // 最下面那排小块（第 19 排）打掉了 16 像素宽，上面那排还在。
    assert!((4..8).all(|mx| !g.bricks[19][mx]), "{:?}", &g.bricks[19][..10]);
    assert!((4..8).all(|mx| g.bricks[18][mx]));
}

#[test]
fn steel_and_water_stop_a_tank_and_steel_stops_a_shot() {
    let mut g = play(2);
    g.tanks.retain(|t| t.is_player());
    // 第三关第 4 行是水：y 48..64。
    assert_eq!(g.ground[6][0], Ground::Water);
    let t = me_mut(&mut g);
    (t.x, t.y, t.dir) = (0, 72, Dir::Up);
    for _ in 0..40 {
        g.drive(0, Dir::Up, 16);
    }
    assert_eq!(me(&g).y, 64, "开进水里了");
    // 第一关中间的钢：打不掉。
    let mut g = play(0);
    g.tanks.retain(|t| t.is_player());
    assert_eq!(g.ground[6][8], Ground::Steel);
    let t = me_mut(&mut g);
    (t.x, t.y, t.dir) = (64, 64, Dir::Up);
    g.fire(0);
    for _ in 0..30 {
        g.move_shots();
    }
    assert_eq!(g.ground[6][8], Ground::Steel);
    assert!(g.shots.is_empty());
    // 火力满了打得掉。
    me_mut(&mut g).level = 3;
    g.fire(0);
    for _ in 0..30 {
        g.move_shots();
    }
    assert_eq!(g.ground[7][8], Ground::Open);
}

#[test]
fn turning_snaps_to_the_grid() {
    let mut g = play(0);
    g.tanks.retain(|t| t.is_player());
    // 第一关第 0 行是空的：在 y = 3 往右开，再往下拐，对齐到 y = 0。
    let t = me_mut(&mut g);
    (t.x, t.y, t.dir) = (0, 3, Dir::Up);
    g.drive(0, Dir::Right, 16);
    assert_eq!(me(&g).y, 0);
    let t = me_mut(&mut g);
    (t.x, t.y, t.dir) = (5, 0, Dir::Right);
    g.drive(0, Dir::Down, 16);
    assert_eq!(me(&g).x, 8, "5 离 8 近");
}

#[test]
fn shield_saves_the_player_and_base_hit_ends_the_game() {
    let mut g = play(0);
    let lives = g.lives[0];
    let (x, y) = (me(&g).x, me(&g).y);
    // 刚出来有护盾：敌车的子弹打中也没事。
    assert!(me(&g).shield > 0);
    g.shots.push(Shot { x: x + 6, y: y + 2, dir: Dir::Down, owner: 99, from_player: false, speed: 3, pierce: false, alive: true });
    g.move_shots();
    g.move_shots();
    assert!(g.tanks.iter().any(|t| t.is_player()));
    // 没护盾：打中少一条命。
    me_mut(&mut g).shield = 0;
    g.shots.push(Shot { x: x + 6, y: y + 2, dir: Dir::Down, owner: 99, from_player: false, speed: 3, pierce: false, alive: true });
    g.move_shots();
    g.move_shots();
    assert!(!g.tanks.iter().any(|t| t.is_player()));
    assert_eq!(g.lives[0], lives - 1);
    assert!(g.over.is_none());
    // 打中营地：输了。
    g.shots.push(Shot { x: BASE.0 + 6, y: BASE.1 - 6, dir: Dir::Down, owner: 99, from_player: false, speed: 3, pierce: false, alive: true });
    // 营地上面那圈砖先挡着：拿掉它。
    for &(cx, cy) in &RING {
        g.set_brick_cell(cx, cy, false);
    }
    for _ in 0..5 {
        g.move_shots();
    }
    assert!(!g.base_alive);
    assert!(g.over.is_some());
}

#[test]
fn shovel_turns_the_ring_to_steel_and_back() {
    let mut g = play(0);
    g.apply(Bonus::Shovel, 0);
    for &(cx, cy) in &RING {
        assert_eq!(g.ground[cy][cx], Ground::Steel);
    }
    g.tanks.retain(|t| t.is_player());
    g.queue.clear();
    for _ in 0..900 {
        g.keys.set(0);
        g.step();
    }
    for &(cx, cy) in &RING {
        assert_eq!(g.ground[cy][cx], Ground::Open);
        assert!(g.bricks[cy * 2][cx * 2]);
    }
}

#[test]
fn clock_freezes_enemies() {
    let mut g = play(0);
    run(&mut g, 0, 120);
    let e = g.tanks.iter().find(|t| !t.is_player() && t.active()).cloned().expect("有敌车出来了");
    let i = g.tanks.iter().position(|t| t.is_player()).unwrap();
    g.apply(Bonus::Clock, i);
    run(&mut g, 0, 100);
    let now = g.tanks.iter().find(|t| t.id == e.id).unwrap();
    assert_eq!((now.x, now.y), (e.x, e.y));
}

#[test]
fn bomb_and_power_up() {
    let mut g = play(0);
    run(&mut g, 0, 200);
    assert!(g.enemies_alive() > 0);
    let i = g.tanks.iter().position(|t| t.is_player()).unwrap();
    g.apply(Bonus::Power, i);
    g.apply(Bonus::Power, i);
    assert_eq!(me(&g).level, 2);
    let i = g.tanks.iter().position(|t| t.is_player()).unwrap();
    g.apply(Bonus::Bomb, i);
    assert_eq!(g.tanks.iter().filter(|t| !t.is_player() && t.active()).count(), 0);
    // 火力 2 能连打两发。
    let i = g.tanks.iter().position(|t| t.is_player()).unwrap();
    me_mut(&mut g).spawn = 0;
    g.fire(i);
    g.fire(i);
    g.fire(i);
    assert_eq!(g.shots.iter().filter(|s| s.from_player).count(), 2);
}

#[test]
fn armor_takes_four_hits() {
    let mut g = play(0);
    let mut t = Tank::new(500, Kind::Armor, (64, 0));
    t.spawn = 0;
    g.tanks.push(t);
    let i = g.tanks.len() - 1;
    for n in 0..3 {
        g.damage(i);
        assert_eq!(g.tanks[i].hp, 3 - n);
    }
    g.damage(i);
    assert!(!g.tanks.iter().any(|t| t.id == 500));
    assert_eq!(g.killed[3], 1);
    assert_eq!(g.score[0], 400);
}

#[test]
fn clearing_a_stage_goes_to_the_tally_and_the_next_stage() {
    let mut g = play(0);
    g.queue.clear();
    g.tanks.retain(|t| t.is_player());
    run(&mut g, 0, 160);
    assert_eq!(g.mode, Mode::Tally);
    run(&mut g, 0, 200);
    run(&mut g, key::A, 1);
    assert_eq!(g.mode, Mode::Play);
    assert_eq!(g.stage, 1);
    assert_eq!(g.unlocked, 1);
}

#[test]
fn twenty_enemies_with_three_carriers() {
    let mut g = play(0);
    assert_eq!(g.queue.len(), PER_STAGE);
    let mut carriers = 0;
    let mut seen = 0;
    // 打掉出来的车，一直到二十辆都出来。
    for _ in 0..20000 {
        g.keys.set(0);
        g.step();
        let fresh: Vec<_> = g.tanks.iter().filter(|t| !t.is_player() && t.spawn == 47).map(|t| t.carrier).collect();
        seen += fresh.len();
        carriers += fresh.iter().filter(|c| **c).count();
        g.tanks.retain(|t| t.is_player() || t.spawn > 0);
        g.base_alive = true;
        g.over = None;
        if let Some(i) = g.tanks.iter().position(|t| t.is_player()) {
            g.tanks[i].shield = 10;
        }
        if g.queue.is_empty() {
            break;
        }
    }
    assert_eq!(seen, PER_STAGE);
    assert_eq!(carriers, 3);
}

