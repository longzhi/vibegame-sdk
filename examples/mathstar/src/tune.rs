//! 曲子和提示音，都是自己写的。旋律在方波 1，低音在三角波，方波 2 给错的提示音。

use vibeplay::full::{Channel, Song, Track};

fn t(ch: Channel, vol: u32, notes: &str) -> Track {
    Track::parse(ch, vol, notes).expect("曲子写错了")
}

/// 标题：蹦蹦跳跳的大调，一直循环。
pub fn title() -> Song {
    Song::new(
        8,
        true,
        vec![
            t(Channel::Square1, 22, "C5 E5 G5 E5 F5 A5 G5:2 E5 G5 C6 G5 A5 F5 D5:2 C5 E5 G5 E5 F5 A5 C6:2 B5 G5 D5 F5 E5:2 C5:2"),
            t(Channel::Triangle, 40, "C3:2 G3:2 F3:2 C3:2 C3:2 E3:2 F3:2 G3:2 C3:2 G3:2 F3:2 A3:2 G3:2 G2:2 C3:4"),
        ],
    )
}

/// 答对了：往上跳的四个音。
pub fn right() -> Song {
    Song::new(
        4,
        false,
        vec![t(Channel::Square1, 30, "C5 E5 G5 C6:4"), t(Channel::Triangle, 40, "C4 r G3 C4:4")],
    )
}

/// 答错了：轻轻的两个低音，不吓人。
pub fn wrong() -> Song {
    Song::new(8, false, vec![t(Channel::Square2, 12, "D3 A2:3")])
}

/// 一局做完。星星多，曲子长一点、高一点。
pub fn done(stars: u8) -> Song {
    let tune = match stars {
        10 => "C5 E5 G5 C6 G5 C6 E6:2 D6 C6 G5 E5 C6:4",
        7..=9 => "C5 E5 G5 C6:2 G5 A5 B5 C6:4",
        _ => "G4 C5 E5 D5 C5 E5 C5:4",
    };
    Song::new(6, false, vec![t(Channel::Square1, 26, tune), t(Channel::Triangle, 40, "C3:2 G3:2 F3:2 G3:2 C3:4")])
}
