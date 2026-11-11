//! `vibegame new`：从模板起一个游戏。`vibegame skill`：给 agent 的说明。
//! 模板是 templates/blank，编进命令行里：命令行装到哪都能起。

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

const LIB: &str = include_str!("../../templates/blank/src/lib.rs");
const MANIFEST: &str = include_str!("../../templates/blank/app.toml");
const COVER: &[u8] = include_bytes!("../../templates/blank/assets/cover.png");
/// 给 agent 的说明（Skill）。Claude Code、Codex 装成技能，别的 agent 跑 `vibegame skill` 读。
pub const SKILL: &str = include_str!("../../skill/SKILL.md");

const REPO: &str = "https://github.com/second-state/vibegame-sdk";

/// 起一个游戏。`sdk` 是本地的 vibegame-sdk 目录：改 SDK 的人用，平常从 GitHub 拿。
pub fn create(dir: &Path, title: Option<&str>, sdk: Option<&Path>) -> Result<()> {
    if dir.exists() && std::fs::read_dir(dir)?.next().is_some() {
        bail!("{} 已经有东西了：换一个新目录", dir.display());
    }
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_lowercase()) || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        bail!("目录名「{name}」就是游戏的名字：要英文小写开头，只用小写字母、数字、-（比如 star-catch）");
    }
    let dep = match sdk {
        Some(p) => {
            let p = std::fs::canonicalize(p).with_context(|| format!("没有 {}", p.display()))?;
            if !p.join("vibegame/Cargo.toml").is_file() {
                bail!("{} 不是 vibegame-sdk 的目录", p.display());
            }
            format!("vibegame = {{ path = \"{}\" }}", p.join("vibegame").display())
        }
        None => format!("vibegame = {{ git = \"{REPO}\", tag = \"v{}\" }}", env!("CARGO_PKG_VERSION")),
    };
    let title = title.unwrap_or(&name);
    let cargo = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
{dep}

# 游戏自己一个工作区：放在别的 Rust 项目里也不串。
[workspace]

# 编成 wasm 用 release：小、快。
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"
"#
    );
    let manifest = MANIFEST.replace("\"blank\"", &format!("\"{name}\"")).replace("bin/blank.cwasm", &format!("bin/{name}.cwasm"));
    // 新游戏的名字只有一个（不写就是目录名）。中文名留一行注释，要的时候打开。
    let manifest: String = manifest
        .lines()
        .map(|l| {
            if l.starts_with("title_zh") {
                "# title_zh  = \"中文名\"          # 系统是中文时用这个名字\n".to_string()
            } else if l.starts_with("title ") {
                format!("title     = \"{}\"\n", title.replace('"', ""))
            } else {
                format!("{l}\n")
            }
        })
        .collect();
    // 标题画面上的名字也换成新的：不然第一次 run 看到的是模板的「STAR CATCH」。
    let shown = format!("{:?}", title.to_uppercase());
    let lib = LIB.replace("\"STAR CATCH\"", &shown).replace("\"捡星星\"", &shown);
    let files: [(&str, &[u8]); 5] = [
        ("Cargo.toml", cargo.as_bytes()),
        ("app.toml", manifest.as_bytes()),
        ("src/lib.rs", lib.as_bytes()),
        ("assets/cover.png", COVER),
        (".gitignore", b"/target\n/bin\n/dist\n/save\n*.png\n!/assets/*.png\n"),
    ];
    for (rel, bytes) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap())?;
        std::fs::write(&p, bytes).with_context(|| format!("写不了 {}", p.display()))?;
    }
    println!("起好了：{}", dir.display());
    println!("  cd {} && vibegame run        开窗口玩", dir.display());
    println!("  规则在 src/lib.rs 的 step，画面在 draw。给 agent 看的说明：vibegame skill");
    Ok(())
}

/// 把说明装成 Claude Code 和 Codex 的技能。`to` 给了就只装到那个目录（比如项目里的 .claude/skills）。
pub fn install_skill(to: Option<&Path>) -> Result<()> {
    let dirs: Vec<PathBuf> = match to {
        Some(d) => vec![d.to_path_buf()],
        None => {
            let home = PathBuf::from(std::env::var_os("HOME").context("没有 HOME")?);
            let mut v: Vec<PathBuf> = [".claude", ".codex"].iter().map(|a| home.join(a)).filter(|p| p.is_dir()).map(|p| p.join("skills")).collect();
            if v.is_empty() {
                v.push(home.join(".claude/skills"));
            }
            v
        }
    };
    for d in dirs {
        let p = d.join("vibegame/SKILL.md");
        std::fs::create_dir_all(p.parent().unwrap())?;
        std::fs::write(&p, SKILL).with_context(|| format!("写不了 {}", p.display()))?;
        println!("{}", p.display());
    }
    println!("别的 agent：让它先跑 vibegame skill 读说明。");
    Ok(())
}
