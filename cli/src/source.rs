//! 作者自愿公开的源码（QingOS 的 ADR 0044）：`vibeplay publish --source` 才打。
//!
//! 不整个目录打包：目录里有 `target/`（几个 GB）、`.env` 这类东西。只收下面这些，别的一概不收：
//!
//! ```text
//! Cargo.toml  Cargo.lock  app.toml  src/…  assets/…
//! ```
//!
//! 打之前查：压缩后不超过 2 MB，没有像 key 的字符串。服务端还会再查一遍。
//!
//! `Cargo.toml` 要让别人也编得了：`vibeplay` 是本机路径的依赖，换成这个 SDK 版本的 git 依赖；
//! 从 workspace 继承的 `version` 这些，换成 workspace 里写的值。别的本机路径依赖不收。

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// 压缩后最多多大。和服务端一样。
pub const MAX: usize = 2 << 20;
/// `app.toml` 没写 `license` 时用的开源协议。
pub const DEFAULT_LICENSE: &str = "MIT";

/// 打好的源码：tar.gz 的字节，和里面的每个文件（相对路径、字节数）。
pub struct Source {
    pub bytes: Vec<u8>,
    pub files: Vec<(String, u64)>,
    pub license: String,
    /// 发出去的 Cargo.toml 改了什么：别人才编得了。
    pub changed: Vec<String>,
}

/// 要收的文件，排好序。
fn pick(dir: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for f in ["Cargo.toml", "Cargo.lock", "app.toml"] {
        if dir.join(f).is_file() {
            out.push(f.to_string());
        }
    }
    for sub in ["src", "assets"] {
        for p in walk(&dir.join(sub))? {
            out.push(p.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
    out.sort();
    Ok(out)
}

/// 一个目录下的文件，跳过隐藏的。没有这个目录是空的。
fn walk(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
            continue;
        }
        if p.is_dir() {
            out.extend(walk(&p)?);
        } else {
            out.push(p);
        }
    }
    Ok(out)
}

/// 一条 key 的样子：开头、后面的字符、至少几个、叫什么。
type Rule = (&'static str, fn(u8) -> bool, usize, &'static str);

/// 像 key 的字符串：回它像什么。和服务端查的一样。
pub fn secret(text: &str) -> Option<&'static str> {
    let run = |at: usize, ok: fn(u8) -> bool| text.as_bytes()[at..].iter().take_while(|b| ok(**b)).count();
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    let alnum = |b: u8| b.is_ascii_alphanumeric();
    let upper = |b: u8| b.is_ascii_uppercase() || b.is_ascii_digit();
    let starts = |at: usize| at == 0 || !word(text.as_bytes()[at - 1]);
    if text.contains("PRIVATE KEY-----") {
        return Some("私钥");
    }
    let rules: [Rule; 9] = [
        ("sk-", word, 20, "以 sk- 开头的 key"),
        ("ghp_", alnum, 30, "GitHub 令牌"),
        ("gho_", alnum, 30, "GitHub 令牌"),
        ("ghu_", alnum, 30, "GitHub 令牌"),
        ("ghs_", alnum, 30, "GitHub 令牌"),
        ("ghr_", alnum, 30, "GitHub 令牌"),
        ("AKIA", upper, 16, "AWS key"),
        ("AIza", word, 35, "Google key"),
        ("xoxb-", word, 10, "Slack 令牌"),
    ];
    for (head, ok, min, what) in rules {
        for (at, _) in text.match_indices(head) {
            if starts(at) && run(at + head.len(), ok) >= min {
                return Some(what);
            }
        }
    }
    None
}

/// 开源协议：`app.toml` 的 `[app] license`，没写是 MIT。
fn license(dir: &Path) -> Result<String> {
    let t: toml::Value = toml::from_str(&std::fs::read_to_string(dir.join("app.toml"))?).context("app.toml 读不了")?;
    Ok(t.get("app").and_then(|a| a.get("license")).and_then(|l| l.as_str()).filter(|l| !l.trim().is_empty()).unwrap_or(DEFAULT_LICENSE).to_string())
}

/// SDK 的仓库：别人编的时候从这里拿 `vibeplay`。
const REPO: &str = "https://github.com/second-state/vibeplay-sdk";

/// 往上找 workspace 的 Cargo.toml。
fn workspace(dir: &Path) -> Option<toml::Table> {
    for d in dir.ancestors().skip(1) {
        if let Ok(Ok(t)) = std::fs::read_to_string(d.join("Cargo.toml")).map(|s| s.parse::<toml::Table>()) {
            if t.contains_key("workspace") {
                return Some(t);
            }
        }
    }
    None
}

