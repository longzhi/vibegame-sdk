//! 开窗口玩：720×480，和掌机的屏一样大。240×160 的放大 3 倍（CPU 做最近邻）；
//! 640×480 的真彩画面不放大，放在中间，左右各 40 个黑点。键盘当键，声音从声卡出。

use crate::player::{Player, FPS};
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use minifb::{Key, Window, WindowOptions};
use qfull::full::{key, Full, H, W};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 键盘到键。
const KEYS: [(Key, u32); 12] = [
    (Key::Up, key::UP),
    (Key::Down, key::DOWN),
    (Key::Left, key::LEFT),
    (Key::Right, key::RIGHT),
    (Key::X, key::A),
    (Key::Z, key::B),
    (Key::S, key::X),
    (Key::A, key::Y),
    (Key::Q, key::L1),
    (Key::W, key::R1),
    (Key::Enter, key::START),
    (Key::RightShift, key::SELECT),
];

pub fn play(name: &str, p: &mut Player) -> Result<()> {
    let (w, h) = (W * 3, H * 3);
    let mut win = Window::new(&format!("vibeplay · {name}"), w, h, WindowOptions::default()).map_err(|e| anyhow!("开不了窗口：{e}"))?;
    // 按帧的节奏自己管，不让窗口库再睡一遍。
    win.set_target_fps(0);
    let _sound = match audio(p.full().clone()) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("vibeplay：没有声音：{e:#}");
            None
        }
    };
    let mut buf = vec![0u32; w * h];
    let period = Duration::from_secs_f64(1.0 / FPS as f64);
    let mut next = Instant::now();
    while win.is_open() && !win.is_key_down(Key::Escape) {
        let bits = KEYS.iter().filter(|(k, _)| win.is_key_down(*k)).fold(0, |acc, (_, b)| acc | b);
        if let Err(e) = p.step(bits) {
            p.full().synth(|s| s.stop());
            return Err(e);
        }
        p.last.blit(&mut buf, w, h);
        win.update_with_buffer(&buf, w, h).map_err(|e| anyhow!("画不了窗口：{e}"))?;
        next += period;
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        } else {
            // 晚了不补：从现在重新数。
            next = now;
        }
    }
    p.full().synth(|s| s.stop());
    Ok(())
}

/// 开一条声音的输出流：声卡要样本时，从合成器混。
fn audio(full: Arc<Full>) -> Result<cpal::Stream> {
    let dev = cpal::default_host().default_output_device().context("没有声卡")?;
    let cfg = dev.default_output_config().context("声卡不给配置")?;
    let channels = cfg.channels() as usize;
    full.synth(|s| s.set_rate(cfg.sample_rate().0));
    let mut mono = Vec::new();
    let err = |e| eprintln!("vibeplay：声音出错：{e}");
    let stream = match cfg.sample_format() {
        cpal::SampleFormat::F32 => dev.build_output_stream(
            &cfg.into(),
            move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                mono.resize(out.len() / channels, 0.0);
                full.synth(|s| s.render(&mut mono));
                for (frame, &x) in out.chunks_mut(channels).zip(&mono) {
                    frame.fill(x);
                }
            },
            err,
            None,
        )?,
        cpal::SampleFormat::I16 => dev.build_output_stream(
            &cfg.into(),
            move |out: &mut [i16], _: &cpal::OutputCallbackInfo| {
                mono.resize(out.len() / channels, 0.0);
                full.synth(|s| s.render(&mut mono));
                for (frame, &x) in out.chunks_mut(channels).zip(&mono) {
                    frame.fill((x * i16::MAX as f32) as i16);
                }
            },
            err,
            None,
        )?,
        f => return Err(anyhow!("不认识声卡的样本格式 {f:?}")),
    };
    stream.play()?;
    Ok(stream)
}
