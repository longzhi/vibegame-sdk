//! 曲子，都是自己写的。旋律在方波 1，低音在三角波，鼓在噪声，方波 2 留给音效。

use vibegame::full::{Channel, Song, Track};

fn t(ch: Channel, vol: u32, notes: &str) -> Track {
    Track::parse(ch, vol, notes).expect("曲子写错了")
}

/// 标题：慢一点，往上走的号角。
pub fn title() -> Song {
    Song::new(
        9,
        true,
        vec![
            t(Channel::Square1, 28, "G4:2 C5:2 D5 E5:3 D5:2 C5:2 G4:4 A4:2 C5:2 E5 G5:3 F5:2 E5:2 D5:4 r:2"),
            t(Channel::Triangle, 45, "C3:4 G2:4 A2:4 E2:4 F2:4 C3:4 G2:4 G2:2 r:2"),
        ],
    )
}

/// 第一关：海岸。明快，大调。
pub fn coast() -> Song {
    Song::new(
        7,
        true,
        vec![
            t(Channel::Square1, 26, "E5 G5 A5:2 G5 E5 D5:2 E5 G5 C6:2 B5 A5 G5:2 F5 A5 C6:2 A5 F5 E5:2 D5 E5 F5 G5 A5:2 G5:2 E5 G5 A5:2 G5 E5 D5:2 C5 D5 E5:2 G5 F5 E5 D5 C5:4 r:4"),
            t(Channel::Triangle, 44, "C3:2 C4:2 C3:2 C4:2 A2:2 A3:2 A2:2 A3:2 F2:2 F3:2 F2:2 F3:2 G2:2 G3:2 G2:2 G3:2 C3:2 C4:2 C3:2 C4:2 A2:2 A3:2 A2:2 A3:2 F2:2 G2:2 G3:2 G2:2 C3:4 r:4"),
            t(Channel::Noise, 12, "C8 r C9 r C8 r C9 C9 C8 r C9 r C8 r C9 C9 C8 r C9 r C8 r C9 C9 C8 r C9 r C8 r C9 C9 C8 r C9 r C8 r C9 C9 C8 r C9 r C8 r C9 C9 C8 r C9 r C8 r C9 C9 C8 r C9 r C8 r C9 C9"),
        ],
    )
}

/// 第二关：沙漠。小调，有点急。
pub fn desert() -> Song {
    Song::new(
        7,
        true,
        vec![
            t(Channel::Square1, 26, "A4 C5 E5 A5:2 G5 E5 F5:2 E5 D5 E5:4 A4 C5 E5 A5:2 B5 C6 B5:2 A5 G#5 A5:4 D5 F5 A5 D6:2 C6 A5 B5:2 A5 G5 E5:4 C5 D5 E5 F5 E5 D5 C5 B4 A4:4 r:4"),
            t(Channel::Triangle, 44, "A2 r A2 A3 A2 r A2 A3 F2 r F2 F3 E2 r E2 E3 A2 r A2 A3 A2 r A2 A3 E2 r E2 E3 E2 r E2 E3 D2 r D2 D3 D2 r D2 D3 E2 r E2 E3 E2 r E2 E3 F2 r F2 F3 E2 r E2 E3 A2:4 r:4"),
            t(Channel::Noise, 12, "C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9"),
        ],
    )
}

/// 第三关：夜里的云。安静一点，长音多。
pub fn clouds() -> Song {
    Song::new(
        8,
        true,
        vec![
            t(Channel::Square1, 22, "D5:3 F5 A5:4 G5:2 F5:2 E5:4 C5:3 E5 G5:4 F5:2 E5:2 D5:4 D5:3 F5 A5:4 C6:2 B5:2 A5:4 G5:2 F5:2 E5:2 C5:2 D5:6 r:2"),
            t(Channel::Triangle, 42, "D3:4 D3:4 A2:4 A2:4 C3:4 C3:4 G2:4 A2:4 D3:4 D3:4 F2:4 F2:4 G2:4 A2:4 D3:6 r:2"),
            t(Channel::Noise, 8, "C9:2 r:2 C8:2 r:2 C9:2 r:2 C8:2 r:2 C9:2 r:2 C8:2 r:2 C9:2 r:2 C8:2 r:2 C9:2 r:2 C8:2 r:2 C9:2 r:2 C8:2 r:2 C9:2 r:2 C8:2 r:2 C9:2 r:2 C8:2 r:2"),
        ],
    )
}