/// 发出去的 `Cargo.toml`：别人拿到也编得了。回新的字节，和改了什么（没改是空的）。
fn portable_cargo(dir: &Path) -> Result<(Vec<u8>, Vec<String>)> {
    let raw = std::fs::read_to_string(dir.join("Cargo.toml")).context("读不了 Cargo.toml")?;
    let mut t: toml::Table = raw.parse().context("Cargo.toml 读不了")?;
    let mut changed = Vec::new();
    if let Some(pkg) = t.get_mut("package").and_then(|p| p.as_table_mut()) {
        let ws = workspace(dir);
        let shared = ws.as_ref().and_then(|w| w.get("workspace")?.get("package")?.as_table().cloned()).unwrap_or_default();
        for (k, v) in pkg.iter_mut() {
            if v.get("workspace").and_then(|w| w.as_bool()) == Some(true) {
                *v = shared.get(k).cloned().with_context(|| format!("Cargo.toml 的 {k} 从 workspace 继承，workspace 里没写"))?;
                changed.push(format!("{k} 写成 workspace 里的值"));
            }
        }
    }
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(deps) = t.get_mut(section).and_then(|d| d.as_table_mut()) else { continue };
        for (name, v) in deps.iter_mut() {
            let local = v.get("path").is_some() || v.get("workspace").and_then(|w| w.as_bool()) == Some(true);
            if !local {
                continue;
            }
            if name != "vibeplay" {
                bail!("Cargo.toml 的 {name} 是本机的依赖，别人编不了。换成 crates.io 或 git 的依赖再发");
            }
            let mut git = toml::Table::new();
            git.insert("git".into(), REPO.into());
            git.insert("tag".into(), format!("v{}", env!("CARGO_PKG_VERSION")).into());
            *v = git.into();
            changed.push(format!("vibeplay 换成 git 依赖（v{}）", env!("CARGO_PKG_VERSION")));
        }
    }
    if changed.is_empty() {
        return Ok((raw.into_bytes(), changed));
    }
    Ok((toml::to_string(&t)?.into_bytes(), changed))
}

