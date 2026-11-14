//! 一个游戏的目录：读清单、查清单、用 cargo 编成 wasm。

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// 清单 `app.toml` 里命令行用得到的几项。
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub app: AppSection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSection {
    pub name: String,
    /// 名字。系统是哪种语言、又没有那种语言的名字时用它。
    #[serde(default)]
    pub title: String,
    /// 中文的名字（可以不写）。
    #[serde(default)]
    pub title_zh: String,
    /// 英文的名字（可以不写）。
    #[serde(default)]
    pub title_en: String,
    pub command: String,
    #[serde(default)]
    pub screen: String,
    /// 交哪种画面：不写是 240×160、256 色；"640x480" 是真彩（RGB565），掌机不放大（QingOS 的 ADR 0036）。
    #[serde(default)]
    pub frame: String,
    #[serde(default)]
    pub rate_hz: u32,
    #[serde(default = "sixteen")]
    pub memory_mb: u64,
    #[serde(default)]
    pub cover: String,
    /// 游戏还是应用：`game`（不写就是）或 `app`。集市按它分两栏。
    #[serde(default)]
    pub kind: String,
}

impl AppSection {
    /// 这种语言（`en`、`zh`）下的名字。
    pub fn title_in(&self, lang: &str) -> &str {
        let t = if lang == "zh" { &self.title_zh } else { &self.title_en };
        [t, &self.title, &self.name].into_iter().find(|s| !s.is_empty()).unwrap()
    }
}

fn sixteen() -> u64 {
    16
}

/// 一个游戏：目录和清单。
pub struct Game {
    pub dir: PathBuf,
    pub m: AppSection,
}

impl Game {
    /// 读一个游戏的目录。清单不对就说哪里不对：掌机装的时候也会这样查。
    pub fn open(dir: &Path) -> Result<Game> {
        let path = dir.join("app.toml");
        let text = std::fs::read_to_string(&path).with_context(|| format!("{} 下没有 app.toml：这不是一个游戏的目录", dir.display()))?;
        let m: Manifest = toml::from_str(&text).with_context(|| format!("{} 写错了", path.display()))?;
        let a = m.app;
        let mut bad = Vec::new();
        if a.name.is_empty() || !a.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') || !a.name.starts_with(|c: char| c.is_ascii_lowercase()) {
            bad.push(format!("name = \"{}\"：要英文小写开头，只用小写字母、数字、-", a.name));
        }
        if a.screen != "full" {
            bad.push("要写 screen = \"full\"（整屏页）".into());
        }
        if !a.frame.is_empty() && a.frame != "640x480" {
            bad.push(format!("frame = \"{}\"：不写是 240×160，要真彩写 \"640x480\"", a.frame));
        }
        if a.rate_hz != 60 {
            bad.push(format!("rate_hz 是 {}，要 60", a.rate_hz));
        }
        if !a.command.ends_with(".cwasm") || !a.command.starts_with("bin/") {
            bad.push(format!("command = \"{}\"：要 bin/<名字>.cwasm", a.command));
        }
        if !a.kind.is_empty() && a.kind != "game" && a.kind != "app" {
            bad.push(format!("kind = \"{}\"：游戏写 game（不写也是），应用写 app", a.kind));
        }
        if !a.cover.is_empty() {
            let c = dir.join(&a.cover);
            if !a.cover.starts_with("assets/") || !a.cover.ends_with(".png") {
                bad.push(format!("cover = \"{}\"：要 assets/ 下的一张 .png", a.cover));
            } else if !c.is_file() {
                bad.push(format!("cover：没有 {}", c.display()));
            }
        }
        if !bad.is_empty() {
            bail!("{} 有几处不对：\n  - {}", path.display(), bad.join("\n  - "));
        }
        Ok(Game { dir: dir.to_path_buf(), m: a })
    }

    /// 用 cargo 编成 wasm（release），返回 .wasm 的路径。
    pub fn build(&self) -> Result<PathBuf> {
        if !self.dir.join("Cargo.toml").is_file() {
            // 没有源码：用 bin/ 下编好的。
            let wasm = self.dir.join(&self.m.command).with_extension("wasm");
            return if wasm.is_file() { Ok(wasm) } else { bail!("{} 下没有 Cargo.toml，也没有编好的 {}", self.dir.display(), wasm.display()) };
        }
        build(&self.dir)
    }
}

/// 用 cargo 把一个目录编成 wasm，返回 .wasm 的路径。
pub fn build(dir: &Path) -> Result<PathBuf> {
    let out = std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["build", "--release", "--quiet", "--target", "wasm32-unknown-unknown", "--message-format=json-render-diagnostics", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("起不来 cargo（vibeplay doctor 看看装好没有）")?;
    if !out.status.success() {
        bail!("{} 编不过（错在上面）。缺 wasm32 目标就跑 vibeplay doctor", dir.display());
    }
    let manifest = std::fs::canonicalize(dir.join("Cargo.toml"))?;
    let mut found = None;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        // cargo 回的路径可能没解开符号链接（macOS 的 /var 是 /private/var）：两边都解开再比。
        if v["reason"] != "compiler-artifact" || v["manifest_path"].as_str().and_then(|p| std::fs::canonicalize(p).ok()).as_deref() != Some(manifest.as_path()) {
            continue;
        }
        for f in v["filenames"].as_array().into_iter().flatten() {
            if let Some(f) = f.as_str().filter(|f| f.ends_with(".wasm")) {
                found = Some(PathBuf::from(f));
            }
        }
    }
    found.with_context(|| format!("{} 编完了，没找到 .wasm（Cargo.toml 要有 crate-type = [\"cdylib\", \"rlib\"]）", dir.display()))
}

/// 命令行给的是一个目录还是一个 .wasm、.cwasm 文件：回要跑的模块、名字、内存上限。
pub fn module_of(path: &Path) -> Result<(PathBuf, String, u64)> {
    if path.is_file() {
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("game").to_string();
        return Ok((path.to_path_buf(), name, 64));
    }
    let g = Game::open(path)?;
    Ok((g.build()?, g.m.name.clone(), g.m.memory_mb))
}
