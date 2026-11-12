//! 曲子，都是自己写的。旋律在 Square1，和声（反拍）在 Square2，低音在 Triangle，鼓点在 Noise。
//! 一小节 8 格（屋顶那首 12 格，三拍子的感觉）。时间不多了，一格少一帧，听起来快一点。

use crate::art::Theme;
use vibeplay::full::{Channel, Song, Track};

fn tr(ch: Channel, vol: u32, s: &str) -> Track {
    Track::parse(ch, vol, s).expect("曲子写错了")
}

fn song(fps: u32, looped: bool, lead: &str, harm: &str, bass: &str, drum: &str) -> Song {
    let mut t = vec![tr(Channel::Square1, 38, lead), tr(Channel::Triangle, 60, bass)];
    if !harm.is_empty() {
        t.push(tr(Channel::Square2, 18, harm));
    }
    if !drum.is_empty() {
        t.push(tr(Channel::Noise, 14, drum));
    }
    Song::new(fps, looped, t)
}

/// 这一关的曲子。
pub fn level(theme: Theme, hurry: bool) -> Song {
    let f = |n: u32| if hurry { n - 2 } else { n };
    match theme {
        // 早班的街道：大调，蹦蹦跳跳。
        Theme::Street => song(
            f(8),
            true,
            "E5 r G5 E5 C5 r D5 E5 F5 r A5 F5 D5 r E5 F5 G5 r C6 G5 E5 r F5 G5 A5 G5 F5 E5 D5:2 r:2 \
             E5 r G5 E5 C5 r D5 E5 F5 r A5 C6 B5 A5 G5 F5 E5 G5 C6:2 B5 G5 A5 B5 C6:4 r:4",
            "r E4 r G4 r E4 r G4 r F4 r A4 r F4 r A4 r E4 r G4 r E4 r G4 r D4 r G4 r B3 r D4 \
             r E4 r G4 r E4 r G4 r F4 r A4 r D4 r B3 r E4 r G4 r D4 r F4 r E4 r G4 E4:4",
            "C3 r G3 r C3 r G3 r F2 r C3 r F2 r C3 r C3 r G3 r E3 r G3 r G2 r D3 r G2 r B2 r \
             C3 r G3 r C3 r G3 r F2 r C3 r D3 r G2 r C3 r E3 r G2 r G2 B2 C3 r G2 r C3:4",
            "C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C9 C9 C8 r \
             C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C9 r C8 r C8 C8 C8 r",
        ),
        // 下水道：小调，一步一步摸着走。
        Theme::Sewer => song(
            f(9),
            true,
            "A4 r C5 r E5 r D5 C5 B4 r D5 r G4:2 r:2 A4 r C5 r E5 r F5 E5 D5 C5 B4 G#4 A4:2 r:2",
            "",
            "A2 r A2 r A2 r E2 r G2 r G2 r G2 r D2 r F2 r F2 r F2 r C3 r E2 r E2 r E2:2 r:2",
            "C8 r r r C9 r r r C8 r r r C9 r r r C8 r r r C9 r r r C8 r C8 r C9 r r r",
        ),
        // 傍晚的屋顶：三拍子，晃晃悠悠。
        Theme::Roof => song(
            f(7),
            true,
            "D5:3 F#5 A5:2 D6:3 C#6 B5 A5 G5:3 E5 G5:2 B5:3 A5 G5 F#5 F#5:3 A5 D6:2 F#5:3 E5 F#5 G5 A5:6 E5:3 C#5:3 \
             D5:3 F#5 A5:2 D6:3 E6 D6 C#6 B5:3 G5 B5:2 D6:3 C#6 B5 A5 G5:3 F#5 E5:2 A5:3 G5 F#5 E5 D5:9 r:3",
            "r:3 F#4:3 r:3 F#4:3 r:3 G4:3 r:3 G4:3 r:3 F#4:3 r:3 F#4:3 r:3 E4:3 r:3 E4:3 \
             r:3 F#4:3 r:3 F#4:3 r:3 G4:3 r:3 G4:3 r:3 E4:3 r:3 C#4:3 r:3 F#4:3 D4:6",
            "D3:3 A3:3 F#3:3 A3:3 G2:3 D3:3 B2:3 D3:3 D3:3 A3:3 F#3:3 A3:3 A2:3 E3:3 C#3:3 E3:3 \
             D3:3 A3:3 F#3:3 A3:3 G2:3 D3:3 B2:3 D3:3 A2:3 E3:3 A2:3 C#3:3 D3:6 D2:6",
            "C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r C8 C9 r r \
             C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 r r C9 r r C8 C8 C8 C8 r r",
        ),
        // 夜里的港口、屋顶：小调，有点紧张。
        Theme::Harbor | Theme::NightRoof => song(
            f(8),
            true,
            "E5 r G5 r B5 A5 G5 r F#5 r A5 r D5:2 r:2 E5 r G5 r B5 C6 B5 A5 G5 F#5 E5 D#5 E5:2 r:2 \
             C5 r E5 r G5 F#5 E5 r D5 r F#5 r A5 G5 F#5 r B4 r D#5 r F#5 A5 G5 F#5 E5:4 r:4",
            "r B3 r E4 r B3 r E4 r A3 r D4 r A3 r D4 r B3 r E4 r B3 r E4 r B3 r D#4 r B3 r D#4 \
             r C4 r E4 r C4 r E4 r D4 r F#4 r D4 r F#4 r B3 r D#4 r B3 r D#4 r B3 r E4 B3:4",
            "E2 r B2 r E3 r B2 r D2 r A2 r D3 r A2 r E2 r B2 r E3 r B2 r B1 r F#2 r B2 r F#2 r \
             C2 r G2 r C3 r G2 r D2 r A2 r D3 r A2 r B1 r F#2 r B2 r D#3 r E2 r B2 r E2:4",
            "C9 r C8 C9 r C9 C8 r C9 r C8 C9 r C9 C8 r C9 r C8 C9 r C9 C8 r C9 r C8 C9 C8 C8 C8 r \
             C9 r C8 C9 r C9 C8 r C9 r C8 C9 r C9 C8 r C9 r C8 C9 r C9 C8 r C9 r C8 C9 C8 C8 C8 r",
        ),
        // 钟楼：低音一直走，像齿轮在转。
        Theme::Tower => song(
            f(8),
            true,
            "C5 r C5 Eb5 r C5 G5 r F5 r Eb5 r D5 r Bb4 r C5 r C5 Eb5 r G5 Ab5 r G5 F5 Eb5 D5 C5:2 r:2",
            "",
            "C2 C3 C2 C3 C2 C3 C2 C3 Bb1 Bb2 Bb1 Bb2 Ab1 Ab2 G1 G2 C2 C3 C2 C3 Ab1 Ab2 Ab1 Ab2 G1 G2 G1 G2 C2:2 r:2",
            "C8 r C8 r C9 r r r C8 r C8 r C9 r r r C8 r C8 r C9 r r r C8 r C8 r C9 C9 C9 r",
        ),
    }
}

