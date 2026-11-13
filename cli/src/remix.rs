//! `vibeplay remix`：拿集市上公开了源码的游戏，起一个新游戏，记下改自哪一版（QingOS ADR 0045）。
//! 只能改集市上最新的那一版：列表里有它源码的 sha256，下载以后对得上才解开。

use anyhow::{bail, Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Component, Path};

/// 新游戏的 `Cargo.toml` 没有这两节就加上：自己一个工作区，release 编得小。和 `vibeplay new` 的一样。
const WORKSPACE: &str = "
# 游戏自己一个工作区：放在别的 Rust 项目里也不串。
[workspace]
";
const PROFILE: &str = "
# 编成 wasm 用 release：小、快。
[profile.release]
opt-level = \"z\"
lto = true
codegen-units = 1
strip = true
panic = \"abort\"
";

/// 名字的规矩和 `vibeplay new` 一样：目录名就是游戏的名字。
fn valid_name(n: &str) -> bool {
    (2..=32).contains(&n.len()) && n.starts_with(|c: char| c.is_ascii_lowercase()) && n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn remix(name: &str, dir: Option<&Path>, market: &str) -> Result<()> {
    let default = format!("{name}-remix");
    let dir = dir.unwrap_or(Path::new(&default));
    let new = dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    if !valid_name(&new) {
        bail!("目录名「{new}」就是新游戏的名字：要英文小写开头，只用小写字母、数字、-，2 到 32 个字");
    }
    if new == name {
        bail!("新游戏要换一个名字：{name} 是原来那个游戏的");
    }
    if dir.exists() && std::fs::read_dir(dir)?.next().is_some() {
        bail!("{} 已经有东西了：换一个新目录", dir.display());
    }

    let a = ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(60))).http_status_as_error(false).build().new_agent();
    let get = |url: &str| -> Result<Vec<u8>> {
        let mut r = a.get(url).call().with_context(|| format!("连不上集市 {market}"))?;
        if r.status() != 200 {
            bail!("{url} 回 {}", r.status());
        }
        let mut b = Vec::new();
        r.body_mut().as_reader().read_to_end(&mut b)?;
        Ok(b)
    };
    let list: Value = serde_json::from_slice(&get(&format!("{market}/list.json"))?).context("集市的列表读不了")?;
    let e = list["apps"].as_array().into_iter().flatten().find(|e| e["name"] == name).with_context(|| format!("集市上没有 {name}"))?;
    let version = e["version"].as_str().unwrap_or("");
    let (Some(src), Some(want)) = (e["source"].as_str(), e["source_sha256"].as_str()) else {
        bail!("{name} 的作者没有公开源码，不能改");
    };
    let license = e["license"].as_str().unwrap_or("MIT");
    let author = e["author"].as_str().unwrap_or("?");
    let bytes = get(&format!("{market}/{src}"))?;
    let got: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    if got != want {
        bail!("下载的源码和集市列表里的 sha256 对不上，不解开");
    }

    unpack(&bytes, dir)?;
    let from = format!("{name}@{version}");
    rename(dir, &new, &from)?;
    println!("起好了：{}，改自 {author} 的 {name} {version}（开源协议 {license}）", dir.display());
    println!("  名字已经换成 {new}，版本从 0.1.0 起，app.toml 里记了 from = \"{from}\"。");
    println!("  title、description、封面还是原来的：发到集市以前改成你的。");
    println!("  cd {} && vibeplay run        开窗口玩", dir.display());
    println!("  照 {license} 开源协议改。发的时候集市会写上改自 {name}。");
    Ok(())
}

/// 解开源码。只解普通文件，路径不能往上走、不能是绝对路径。
fn unpack(bytes: &[u8], dir: &Path) -> Result<()> {
    let mut t = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    for f in t.entries()? {
        let mut f = f?;
        if !f.header().entry_type().is_file() {
            continue;
        }
        let rel = f.path()?.into_owned();
        if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
            bail!("源码里有不对的路径：{}", rel.display());
        }
        let p = dir.join(&rel);
        std::fs::create_dir_all(p.parent().unwrap())?;
        let mut b = Vec::new();
        f.read_to_end(&mut b)?;
        std::fs::write(&p, b).with_context(|| format!("写不了 {}", p.display()))?;
    }
    Ok(())
}

