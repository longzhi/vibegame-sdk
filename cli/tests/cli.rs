//! 命令行本身：拿模板游戏（templates/blank）跑一遍 shot、run --headless、pack、web、new。

use std::path::{Path, PathBuf};
use std::process::Command;

fn blank() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../templates/blank")
}

fn tmp(name: &str) -> PathBuf {
    let p = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("vibegame-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// 一次只跑一个：它们都调 cargo 编 wasm。
static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn vibegame(args: &[&str], cwd: &Path) -> Result<String, String> {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let out = Command::new(env!("CARGO_BIN_EXE_vibegame"))
        .args(args)
        .current_dir(cwd)
        .env("CARGO_TARGET_DIR", Path::new(env!("CARGO_TARGET_TMPDIR")).join("vibegame-wasm"))
        .output()
        .unwrap();
    if out.status.success() {
        Ok(String::from_utf8(out.stdout).unwrap())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

fn ok(args: &[&str], cwd: &Path) -> String {
    vibegame(args, cwd).unwrap_or_else(|e| panic!("vibegame {args:?} 失败：{e}"))
}

#[test]
fn shot_saves_each_frame_and_crops() {
    let d = tmp("shot");
    let script = d.join("s.txt");
    std::fs::write(&script, "0 start\n").unwrap();
    let out = ok(&["shot", blank().to_str().unwrap(), "--frames", "60,10", "--input", script.to_str().unwrap(), "--out", "a.png", "--scale", "2"], &d);
    assert_eq!(out.lines().count(), 2, "{out}");
    for (n, (w, h)) in [(10, (480, 320)), (60, (480, 320))] {
        let r = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(d.join(format!("a-{n}.png"))).unwrap())).read_info().unwrap();
        assert_eq!((r.info().width, r.info().height), (w, h));
    }
    ok(&["shot", blank().to_str().unwrap(), "--frames", "5", "--crop", "72,30,96,80", "--scale", "1", "--out", "c.png"], &d);
    let r = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(d.join("c.png")).unwrap())).read_info().unwrap();
    assert_eq!((r.info().width, r.info().height), (96, 80));
    assert!(vibegame(&["shot", blank().to_str().unwrap(), "--frames", "5", "--crop", "200,0,96,80"], &d).unwrap_err().contains("出了画面"));
}

#[test]
fn headless_prints_the_tones_of_a_star() {
    let d = tmp("headless");
    let script = d.join("s.txt");
    // 开局主角在 (112, 72)，星星在 (60, 40)：往左上走，第 18 帧左右碰上。
    std::fs::write(&script, "0 start\n1-40 left up\n").unwrap();
    let out = ok(&["run", blank().to_str().unwrap(), "--headless", "--frames", "60", "--input", script.to_str().unwrap()], &d);
    assert!(out.contains("声道=1 频率=1320"), "捡到星星响一声：{out}");
    assert!(out.contains("声道=2"), "低音在响：{out}");
    assert!(out.ends_with("跑完 60 帧\n"));
}

#[test]
fn pack_holds_the_wasm_the_native_code_and_the_cover() {
    let d = tmp("pack");
    let out = ok(&["pack", blank().to_str().unwrap(), "--out", "o"], &d);
    assert!(out.contains("blank.vg"), "{out}");
    let f = std::fs::File::open(d.join("o/blank.vg")).unwrap();
    let mut a = tar::Archive::new(flate2::read::GzDecoder::new(f));
    let names: Vec<String> = a.entries().unwrap().map(|e| e.unwrap().path().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(names, ["app.toml", "bin/blank.wasm", "bin/blank.cwasm", "assets/cover.png"]);
    // 包每次打出来一样。
    ok(&["pack", blank().to_str().unwrap(), "--out", "p"], &d);
    assert_eq!(std::fs::read(d.join("o/blank.vg")).unwrap(), std::fs::read(d.join("p/blank.vg")).unwrap());
}

#[test]
fn web_is_one_file_with_the_game_inside() {
    let d = tmp("web");
    ok(&["web", blank().to_str().unwrap()], &d);
    let html = std::fs::read_to_string(d.join("dist/blank.html")).unwrap();
    assert!(html.contains("<title>Star Catch</title>"), "默认英文的名字");
    assert!(html.contains("\"zh\":\"捡星星\""), "中文的名字也带着");
    assert!(html.contains("vgLang") && html.contains("lang: () =>"), "能换语言，游戏问得到");
    assert!(html.contains("\"wasm\":\"AGFzbQ"), "wasm 用 base64 塞在里面");
    assert!(html.contains("cover.png"));
    assert!(!html.contains("{{"), "模板的空都填上了");
    assert!(html.contains("classList.add(\"embed\")") && html.contains("vgKey"), "能嵌进别的网页，外面能按键");
}

#[test]
fn new_writes_a_game_that_names_itself_after_the_folder() {
    let d = tmp("new");
    let sdk = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    ok(&["new", "star-catch", "--title", "抓星星", "--sdk", sdk.to_str().unwrap()], &d);
    let g = d.join("star-catch");
    let m = std::fs::read_to_string(g.join("app.toml")).unwrap();
    assert!(m.contains("name      = \"star-catch\"") && m.contains("\"抓星星\"") && m.contains("bin/star-catch.cwasm"), "{m}");
    let c = std::fs::read_to_string(g.join("Cargo.toml")).unwrap();
    assert!(c.contains("name = \"star-catch\"") && c.contains("vibegame = { path = ") && c.contains("[workspace]"), "{c}");
    assert!(g.join("src/lib.rs").is_file() && g.join("assets/cover.png").is_file());
    assert!(vibegame(&["new", "star-catch"], &d).unwrap_err().contains("已经有东西"));
    assert!(vibegame(&["new", "Star"], &d).unwrap_err().contains("小写"));
}

#[test]
fn typos_are_errors_not_ignored() {
    let d = tmp("typo");
    assert!(vibegame(&["shot", ".", "--frame", "5"], &d).unwrap_err().contains("--frame"));
    assert!(vibegame(&["fly"], &d).unwrap_err().contains("不认识的命令"));
}
