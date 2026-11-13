//! 集市：登录、发布、看状态、下架（QingOS 的 ADR 0037）。服务端是 vibeplay-api。
//!
//! ```text
//! POST /v1/login/start                    → {device_code, user_code, verification_uri, interval}
//! POST /v1/login/poll  {device_code}      → {pending} 或 {token, login}
//! POST /v1/publish     multipart：package、agree=yes、shot×3   头 authorization: Bearer <令牌>
//! GET  /v1/status?name=                   → {login, apps: [...], rejected: [...]}
//! POST /v1/unpublish   {name, version?}
//! ```
//!
//! 登录走 GitHub 的设备码：人在浏览器里输码，agent 替不了。令牌存在 ~/.config/vibeplay/market.json，不进仓库。
//! 发布前在这台电脑上先查一遍（服务端还会再查）：清单、不开窗口跑 600 帧、截三张图、打不带 `.cwasm` 的包。

use crate::game::Game;
use crate::player::Player;
use crate::{pack, script};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// 集市的地址。`--market` 和环境变量 `VIBEPLAY_MARKET` 能换（本机试的时候指 wrangler dev）。
pub const DEFAULT: &str = "https://api.vibeplay.korekore.ai";
/// 不开窗口跑几帧不出错才发（ADR 0037 第 5 条）。
const FRAMES: u64 = 600;
/// 在哪几帧截图，跟着包一起传。
const SHOTS: [u64; 3] = [60, 180, 420];
/// 系统的名字：集市上谁都不能用。服务端也查。
/// 掌机预装的四款游戏（courier、ironcamp、skyfire、thunder）不在这里：只有官方账号能发，由服务端判断。
const RESERVED: [&str; 10] = ["qing", "market", "settings", "agenda", "list", "family", "duck", "vibebuddy", "vibeplay", "full-block"];

#[derive(Serialize, Deserialize, Default)]
struct Saved {
    market: String,
    token: String,
    login: String,
    /// 同意过条款：有权发布包里的全部内容。只问一次。
    #[serde(default)]
    agreed: bool,
}

fn file() -> Result<PathBuf> {
    Ok(crate::device::config_dir().context("找不到 HOME")?.join("market.json"))
}

fn saved() -> Option<Saved> {
    serde_json::from_slice(&std::fs::read(file().ok()?).ok()?).ok()
}

fn save(s: &Saved) -> Result<()> {
    let f = file()?;
    std::fs::create_dir_all(f.parent().unwrap())?;
    std::fs::write(&f, serde_json::to_vec_pretty(s)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// 用哪个集市：`--market`，再看环境变量，再看登录时存的，最后是默认的。
pub fn url(arg: Option<&str>) -> String {
    let u = arg.map(str::to_string).or_else(|| std::env::var("VIBEPLAY_MARKET").ok()).or_else(|| saved().map(|s| s.market)).unwrap_or_else(|| DEFAULT.into());
    u.trim().trim_end_matches('/').to_string()
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(120))).http_status_as_error(false).build().into()
}

/// 回的 JSON；不是 2xx 时拿服务端说的原因报错。
fn reply(r: Result<ureq::http::Response<ureq::Body>, ureq::Error>, market: &str) -> Result<Value> {
    let mut r = r.with_context(|| format!("连不上集市 {market}"))?;
    let status = r.status().as_u16();
    let v: Value = r.body_mut().read_json().unwrap_or(Value::Null);
    match status {
        200..=299 => Ok(v),
        401 => bail!("集市不认这个登录：vibeplay login 再登一次"),
        _ => bail!("集市回 {status}：{}", v["error"].as_str().unwrap_or("没说原因")),
    }
}

fn token(market: &str) -> Result<String> {
    match saved() {
        Some(s) if s.market == market && !s.token.is_empty() => Ok(s.token),
        Some(s) if !s.token.is_empty() => bail!("登录的是 {}，不是 {market}：vibeplay login --market {market}", s.market),
        _ => bail!("还没登录：先 vibeplay login（要人在浏览器里点）"),
    }
}

