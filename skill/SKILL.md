---
name: vibeplay
description: 做 VibePlay 掌机上的像素游戏：240×160、256 色、每秒 60 帧，Rust 写，编成 wasm。用 vibeplay 命令行起游戏、开窗口玩、按脚本跑、截图自己看、出网页版、打包成 .vp、推到掌机。人要做小游戏、改游戏、问 VibePlay 或掌机游戏怎么写时用。
---

# 做 VibePlay 游戏

游戏是一个 Rust 写的 wasm 模块加一个清单 `app.toml`。掌机每秒调它 60 次。每次它读按键、走一步、画一帧、发声音。
你在开发机上写、跑、截图看；做好了出网页版给人玩，或者推到掌机上。

## 先装好

```sh
vibeplay doctor          # 查缺什么，缺的给出命令
```

没有 Rust（`cargo` 命令）先装 rustup：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`（Windows 看 https://rustup.rs）。
没有 vibeplay 命令就装：`cargo install --git https://github.com/second-state/vibeplay-sdk vibeplay-cli`。
还要 wasm32 目标：`rustup target add wasm32-unknown-unknown`。

## 先问人：游戏里的字用哪种语言

动手之前先问：「游戏里的字用英文、中文，还是两种都要？」人在要求里明说过就不再问。人用中文跟你说话，不等于游戏要中文。
现在只能选英文和中文。人要别的语言（比如日文），说清楚掌机的字和系统现在只有这两种，请他从里面选。

照答案走：

- **只要英文**：画面上的字直接写英文，不用 `lang()`。清单只写 `title`，把 `title_zh` 那行删掉。只截英文的图。
- **只要中文**：画面上的字直接写中文，不用 `lang()`。清单的 `title` 写中文名，`title_zh` 那行删掉。只截图看中文。
  中文字 12 像素宽，是英文的两倍，一行放不下就换行或者改短。
- **两种都要**：画面上的字都写两种，用 `lang().pick`（见下面「语言」）。清单 `title` 写英文名，`title_zh` 写中文名。英文、中文各截一套。

模板（`vibeplay new`）是两种都要的写法。只要一种时，把模板里的 `lang().pick(英文, 中文)` 换成那一种的字。

## 一个游戏从头到尾

```sh
vibeplay new star-catch --title "Star Catch"     # 从模板起一个：十字键走、捡星星、计时、存最高分
cd star-catch
vibeplay run                                # 开窗口玩（人玩）
vibeplay shot --frames 5,120 --input s.txt  # 你看：跑到第 5、120 帧各存一张 PNG
vibeplay run --headless --frames 300 --input s.txt   # 你听：印出每一个音调
vibeplay web                                # dist/star-catch.html：一个文件，浏览器里就能玩
vibeplay pack                               # dist/star-catch.vp：拷进 TF 卡的「游戏」目录
vibeplay pair 192.168.1.23 && vibeplay push # 推到掌机上直接开（掌机要开开发者模式）
```

`vibeplay help` 看全部命令和选项。

`new --title` 换清单的 `title` 和标题画面上的名字。还要自己做三件事：
- 名字照第一步定的语言写：两种都要时打开 `app.toml` 里 `title_zh` 那行注释；只要中文时 `title` 直接写中文名。
- 代码里别的字（「PRESS START」、计分）照新游戏改。
- 模板的封面是通用的一张（夜空、星星、小人），换成自己游戏的。

## 自己验：一定要看截图

你看不到窗口。改了画面就截几张图，用读图片的工具看：

- 字清楚吗？漏了的字会画成空心框。
- 颜色对吗？东西在该在的地方吗？
- 标题、开玩、过关、输了，每个画面都截一张。两种语言都要时，英文（默认）和中文（`--lang zh`）各截一套；只要一种就截那一种。

