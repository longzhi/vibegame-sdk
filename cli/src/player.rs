//! 跑一个游戏：一帧交按键、调一次 `qing_tick`、取画面和音调。窗口和不开窗口都用它。
//! 装载器和画面、声音的代码是 qfull 的，掌机上用的是同一份。

use crate::script::Script;
use anyhow::{Context, Result};
use qfull::full::{Frame, Full, Tone, H, W};
use std::path::Path;
use std::sync::Arc;

/// 整屏页每秒几帧。
pub const FPS: u32 = 60;

pub struct Player {
    inner: qfull::Player,
    /// 跑过几帧。
    pub frame_no: u64,
    /// 最新的画面：这一帧没交就还是上一帧的。
    pub last: Frame,
}

impl Player {
    /// `data` 是游戏存档的目录（`data_write`）；None 时存不了。
    pub fn load(module: &Path, data: Option<std::path::PathBuf>, memory_mb: u64) -> Result<Player> {
        Ok(Player { inner: qfull::Player::load(module, data, memory_mb)?, frame_no: 0, last: Frame::default() })
    }

    pub fn full(&self) -> &Arc<Full> {
        self.inner.full()
    }

    /// 走一帧：交这一帧的按键，调 `qing_tick`，取画面。返回这一帧发的音调。游戏出错（panic、死循环）回错。
    pub fn step(&mut self, buttons: u32) -> Result<Vec<Tone>> {
        self.inner.tick(buttons).with_context(|| format!("第 {} 帧", self.frame_no))?;
        let full = self.inner.full();
        if let Some(f) = full.take_frame() {
            self.last = f;
        }
        self.frame_no += 1;
        Ok(full.take_tones())
    }

    /// 不开窗口跑 `frames` 帧，按脚本按键。合成器照样往前走（一帧的样本混出来丢掉），音调有长短才对。
    pub fn run(&mut self, frames: u64, script: &Script, mut on_tone: impl FnMut(u64, &Tone)) -> Result<()> {
        let full = self.inner.full().clone();
        let mut buf = vec![0.0f32; (full.synth(|s| s.rate()) / FPS) as usize];
        for _ in 0..frames {
            let n = self.frame_no;
            for t in self.step(script.at(n))? {
                on_tone(n, &t);
            }
            full.synth(|s| s.render(&mut buf));
        }
        Ok(())
    }

    /// 最新的一帧存成 PNG，放大 `scale` 倍（最近邻）。`crop` 是只要的一块：x、y、宽、高，原大小的像素。
    pub fn shot(&self, out: &Path, scale: usize, crop: Option<(usize, usize, usize, usize)>) -> Result<()> {
        let k = scale.max(1);
        let (x0, y0, cw, ch) = crop.unwrap_or((0, 0, W, H));
        if cw == 0 || ch == 0 || x0 + cw > W || y0 + ch > H {
            anyhow::bail!("--crop 出了画面：画面是 {W}×{H}");
        }
        let file = std::fs::File::create(out).with_context(|| format!("写不了 {}", out.display()))?;
        let full = self.last.rgb_scaled(1);
        let mut rgb = Vec::with_capacity(cw * ch * 3 * k * k);
        for y in 0..ch * k {
            for x in 0..cw * k {
                let i = ((y0 + y / k) * W + x0 + x / k) * 3;
                rgb.extend_from_slice(&full[i..i + 3]);
            }
        }
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), (cw * k) as u32, (ch * k) as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()?.write_image_data(&rgb)?;
        Ok(())
    }
}