/// 换成新游戏：名字、版本、`from`。app.toml 一行一行改，留着作者的注释。
fn rename(dir: &Path, new: &str, from: &str) -> Result<()> {
    let path = dir.join("app.toml");
    let text = std::fs::read_to_string(&path).context("源码里没有 app.toml")?;
    let key = |l: &str| l.split('=').next().unwrap_or("").trim().to_string();
    let mut out = Vec::new();
    let mut section = String::new();
    let mut wrote_from = false;
    for l in text.lines() {
        let t = l.trim_start();
        if t.starts_with('[') {
            section = t.to_string();
        }
        if section == "[app]" && t.contains('=') && !t.starts_with('#') {
            match key(t).as_str() {
                "name" => {
                    out.push(format!("name      = \"{new}\""));
                    continue;
                }
                "command" => {
                    out.push(format!("command   = \"bin/{new}.cwasm\""));
                    continue;
                }
                "from" => continue,
                "version" => {
                    out.push("version   = \"0.1.0\"".into());
                    out.push(format!("from      = \"{from}\"   # 改自哪个游戏的哪一版：集市上会写出来"));
                    wrote_from = true;
                    continue;
                }
                _ => {}
            }
        }
        out.push(l.to_string());
    }
    if !wrote_from {
        bail!("源码里的 app.toml 没有 version");
    }
    let text = out.join("\n") + "\n";
    let v: toml::Value = toml::from_str(&text).context("改完的 app.toml 读不了")?;
    if v["app"]["name"].as_str() != Some(new) {
        bail!("app.toml 的 name 没改成 {new}");
    }
    std::fs::write(&path, text)?;

    let path = dir.join("Cargo.toml");
    let mut c: toml::Table = std::fs::read_to_string(&path).context("源码里没有 Cargo.toml")?.parse().context("Cargo.toml 读不了")?;
    let pkg = c.get_mut("package").and_then(|p| p.as_table_mut()).context("Cargo.toml 里没有 [package]")?;
    pkg.insert("name".into(), new.into());
    pkg.insert("version".into(), "0.1.0".into());
    let mut s = toml::to_string(&c)?;
    if !c.contains_key("workspace") {
        s.push_str(WORKSPACE);
    }
    if c.get("profile").and_then(|p| p.get("release")).is_none() {
        s.push_str(PROFILE);
    }
    std::fs::write(&path, s)?;
    // 编出来的东西不进 git，和 vibeplay new 的一样。
    std::fs::write(dir.join(".gitignore"), "/target\n/bin\n/dist\n/save\n*.png\n!/assets/*.png\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("vibeplay-remix-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(
            d.join("app.toml"),
            "[app]\nname      = \"skyfire\"\ntitle     = \"Skyfire\"   # 名字\nversion   = \"0.1.1\"\ncommand   = \"bin/skyfire.cwasm\"\nlicense = \"MIT\"\n",
        )
        .unwrap();
        std::fs::write(d.join("Cargo.toml"), "[package]\nname = \"skyfire\"\nversion = \"0.3.0\"\nedition = \"2021\"\n\n[dependencies]\nvibeplay = { git = \"x\", tag = \"v0.3.0\" }\n").unwrap();
        d
    }

    #[test]
    fn rename_换名字_版本_记下_from_留着注释() {
        let d = game("rename");
        rename(&d, "my-sky", "skyfire@0.1.1").unwrap();
        let app = std::fs::read_to_string(d.join("app.toml")).unwrap();
        let v: toml::Value = toml::from_str(&app).unwrap();
        assert_eq!(v["app"]["name"].as_str(), Some("my-sky"));
        assert_eq!(v["app"]["command"].as_str(), Some("bin/my-sky.cwasm"));
        assert_eq!(v["app"]["version"].as_str(), Some("0.1.0"));
        assert_eq!(v["app"]["from"].as_str(), Some("skyfire@0.1.1"));
        assert_eq!(v["app"]["license"].as_str(), Some("MIT"));
        assert!(app.contains("# 名字"));
        let c: toml::Table = std::fs::read_to_string(d.join("Cargo.toml")).unwrap().parse().unwrap();
        assert_eq!(c["package"]["name"].as_str(), Some("my-sky"));
        assert_eq!(c["package"]["version"].as_str(), Some("0.1.0"));
        assert!(c.contains_key("workspace"));
        assert_eq!(c["profile"]["release"]["opt-level"].as_str(), Some("z"));
    }

    #[test]
    fn rename_改过的再改_from_换成新的() {
        let d = game("again");
        rename(&d, "sky-a", "skyfire@0.1.1").unwrap();
        rename(&d, "sky-b", "sky-a@0.1.0").unwrap();
        let app = std::fs::read_to_string(d.join("app.toml")).unwrap();
        assert_eq!(app.lines().filter(|l| l.trim_start().starts_with("from")).count(), 1);
        let v: toml::Value = toml::from_str(&app).unwrap();
        assert_eq!(v["app"]["from"].as_str(), Some("sky-a@0.1.0"));
    }

    #[test]
    fn unpack_不解往上走的路径() {
        let mut t = tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast()));
        let mut h = tar::Header::new_gnu();
        h.set_size(1);
        h.set_mode(0o644);
        h.as_gnu_mut().unwrap().name[..9].copy_from_slice(b"../evil.r");
        h.set_cksum();
        t.append(&h, &b"x"[..]).unwrap();
        let bytes = t.into_inner().unwrap().finish().unwrap();
        let d = std::env::temp_dir().join(format!("vibeplay-remix-unpack-{}", std::process::id()));
        assert!(unpack(&bytes, &d.join("g")).is_err());
        assert!(!d.join("evil.r").exists());
    }

    #[test]
    fn 名字的规矩() {
        assert!(valid_name("my-sky"));
        assert!(!valid_name("My-sky"));
        assert!(!valid_name("s"));
        assert!(!valid_name("sky_fire"));
    }
}
