//! 网页版：一个单独的 HTML 文件，游戏的 .wasm 和素材都塞在里面。双击就能玩，也能发给别人、放到网上。
//! 宿主在 web/player.js：画面、按键、合成器、存档照着掌机做。

use crate::game::Game;
use anyhow::Result;
use std::path::Path;

const PAGE: &str = include_str!("../../web/page.html");
const PLAYER: &str = include_str!("../../web/player.js");

fn b64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            out.push(if i <= c.len() { T[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    out
}

/// HTML 里的字：标题里的 `<`、`&` 换掉。
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// 出一个网页版。回 HTML。
pub fn page(dir: &Path) -> Result<String> {
    let g = Game::open(dir)?;
    let wasm = std::fs::read(g.build()?)?;
    let mut assets = serde_json::Map::new();
    let adir = dir.join("assets");
    if adir.is_dir() {
        for e in std::fs::read_dir(&adir)? {
            let p = e?.path();
            if p.is_file() {
                let name = p.file_name().unwrap().to_string_lossy().into_owned();
                assets.insert(name, b64(&std::fs::read(&p)?).into());
            }
        }
    }
    let title = if g.m.title.is_empty() { g.m.name.clone() } else { g.m.title.clone() };
    let data = serde_json::json!({ "name": g.m.name, "title": title, "wasm": b64(&wasm), "assets": assets });
    // JSON 放进 <script>：`</` 换成 `<\/`，不会提前把 script 关上。
    let data = data.to_string().replace("</", "<\\/");
    Ok(PAGE.replace("{{title}}", &escape(&title)).replace("{{data}}", &data).replace("{{player}}", PLAYER))
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(super::b64(b""), "");
        assert_eq!(super::b64(b"f"), "Zg==");
        assert_eq!(super::b64(b"fo"), "Zm8=");
        assert_eq!(super::b64(b"foobar"), "Zm9vYmFy");
    }
}
