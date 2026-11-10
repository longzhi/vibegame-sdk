//! 和掌机说话：配对、推游戏。走局域网的 HTTP，到掌机上 QingOS 的控制台（只在开发者模式下开）。
//!
//! ```text
//! GET  /api/apps/push     头 x-qing-code: <口令>   → 200 {"device": 名字, "apps": [名字…]}
//! POST /api/apps/push     头 x-qing-code，身子是 .vg 包
//!                  → 200 {"name": 名字, "state": "updated" | "installed" | "declined" | "timeout"}
//! ```
//!
//! 同名的游戏直接换掉（存档留着）再打开。新游戏掌机上问一次「装不装」，问的时候这个请求等着。
//! 口令存在 ~/.config/vibegame/device.json：只在这台开发机上，不进仓库。

use crate::pack;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    pub host: String,
    pub code: String,
}

fn file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("vibegame/device.json"))
}

/// 配过的掌机。
pub fn saved() -> Option<Device> {
    serde_json::from_slice(&std::fs::read(file()?).ok()?).ok()
}

fn base(host: &str) -> String {
    let h = host.trim().trim_end_matches('/');
    if h.starts_with("http://") || h.starts_with("https://") {
        h.to_string()
    } else {
        format!("http://{h}")
    }
}

fn agent(secs: u64) -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(secs))).http_status_as_error(false).build().into()
}

/// 问一下掌机：回 (状态码, 身子)。
fn hello(d: &Device) -> Result<(u16, serde_json::Value)> {
    let mut r = agent(5).get(format!("{}/api/apps/push", base(&d.host))).header("x-qing-code", &d.code).call().with_context(|| format!("连不上 {}：掌机开着吗？和开发机在一个 Wi‑Fi 里吗？", d.host))?;
    let status = r.status().as_u16();
    let body = r.body_mut().read_json().unwrap_or(serde_json::Value::Null);
    Ok((status, body))
}

/// 状态码说成人话。
fn explain(status: u16, host: &str) -> anyhow::Error {
    match status {
        401 => anyhow::anyhow!("口令不对：看掌机屏幕上的口令（设置 → 系统），再 vibegame pair {host}"),
        404 => anyhow::anyhow!("掌机没开开发者模式：在掌机的设置 → 系统里打开，重启后再试"),
        413 => anyhow::anyhow!("包太大了，掌机不收"),
        s => anyhow::anyhow!("掌机回了 {s}"),
    }
}

pub fn reachable(d: &Device) -> bool {
    matches!(hello(d), Ok((200, _)))
}

/// 配对：口令对了就存下来。
pub fn pair(host: &str, code: Option<&str>) -> Result<()> {
    let code = match code {
        Some(c) => c.to_string(),
        None => {
            eprint!("掌机屏幕上的口令（设置 → 系统）：");
            let mut s = String::new();
            std::io::stdin().read_line(&mut s)?;
            s.trim().to_string()
        }
    };
    let d = Device { host: host.trim().to_string(), code };
    let (status, body) = hello(&d)?;
    if status != 200 {
        return Err(explain(status, host));
    }
    let f = file().context("没有 HOME，存不了口令")?;
    std::fs::create_dir_all(f.parent().unwrap())?;
    std::fs::write(&f, serde_json::to_vec_pretty(&d)?)?;
    let games: Vec<&str> = body["apps"].as_array().into_iter().flatten().filter_map(|v| v.as_str()).collect();
    println!("配好了：{}（{}）。上面有 {} 个游戏{}", body["device"].as_str().unwrap_or("掌机"), d.host, games.len(), if games.is_empty() { String::new() } else { format!("：{}", games.join("、")) });
    println!("vibegame push 推当前目录的游戏。");
    Ok(())
}

/// 推一次：打包、发过去、说结果。
fn push_once(dir: &Path, d: &Device) -> Result<()> {
    let (bytes, g) = pack::pack(dir, pack::DEVICE_TARGET)?;
    println!("推 {}（{} KB）到 {}…", g.m.name, bytes.len() / 1024, d.host);
    // 新游戏要人在掌机上按，最多等 90 秒。
    let mut r = agent(90)
        .post(format!("{}/api/apps/push", base(&d.host)))
        .header("x-qing-code", &d.code)
        .header("content-type", "application/x-vibegame")
        .send(&bytes[..])
        .with_context(|| format!("连不上 {}", d.host))?;
    let status = r.status().as_u16();
    if status != 200 {
        let text = r.body_mut().read_to_string().unwrap_or_default();
        return Err(explain(status, &d.host).context(text));
    }
    let body: serde_json::Value = r.body_mut().read_json().context("掌机回的不是 JSON")?;
    match body["state"].as_str().unwrap_or("") {
        "updated" => println!("换好了，掌机上开着 {}（存档留着）", g.m.name),
        "installed" => println!("装好了，掌机上开着 {}", g.m.name),
        "declined" => bail!("掌机上按了「不装」"),
        "timeout" => bail!("掌机上没人按：新游戏要在掌机上按 A 才装"),
        s => bail!("掌机回了不认识的 {s}"),
    }
    Ok(())
}

/// 游戏目录里最新的改动时间：源码、素材、清单。
fn newest(dir: &Path) -> SystemTime {
    fn walk(p: &Path, best: &mut SystemTime) {
        let Ok(m) = std::fs::metadata(p) else { return };
        if m.is_dir() {
            for e in std::fs::read_dir(p).into_iter().flatten().flatten() {
                walk(&e.path(), best);
            }
        } else if let Ok(t) = m.modified() {
            *best = (*best).max(t);
        }
    }
    let mut best = SystemTime::UNIX_EPOCH;
    for rel in ["src", "assets", "app.toml", "Cargo.toml"] {
        walk(&dir.join(rel), &mut best);
    }
    best
}

pub fn push(dir: &Path, host: Option<&str>, watch: bool) -> Result<()> {
    let mut d = saved().context("还没配对：vibegame pair <掌机的地址>")?;
    if let Some(h) = host {
        d.host = h.to_string();
    }
    if !watch {
        return push_once(dir, &d);
    }
    println!("盯着 {}：一改就再推。Ctrl‑C 停。", dir.display());
    let mut seen = SystemTime::UNIX_EPOCH;
    loop {
        let t = newest(dir);
        if t > seen {
            seen = t;
            if let Err(e) = push_once(dir, &d) {
                eprintln!("vibegame：{e:#}");
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
