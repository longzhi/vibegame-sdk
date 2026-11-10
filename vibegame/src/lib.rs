//! VibeGame 写游戏的库。游戏是 QingOS 上的一个整屏 app：240×160、256 色、每秒 60 帧。
//!
//! ```ignore
//! use vibegame::full::{Frame, Keys};
//! use std::sync::Mutex;
//! static G: Mutex<Option<(Frame, Keys)>> = Mutex::new(None);
//! vibegame::main!(|app| {
//!     app.every(|| {
//!         let mut g = G.lock().unwrap();
//!         let (f, k) = g.get_or_insert_with(Default::default);
//!         k.read();
//!         f.clear(0);
//!         f.show();
//!     });
//! });
//! ```
//!
//! - [`full`]：画面（`Frame`）、按键（`Keys`）、音调（`tone`）、精灵、地图块、镜头、音乐。
//! - [`FONT`]：自带的点阵字，中文英文都有。
//! - [`save`]、[`load`]：存档。存在掌机上这个游戏自己的地方。
//! - [`main!`]：游戏的入口。每帧调一次你交给 `app.every` 的函数。

pub use qapp::full;
pub use qapp::main;

mod font;
/// 自带的点阵字：12 像素高，ASCII 和 GB2312 的全部字（缝合像素字体，OFL-1.1）。`frame.text(&FONT, x, y, "字", 颜色)`。
pub use font::FONT;

/// 存一份档。`name` 是一个文件名（没有斜杠，不以 `.` 开头）。
pub fn save(name: &str, bytes: &[u8]) -> bool {
    qapp::host::data_write(name, bytes).is_ok()
}

/// 读一份档。没有回 None。
pub fn load(name: &str) -> Option<Vec<u8>> {
    qapp::host::data_read(name)
}

/// 删一份档。
pub fn erase(name: &str) -> bool {
    qapp::host::data_delete(name).is_ok()
}

/// 记一行日志：`vibegame run` 印在终端里，掌机上进系统日志。
pub fn log(s: &str) {
    qapp::host::log(s)
}