/// 第四关：要塞。硬，一直往前推。
pub fn fortress() -> Song {
    Song::new(
        6,
        true,
        vec![
            t(Channel::Square1, 26, "E5 E5 G5 E5 A5 E5 G5 E5 D5 D5 F5 D5 G5 D5 F5 D5 C5 C5 E5 C5 F5 C5 E5 C5 B4 D5 F5 G#5 B5:4 E5 E5 G5 E5 A5 E5 B5 E5 C6 B5 A5 G5 F5 E5 D5 C5 D5 D5 F5 A5 G5 F5 E5 D5 E5:8"),
            t(Channel::Triangle, 46, "E2 E3 E2 E3 E2 E3 E2 E3 D2 D3 D2 D3 D2 D3 D2 D3 C2 C3 C2 C3 C2 C3 C2 C3 B1 B2 B1 B2 E2:4 E2 E3 E2 E3 E2 E3 E2 E3 A1 A2 A1 A2 A1 A2 A1 A2 D2 D3 D2 D3 B1 B2 B1 B2 E2:8"),
            t(Channel::Noise, 14, "C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C8 C8 C8 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C8 C8 C8 C8 C8 C8 C8"),
        ],
    )
}

/// 大怪：低音一直在下面滚，旋律一句一句压上来。
pub fn boss() -> Song {
    Song::new(
        6,
        true,
        vec![
            t(Channel::Square1, 28, "E5:2 r E5 F5:2 r F5 G5:2 F5 E5 D#5:4 E5:2 r E5 F5:2 r F5 G5:2 A5 B5 C6:4 B5:2 A5:2 G5:2 F5:2 E5:2 D5:2 C5:2 B4:2 C5 D5 D#5 E5:5 r"),
            t(Channel::Triangle, 46, "E2 E2 E3 E2 E2 E3 E2 E3 E2 E2 E3 E2 D#2 D#3 D#2 D#3 E2 E2 E3 E2 E2 E3 E2 E3 F2 F2 F3 F2 F#2 F#3 G2 G#2 A2 A2 A3 A2 G2 G2 G3 G2 F2 F2 F3 F2 B1 B2 B1 B2 E2:8"),
            t(Channel::Noise, 14, "C8 r C9 C8 C8 r C9 r C8 r C9 C8 C8 r C9 C9 C8 r C9 C8 C8 r C9 r C8 r C9 C8 C8 C9 C8 C9 C8 r C9 C8 C8 r C9 r C8 r C9 C8 C8 r C9 C9 C8 r C9 C8 C8 r C9 r C8 C8 C8 C8 C8 C8 C8 C8"),
        ],
    )
}

/// 过关：短短一句。
pub fn clear() -> Song {
    Song::new(
        8,
        false,
        vec![t(Channel::Square1, 30, "C5 E5 G5 C6:2 G5 C6:5"), t(Channel::Triangle, 45, "C3 E3 G3 C4:2 G3 C3:5")],
    )
}

/// 没命了。
pub fn over() -> Song {
    Song::new(10, false, vec![t(Channel::Square1, 26, "E5:2 D#5:2 D5:2 C#5:2 C5:6"), t(Channel::Triangle, 40, "A2:2 G#2:2 G2:2 F#2:2 F2:6")])
}

/// 每关的曲子。
pub fn stage(i: usize) -> Song {
    match i {
        0 => coast(),
        1 => desert(),
        2 => clouds(),
        _ => fortress(),
    }
}
