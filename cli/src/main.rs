//! vibegame：做掌机游戏的命令行。人和 agent 用的是同一套命令，`vibegame help` 看全部。

mod device;
mod game;
mod new;
mod pack;
mod player;
mod script;
mod web;
mod window;

use anyhow::{bail, Context, Result};
use player::Player;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const USAGE: &str = "vibegame：做掌机游戏（240×160、256 色、每秒 60 帧）

  vibegame doctor                       查开发机缺什么（Rust、wasm32 目标、掌机连没连上）
  vibegame new <目录> [--title 名字]     从模板起一个新游戏
  vibegame run [游戏]                    开窗口玩，有声音。存档放在 <游戏>/save/
  vibegame run [游戏] --headless --frames N [--input 脚本] [--data 目录]
                                        不开窗口，按脚本按键，印出每一个音调
  vibegame shot [游戏] --frames N[,N…] [--input 脚本] [--out a.png] [--scale K] [--crop x,y,w,h]
                                        跑到第 N 帧，存成 PNG（默认放大 3 倍）。几个帧号就存几张
  vibegame web [游戏] [--out a.html]     出网页版：一个 HTML 文件，浏览器里就能玩，能发给别人
  vibegame pack [游戏] [--out 目录]      打包成 <名字>.vg：拷进 TF 卡的「游戏」目录，或者 push
  vibegame pair <掌机的地址>             和掌机配对（掌机要开开发者模式，屏幕上有口令）
  vibegame push [游戏] [--watch]         打包、推到配好的掌机上、打开。--watch：一改就再推
  vibegame skill [install [--to 目录]]   印出给 agent 的说明；install 装进 Claude Code 和 Codex

[游戏] 是游戏的目录（有 app.toml），不写是当前目录。run、shot 也能直接给一个 .wasm 文件。
按键脚本一行一条：「帧 键…」或「起-止 键…」，帧从 0 数，# 后面是注释。
键：up down left right a b x y l1 r1 start select。
窗口的键：方向键，X=A，Z=B，S=X，A=Y，Q=L1，W=R1，回车=Start，右 Shift=Select，Esc 退出。";

/// 命令行的参数：位置参数和 `--名字 值`。
struct Args {
    pos: Vec<String>,
    opts: HashMap<String, String>,
}

/// 不带值的开关。
const SWITCHES: [&str; 2] = ["headless", "watch"];