/// 打源码。不对就报错，说是哪个文件。
pub fn pack(dir: &Path) -> Result<Source> {
    let names = pick(dir)?;
    for need in ["Cargo.toml", "app.toml"] {
        if !names.iter().any(|n| n == need) {
            bail!("源码里要有 {need}");
        }
    }
    if !names.iter().any(|n| n.starts_with("src/")) {
        bail!("源码里要有 src/");
    }
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best()));
    let mut files = Vec::new();
    let (cargo, changed) = portable_cargo(dir)?;
    for n in &names {
        let bytes = if n == "Cargo.toml" { cargo.clone() } else { std::fs::read(dir.join(n)).with_context(|| format!("读不了 {n}"))? };
        if let Some(what) = secret(&String::from_utf8_lossy(&bytes)) {
            bail!("{n} 里像有{what}。拿掉再发（服务端也会查）");
        }
        let mut h = tar::Header::new_gnu();
        h.set_size(bytes.len() as u64);
        h.set_mode(0o644);
        h.set_mtime(0);
        h.set_cksum();
        tar.append_data(&mut h, n, &bytes[..])?;
        files.push((n.clone(), bytes.len() as u64));
    }
    let bytes = tar.into_inner()?.finish()?;
    if bytes.len() > MAX {
        let mut big = files.clone();
        big.sort_by_key(|f| std::cmp::Reverse(f.1));
        let top: Vec<String> = big.iter().take(3).map(|(n, s)| format!("{n}（{} KB）", s / 1024)).collect();
        bail!("源码压缩后 {} KB，超过 {} MB。最大的是：{}", bytes.len() / 1024, MAX >> 20, top.join("、"));
    }
    Ok(Source { bytes, files, license: license(dir)?, changed })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(tag: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vibeplay-src-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (n, b) in files {
            let p = dir.join(n);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, b).unwrap();
        }
        dir
    }

    #[test]
    fn only_the_listed_files_go_in() {
        let dir = game("listed", &[
            ("Cargo.toml", b"[package]\nname = \"sky\"\n"),
            ("app.toml", b"[app]\nname = \"sky\"\nlicense = \"Apache-2.0\"\n"),
            ("src/lib.rs", b"fn x() {}\n"),
            ("assets/cover.png", b"png"),
            ("target/release/sky.wasm", b"big"),
            (".env", b"KEY=1"),
            ("src/.secret", b"x"),
            ("notes.md", b"x"),
        ]);
        let s = pack(&dir).unwrap();
        let names: Vec<&str> = s.files.iter().map(|f| f.0.as_str()).collect();
        assert_eq!(names, ["Cargo.toml", "app.toml", "assets/cover.png", "src/lib.rs"]);
        assert_eq!(s.license, "Apache-2.0");
        assert_eq!(pack(&dir).unwrap().bytes, s.bytes, "每次打出来一样");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_local_vibeplay_becomes_a_git_dependency() {
        let root = game("ws", &[("Cargo.toml", b"[workspace]\nmembers = []\n[workspace.package]\nversion = \"0.3.0\"\nedition = \"2021\"\n")]);
        let dir = root.join("examples/sky");
        for (n, b) in [
            ("Cargo.toml", "[package]\nname = \"sky\"\nversion.workspace = true\nedition.workspace = true\n\n[dependencies]\nvibeplay = { path = \"../../vibeplay\" }\n"),
            ("app.toml", "[app]\nname = \"sky\"\n"),
            ("src/lib.rs", "fn x() {}\n"),
        ] {
            std::fs::create_dir_all(dir.join(n).parent().unwrap()).unwrap();
            std::fs::write(dir.join(n), b).unwrap();
        }
        let (bytes, changed) = portable_cargo(&dir).unwrap();
        let t: toml::Table = String::from_utf8(bytes).unwrap().parse().unwrap();
        assert_eq!(t["package"]["version"].as_str(), Some("0.3.0"));
        assert_eq!(t["package"]["edition"].as_str(), Some("2021"));
        assert_eq!(t["dependencies"]["vibeplay"]["git"].as_str(), Some(REPO));
        assert_eq!(changed.len(), 3);
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"sky\"\n[dependencies]\nmine = { path = \"../mine\" }\n").unwrap();
        assert!(portable_cargo(&dir).unwrap_err().to_string().contains("mine"));
        std::fs::write(dir.join("Cargo.toml"), "# 注释留着\n[package]\nname = \"sky\"\n").unwrap();
        assert_eq!(portable_cargo(&dir).unwrap(), (b"# \xe6\xb3\xa8\xe9\x87\x8a\xe7\x95\x99\xe7\x9d\x80\n[package]\nname = \"sky\"\n".to_vec(), vec![]), "不用改就原样");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// 四款样板的源码拿出 SDK 也编得了（要网：从 git 拿 vibeplay）。`cargo test -- --ignored` 跑。
    #[test]
    #[ignore]
    fn the_examples_build_from_their_source() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for name in ["courier", "ironcamp", "skyfire", "thunder"] {
            let src = pack(&root.join("examples").join(name)).unwrap();
            let out = std::env::temp_dir().join(format!("vibeplay-src-build-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&out);
            std::fs::create_dir_all(&out).unwrap();
            tar::Archive::new(flate2::read::GzDecoder::new(&src.bytes[..])).unpack(&out).unwrap();
            let ok = std::process::Command::new("cargo")
                .args(["build", "--release", "--quiet", "--target", "wasm32-unknown-unknown", "--manifest-path"])
                .arg(out.join("Cargo.toml"))
                .env("CARGO_TARGET_DIR", std::env::temp_dir().join("vibeplay-src-build-target"))
                .status()
                .unwrap()
                .success();
            assert!(ok, "{name} 的源码拿出来编不了");
            let _ = std::fs::remove_dir_all(&out);
        }
    }

    #[test]
    fn keys_and_missing_files_are_refused() {
        let dir = game("key", &[("Cargo.toml", b"[package]\nname = \"sky\"\n"), ("app.toml", b"[app]\n"), ("src/lib.rs", b"const K: &str = \"sk-abcdefghijklmnopqrstuvwxyz\";")]);
        assert!(pack(&dir).err().unwrap().to_string().contains("src/lib.rs"));
        let _ = std::fs::remove_dir_all(&dir);
        let dir = game("nocargo", &[("app.toml", b"[app]\n")]);
        assert!(pack(&dir).err().unwrap().to_string().contains("Cargo.toml"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn what_looks_like_a_key() {
        assert_eq!(secret("-----BEGIN PRIVATE KEY-----"), Some("私钥"));
        assert_eq!(secret("token = ghp_0123456789abcdefghijklmnopqrstuvwxyz"), Some("GitHub 令牌"));
        assert_eq!(secret("AKIAABCDEFGHIJKLMNOP"), Some("AWS key"));
        assert_eq!(secret("let task-done = 1; // ask-me"), None, "单词里的 sk- 不算");
        assert_eq!(secret("sk-short"), None);
        assert_eq!(secret("fn main() {}"), None);
    }
}
