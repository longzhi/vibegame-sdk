//! 打包：`<名字>.vp`。就是一个 tar.gz，解开是一个 app 目录：
//!
//! ```text
//! app.toml
//! bin/<名字>.wasm     游戏本身，浏览器版也用它
//! bin/<名字>.cwasm    给掌机编好的本机码（清单的 command 指着它）
//! assets/…            封面和别的素材
//! ```
//!
//! 推到掌机、拷进 TF 卡的包带 `.cwasm`：掌机装的时候不编，所以在这里编。掌机的系统升级换了 wasmtime，旧包要重新打。
//! 发到集市的包不带 `.cwasm`：掌机从 `.wasm` 自己编，不用别人编的本机码（QingOS 的 ADR 0039）。

use crate::game::Game;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// 掌机的目标三元组。
pub const DEVICE_TARGET: &str = "aarch64-unknown-linux-musl";

/// 打一个包，回包的字节和游戏。`target` 是 None 时不带 `.cwasm`（发到集市的包）。
pub fn pack(dir: &Path, target: Option<&str>) -> Result<(Vec<u8>, Game)> {
    let g = Game::open(dir)?;
    let wasm_path = g.build()?;
    let wasm = std::fs::read(&wasm_path)?;
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best()));
    let add = |tar: &mut tar::Builder<_>, name: &str, bytes: &[u8]| -> Result<()> {
        let mut h = tar::Header::new_gnu();
        h.set_size(bytes.len() as u64);
        h.set_mode(0o644);
        h.set_mtime(0);
        h.set_cksum();
        tar.append_data(&mut h, name, bytes)?;
        Ok(())
    };
    add(&mut tar, "app.toml", &std::fs::read(dir.join("app.toml"))?)?;
    let cmd = g.m.command.clone();
    add(&mut tar, &cmd.replace(".cwasm", ".wasm"), &wasm)?;
    if let Some(target) = target {
        let cwasm = qfull::precompile(&wasm, Some(target)).with_context(|| format!("给 {target} 编不了"))?;
        add(&mut tar, &cmd, &cwasm)?;
    }
    for f in files(&dir.join("assets"))? {
        let rel = f.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/");
        add(&mut tar, &rel, &std::fs::read(&f)?)?;
    }
    let bytes = tar.into_inner()?.finish()?;
    Ok((bytes, g))
}

/// 一个目录下所有的文件，排好序（包每次打出来一样）。没有这个目录是空的。
fn files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            out.extend(files(&p)?);
        } else if !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}
