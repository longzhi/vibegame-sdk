# VibeGame SDK

做 VibeGame 掌机上的像素游戏：240×160、256 色、每秒 60 帧，四个声道的芯片音。
用 Rust 写，编成 wasm。你自己写，或者让 agent 写：Claude Code、Codex 都行，别的 agent 也行。

```sh
cargo install --git https://github.com/longzhi/vibegame-sdk vibegame-cli
rustup target add wasm32-unknown-unknown
vibegame skill install        # 给 Claude Code、Codex 装上说明；别的 agent 让它跑 vibegame skill

vibegame new star-catch && cd star-catch
vibegame run                  # 开窗口玩
vibegame web                  # dist/star-catch.html：一个文件，浏览器里就能玩，能发给别人
vibegame pack                 # dist/star-catch.vg：拷进 TF 卡的「游戏」目录
vibegame pair 192.168.1.23    # 和掌机配对一次（掌机开开发者模式，设置 → 系统里有口令）
vibegame push --watch         # 推到掌机上直接开，一改就再推
```

## 里面有什么

| 目录 | 是什么 |
|---|---|
| `vibegame/` | 写游戏的库：画面、按键、声音、精灵、地图、镜头、音乐、存档，自带中英文点阵字 |
| `cli/` | `vibegame` 命令行：起游戏、开窗口玩、按脚本跑、截图、出网页版、打包、推到掌机 |
| `web/` | 网页版的播放器：`vibegame web` 把它和游戏塞进一个 HTML |
| `skill/SKILL.md` | 给 agent 的说明。`vibegame skill` 印出来 |
| `templates/blank/` | `vibegame new` 起的模板：《捡星星》 |
| `examples/` | 四款做完的游戏，抄着做 |
| `tools/` | 生成自带字体（`make-font.py`）、画样板游戏的封面（`game-covers.py`） |

## 四款样板

- `courier`《疾风邮差》：横版跳跃，六关。
- `thunder`《雷霆突击》：横版射击，四关，两个突击队员选一个。
- `ironcamp`《铁壳守营》：俯视的坦克战，二十关。
- `duckpush`《鸭鸭搬面包》：推箱子，二十关。

`vibegame run examples/courier` 开窗口玩，`vibegame web examples/courier` 出网页版。

## agent 怎么自己验

agent 看不到窗口，所以命令行能不开窗口跑：

```sh
vibegame shot --frames 60,300 --input s.txt     # 跑到第 60、300 帧各存一张 PNG，agent 读图看
vibegame run --headless --frames 300 --input s.txt   # 印出每一个音调
```

按键脚本一行一条：`0-29 right` 是前 30 帧按着右，`30 a` 是第 30 帧按 A。
画面、声音、按键的代码和掌机上的是同一份，开发机上看到什么，掌机上就是什么。

## 装到掌机上

- **Wi‑Fi**：掌机打开开发者模式。`vibegame pair <地址>` 输屏幕上的口令，配一次。
  以后 `vibegame push` 打包、推过去、直接打开。同一个游戏再推直接换，存档留着。新游戏掌机上问一次，要人按 A。
- **TF 卡**：`vibegame pack` 出的 `.vg` 拷进卡上的「游戏」目录。插上掌机开机就装。

`.vg` 是一个 tar.gz：`app.toml`、`bin/<名字>.wasm`、给掌机编好的 `bin/<名字>.cwasm`、`assets/`。
掌机上不带编译器，所以 `pack` 在开发机上编好。掌机的系统升级换了运行时，旧包要重新打。

## 建在什么上

游戏是 [QingOS](https://github.com/longzhi/qingos-sdk) 上的一个整屏 app。写 app 的库（qapp）和放整屏 app 的库（qingos-host）在 qingos-sdk 里，这里用 git 依赖。

## 开源协议

MIT。自带的点阵字是缝合像素字体（OFL-1.1），全文在 `vibegame/fonts/fusion-pixel-font/`。