pub fn login(market: &str) -> Result<()> {
    let a = agent();
    let start = reply(a.post(format!("{market}/v1/login/start")).send_json(json!({})), market)?;
    let code = start["device_code"].as_str().context("集市没给 device_code")?;
    let mut every = start["interval"].as_u64().unwrap_or(5);
    println!("在浏览器里打开 {}", start["verification_uri"].as_str().unwrap_or("https://github.com/login/device"));
    println!("输这个码：{}", start["user_code"].as_str().unwrap_or("?"));
    println!("等你在浏览器里点完……");
    loop {
        std::thread::sleep(Duration::from_secs(every));
        let r = reply(a.post(format!("{market}/v1/login/poll")).send_json(json!({ "device_code": code })), market)?;
        if let Some(t) = r["token"].as_str() {
            let who = r["login"].as_str().unwrap_or("").to_string();
            let agreed = saved().is_some_and(|s| s.agreed && s.login == who);
            save(&Saved { market: market.into(), token: t.into(), login: who.clone(), agreed })?;
            println!("登录好了：{who}。令牌存在 {}", file()?.display());
            return Ok(());
        }
        match r["pending"].as_str() {
            Some("authorization_pending") => {}
            Some("slow_down") => every += 5,
            Some("expired_token") => bail!("码过期了：再 vibeplay login"),
            Some("access_denied") => bail!("在浏览器里点了取消"),
            other => bail!("登录没成：{}", other.unwrap_or("集市没说原因")),
        }
    }
}

/// 清单里集市要、命令行别处不查的几项。服务端还会再查一遍。
fn check_manifest(g: &Game) -> Result<()> {
    let text = std::fs::read_to_string(g.dir.join("app.toml"))?;
    let v: toml::Value = toml::from_str(&text)?;
    let app = v.get("app").and_then(|a| a.as_table()).context("app.toml 里没有 [app]")?;
    let s = |k: &str| app.get(k).and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let b = |k: &str| app.get(k).and_then(|x| x.as_bool()).unwrap_or(false);
    let has = |k: &str| app.iter().any(|(key, x)| (key == k || key.strip_prefix(k).is_some_and(|r| r.len() == 3 && r.starts_with('_'))) && x.as_str().is_some_and(|s| !s.trim().is_empty()));
    let mut bad = Vec::new();
    let version = s("version");
    let parts: Vec<_> = version.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.bytes().all(|c| c.is_ascii_digit())) {
        bad.push(format!("version = \"{version}\"：要写成 x.y.z，每发一次往上加"));
    }
    if RESERVED.contains(&g.m.name.as_str()) {
        bad.push(format!("name = \"{}\"：是系统的名字，换一个", g.m.name));
    }
    if !has("title") {
        bad.push("要有 title（可以再加 title_zh）".into());
    }
    if !has("description") {
        bad.push("要有 description：集市上的一两句介绍（可以再加 description_zh）".into());
    }
    if g.m.cover.is_empty() {
        bad.push("要有 cover：assets/ 下的一张 PNG".into());
    }
    if let Some(langs) = app.get("langs") {
        let ok = langs.as_array().is_some_and(|l| !l.is_empty() && l.iter().all(|x| matches!(x.as_str(), Some("en" | "zh"))));
        if !ok {
            bad.push("langs 只能写 en、zh（游戏暂时不支持别的语言）".into());
        }
    }
    if b("mcu") {
        bad.push("要 mcu 的 app 集市不收".into());
    }
    if v.get("agent").is_some() {
        bad.push("带人格（[[agent]]）的 app 集市不收".into());
    }
    if b("network") && app.get("hosts").and_then(|h| h.as_array()).is_none_or(|h| h.is_empty()) {
        bad.push("要网（network = true）就要写 hosts：要连的地址".into());
    }
    if !bad.is_empty() {
        bail!("发到集市前，app.toml 还要改几处：\n  - {}", bad.join("\n  - "));
    }
    Ok(())
}

