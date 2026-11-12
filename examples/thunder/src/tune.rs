//! 曲子，都是自己写的。旋律在 Square1，和声在 Square2，低音在 Triangle，鼓点在 Noise。
//! 一小节 8 格。打仗的曲子多用小调，低音一直在跑。

use crate::art::Theme;
use vibeplay::full::{Channel, Song, Track};

fn tr(ch: Channel, vol: u32, s: &str) -> Track {
    Track::parse(ch, vol, s).expect("曲子写错了")
}

fn song(fps: u32, looped: bool, lead: &str, harm: &str, bass: &str, drum: &str) -> Song {
    let mut t = vec![tr(Channel::Square1, 36, lead), tr(Channel::Triangle, 60, bass)];
    if !harm.is_empty() {
        t.push(tr(Channel::Square2, 16, harm));
    }
    if !drum.is_empty() {
        t.push(tr(Channel::Noise, 14, drum));
    }
    Song::new(fps, looped, t)
}

/// 这一关的曲子。
pub fn level(theme: Theme) -> Song {
    match theme {
        // 丛林：小调，往前冲，低音八分音符一直跑。
        Theme::Jungle => song(
            7,
            true,
            "D5:2 r F5 D5 A5:2 G5 F5 E5:2 r C5 E5 G5 F5 E5 D5:2 r F5 D5 A5:2 C6 A5 Bb5:2 A5 G5 F5:2 E5 C5 \
             D5:2 r F5 D5 A5:2 G5 F5 E5:2 r G5 A5 Bb5 C6 Bb5 A5:2 F5 D5 E5 F5 G5 A5 D5:4 r:4",
            "r A4 r A4 r A4 r A4 r G4 r G4 r G4 r G4 r A4 r A4 r F4 r F4 r G4 r G4 r A4 r C#5 \
             r A4 r A4 r A4 r A4 r G4 r G4 r Bb4 r Bb4 r A4 r A4 r G4 r G4 r A4 r A4 A4:4",
            "D3 D3 D4 D3 D3 D3 D4 D3 C3 C3 C4 C3 C3 C3 C4 C3 Bb2 Bb2 Bb3 Bb2 Bb2 Bb2 Bb3 Bb2 A2 A2 A3 A2 A2 A2 C#3 E3 \
             D3 D3 D4 D3 D3 D3 D4 D3 C3 C3 C4 C3 C3 C3 C4 C3 Bb2 Bb2 Bb3 Bb2 A2 A2 A3 A2 D3 D3 A2 A2 D3:4",
            "C9 r C8 C9 C9 r C8 r C9 r C8 C9 C9 r C8 C8 C9 r C8 C9 C9 r C8 r C9 r C8 C9 C8 C8 C8 C8 \
             C9 r C8 C9 C9 r C8 r C9 r C8 C9 C9 r C8 C8 C9 r C8 C9 C9 r C8 r C9 C8 C9 C8 C8 C8 C8 r",
        ),
        // 瀑布：往上爬，旋律一级一级往上走。
        Theme::Falls => song(
            7,
            true,
            "E5 r E5 G5 A5 r B5 r C6:2 B5 A5 G5:2 E5 r F5 r F5 A5 B5 r C6 r D6:2 C6 B5 A5:4 \
             E5 r E5 G5 A5 r B5 r C6:2 D6 E6 D6:2 C6 B5 A5 G5 F5 E5 F5 G5 A5 B5 E6:4 r:4",
            "r C5 r C5 r D5 r D5 r E5 r E5 r B4 r B4 r D5 r D5 r E5 r E5 r F5 r F5 r E5 r E5 \
             r C5 r C5 r D5 r D5 r E5 r E5 r G5 r G5 r D5 r D5 r B4 r B4 r G#4 r G#4 G#4:4",
            "A2 A3 A2 A3 G2 G3 G2 G3 F2 F3 F2 F3 E2 E3 E2 E3 D2 D3 D2 D3 F2 F3 F2 F3 E2 E3 E2 E3 A2 A3 A2 A3 \
             A2 A3 A2 A3 G2 G3 G2 G3 F2 F3 F2 F3 C3 C4 C3 C4 D2 D3 D2 D3 E2 E3 E2 E3 E2 E3 G#2 B2 E2:4",
            "C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C9 C9 C9 \
             C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 C8 C9 C9 C8 C8 C8 r",
        ),
        // 雪原：冷，慢一点，长音多。
        Theme::Snow => song(
            8,
            true,
            "B4:2 E5 F#5 G5:3 F#5 E5:2 D5 E5 F#5:4 B4:2 G5 A5 B5:3 A5 G5:2 F#5 G5 E5:4 \
             C5:2 E5 G5 C6:3 B5 A5:2 G5 A5 B5:4 A5:2 G5 F#5 E5:2 D#5 F#5 E5:4 r:4",
            "r G4 r G4 r G4 r G4 r A4 r A4 r F#4 r F#4 r G4 r G4 r G4 r G4 r A4 r A4 r G4 r G4 \
             r E4 r E4 r G4 r G4 r F#4 r F#4 r D#4 r D#4 r C4 r C4 r B3 r B3 r G4 r G4 G4:4",
            "E2:2 B2:2 E3:2 B2:2 D2:2 A2:2 D3:2 A2:2 C2:2 G2:2 C3:2 G2:2 B1:2 F#2:2 B2:2 F#2:2 \
             A1:2 E2:2 A2:2 E2:2 C2:2 G2:2 C3:2 G2:2 A1:2 E2:2 B1:2 F#2:2 E2:4 B1:4",
            "C8 r r r C9 r r C8 C8 r r r C9 r r r C8 r r r C9 r r C8 C8 r r r C9 r C9 r \
             C8 r r r C9 r r C8 C8 r r r C9 r r r C8 r r r C9 r r C8 C8 r C8 r C9 C9 C9 r",
        ),
        // 要塞：低音像机器一样敲，旋律短促。
        Theme::Fort => song(
            6,
            true,
            "C5 r C5 r Eb5 r C5 r G5 r F5 Eb5 D5 r Bb4 r C5 r C5 r Eb5 r G5 r Ab5 r G5 F5 Eb5 r D5 r \
             C5 r C5 r Eb5 r C5 r G5 r Bb5 r C6 r Bb5 Ab5 G5 r F5 r Eb5 r D5 r B4 r D5 r C5:4",
            "",
            "C2 C2 C3 C2 C2 C2 C3 C2 Ab1 Ab1 Ab2 Ab1 Bb1 Bb1 Bb2 Bb1 C2 C2 C3 C2 C2 C2 C3 C2 Ab1 Ab1 Ab2 Ab1 G1 G1 G2 G1 \
             C2 C2 C3 C2 C2 C2 C3 C2 Eb2 Eb2 Eb3 Eb2 F2 F2 F3 F2 Ab1 Ab1 Ab2 Ab1 G1 G1 G2 G1 C2 C2 G1 G1 C2:4",
            "C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C8 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C8 C8 C8 C8 \
             C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C8 C8 C9 C8 C9 C8 C9 C8 C9 C8 C9 C8 C8 C8 C8 C8 C8 r",
        ),
    }
}

