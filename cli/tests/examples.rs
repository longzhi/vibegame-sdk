//! 不开窗口跑 examples/ 下的四款游戏：按脚本按键，看发了哪些音调、截图里的颜色，推箱子还看过关以后存档在不在。
//! vibegame 自己把游戏编成 wasm，编到这次测试的临时目录里。

use std::path::{Path, PathBuf};
use std::process::Command;

fn app_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples").join(name)
}

fn tmp(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("vibegame-examples-{}-{name}", std::process::id()))
}

/// 一次只跑一个 vibegame：它们都调 cargo 编 wasm。
static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 跑一次 vibegame，回 stdout。不成就失败，带上 stderr。
fn vibegame(app: &str, args: &[&str], script: &str) -> String {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let input = tmp(&format!("{app}-input.txt"));
    std::fs::write(&input, script).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_vibegame"))
        .args(args)
        .arg(app_dir(app))
        .arg("--input")
        .arg(&input)
        .env("CARGO_TARGET_DIR", Path::new(env!("CARGO_TARGET_TMPDIR")).join("vibegame-wasm"))
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&input);
    assert!(out.status.success(), "vibegame {app} {args:?} 失败：{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

/// 截第 `frames` 帧（不放大），回 RGB，240×160。
fn shot(app: &str, frames: u32, script: &str, extra: &[&str]) -> Vec<u8> {
    let png = tmp(&format!("{app}-{frames}.png"));
    let n = frames.to_string();
    let mut args = vec!["shot", "--frames", &n, "--scale", "1", "--out", png.to_str().unwrap()];
    args.extend_from_slice(extra);
    vibegame(app, &args, script);
    let mut r = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(&png).unwrap())).read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    let _ = std::fs::remove_file(&png);
    assert_eq!((info.width, info.height), (240, 160));
    buf[..info.buffer_size()].to_vec()
}

fn px(rgb: &[u8], x: usize, y: usize) -> [u8; 3] {
    let i = (y * 240 + x) * 3;
    [rgb[i], rgb[i + 1], rgb[i + 2]]
}

fn count(rgb: &[u8], c: [u8; 3]) -> usize {
    rgb.as_chunks::<3>().0.iter().filter(|p| **p == c).count()
}

/// 「帧 N 音调 声道=… 频率=…」里，满足条件的那些行的帧号。
fn tone_frames(out: &str, has: &str) -> Vec<u64> {
    out.lines().filter(|l| l.contains("音调") && l.contains(has)).filter_map(|l| l.split_whitespace().nth(1)?.parse().ok()).collect()
}

/// 《疾风邮差》小邮差的帽子。
const CAP: [u8; 3] = [0x24, 0x34, 0x7c];

#[test]
fn courier_runs_jumps_and_scrolls() {
    // 第 5 帧开始，「1-1 早班」的字幕放两秒，往右跑，第 200 帧按 A 跳。
    let script = "5 start\n140-399 right\n200-215 a\n";
    let out = vibegame("courier", &["run", "--headless", "--frames", "400"], script);
    let jump: Vec<u64> = [392, 523, 784].iter().flat_map(|hz| tone_frames(&out, &format!("声道=1 频率={hz} 毫秒=53"))).collect();
    assert_eq!(jump, [201, 202, 204], "跳的音效是三个往上走的音：{out}");
    assert!(tone_frames(&out, "声道=2").len() > 10, "低音一直在响：{out}");
    // 画面跟着走：远处的房子挪了；小邮差还在画面里。
    let a = shot("courier", 140, script, &[]);
    let b = shot("courier", 400, script, &[]);
    assert!(count(&b, CAP) > 10, "看得见小邮差");
    let row = |rgb: &[u8]| (0..240).map(|x| px(rgb, x, 90)).collect::<Vec<_>>();
    assert_ne!(row(&a), row(&b), "房子跟着画面挪了");
}

/// 《雷霆突击》雷的红头巾。
const BAND: [u8; 3] = [0xe0, 0x28, 0x28];