/// 和邮件大盗打的时候。
pub fn boss(hurry: bool) -> Song {
    song(
        if hurry { 4 } else { 5 },
        true,
        "C5 D5 Eb5 C5 G5 r G5 r Ab5 G5 F5 Eb5 D5 r B4 r C5 D5 Eb5 C5 G5 r C6 r B5 Ab5 G5 F5 Eb5 D5 C5 r",
        "",
        "C2 r C2 C3 C2 r C2 C3 Ab1 r Ab1 Ab2 G1 r G1 G2 C2 r C2 C3 C2 r C2 C3 Ab1 r Ab1 Ab2 G1 G2 C2 r",
        "C8 C9 C9 C8 C8 C9 C9 C9 C8 C9 C9 C8 C8 C9 C8 C8 C8 C9 C9 C8 C8 C9 C9 C9 C8 C9 C9 C8 C8 C8 C8 r",
    )
}

/// 拿了金邮票：快，一路往上爬。
pub fn star() -> Song {
    song(
        5,
        true,
        "C5 E5 G5 C6 D5 F5 A5 D6 E5 G5 B5 E6 F5 A5 C6 F6",
        "",
        "C3 r C3 r D3 r D3 r E3 r E3 r F3 r F3 r",
        "C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C8 C8",
    )
}

/// 倒下：一路往下滑。
pub fn death() -> Song {
    song(7, false, "r:4 E5 D#5 D5 C#5 C5:2 r G4 C4:6", "", "r:4 C3:4 G2:4 C2:6", "")
}

/// 进了邮局的门。
pub fn clear() -> Song {
    song(6, false, "C5 E5 G5 C6:2 G5 C6:2 E6:6 r:4", "", "C3:2 G3:2 C4:2 E3:2 G3:2 C3:6 r:4", "")
}

pub fn game_over() -> Song {
    song(9, false, "A4:2 G4:2 F4:2 E4:4 D4:2 E4:2 C4:8", "", "F2:4 C3:4 G2:4 C2:8", "")
}

/// 六关都送到了。
pub fn ending() -> Song {
    song(
        7,
        false,
        "G5 E5 C5 E5 G5:2 C6:2 B5 A5 G5 F5 E5:4 F5 G5 A5 F5 E5 D5 C5 D5 E5:2 G5:2 C6:8",
        "",
        "C3:4 E3:4 F3:4 C3:4 F2:4 G2:4 C3:4 G2:4 C3:8",
        "",
    )
}
