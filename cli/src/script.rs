//! 按键脚本：一行一条，「帧 键…」或「起-止 键…」，帧从 0 数，两头都算。`#` 后面是注释。
//!
//! ```text
//! 0-29 right     # 前 30 帧按着右
//! 30 a           # 第 30 帧按 A
//! ```

use anyhow::{bail, Context, Result};
use qingos_host::full::key;

#[derive(Default)]
pub struct Script {
    spans: Vec<(u64, u64, u32)>,
}

impl Script {
    pub fn parse(text: &str) -> Result<Script> {
        let mut spans = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut words = line.split_whitespace();
            let at = words.next().unwrap_or("");
            let (a, b) = match at.split_once('-') {
                Some((a, b)) => (a, b),
                None => (at, at),
            };
            let what = || format!("脚本第 {} 行：{line}", n + 1);
            let (a, b): (u64, u64) = (a.parse().with_context(what)?, b.parse().with_context(what)?);
            if b < a {
                bail!("{}：止在起之前", what());
            }
            let mut bits = 0;
            for w in words {
                bits |= key::of(w).with_context(|| format!("{}：不认识的键 {w}", what()))?;
            }
            spans.push((a, b, bits));
        }
        Ok(Script { spans })
    }

    /// 第 `frame` 帧按着哪些键。
    pub fn at(&self, frame: u64) -> u32 {
        self.spans.iter().filter(|(a, b, _)| (*a..=*b).contains(&frame)).fold(0, |acc, s| acc | s.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_single_frames_add_up() {
        let s = Script::parse("# 试一下\n0-2 right\n2 A up # 第三帧\n").unwrap();
        assert_eq!(s.at(0), key::RIGHT);
        assert_eq!(s.at(2), key::RIGHT | key::A | key::UP);
        assert_eq!(s.at(3), 0);
        assert!(Script::parse("1 jump").is_err());
        assert!(Script::parse("5-1 a").is_err());
    }
}