#[test]
fn thunder_shoots_while_the_music_plays() {
    // 字幕放完以后往右走，按住 B 开枪。
    let script = "5 start\n170-260 right\n170-260 b\n";
    let out = vibegame("thunder", &["run", "--headless", "--frames", "260"], script);
    let shots = tone_frames(&out, "声道=3 频率=2000");
    assert!(shots.len() >= 5, "按住 B 连着打：{out}");
    assert_eq!(shots[0], 170, "按下就打");
    assert_eq!(shots[1] - shots[0], 14, "普通枪按住 14 帧一发");
    // 枪声和音乐一起响：开枪的这段时间里，旋律和低音也在发。
    let (from, to) = (shots[0], shots[shots.len() - 1]);
    let during = |has: &str| tone_frames(&out, has).into_iter().filter(|n| (from..=to).contains(n)).count();
    assert!(during("声道=0") >= 3 && during("声道=2") >= 3, "{out}");
    // 开局一秒里主角一闪一闪，等它不闪了再截。
    let rgb = shot("thunder", 240, script, &[]);
    assert!(count(&rgb, BAND) > 4, "看得见雷的头巾");
}

const SHOT: [u8; 3] = [0xff, 0xd2, 0x3f];

#[test]
fn skyfire_fires_volleys_after_the_banner() {
    // 标题按 START，关头的字放两秒，再按住 A 连发。
    let script = "5 start\n130-250 a\n";
    let out = vibegame("skyfire", &["run", "--headless", "--frames", "250"], script);
    let shots = tone_frames(&out, "声道=1 频率=1760");
    assert!(shots.len() >= 8, "按住 A 一直打：{out}");
    assert!(shots[0] >= 130, "关头的字放完才能打");
    assert!(!tone_frames(&out, "声道=0").is_empty(), "关里的曲子在放");
    // 截一张：子弹（黄的）在往上飞。
    let rgb = shot("skyfire", 200, script, &[]);
    assert!(count(&rgb, SHOT) > 20, "看得见子弹");
}

#[test]
fn duckpush_clears_level_one_and_keeps_the_save() {
    let data = tmp("pushdata");
    let _ = std::fs::remove_dir_all(&data);
    let d = data.to_str().unwrap();
    // 选关里按 A 进第一关，往右三步：面包推上盘子。
    let script = "5 a\n10 right\n14 right\n18 right\n";
    let out = vibegame("duckpush", &["run", "--headless", "--frames", "40", "--data", d], script);
    assert_eq!(tone_frames(&out, "声道=1 频率=196"), [14], "推了一下：{out}");
    assert_eq!(tone_frames(&out, "声道=1 频率=1320"), [18], "推上盘子：{out}");
    assert_eq!(std::fs::read_to_string(data.join("progress")).unwrap(), "10000000000000000000");
    // 再开一次：第一关的格是绿的（过了），光标（金框）在第二关。
    let rgb = shot("duckpush", 2, "", &["--data", d]);
    assert_eq!(px(&rgb, 10, 40), [0x50, 0xb8, 0x60]);
    assert_eq!(px(&rgb, 52, 36), [0xff, 0xe0, 0x70]);
    assert_ne!(px(&rgb, 6, 36), [0xff, 0xe0, 0x70]);
    std::fs::remove_dir_all(&data).unwrap();
}

#[test]
fn ironcamp_shoots_a_brick_and_draws_the_camp() {
    // 标题按 START，选关按 A。出车的光闪完（第 88 帧）以后打一发：一号车正上面是砖。
    let script = "20 start\n40 a\n100 a\n";
    let out = vibegame("ironcamp", &["run", "--headless", "--frames", "140"], script);
    assert_eq!(tone_frames(&out, "声道=1 频率=523"), [100], "开炮：{out}");
    assert_eq!(tone_frames(&out, "声道=3 频率=1800"), [100], "砖打掉一条：{out}");
    assert!(tone_frames(&out, "声道=0").len() >= 5, "开局的号角：{out}");
    let rgb = shot("ironcamp", 120, script, &[]);
    assert_eq!(px(&rgb, 56, 152), [0xa4, 0xd4, 0x68], "一号车在营地左边");
    assert!(count(&rgb, [0xe8, 0xd4, 0x9c]) > 20, "看得见帐篷");
    // 一号车头上那块砖下面少了一条（4 像素）。
    assert_eq!(px(&rgb, 56, 141), [0x12, 0x12, 0x1a]);
    assert_ne!(px(&rgb, 56, 137), [0x12, 0x12, 0x1a]);
}
