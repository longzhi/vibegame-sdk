use super::*;
use vibeplay::full::native::set_lang;
use vibeplay::full::Lang;

/// 2026-11-14（星期六）hh:mm:ss，UTC。
fn at(h: i64, m: i64, s: i64) -> Time {
    Time::at(1_794_614_400 + h * 3600 + m * 60 + s, 0)
}

fn digits(hour: u32, minute: u32, h12: bool) -> (String, Option<Half>) {
    let (d, half) = clock(hour, minute, h12);
    let s = d.iter().map(|d| d.map_or(' ', |d| (b'0' + d) as char)).collect();
    (s, half)
}

#[test]
fn the_test_day_is_a_saturday() {
    let t = at(0, 0, 0);
    assert_eq!((t.year, t.month, t.day, t.weekday), (2026, 11, 14, 5));
}

#[test]
fn twenty_four_hours_pads_with_zero() {
    assert_eq!(digits(0, 0, false), ("0000".into(), None));
    assert_eq!(digits(7, 5, false), ("0705".into(), None));
    assert_eq!(digits(12, 0, false), ("1200".into(), None));
    assert_eq!(digits(23, 59, false), ("2359".into(), None));
}

#[test]
fn twelve_hours_midnight_and_noon() {
    // 零点是 12 AM，中午是 12 PM。
    assert_eq!(digits(0, 0, true), ("1200".into(), Some(Half::Am)));
    assert_eq!(digits(0, 59, true), ("1259".into(), Some(Half::Am)));
    assert_eq!(digits(11, 59, true), ("1159".into(), Some(Half::Am)));
    assert_eq!(digits(12, 0, true), ("1200".into(), Some(Half::Pm)));
    assert_eq!(digits(12, 30, true), ("1230".into(), Some(Half::Pm)));
    // 1 到 9 点前面空着，不写 0。
    assert_eq!(digits(1, 5, true), (" 105".into(), Some(Half::Am)));
    assert_eq!(digits(13, 0, true), (" 100".into(), Some(Half::Pm)));
    assert_eq!(digits(22, 10, true), ("1010".into(), Some(Half::Pm)));
    assert_eq!(digits(23, 59, true), ("1159".into(), Some(Half::Pm)));
}

#[test]
fn light_follows_the_hour() {
    assert_eq!(light(5), Light::Night);
    assert_eq!(light(6), Light::Day);
    assert_eq!(light(12), Light::Day);
    assert_eq!(light(16), Light::Day);
    assert_eq!(light(17), Light::Evening);
    assert_eq!(light(19), Light::Evening);
    assert_eq!(light(20), Light::Night);
    assert_eq!(light(0), Light::Night);
}

#[test]
fn date_line_in_both_languages() {
    set_lang(Lang::En);
    assert_eq!(date_line(&at(9, 0, 0)), "Saturday, Nov 14");
    set_lang(Lang::Zh);
    assert_eq!(date_line(&at(9, 0, 0)), "11月14日 星期六");
    // 过了零点是下一天，星期日。
    assert_eq!(date_line(&at(24, 0, 0)), "11月15日 星期日");
    set_lang(Lang::En);
    assert_eq!(date_line(&at(24, 0, 0)), "Sunday, Nov 15");
}

#[test]
fn every_character_is_in_the_font() {
    // 漏了的字会画成空心框。把所有会出的字都查一遍。
    let mut all = String::new();
    for d in 0..7 {
        for m in 1..=12 {
            let t = Time { unix: 0, year: 2026, month: m, day: 30, weekday: d, hour: 0, minute: 0, second: 0, tz_offset: 0 };
            for l in [Lang::En, Lang::Zh] {
                set_lang(l);
                all += &date_line(&t);
            }
        }
    }
    for l in [Lang::En, Lang::Zh] {
        set_lang(l);
        let mut g = Game::default();
        for toast in [Toast::Help, Toast::Hours, Toast::Theme] {
            g.toast = Some((toast, 10));
            for theme in 0..THEMES.len() {
                g.theme = theme;
                g.h12 = theme % 2 == 0;
                g.draw(&at(13, 0, 0));
            }
        }
        all += lang().pick("AM PM", "上午下午");
        all += "A：12/24 小时  上下：颜色 12-hour clock 24-hour clock 颜色：Color: ";
    }
    for th in &THEMES {
        all += th.en;
        all += th.zh;
    }
    set_lang(Lang::En);
    for c in all.chars() {
        assert!(FONT.glyph(c).is_some(), "字库里没有「{c}」");
    }
}

#[test]
fn a_toggles_twelve_hours_and_it_is_saved() {
    let mut g = Game::default();
    let before = g.h12;
    g.keys.set(key::A);
    g.step(&at(9, 0, 0));
    assert_eq!(g.h12, !before);
    assert_eq!(g.toast.map(|t| t.0), Some(Toast::Hours));
    let saved = vibeplay::load(SAVE).unwrap();
    assert_eq!(saved[0], g.h12 as u8);
    // 按住不算再按一次。
    g.keys.set(key::A);
    g.step(&at(9, 0, 0));
    assert_eq!(g.h12, !before);
}

#[test]
fn up_and_down_cycle_the_theme() {
    let mut g = Game { theme: 0, ..Game::default() };
    g.keys.set(key::DOWN);
    g.step(&at(9, 0, 0));
    assert_eq!(g.theme, THEMES.len() - 1);
    g.keys.set(0);
    g.step(&at(9, 0, 0));
    g.keys.set(key::UP);
    g.step(&at(9, 0, 0));
    assert_eq!(g.theme, 0);
    assert_eq!(g.toast.map(|t| t.0), Some(Toast::Theme));
}

#[test]
fn bad_save_falls_back_to_defaults() {
    vibeplay::save(SAVE, &[1, 200]);
    let g = Game::default();
    assert!(g.h12);
    assert!(g.theme < THEMES.len());
}

#[test]
fn colon_blinks_once_a_second() {
    let mut g = Game::default();
    let mut ons = 0;
    for i in 0..120 {
        g.keys.set(0);
        g.step(&at(9, 0, i / 60));
        if g.colon_on() {
            ons += 1;
        }
    }
    assert_eq!(ons, 60);
}

#[test]
fn the_help_goes_away_by_itself() {
    let mut g = Game::default();
    for _ in 0..HELP_FRAMES {
        g.keys.set(0);
        g.step(&at(9, 0, 0));
    }
    assert_eq!(g.toast, None);
}

#[test]
fn the_same_frame_draws_the_same_pixels() {
    // 鱼只看帧数：同一帧、同一时间画出来一样，截图才稳。
    let draw = |hour| {
        let mut g = Game { theme: 0, h12: false, ..Game::default() };
        for _ in 0..500 {
            g.keys.set(0);
            g.step(&at(hour, 0, 0));
        }
        g.draw(&at(hour, 0, 0));
        g.frame.pixels.clone()
    };
    for h in [9, 18, 23] {
        assert_eq!(draw(h), draw(h));
    }
}

#[test]
fn night_dims_the_tank() {
    let water = |hour| {
        let mut g = Game::default();
        g.draw(&at(hour, 0, 0));
        g.frame.palette[WATER[1] as usize]
    };
    let sum = |c: [u8; 3]| c.iter().map(|&v| v as u32).sum::<u32>();
    assert!(sum(water(23)) * 3 < sum(water(9)));
    assert_ne!(water(18), water(9));
}
