// 网页版拉一个 .vp 再玩（集市的网页用）：地址写 ?pkg=<包的地址>。
// 包是 gzip 的 tar：解开，取 bin/<名字>.wasm 和 assets/ 下的素材，交给 player.js（照 window.VP 的样子）。
(async () => {
  "use strict";
  const status = document.getElementById("status");
  const say = (en, zh) => { status.textContent = (navigator.language || "").startsWith("zh") ? zh : en; };
  const toB64 = (u8) => { let s = ""; for (let i = 0; i < u8.length; i += 0x8000) s += String.fromCharCode.apply(null, u8.subarray(i, i + 0x8000)); return btoa(s); };
  function untar(d) {
    const files = {};
    const text = (a, n) => new TextDecoder().decode(d.subarray(a, a + n)).replace(/\0.*$/s, "");
    let at = 0, long = null;
    while (at + 512 <= d.length && d[at] !== 0) {
      const size = parseInt(text(at + 124, 12).trim() || "0", 8);
      const type = String.fromCharCode(d[at + 156] || 48);
      const prefix = text(at + 345, 155);
      const name = long || (prefix ? prefix + "/" + text(at, 100) : text(at, 100));
      long = null;
      const body = d.subarray(at + 512, at + 512 + size);
      if (type === "L") long = new TextDecoder().decode(body).replace(/\0.*$/s, "");
      else if (type === "0" || type === "\0") files[name.replace(/^\.\//, "")] = body;
      at += 512 + Math.ceil(size / 512) * 512;
    }
    return files;
  }
  try {
    const pkg = new URLSearchParams(location.search).get("pkg");
    if (!pkg) throw new Error("no ?pkg=");
    say("Loading the game…", "在拿游戏……");
    const r = await fetch(pkg);
    if (!r.ok) throw new Error("HTTP " + r.status);
    const tar = new Uint8Array(await new Response(r.body.pipeThrough(new DecompressionStream("gzip"))).arrayBuffer());
    const files = untar(tar);
    const manifest = new TextDecoder().decode(files["app.toml"] || new Uint8Array());
    const field = (k) => (manifest.match(new RegExp("^" + k + "\\s*=\\s*\"([^\"]*)\"", "m")) || [])[1] || "";
    const name = field("name");
    const wasm = files["bin/" + name + ".wasm"];
    if (!wasm) throw new Error("no bin/" + name + ".wasm");
    if (field("frame")) throw new Error("the web version only plays 240×160 games");
    const assets = {};
    for (const [p, b] of Object.entries(files)) if (p.startsWith("assets/") && !p.slice(7).includes("/")) assets[p.slice(7)] = toB64(b);
    const en = field("title_en") || field("title") || name;
    window.VP = { name, title: { en, zh: field("title_zh") || en }, wasm: toB64(wasm), assets };
    document.title = en;
    document.querySelector("h1").textContent = en;
    status.textContent = "";
    const s = document.createElement("script");
    s.textContent = document.getElementById("vp-player").textContent;
    document.body.append(s);
  } catch (e) {
    console.error(e);
    say("Could not load the game: " + e.message, "游戏拿不到：" + e.message);
  }
})();