/// 打大怪的时候：快，半音往上挤。
pub fn boss() -> Song {
    song(
        5,
        true,
        "E5 E5 r E5 F5 r E5 r G5 G5 r G5 Ab5 r G5 r E5 E5 r E5 F5 r E5 r B5:2 Bb5:2 A5:2 Ab5:2",
        "B4 B4 r B4 C5 r B4 r D5 D5 r D5 Eb5 r D5 r B4 B4 r B4 C5 r B4 r F5:2 E5:2 Eb5:2 D5:2",
        "E2 E2 E3 E2 F2 F2 F3 F2 E2 E2 E3 E2 F2 F2 F3 F2 E2 E2 E3 E2 F2 F2 F3 F2 E2 E3 E2 E3 E2 E3 E2 E3",
        "C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C9 C8 C9 C9 C8 C9 C9 C8 C8 C8 C8 C8 C8 C8 C8 C8",
    )
}

/// 一关开头：短的号声。
pub fn intro() -> Song {
    song(6, false, "G4 C5 E5 G5:3 E5 G5:6 r:2", "", "C3:3 G2:3 C3:6 r:2", "C9 C9 C9 C8:3 C9 C8:6")
}

/// 打完大怪。
pub fn clear() -> Song {
    song(6, false, "C5 E5 G5 C6:2 B5 C6:2 D6 E6:6 r:4", "G4 C5 E5 G5:2 G5 G5:2 B5 C6:6 r:4", "C3:2 G3:2 C3:2 G2:2 C3:6 r:4", "")
}

pub fn game_over() -> Song {
    song(9, false, "E5:2 D5:2 C5:2 B4:4 A4:2 G#4:2 A4:8", "", "A2:4 E2:4 F2:4 E2:2 A1:6", "")
}

/// 四关都打下来了。
pub fn ending() -> Song {
    song(
        7,
        false,
        "C5:2 E5 G5 C6:4 B5 A5 G5 E5 F5:4 D5 F5 A5 C6 B5:2 G5:2 C6:8 r:4",
        "E4:2 G4 C5 E5:4 D5 C5 B4 C5 A4:4 B4 D5 F5 A5 G5:2 F5:2 E5:8 r:4",
        "C3:4 C3:4 F2:4 F2:4 G2:4 G2:4 C3:8 r:4",
        "",
    )
}

/// 标题。
pub fn title() -> Song {
    song(
        7,
        true,
        "A4:2 r A4 C5:2 E5:2 D5:2 C5 B4 C5:4 A4:2 r A4 C5:2 E5:2 G5:2 F5 E5 E5:4 \
         F5:2 r F5 E5:2 D5:2 C5:2 B4 C5 D5:4 E5:2 r D5 C5 B4 G#4 B4 A4:4 r:4",
        "",
        "A2 A3 A2 A3 A2 A3 A2 A3 F2 F3 F2 F3 F2 F3 F2 F3 A2 A3 A2 A3 C3 C4 C3 C4 E2 E3 E2 E3 E2 E3 E2 E3 \
         D2 D3 D2 D3 D2 D3 D2 D3 G2 G3 G2 G3 G2 G3 G2 G3 E2 E3 E2 E3 E2 E3 G#2 B2 A2:4 r:4",
        "C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C9 C9 C9 \
         C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 r C9 r C8 C8 C9 r C8 C9 C8 C9 C8 C8 C8 r",
    )
}
