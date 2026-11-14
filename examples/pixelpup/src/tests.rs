use super::pet::*;
use super::*;

const H8: i64 = 8 * 3600;
/// 2026-11-14 12:00:00 UTC。测试里时区都给 0：本地钟点就是 UTC 的。
const NOON: i64 = 1_794_657_600;

fn at(unix: i64) -> Time {
    Time::at(unix, 0)
}

/// 一个白天开着的 app，小狗是刚领养的。
fn home(now: i64) -> Game {
    vibeplay::erase(SAVE);
    Game::new(at(now))
}

fn press(g: &mut Game, k: u32, now: i64) {
    g.keys.set(k);
    g.step_at(at(now));
    g.keys.set(0);
}

#[test]
fn eight_hours_make_the_pup_hungry() {
    let mut p = Pet::new(0);
    assert_eq!(p.catch_up(H8), H8);
    assert_eq!(p.food, 80_000 - 480 * FOOD_PER_MIN);
    assert_eq!(p.fun, 80_000 - 480 * FUN_PER_MIN);
    assert_eq!(p.mood(), Mood::Hungry);
    assert_eq!(p.last, H8);
}

#[test]
fn once_a_second_is_the_same_as_once_at_the_end() {
    let mut a = Pet::new(7);
    let mut b = a;
    for s in 1..=H8 {
        a.catch_up(7 + s);
    }
    b.catch_up(7 + H8);
    assert_eq!(a, b);
    // 不到一分钟的零头留着，不丢。
    let mut c = Pet::new(0);
    c.catch_up(59);
    assert_eq!((c.food, c.last), (80_000, 0));
    c.catch_up(61);
    assert_eq!((c.food, c.last), (80_000 - FOOD_PER_MIN, 60));
}

#[test]
fn a_long_absence_counts_at_most_twelve_hours() {
    let mut full = Pet { food: MAX, fun: MAX, ..Pet::new(0) };
    let mut twelve = full;
    twelve.catch_up(CAP);
    let gap = full.catch_up(3 * 86_400);
    assert_eq!(gap, 3 * 86_400, "离开的真时间照样回");
    assert_eq!((full.food, full.fun), (twelve.food, twelve.fun));
    assert!(full.food > 0, "出门三天回来，狗饿了，但没空");
    assert_eq!(full.last, 3 * 86_400, "超过上限的直接跳到现在");
}

#[test]
fn a_clock_set_back_takes_nothing() {
    let mut p = Pet::new(NOON);
    p.catch_up(NOON - 5 * 3600);
    assert_eq!((p.food, p.fun, p.last), (80_000, 80_000, NOON - 5 * 3600));
}

#[test]
fn night_is_ten_to_seven() {
    for h in [22, 23, 0, 3, 6] {
        assert!(is_night(h), "{h} 点");
    }
    for h in [7, 8, 12, 21] {
        assert!(!is_night(h), "{h} 点");
    }
    assert!(is_night(at(NOON + 11 * 3600).hour));
    assert!(!is_night(at(NOON).hour));
}

#[test]
fn feeding_and_walking() {
    let mut p = Pet { food: 10_000, fun: 10_000, ..Pet::new(0) };
    p.feed(0);
    assert_eq!((p.food, p.fun), (45_000, 13_000));
    p.walk(0);
    assert_eq!((p.food, p.fun), (40_000, 43_000));
    for _ in 0..5 {
        p.feed(0);
    }
    assert_eq!(p.food, MAX);
}

#[test]
fn age_counts_whole_days() {
    let p = Pet::new(NOON);
    assert_eq!(p.age_days(NOON + 86_399), 0);
    assert_eq!(p.age_days(NOON + 3 * 86_400), 3);
}

#[test]
fn the_save_round_trips() {
    let p = Pet { born: 1, last: 2, food: 3, fun: 4 };
    assert_eq!(Pet::decode(&p.encode()), Some(p));
    assert_eq!(Pet::decode("2,1,2,3,4"), None);
    assert_eq!(Pet::decode("junk"), None);
    assert_eq!(Pet::decode("1,1,2,999999,-5").map(|p| (p.food, p.fun)), Some((MAX, 0)));
}

#[test]
fn a_by_day_feeds_and_b_walks() {
    let mut g = home(NOON);
    press(&mut g, key::A, NOON);
    assert_eq!(g.mode, Mode::Eat);
    assert_eq!(g.pet.food, MAX);
    for _ in 0..EAT {
        g.step_at(at(NOON));
    }
    assert_eq!(g.mode, Mode::Home);
    press(&mut g, key::B, NOON);
    assert_eq!(g.mode, Mode::Walk);
    assert_eq!(g.pet.fun, MAX);
}

#[test]
fn a_sleeping_pup_only_grumbles() {
    let night = NOON + 11 * 3600;
    let mut g = home(night);
    let before = g.pet;
    press(&mut g, key::A, night);
    assert_eq!(g.mode, Mode::Grumpy);
    press(&mut g, key::B, night);
    assert_eq!(g.pet, before);
    for _ in 0..GRUMPY {
        g.step_at(at(night));
    }
    assert_eq!(g.mode, Mode::Home);
}

#[test]
fn reopening_catches_up_and_says_how_long() {
    // 用 Game::new 读存档：先存一只八小时前的狗。
    let old = Pet::new(NOON - H8);
    vibeplay::save(SAVE, old.encode().as_bytes());
    let g = Game::new(at(NOON));
    assert_eq!(g.pet.born, NOON - H8);
    assert_eq!(g.pet.food, 80_000 - 480 * FOOD_PER_MIN);
    assert_eq!(g.away, (H8, AWAY));
}

#[test]
fn words_follow_the_system_language() {
    use vibeplay::full::{native::set_lang, Lang};
    let mut g = home(NOON);
    g.draw();
    let en = g.frame.pixels.clone();
    set_lang(Lang::Zh);
    g.draw();
    let zh = g.frame.pixels.clone();
    set_lang(Lang::En);
    assert_ne!(en, zh);
}