impl Args {
    fn parse(it: impl Iterator<Item = String>) -> Result<Args> {
        let mut a = Args { pos: Vec::new(), opts: HashMap::new() };
        let mut it = it.peekable();
        while let Some(x) = it.next() {
            if let Some(name) = x.strip_prefix("--") {
                let v = if SWITCHES.contains(&name) { String::new() } else { it.next().with_context(|| format!("--{name} 后面要一个值"))? };
                a.opts.insert(name.to_string(), v);
            } else {
                a.pos.push(x);
            }
        }
        Ok(a)
    }
    fn has(&self, k: &str) -> bool {
        self.opts.contains_key(k)
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.opts.get(k).map(String::as_str)
    }
    fn path(&self, k: &str) -> Option<PathBuf> {
        self.get(k).map(PathBuf::from)
    }
    /// 第 `i` 个位置参数当游戏的目录，没有就是当前目录。
    fn game(&self, i: usize) -> PathBuf {
        PathBuf::from(self.pos.get(i).map(String::as_str).unwrap_or("."))
    }
    /// 只认这几个 `--名字`，别的报错：打错了不会悄悄被忽略。
    fn only(&self, known: &[&str]) -> Result<()> {
        for k in self.opts.keys() {
            if !known.contains(&k.as_str()) {
                bail!("这个命令没有 --{k}");
            }
        }
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || matches!(args[0].as_str(), "-h" | "--help" | "help") {
        println!("{USAGE}");
        return;
    }
    if matches!(args[0].as_str(), "-V" | "--version" | "version") {
        println!("vibegame {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if let Err(e) = run(args) {
        eprintln!("vibegame：{e:#}");
        std::process::exit(1);
    }
}

fn run(mut args: Vec<String>) -> Result<()> {
    let cmd = args.remove(0);
    let a = Args::parse(args.into_iter())?;
    match cmd.as_str() {
        "doctor" => doctor(),
        "new" => {
            a.only(&["title", "sdk"])?;
            let dir = a.pos.first().context("要一个目录：vibegame new mygame")?;
            new::create(Path::new(dir), a.get("title"), a.path("sdk").as_deref())
        }
        "run" if a.has("headless") => {
            a.only(&["headless", "frames", "input", "data"])?;
            let frames: u64 = a.get("frames").context("--headless 要 --frames N")?.parse().context("--frames 要整数")?;
            let mut p = load(&a, a.path("data"))?;
            p.run(frames, &script(&a)?, |n, t| println!("帧 {n} 音调 声道={} 频率={} 毫秒={} 音量={}", t.channel, t.hz, t.ms, t.volume))?;
            println!("跑完 {frames} 帧");
            Ok(())
        }
        "run" => {
            a.only(&["data"])?;
            let g = a.game(0);
            let data = a.path("data").or_else(|| g.is_dir().then(|| g.join("save")));
            let name = game::module_of(&g)?.1;
            let mut p = load(&a, data)?;
            window::play(&name, &mut p)
        }
        "shot" => {
            a.only(&["frames", "input", "data", "out", "scale", "crop"])?;
            shot(&a)
        }
        "web" => {
            a.only(&["out"])?;
            let g = game::Game::open(&a.game(0))?;
            let html = web::page(&g.dir)?;
            let out = a.path("out").unwrap_or_else(|| PathBuf::from("dist").join(format!("{}.html", g.m.name)));
            write(&out, html.as_bytes())?;
            println!("{}（{} KB，双击就能在浏览器里玩）", out.display(), html.len() / 1024);
            Ok(())
        }
        "pack" => {
            a.only(&["out", "target"])?;
            let (bytes, g) = pack::pack(&a.game(0), a.get("target").unwrap_or(pack::DEVICE_TARGET))?;
            let out = a.path("out").unwrap_or_else(|| PathBuf::from("dist")).join(format!("{}.vg", g.m.name));
            write(&out, &bytes)?;
            println!("{}（{} KB）", out.display(), bytes.len() / 1024);
            Ok(())
        }
        "pair" => {
            a.only(&["code"])?;
            device::pair(a.pos.first().context("要掌机的地址：vibegame pair 192.168.1.23")?, a.get("code"))
        }
        "push" => {
            a.only(&["watch", "device"])?;
            device::push(&a.game(0), a.get("device"), a.has("watch"))
        }
        "skill" => {
            a.only(&["to"])?;
            match a.pos.first().map(String::as_str) {
                None => {
                    print!("{}", new::SKILL);
                    Ok(())
                }
                Some("install") => new::install_skill(a.path("to").as_deref()),
                Some(x) => bail!("skill 后面只认 install，不认 {x}"),
            }
        }
        c => bail!("不认识的命令 {c}。vibegame help 看全部命令"),
    }
}

fn script(a: &Args) -> Result<script::Script> {
    match a.path("input") {
        Some(p) => script::Script::parse(&std::fs::read_to_string(&p).with_context(|| format!("读不了 {}", p.display()))?),
        None => Ok(script::Script::default()),
    }
}

/// 编好游戏、装进播放器。`data` 是存档的目录，None 时存不了。
fn load(a: &Args, data: Option<PathBuf>) -> Result<Player> {
    let (module, _, memory_mb) = game::module_of(&a.game(0))?;
    if let Some(d) = &data {
        std::fs::create_dir_all(d).with_context(|| format!("建不了 {}", d.display()))?;
    }
    Player::load(&module, data, memory_mb)
}

fn shot(a: &Args) -> Result<()> {
    let mut at: Vec<u64> = Vec::new();
    for s in a.get("frames").context("shot 要 --frames N（几张就写 --frames 60,120）")?.split(',') {
        at.push(s.trim().parse().with_context(|| format!("--frames 里的 {s} 不是整数"))?);
    }
    at.sort();
    let scale: usize = a.get("scale").map(str::parse).transpose().context("--scale 要整数")?.unwrap_or(3);
    let crop = match a.get("crop") {
        Some(c) => {
            let v: Vec<usize> = c.split(',').map(|x| x.trim().parse()).collect::<Result<_, _>>().context("--crop 要 x,y,w,h 四个整数")?;
            let [x, y, w, h] = v[..] else { bail!("--crop 要 x,y,w,h 四个整数") };
            Some((x, y, w, h))
        }
        None => None,
    };
    let name = game::module_of(&a.game(0))?.1;
    let mut p = load(a, a.path("data"))?;
    let s = script(a)?;
    for n in &at {
        p.run(n - p.frame_no, &s, |_, _| {})?;
        let out = match (a.path("out"), at.len()) {
            (Some(o), 1) => o,
            (Some(o), _) => o.with_file_name(format!("{}-{n}.png", o.file_stem().unwrap_or_default().to_string_lossy())),
            (None, _) => PathBuf::from(format!("{name}-{n}.png")),
        };
        if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        p.shot(&out, scale, crop)?;
        println!("{}", out.display());
    }
    Ok(())
}

fn write(out: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).with_context(|| format!("建不了 {}", parent.display()))?;
    }
    std::fs::write(out, bytes).with_context(|| format!("写不了 {}", out.display()))
}

/// 查开发机：一样一样说好没好，没好的给出修的命令。
fn doctor() -> Result<()> {
    let mut bad = 0;
    let mut check = |ok: bool, what: &str, fix: &str| {
        if ok {
            println!("  好  {what}");
        } else {
            bad += 1;
            println!("  缺  {what}：{fix}");
        }
    };
    let out = |cmd: &str, args: &[&str]| std::process::Command::new(cmd).args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
    check(out("cargo", &["--version"]).is_some(), "Rust（cargo）", "装 Rust：https://rustup.rs");
    let targets = out("rustup", &["target", "list", "--installed"]).unwrap_or_default();
    check(targets.lines().any(|l| l.trim() == "wasm32-unknown-unknown"), "wasm32 目标", "rustup target add wasm32-unknown-unknown");
    match device::saved() {
        Some(d) => check(device::reachable(&d), &format!("掌机 {}", d.host), "掌机开着吗？和开发机在一个 Wi‑Fi 里吗？开发者模式开了吗？"),
        None => println!("  --  掌机：还没配对（要推到掌机上再 vibegame pair <地址>）"),
    }
    if bad == 0 {
        println!("都好了。vibegame new mygame 起一个游戏。");
    }
    Ok(())
}
