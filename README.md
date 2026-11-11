# VibeGame SDK

English · [中文](README.zh-CN.md)

Make pixel games for the VibeGame handheld: 240×160, 256 colors, 60 frames a second, four chip-sound channels.
You write games in Rust and build them to wasm. Write them yourself, or let an agent write them. Claude Code and Codex work, and so do other agents.

You need Rust. If you do not have it, install it from [rustup.rs](https://rustup.rs) first.

```sh
cargo install --git https://github.com/longzhi/vibegame-sdk vibegame-cli
rustup target add wasm32-unknown-unknown
vibegame doctor               # checks what is missing and prints the command to fix it
vibegame skill install        # gives Claude Code and Codex the instructions; other agents run `vibegame skill`

vibegame new star-catch && cd star-catch
vibegame run                  # play in a window
vibegame web                  # dist/star-catch.html: one file that plays in a browser and that you can send to anyone
vibegame pack                 # dist/star-catch.vg: copy it into the games folder on a TF card
vibegame pair 192.168.1.23    # pair with the handheld once (turn on developer mode; the code is in Settings → System)
vibegame push --watch         # send it to the handheld and open it; send again after each change
```

## What is in this repository

| Folder | What it is |
|---|---|
| `vibegame/` | The library for games: screen, buttons, sound, sprites, tile maps, camera, music, saves, and a built-in English and Chinese pixel font |
| `cli/` | The `vibegame` command line: start a game, play it in a window, run it from a key script, take screenshots, make a web version, pack it, push it to the handheld |
| `web/` | The web player. `vibegame web` puts it and the game into one HTML file |
| `skill/SKILL.md` | The instructions for agents. `vibegame skill` prints them |
| `templates/blank/` | The template that `vibegame new` starts from: Star Catch |
| `examples/` | Four finished games to copy from |
| `tools/` | Makes the built-in font (`make-font.py`) and draws the covers of the sample games (`game-covers.py`) |

## Four sample games

- `courier`, Gale Courier: side-scrolling platformer, six stages.
- `thunder`, Thunder Strike: run-and-gun, four stages, pick one of two soldiers.
- `skyfire`, Skyfire: vertical shooter, four stages, a boss at the end of each.
- `ironcamp`, Iron Camp: top-down tank battle, twenty stages.

`vibegame run examples/courier` plays one in a window. `vibegame web examples/courier` makes its web version.

To put a web version in your own page, use `<iframe src="courier.html?embed">`. Only the screen shows.
The outer page presses keys with `postMessage({ vgKey: "a", down: true })` and releases them with `down: false`.

## English and Chinese

The handheld system has two languages, English and Chinese. English is the default, and the person changes it in Settings.
A game asks for the current language with `vibegame::full::lang()` and picks the text to draw:

```rust
use vibegame::full::lang;
frame.text_center(&FONT, 70, lang().pick("PRESS START", "按 START 开始"), WHITE);
```

Ask each time you draw text. Do not keep the answer: the person can change the language from the menu while the game is paused, and the text must follow at once.
In `app.toml`, put the English name in `title` and the Chinese name in `title_zh`. The handheld home screen shows the one for the system language.
On your computer, add `--lang zh` to `vibegame run` or `shot` to see the Chinese text.
A web version reads `?lang=zh` from its address. Without it, the browser language decides, and the page has a switch at the bottom. A page that embeds it sends `postMessage({ vgLang: "zh" })`.

## How an agent checks its own work

An agent cannot see a window, so the command line can run without one:

```sh
vibegame shot --frames 60,300 --input s.txt          # saves a PNG at frames 60 and 300 for the agent to look at
vibegame run --headless --frames 300 --input s.txt   # prints every note that plays
```

A key script has one entry per line. `0-29 right` holds right for the first 30 frames. `30 a` presses A at frame 30.
The screen, sound and button code is the same code that runs on the handheld. What you see on your computer is what the handheld shows.

## Putting games on the handheld

- **Wi-Fi**: turn on developer mode on the handheld. Run `vibegame pair <address>` and type the code from the screen. You pair once.
  After that, `vibegame push` packs the game, sends it and opens it. Push the same game again and the handheld swaps it in and keeps the saves. For a new game, the handheld asks once, and a person must press A.
- **TF card**: copy the `.vg` file from `vibegame pack` into the `games` folder on the card (a folder named `游戏` also works). Put the card in and turn the handheld on to install it.
- **Wi-Fi on the handheld**: type the password with the D-pad in Settings, or put a `wifi.toml` file at the top of the TF card and turn the handheld on:

  ```toml
  ssid = "your network"
  password = "your password"
  ```

A `.vg` file is a tar.gz with `app.toml`, `bin/<name>.wasm`, `bin/<name>.cwasm` (built for the handheld) and `assets/`.
The handheld has no compiler, so `pack` builds on your computer. When a system update changes the handheld runtime, pack old games again.

## What it is built on

Each game is a full-screen app on [QingOS](https://github.com/longzhi/qingos-sdk). The library for apps (qapp) and the library that runs full-screen apps (qfull) are in qingos-sdk. This repository uses them as git dependencies.

## License

MIT. The built-in pixel font is Fusion Pixel Font (OFL-1.1). The full text is in `vibegame/fonts/fusion-pixel-font/`.