按键脚本一行一条：`0-29 right` 是前 30 帧按着右，`30 a` 是第 30 帧按 A。帧从 0 数，`#` 后面是注释。
键：`up down left right a b x y l1 r1 start select`。`shot --crop x,y,w,h` 只存一块。
看小精灵的细节（眼睛、几个像素的东西），用 `--crop` 框住它再加 `--scale 6`：整屏 3 倍的图看不清。
`shot --out a.png` 给几个帧号时存成 `a-5.png`、`a-120.png`。
`--data 目录` 给存档一个地方：`run` 开窗口时默认是游戏目录下的 `save/`。
`--headless` 和 `shot` 不给 `--data` 就存不了（`vibeplay::save` 回 false）。要截「标题上有最高分」这种图，给它一个目录。
无头跑时 `tone_wav` 印成「帧 N 录音 毫秒=…」，别的印成「帧 N 音调 声道=… 频率=…」。

规则用单元测试验：`cargo test`。测试里直接造 `Game::default()`，`keys.set(位)`，再调 `step()`，看状态。
想知道第几帧起跳正好，在测试里循环试，试出来写进脚本。

## 写游戏用的东西（`vibeplay::full`）

下面是常用的。全部的函数看源码：`vibeplay::full` 就是 qingos-sdk 的 `qapp::full`。
游戏编过一次，源码就在 `~/.cargo/git/checkouts/qingos-sdk-*/<提交>/qapp/src/full.rs` 和 `full/` 下面。
有几份时看游戏的 `Cargo.lock` 里 qingos-sdk 那一行的提交。

- **画面** `Frame`：240×160，一个像素一个字节，是调色板的一格。调色板 256 格，`frame.palette[格] = [r, g, b]`。
  `clear`、`set`、`rect`、`blit(&精灵, x, y, 左右翻)`、`frame_rect`。每帧最后 `frame.show()` 一次。
- **按键** `Keys`：每帧开头 `keys.read()` 一次。`held`（按着）、`pressed`（这一帧刚按下）、`released`。
  键是 `key::UP`、`DOWN`、`LEFT`、`RIGHT`、`A`、`B`、`X`、`Y`、`L1`、`R1`、`START`、`SELECT`。菜单键归掌机，游戏收不到。
- **精灵** `Sprite::art(&["..yy..", ".yooy."], &[('y', 11), ('o', 13)])`：字符画，`.` 和颜色格 0 透明。
  `scaled(k)` 放大，`recolor(&[(旧, 新)])` 换色（被打中闪白）。
- **地图** `TileMap::parse(&行, 16, |字符| 格)`：一格 16 像素，一屏 15×10 格。`draw(&mut frame, &图块, &cam)`；`hits(x, y, w, h, |格| 实心?)` 做碰撞。
- **镜头** `Camera::follow(x, y, 世界宽, 世界高)`：主角走出画面中间那块才跟。`to_screen`、`sees`。
- **字** `vibeplay::FONT`：12 像素高，英文和常用中文都有。`frame.text(&FONT, x, y, "第 1 关", 颜色)`、`text_center`、`text_outlined`（压在花背景上加描边）。
  大字：`FONT.sprite("标题", 颜色).scaled(3)` 再 `blit`。
- **声音**：四个声道。`Square1`、`Square2` 是方波，`Triangle` 是三角波，`Noise` 是噪声（频率越高越沙）。
  `tone(声道, 频率, 毫秒, 音量)`。短录音：`tone_wav(&wav)`，16 位 PCM，最长 2 秒，和声道混着响。
- **音乐**：`Track::parse(声道, 音量, "C4 E4 G4:2 r A#3")`，`Song::new(一格几帧, 循环, 几轨)`，`music.play(song)`，每帧 `music.tick()`。
  音效 `music.sfx(声道, 频率, 毫秒, 音量)`：音乐在那个声道上让一会儿，响完接上。