/// 不开窗口跑 600 帧，顺路截三张图：回每张 PNG 的字节。
fn trial(g: &Game, wasm: &Path) -> Result<Vec<Vec<u8>>> {
    qfull::full::set_lang("en");
    let mut p = Player::load(wasm, None, g.m.memory_mb)?;
    let scale = if p.full().format() == qfull::full::Format::Rgb565 { 1 } else { 3 };
    let dir = std::env::temp_dir().join(format!("vibeplay-publish-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let none = script::Script::default();
    let mut shots = Vec::new();
    for at in SHOTS {
        p.run(at - p.frame_no, &none, |_, _| {}).context("不开窗口跑的时候出错了：先修好再发")?;
        let f = dir.join(format!("{at}.png"));
        p.shot(&f, scale, None)?;
        shots.push(std::fs::read(&f)?);
    }
    p.run(FRAMES - p.frame_no, &none, |_, _| {}).context("不开窗口跑的时候出错了：先修好再发")?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(shots)
}

/// 问一次条款。只在有人的终端上能答。
fn agree(s: &mut Saved) -> Result<()> {
    if s.agreed {
        return Ok(());
    }
    println!("发到集市以前要同意一条：包里的代码、画面、声音、名字，你都有权发布，不侵犯别人的权利。");
    println!("违反的 app 会被下架，已经装了的也可能被停用。");
    print!("同意吗？输 yes 回车：");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    if line.trim() != "yes" {
        bail!("没同意，不发。要人自己在终端里答这一条");
    }
    s.agreed = true;
    save(s)
}

/// multipart 的身子：几个 (名字, 文件名, 字节)。
fn multipart(parts: &[(&str, Option<&str>, &[u8])]) -> (String, Vec<u8>) {
    let boundary = format!("vibeplay{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
    let mut body = Vec::new();
    for (name, file, bytes) in parts {
        body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"").as_bytes());
        if let Some(f) = file {
            body.extend_from_slice(format!("; filename=\"{f}\"\r\nContent-Type: application/octet-stream").as_bytes());
        }
        body.extend_from_slice(b"\r\n\r\n");
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

pub fn publish(dir: &Path, market: &str) -> Result<()> {
    let token = token(market)?;
    let g = Game::open(dir)?;
    check_manifest(&g)?;
    println!("编、不开窗口跑 {FRAMES} 帧、截图……");
    let wasm = g.build()?;
    let shots = trial(&g, &wasm)?;
    let (pkg, _) = pack::pack(dir, None)?;
    let mut s = saved().unwrap_or_default();
    agree(&mut s)?;
    let mut parts: Vec<(&str, Option<&str>, &[u8])> = vec![("package", Some("app.vp"), &pkg), ("agree", None, b"yes")];
    for (i, shot) in shots.iter().enumerate() {
        parts.push(("shot", Some(["1.png", "2.png", "3.png"][i]), shot));
    }
    let (ctype, body) = multipart(&parts);
    println!("传 {} KB……", pkg.len() / 1024);
    let r = reply(agent().post(format!("{market}/v1/publish")).header("authorization", format!("Bearer {token}")).header("content-type", ctype).send(&body[..]), market)?;
    let sha = r["sha256"].as_str().unwrap_or("");
    println!("上架了：{} {}（sha256 {}）。掌机下次打开集市就看得到。", r["name"].as_str().unwrap_or(""), r["version"].as_str().unwrap_or(""), sha.get(..8).unwrap_or(sha));
    Ok(())
}

pub fn status(name: Option<&str>, market: &str) -> Result<()> {
    let token = token(market)?;
    let mut req = agent().get(format!("{market}/v1/status")).header("authorization", format!("Bearer {token}"));
    if let Some(n) = name {
        req = req.query("name", n);
    }
    let r = reply(req.call(), market)?;
    println!("{} 在 {market} 上的 app：", r["login"].as_str().unwrap_or("?"));
    let apps = r["apps"].as_array().cloned().unwrap_or_default();
    if apps.is_empty() {
        println!("  还没有。vibeplay publish 发一个。");
    }
    for a in &apps {
        let state = match a["state"].as_str() {
            Some("stopped") => format!("被集市停用了：{}", a["stop_reason"].as_str().unwrap_or("")),
            Some("removed") => "被集市下架了".into(),
            _ => "在集市上".into(),
        };
        println!("  {}  {state}  掌机上装了 {} 次，网页里玩了 {} 次", a["name"].as_str().unwrap_or("?"), a["installs"], a["plays"]);
        for v in a["versions"].as_array().into_iter().flatten() {
            let listed = if v["listed"].as_bool() == Some(true) { "上架" } else { "下架了" };
            println!("    {}  {listed}  {}", v["version"].as_str().unwrap_or("?"), v["created"].as_str().unwrap_or("").get(..10).unwrap_or(""));
        }
    }
    let rejected = r["rejected"].as_array().cloned().unwrap_or_default();
    if !rejected.is_empty() {
        println!("最近没过的：");
        for x in &rejected {
            println!("  {}  {}  {}", x["at"].as_str().unwrap_or("").get(..16).unwrap_or(""), x["name"].as_str().unwrap_or("-"), x["error"].as_str().unwrap_or(""));
        }
    }
    Ok(())
}

pub fn unpublish(name: &str, version: Option<&str>, market: &str) -> Result<()> {
    let token = token(market)?;
    let body = match version {
        Some(v) => json!({ "name": name, "version": v }),
        None => json!({ "name": name }),
    };
    reply(agent().post(format!("{market}/v1/unpublish")).header("authorization", format!("Bearer {token}")).send_json(body), market)?;
    match version {
        Some(v) => println!("{name} {v} 下架了。已经装了的掌机照常用。"),
        None => println!("{name} 下架了。已经装了的掌机照常用，只是没有更新。"),
    }
    Ok(())
}
