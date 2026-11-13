//! 作者自愿公开的源码（QingOS 的 ADR 0044）：`vibeplay publish --source` 才打。
//!
//! 不整个目录打包：目录里有 `target/`（几个 GB）、`.env` 这类东西。只收下面这些，别的一概不收：
//!
//! ```text
//! Cargo.toml  Cargo.lock  app.toml  src/…  assets/…
//! ```
//!
//! 打之前查：压缩后不超过 2 MB，没有像 key 的字符串。服务端还会再查一遍。

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
    for n in &names {
        let bytes = std::fs::read(dir.join(n)).with_context(|| format!("读不了 {n}"))?;
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
    Ok(Source { bytes, files, license: license(dir)? })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(files: &[(&str, &[u8])]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vibeplay-src-{}-{}", std::process::id(), files.len()));
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
        let dir = game(&[
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
    fn keys_and_missing_files_are_refused() {
        let dir = game(&[("Cargo.toml", b"x"), ("app.toml", b"[app]\n"), ("src/lib.rs", b"const K: &str = \"sk-abcdefghijklmnopqrstuvwxyz\";")]);
        assert!(pack(&dir).err().unwrap().to_string().contains("src/lib.rs"));
        let _ = std::fs::remove_dir_all(&dir);
        let dir = game(&[("app.toml", b"[app]\n")]);
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