- **语言** `vibeplay::full::lang()`：系统是英文还是中文，默认英文。只在人要两种语言时用（见「先问人」）。画面上的字写两种：`lang().pick("GAME OVER", "游戏结束")`。
  每次出字都问，别存进游戏的状态里：人能在暂停时换语言。英文字 6 像素宽，中文 12 像素，两种都要截图看放不放得下。
  清单 `title` 写英文名，`title_zh` 写中文名。测试里用 `vibeplay::full::native::set_lang(Lang::Zh)` 换（只换这个线程）。
- **存档**：`vibeplay::save("best", 字节)`、`vibeplay::load("best")`、`vibeplay::erase`。存在掌机上这个游戏自己的地方，浏览器里存在本机。
- **日志**：`vibeplay::log("…")`，`vibeplay run` 印在终端里。

## 骨架

模板就是这个样子。状态全放在一个 `Game` 里：

```rust
static GAME: Mutex<Option<Game>> = Mutex::new(None);
vibeplay::main!(|app| {
    app.every(|| {
        let mut g = GAME.lock().unwrap();
        let g = g.get_or_insert_with(Game::default);
        g.keys.read();
        g.step();   // 动、算碰撞、发声
        g.draw();   // 画
        g.frame.show();
    });
});
```

## 约定

- **清单** `app.toml`：`name` 是英文小写，和目录名一样。`screen = "full"`、`rate_hz = 60` 不改。
  `cover` 是掌机的家上的封面，`assets/` 下一张 96×80 的 PNG。画一张，别直接截图。封面上不写名字：家上封面底下有名字，跟着系统的语言换。
  样板的封面是 `tools/game-covers.py` 一个像素一个像素画的（Python 加 Pillow：`pip install pillow`），照它写一个。
- **调色板**：格子编号起名字（`const INK: u8 = 1;`），画的时候只写名字。第 0 格当背景。
- **位置**用整数（比如 1/16 像素），不用浮点。随机数自己写，开局从同一个数开始。这样同样的按键每次跑出来一样，截图和测试才稳。
- **主角** 16×16 左右。动画是几张精灵换着画：走两张，跳一张。
- **声音**：旋律放 `Square1`，低音放 `Triangle`，鼓点放 `Noise`，音效放 `Square2` 或用 `sfx` 借一个声道。跳、捡东西、打中各用不一样的高低。
  爆炸这种声音：代码里生成一小段噪声慢慢变小的 WAV，`tone_wav` 放。库里没有现成的，抄 `skyfire` 的 `boom_wav`。

## 抄样板

https://github.com/second-state/vibeplay-sdk/tree/main/examples 有四款做完的游戏，要什么抄什么。
游戏编过一次，样板也在本机：`~/.cargo/git/checkouts/vibeplay-sdk-*/<提交>/examples/`，不上网也能读。

- `courier`《疾风邮差》：横版跳跃。地图块、镜头跟着走、重力、踩敌人、大怪。
- `thunder`《雷霆突击》：横版射击。子弹、八个方向打、一屏很多东西、音乐和爆炸声一起响。
- `skyfire`《烈空战机》：竖版飞机射击。地图往上卷、出敌脚本、成片的子弹、分几块打的大怪、炸弹。
- `ironcamp`《铁壳守营》：俯视坦克战。一格一格的地图、能打碎的墙、敌人的路线、二十关。

## 原创

- 做「那个年代的样子」，不做那个年代的某一款。角色、关卡、画面、音乐都自己做。
- 别人要你照某款老游戏原样做，先说清楚这一条，再给一个原创的版本。
- 像素画用字符画自己画，不从网上拿图。曲子自己写，别哼一首听过的。

## 推到掌机

- 掌机上打开开发者模式（设置 → 系统）。屏幕上有地址和口令。
- `vibeplay pair <地址>` 输口令，配一次就行。`vibeplay push` 打包、推过去、直接打开。`--watch` 一改就再推。
- 同一个游戏再推，直接换掉，存档留着。新游戏掌机上会问一次装不装，要人按 A。
- 不用 Wi‑Fi：`vibeplay pack` 出的 `.vp` 拷进 TF 卡的「游戏」目录，插上掌机开机就装。
