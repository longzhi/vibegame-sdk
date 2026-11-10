// VibeGame 的网页版宿主：在浏览器里跑一个游戏的 .wasm。
// 游戏只用 12 个导入函数（qingos-sdk 的 docs/ABI.md「整屏 app 用到的」）。这里一样一样补上：
// 画面画到 canvas，键盘、手柄、屏上的按钮当键，合成器照 qfull 的 Synth 一行一行抄过来，存档放 localStorage。
// 页面在 window.VG 里给：{ name, title, wasm（base64）, assets: { 名字: base64 } }。
(() => {
  "use strict";
  const W = 240, H = 160, FPS = 60;
  const KEY = { up: 1, down: 2, left: 4, right: 8, a: 16, b: 32, x: 64, y: 128, l1: 256, r1: 512, start: 1024, select: 2048 };
  // 和 vibegame run 的窗口一样的键位。
  const KEYBOARD = {
    ArrowUp: KEY.up, ArrowDown: KEY.down, ArrowLeft: KEY.left, ArrowRight: KEY.right,
    KeyX: KEY.a, KeyZ: KEY.b, KeyS: KEY.x, KeyA: KEY.y, KeyQ: KEY.l1, KeyW: KEY.r1,
    Enter: KEY.start, ShiftRight: KEY.select,
  };
  // 手柄的标准布局：按位置算，右边下面那个是 A。
  const PAD = [[0, KEY.a], [1, KEY.b], [2, KEY.y], [3, KEY.x], [4, KEY.l1], [5, KEY.r1], [8, KEY.select], [9, KEY.start],
    [12, KEY.up], [13, KEY.down], [14, KEY.left], [15, KEY.right]];

  const b64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));
  const toB64 = (u8) => { let s = ""; for (let i = 0; i < u8.length; i += 0x8000) s += String.fromCharCode.apply(null, u8.subarray(i, i + 0x8000)); return btoa(s); };

  // ---- 合成器：照 qfull/src/full.rs 的 Synth 抄，声音才和掌机一样 ----
  const PEAK = 0.25, WAV_MAX_SECS = 2;
  class Synth {
    constructor(rate) { this.rate = rate; this.voices = [0, 1, 2, 3].map(() => ({ hz: 0, left: 0, amp: 0, phase: 0, lfsr: 1 })); this.wavv = null; }
    tone(ch, hz, ms, vol) {
      const v = this.voices[ch]; if (!v) return false;
      v.hz = hz; v.left = hz === 0 ? 0 : Math.floor(Math.min(ms, 60000) * this.rate / 1000); v.amp = Math.min(vol, 100) / 100 * PEAK; return true;
    }
    wav(bytes) { const w = parseWav(bytes); if (!w || w.samples.length > w.rate * WAV_MAX_SECS) return false; this.wavv = { samples: w.samples, rate: w.rate, at: 0 }; return true; }
    stop() { for (const v of this.voices) v.left = 0; this.wavv = null; }
    render(out) {
      const rate = this.rate;
      for (let i = 0; i < out.length; i++) {
        let mix = 0;
        for (let ch = 0; ch < 4; ch++) {
          const v = this.voices[ch]; if (v.left === 0) continue;
          v.left--;
          const step = v.hz / rate;
          let wave;
          if (ch < 2) wave = v.phase < 0.5 ? 1 : -1;
          else if (ch === 2) wave = 1 - 4 * Math.abs(v.phase - 0.5);
          else wave = (v.lfsr & 1) === 0 ? 1 : -1;
          v.phase = Math.fround(v.phase + step);
          while (v.phase >= 1) {
            v.phase = Math.fround(v.phase - 1);
            if (ch === 3) { const bit = (v.lfsr ^ (v.lfsr >> 1)) & 1; v.lfsr = (v.lfsr >> 1) | (bit << 14); }
          }
          mix += wave * v.amp;
        }
        const w = this.wavv;
        if (w) {
          const x = w.samples[Math.floor(w.at)];
          if (x !== undefined) { mix += x / 32768 * 0.5; w.at += w.rate / rate; }
          if (Math.floor(w.at) >= w.samples.length) this.wavv = null;
        }
        out[i] = Math.max(-1, Math.min(1, mix));
      }
    }
  }
  function parseWav(b) {
    const dv = new DataView(b.buffer, b.byteOffset, b.byteLength);
    const tag = (o) => String.fromCharCode(b[o], b[o + 1], b[o + 2], b[o + 3]);
    if (b.length < 12 || tag(0) !== "RIFF" || tag(8) !== "WAVE") return null;
    let fmt = null, data = null;
    for (let i = 12; i + 8 <= b.length;) {
      const id = tag(i), n = dv.getUint32(i + 4, true);
      const body = [i + 8, Math.min(i + 8 + n, b.length)];
      if (id === "fmt " && n >= 16) fmt = body; else if (id === "data") data = body;
      i += 8 + n + (n & 1);
    }
    if (!fmt || !data) return null;
    const format = dv.getUint16(fmt[0], true), channels = Math.max(1, dv.getUint16(fmt[0] + 2, true));
    const rate = dv.getUint32(fmt[0] + 4, true), bits = dv.getUint16(fmt[0] + 14, true);
    if (format !== 1 || bits !== 16 || rate === 0) return null;
    const samples = [];
    for (let o = data[0]; o + 2 * channels <= data[1]; o += 2 * channels) samples.push(dv.getInt16(o, true));
    return { samples, rate };
  }

  // ---- 页面 ----
  const vg = window.VG;
  const canvas = document.getElementById("screen");
  const ctx = canvas.getContext("2d");
  const img = ctx.createImageData(W, H);
  const status = document.getElementById("status");
  let held = 0, touch = 0, tapped = 0, synth = null, audio = null, inst = null, mem = null, dead = false;

  addEventListener("keydown", (e) => { const k = KEYBOARD[e.code]; if (k) { held |= k; tapped |= k; e.preventDefault(); } startAudio(); });
  addEventListener("keyup", (e) => { const k = KEYBOARD[e.code]; if (k) { held &= ~k; e.preventDefault(); } });
  addEventListener("blur", () => { held = 0; touch = 0; });
  for (const el of document.querySelectorAll("[data-key]")) {
    const k = KEY[el.dataset.key];
    const on = (e) => { e.preventDefault(); touch |= k; tapped |= k; startAudio(); }, off = (e) => { e.preventDefault(); touch &= ~k; };
    el.addEventListener("pointerdown", on); el.addEventListener("pointerup", off); el.addEventListener("pointerleave", off); el.addEventListener("pointercancel", off);
  }
  canvas.addEventListener("pointerdown", startAudio);
  // 嵌着这个页面的网页按键：postMessage({ vgKey: "a", down: true })。键名和 data-key 一样。
  addEventListener("message", (e) => {
    const d = e.data, k = d && KEY[d.vgKey];
    if (!k) return;
    if (d.down) { touch |= k; tapped |= k; startAudio(); } else touch &= ~k;
  });
  // 页面看不见了（切走、锁屏）：游戏停下，声音也停。回来接着走。
  document.addEventListener("visibilitychange", () => {
    if (!audio) return;
    if (document.hidden) audio.suspend(); else audio.resume();
  });

  function padBits() {
    let bits = 0;
    for (const p of navigator.getGamepads ? navigator.getGamepads() : []) {
      if (!p) continue;
      for (const [i, k] of PAD) if (p.buttons[i] && p.buttons[i].pressed) bits |= k;
      const [x, y] = [p.axes[0] || 0, p.axes[1] || 0];
      if (x < -0.5) bits |= KEY.left; if (x > 0.5) bits |= KEY.right; if (y < -0.5) bits |= KEY.up; if (y > 0.5) bits |= KEY.down;
    }
    return bits;
  }

  // 浏览器要人先按一下才肯出声：第一次按键或点画面时开声音。
  function startAudio() {
    if (audio) { if (audio.state === "suspended" && !document.hidden) audio.resume(); return; }
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return;
    audio = new AC();
    synth.rate = audio.sampleRate;
    const node = audio.createScriptProcessor(1024, 0, 1);
    node.onaudioprocess = (e) => synth.render(e.outputBuffer.getChannelData(0));
    node.connect(audio.destination);
    status.textContent = "";
  }

  // ---- 系统给游戏的函数 ----
  const store = (k) => `vibegame:${vg.name}:${k}`;
  const bytes = (p, n) => new Uint8Array(mem.buffer, p, n).slice();
  const text = (p, n) => new TextDecoder().decode(bytes(p, n));
  const okName = (k) => k.length > 0 && !k.includes("/") && !k.includes("..") && !k.startsWith(".");
  function give(u8) {
    const p = inst.exports.qing_alloc(u8.length) >>> 0;
    new Uint8Array(mem.buffer, p, u8.length).set(u8);
    return (BigInt(p) << 32n) | BigInt(u8.length);
  }
  function tryLS(f, fallback) { try { return f(); } catch (_) { return fallback; } }
  const qing = {
    frame(px, pal) {
      const m = new Uint8Array(mem.buffer);
      if (px + W * H > m.length || pal + 768 > m.length) return -1;
      const d = img.data;
      for (let i = 0; i < W * H; i++) { const c = pal + m[px + i] * 3; d[i * 4] = m[c]; d[i * 4 + 1] = m[c + 1]; d[i * 4 + 2] = m[c + 2]; d[i * 4 + 3] = 255; }
      ctx.putImageData(img, 0, 0);
      return 0;
    },
    // 按下又松开都在一帧里的键也算这一帧按着：不然点得快的一下会丢。
    buttons() { const b = (held | touch | tapped | padBits()) & 0xfff; tapped = 0; return b; },
    tone: (ch, hz, ms, vol) => (ch <= 3 && synth.tone(ch, hz >>> 0, ms >>> 0, vol >>> 0) ? 0 : -1),
    tone_wav: (p, n) => (synth.wav(bytes(p, n)) ? 0 : -1),
    log: (p, n) => console.log(`[${vg.name}]`, text(p, n)),
    data_read(kp, kn) { const k = text(kp, kn); if (!okName(k)) return 0n; const v = tryLS(() => localStorage.getItem(store(k)), null); return v === null ? 0n : give(b64(v)); },
    data_write(kp, kn, vp, vn) { const k = text(kp, kn); if (!okName(k)) return 1; return tryLS(() => (localStorage.setItem(store(k), toB64(bytes(vp, vn))), 0), 1); },
    data_delete(kp, kn) { const k = text(kp, kn); if (!okName(k)) return 1; tryLS(() => localStorage.removeItem(store(k))); return 0; },
    data_list() {
      const pre = store(""), keys = tryLS(() => Object.keys(localStorage), []).filter((k) => k.startsWith(pre)).map((k) => k.slice(pre.length)).sort();
      return give(new TextEncoder().encode(JSON.stringify(keys)));
    },
    asset_read(p, n) { const k = text(p, n); const v = vg.assets[k]; return v === undefined ? 0n : give(b64(v)); },
    now: () => BigInt(Math.floor(Date.now() / 1000)),
    tz_offset: () => -new Date().getTimezoneOffset() * 60,
  };

  function fail(e) { dead = true; status.textContent = "游戏出错了，看浏览器的控制台。"; console.error(e); if (synth) synth.stop(); }

  async function main() {
    synth = new Synth(48000);
    const module = await WebAssembly.compile(b64(vg.wasm));
    const imports = { qing: {} };
    // 整屏 app 用不到的导入函数：装得上，调了就出错（和 vibegame run 一样）。
    for (const i of WebAssembly.Module.imports(module)) {
      if (i.kind !== "function") continue;
      imports[i.module] = imports[i.module] || {};
      imports[i.module][i.name] = i.module === "qing" && qing[i.name] ? qing[i.name] : () => { throw new Error(`整屏 app 不能用 ${i.module}.${i.name}`); };
    }
    inst = await WebAssembly.instantiate(module, imports);
    mem = inst.exports.memory;
    if (inst.exports.qinit) inst.exports.qinit();
    let last = performance.now(), acc = 0;
    const loop = (t) => {
      if (dead) return;
      acc += document.hidden ? 0 : Math.min(t - last, 100); last = t;
      try {
        while (acc >= 1000 / FPS) { inst.exports.qing_tick(); acc -= 1000 / FPS; }
      } catch (e) { return fail(e); }
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
  }
  main().catch(fail);
})();
