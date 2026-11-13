//! 网页版：一个单独的 HTML 文件，游戏的 .wasm 和素材都塞在里面。双击就能玩，也能发给别人、放到网上。
//! 宿主在 web/player.js：画面、按键、合成器、存档照着掌机做。

use crate::game::Game;
use anyhow::Result;
use std::path::Path;

const PAGE: &str = include_str!("../../web/page.html");
const PLAYER: &str = include_str!("../../web/player.js");
const REMOTE: &str = include_str!("../../web/remote.js");

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
    if !g.m.frame.is_empty() {
        anyhow::bail!("网页版还只认 240×160 的游戏：{} 写了 frame = \"{}\"", g.m.name, g.m.frame);
    }
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
    let title = g.m.title_in("en").to_string();
    let data = serde_json::json!({ "name": g.m.name, "title": { "en": title, "zh": g.m.title_in("zh") }, "wasm": b64(&wasm), "assets": assets });
    // JSON 放进 <script>：`</` 换成 `<\/`，不会提前把 script 关上。
    let data = data.to_string().replace("</", "<\\/");
    Ok(PAGE.replace("{{title}}", &escape(&title)).replace("{{data}}", &data).replace("{{player}}", PLAYER))
}

/// 不带游戏的网页版：打开时按地址里的 `?pkg=` 拉一个 .vp 再玩。集市的网页用它。
pub fn remote_page() -> String {
    PAGE.replace("{{title}}", "VibePlay")
        .replace("<script>window.VP = {{data}};</script>", "")
        .replace("<script>{{player}}</script>", &format!("<script type=\"text/plain\" id=\"vp-player\">{PLAYER}</script>\n<script>{REMOTE}</script>"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_remote_page_waits_for_a_package() {
        let p = super::remote_page();
        assert!(p.contains("id=\"vp-player\"") && p.contains("?pkg="));
        assert!(!p.contains("{{"), "模板里的空都填上了");
        assert!(!super::PLAYER.contains("</script"), "player.js 放进 <script> 里不能提前关上");
    }

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(super::b64(b""), "");
        assert_eq!(super::b64(b"f"), "Zg==");
        assert_eq!(super::b64(b"fo"), "Zm8=");
        assert_eq!(super::b64(b"foobar"), "Zm9vYmFy");
    }
}
