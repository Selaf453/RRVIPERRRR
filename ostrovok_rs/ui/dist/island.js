"use strict";
if (!IN_TAURI) document.body.classList.add("dev");
const isl = $("isl");
let cfg = {}, open = false, toasting = false, speaking = false, sleeping = false, queue = [], toastT = 0, sayT = 0, typeT = 0;
let music = {}, game = null, study = null, focusUntil = 0, lyricLines = [], mpos = { pos: 0, at: 0, playing: false }, weatherW = null, curMood = "neutral";
let lastAct = Date.now(), pokes = [], danceT = 0, danceOn = false, stats = { cpu: 0, ram: 0 };
const eyesAll = ["eIdle", "eMini", "eToast", "eBig", "eSay"].map($);
const pad = n => String(n).padStart(2, "0");
const quiet = () => open || toasting || speaking || sleeping;

/* ── настроения, эффекты, движения ── */
function setMood(m, mv) {
  curMood = m || "neutral";
  eyesAll.forEach(e => { e.className = e.className.split(" ").filter(c => !/^(m|mv)-/.test(c)).join(" "); if (curMood !== "neutral") e.classList.add("m-" + curMood); if (mv) e.classList.add("mv-" + mv); });
}
const mood = setMood;
function look(x, y) { eyesAll.forEach(e => e.querySelectorAll("i").forEach(i => { i.style.setProperty("--lx", x); i.style.setProperty("--ly", y); })); }
function fx(chars, n = 5) {
  if (!chars) return; const box = $("fx"), base = isl.offsetHeight + 14, list = [...chars];
  for (let k = 0; k < n; k++) {
    const b = document.createElement("b"); b.textContent = list[k % list.length]; b.style.left = (Math.random() * 220 - 110) + "px"; b.style.top = base + "px"; box.appendChild(b);
    const dx = (Math.random() - .5) * 90, dy = 30 + Math.random() * 60, d = 1100 + Math.random() * 900;
    b.animate([{ transform: "translate(0,0) scale(.4)", opacity: 0 }, { opacity: 1, offset: .2 }, { transform: `translate(${dx}px,${dy}px) scale(1.1)`, opacity: 0 }], { duration: d, delay: k * 90, easing: "ease-out", fill: "both" }).onfinish = () => b.remove();
  }
}
function anim(cls, ms = 1100) { ISL_ANIMS.forEach(a => isl.classList.remove(a)); void isl.offsetWidth; isl.classList.add(cls); setTimeout(() => isl.classList.remove(cls), ms); }
function react(m, a, fxn) { setMood(m); if (a) anim(a); if (fxn !== 0) fx(MOOD_FX[m], fxn || 4); }
window.addEventListener("pointermove", e => look(Math.max(-1, Math.min(1, (e.clientX / innerWidth - .5) * 4)), Math.max(-1, Math.min(1, e.clientY / 200 - .3))), { passive: true });
T.listen("cur", c => { const r = isl.getBoundingClientRect(); look(Math.max(-1, Math.min(1, (c.x - (r.left + r.width / 2)) / 300)), Math.max(-1, Math.min(1, (c.y - (r.top + r.height / 2)) / 200))); });

/* ── зоны кликов для Rust ── */
function sendRects() {
  const r = [isl, $("news")].filter(e => e && (e.id !== "news" || e.classList.contains("in"))).map(e => { const b = e.getBoundingClientRect(); return [b.left - 6, b.top - 6, b.width + 12, b.height + 12]; });
  T.invoke("set_rects", { rects: r });
}
setInterval(sendRects, 120);

/* ── вид ── */
function applyStyle() { isl.className = isl.className.replace(/\bs-\w+/g, "").trim(); isl.classList.add("s-" + (cfg.island_style || "pill")); }
function chipText() {
  if (game) return "🎮 " + game.name + " · " + Math.round(stats.cpu) + "% / " + Math.round(stats.ram) + "%";
  if (study) { const l = Math.max(0, Math.round((study.end - Date.now()) / 1000)); return "📚 " + pad(Math.floor(l / 60)) + ":" + pad(l % 60); }
  return weatherW && weatherW.temp !== undefined ? Math.round(weatherW.temp) + "°" : "";
}
function setState() {
  isl.classList.toggle("open", open);
  isl.classList.toggle("toast", toasting && !open);
  isl.classList.toggle("speak", speaking && !open && !toasting);
  isl.classList.toggle("zzz", sleeping && !open && !toasting && !speaking);
  isl.classList.toggle("music", !!(music.playing) && !open && !toasting && !speaking && !sleeping);
  const w = $("weather"); w.textContent = chipText(); w.className = (game ? "chip game" : study ? "chip study" : "");
  isl.classList.toggle("wide", !!(game || study));
  updateDance();
}

/* ── реплики ── */
function say(text, m, secs, opts) {
  opts = opts || {};
  if (!text || (!opts.force && quiet())) return;
  if (open) return; clearTimeout(sayT); clearInterval(typeT); speaking = true; sleeping = false;
  setMood(m || "happy"); if (opts.anim) anim(opts.anim); if (opts.fx !== false) fx(MOOD_FX[m || "happy"], 3);
  const el = $("sayT"); el.textContent = ""; el.classList.add("cur"); let i = 0;
  typeT = setInterval(() => { el.textContent = text.slice(0, ++i); if (i >= text.length) { clearInterval(typeT); el.classList.remove("cur"); } }, 32);
  setState();
  sayT = setTimeout(() => { speaking = false; setMood("neutral"); setState(); setTimeout(() => { if (!quiet() && queue.length) toast(queue.shift()); }, 400); }, (secs || 3.5) * 1000 + text.length * 40);
  if (cfg.speak && IN_TAURI) T.invoke("say_text", { text });
}
const chatty = () => cfg.chatty === undefined ? 2 : cfg.chatty;
function sayCtx(key, m, secs, opts) { if (chatty() > 0 || (opts && opts.force)) say(pickPh(key), m, secs, opts); }

/* ── уведомления ── */
function toast(d) {
  if (open || toasting) { if (queue.length < 8) queue.push(d); return; }
  clearTimeout(sayT); clearInterval(typeT); speaking = false; sleeping = false;
  toasting = true;
  $("tIcon").textContent = d.icon || "🔔"; $("tTitle").textContent = d.title || ""; $("tSub").textContent = d.sub || "";
  isl.style.setProperty("--tc", d.color || "#a78bfa"); setMood(d.mood);
  const ms = d.ms || 4200, pr = $("tProg"); pr.style.transition = "none"; pr.style.transform = "scaleX(1)"; void pr.offsetWidth; pr.style.transition = `transform ${ms}ms linear`; pr.style.transform = "scaleX(0)";
  setState();
  if (d.snd) { Snd.volume(d.vol ?? cfg.sound_vol ?? 60); Snd.enable(d.sounds !== false && cfg.sounds !== false); Snd.play(d.snd); }
  if (d.shake) { isl.classList.add("shake"); setTimeout(() => isl.classList.remove("shake"), 800); } else if (d.mood && d.mood !== "neutral") anim(rnd(["a-pulse", "a-jelly", "a-zoom"]), 700);
  fx(MOOD_FX[d.mood] || "", 3);
  clearTimeout(toastT); toastT = setTimeout(endToast, ms);
}
function endToast() { toasting = false; setMood("neutral"); setState(); setTimeout(() => { if (!open && !toasting && queue.length) toast(queue.shift()); }, 450); }
T.listen("toast", t => { wakeUp(true); toast(t); });

/* ── раскрытие ── */
async function openIsl() {
  if (open) return; wakeUp(true); open = true; clearTimeout(toastT); clearTimeout(sayT); clearInterval(typeT); toasting = false; speaking = false; setState(); Snd.play("pop"); setMood("happy"); fx("✨", 4);
  const h = new Date().getHours(); $("pHi").textContent = (cfg.user_name ? cfg.user_name + ", " : "") + (h < 5 ? "доброй ночи" : h < 12 ? "доброе утро" : h < 18 ? "добрый день" : "добрый вечер");
  $("pDate").textContent = new Date().toLocaleDateString("ru-RU", { weekday: "long", day: "numeric", month: "long" });
  refreshHw(); refreshExam(); paintFocus(); paintStudy(); if (window.panelOpen) panelOpen();
}
function closeIsl() { if (!open) return; open = false; setState(); setMood("neutral"); setTimeout(() => { if (!toasting && queue.length) toast(queue.shift()); }, 500); }
isl.addEventListener("click", e => { if (!open && !e.target.closest("button")) openIsl(); });
isl.addEventListener("contextmenu", e => { e.preventDefault(); if (!open) poke(); });
$("bClose").addEventListener("click", e => { e.stopPropagation(); closeIsl(); });
$("bSet").addEventListener("click", e => { e.stopPropagation(); T.invoke("open_settings"); closeIsl(); });
$("bDiary").addEventListener("click", e => { e.stopPropagation(); T.invoke("open_diary"); closeIsl(); });
$("bGame").addEventListener("click", e => { e.stopPropagation(); T.invoke("launch_gaming"); closeIsl(); });
addEventListener("keydown", e => { if (e.key === "Escape") closeIsl(); });
isl.addEventListener("pointerleave", () => { if (open) { clearTimeout(isl._lv); isl._lv = setTimeout(() => { if (!isl.matches(":hover") && !document.activeElement.matches?.("input")) closeIsl(); }, 1800); } });

/* ── тыкаем в остров ── */
function poke() {
  const now = Date.now(); pokes = pokes.filter(t => now - t < 6000); pokes.push(now); lastAct = now; wakeUp(false);
  if (pokes.length >= 5) { say(pickPh("angry"), "angry", 3, { force: true, anim: "a-wiggle" }); Snd.play("alert"); }
  else if (pokes.length >= 3) { say(pickPh("poke"), rnd(["dizzy", "shock", "wink", "shy"]), 2.6, { force: true, anim: rnd(["a-jelly", "a-hop", "a-flip"]) }); }
  else say(pickPh("poke"), rnd(["happy", "wink", "love", "joy", "cat", "sparkle"]), 2.6, { force: true, anim: rnd(["a-jelly", "a-nod", "a-pulse", "a-wiggle"]) });
  Snd.play("tick");
}

/* ── сон и скука ── */
function wakeUp(silent) { if (!sleeping) return; sleeping = false; setMood("neutral"); setState(); if (!silent) say(pickPh("wake"), "joy", 3, { force: true, anim: "a-jelly" }); }
let boredSaid = false;
T.listen("idle", s => {
  const sleepAfter = (cfg.sleep_after_min || 5) * 60;
  if (s < 5) { lastAct = Date.now(); boredSaid = false; if (sleeping) wakeUp(false); return; }
  if (!sleeping && !quiet() && !music.playing && !game && s >= sleepAfter) { say(pickPh("sleep"), "sleepy", 2.5, { force: true, fx: false }); setTimeout(() => { if (!open && !toasting) { sleeping = true; setMood("sleepy"); setState(); } }, 3500); }
  else if (!boredSaid && s >= 120 && !quiet() && !music.playing && chatty() > 0) { boredSaid = true; say(pickPh("bored"), rnd(["sad", "think", "sleepy", "cat"]), 3.5, { anim: "a-nod" }); }
});

/* ── танцы ── */
function updateDance() {
  const should = !!(music.playing && cfg.dance !== false && !open && !toasting && !speaking && !sleeping);
  if (should && !danceOn) { danceOn = true; newDance(); clearInterval(danceT); danceT = setInterval(newDance, 7000 + Math.random() * 3000); }
  if (!should && danceOn) { danceOn = false; clearInterval(danceT); clearDanceCls(); if (!sleeping) setMood("neutral"); }
}
function clearDanceCls() { [...isl.classList].filter(c => /^d-/.test(c)).forEach(c => isl.classList.remove(c)); eyesAll.forEach(e => e.className = e.className.split(" ").filter(c => !/^mv-/.test(c)).join(" ")); }
function newDance() {
  if (!danceOn) return; clearDanceCls(); const [d, mv] = rnd(DANCES), m = rnd(DANCE_MOODS);
  isl.style.setProperty("--beat", rnd([".4s", ".47s", ".5s", ".55s"])); isl.classList.add(d); setMood(m, mv); fx(rnd(["♪♫", "♪", "✨♪", MOOD_FX[m] || "♫"]), 4);
  if (Math.random() < .3 && chatty() > 0) setTimeout(() => { if (danceOn && !quiet()) { /* реплика в танце */ clearDanceCls(); danceOn = false; say(pickPh("dance"), m, 3, { force: true, fx: false }); setTimeout(() => { updateDance(); }, 4500); } }, 2500);
}

/* ── домашка, отсчёт ── */
function refreshHw() { if (window.panelHw) panelHw(); }
T.listen("hw", refreshHw);
async function refreshExam() { const e = await T.invoke("exam_info"); $("exam").textContent = ""; if (!e) { $("exam").textContent = "Добавь ЕГЭ в настройках"; return; } $("exam").append(e.text); }

/* ── музыка и тексты ── */
let lastTrack = "";
T.listen("music", m => {
  const was = music.playing; music = m || {}; $("mTitle").textContent = m.title || "Ничего не играет"; $("mArt").textContent = m.title ? (m.artist || "") + (m.playing ? "" : " · пауза") : "Включи трек, я услышу";
  renderMini(); setState();
  const key = (m.title || "") + "|" + (m.artist || "");
  if (m.playing && key !== lastTrack && m.title) { lastTrack = key; wakeUp(true); if (Math.random() < .5) setTimeout(() => sayCtx("music", rnd(["joy", "love", "excited", "star"]), 3, { anim: "a-hop" }), 600); }
  if (m.playing && !was) fx("♪♫", 5);
});
function renderMini() {
  const t = $("miniT"); t.textContent = ""; if (!music.title) return; t.append(music.title);
  const ly = cfg.show_lyrics !== false && currentLyric(); const s = el("span", ly ? "lyric" : "", ly ? "  ·  " + ly : (music.artist ? "  ·  " + music.artist : "")); t.appendChild(s);
}
T.listen("lyrics", l => { lyricLines = (l.lines || []).map(x => [x[0], x[1]]); renderMini(); });
T.listen("mpos", p => { mpos = { pos: p.pos, at: Date.now(), playing: true }; });
function currentLyric() {
  if (!lyricLines.length || !music.playing) return ""; const pos = mpos.pos + (Date.now() - mpos.at) / 1000 + (cfg.lyrics_offset || 0); let cur = "";
  for (const [t, s] of lyricLines) { if (t <= pos) cur = s; else break; } return cur;
}
setInterval(() => { if (lyricLines.length && music.playing && isl.classList.contains("music")) renderMini(); }, 500);

T.listen("stats", s => {
  stats = s; $("bCpu").style.width = s.cpu + "%"; $("vCpu").textContent = Math.round(s.cpu) + "%"; $("bRam").style.width = s.ram + "%"; $("vRam").textContent = Math.round(s.ram) + "%"; if (game) setState();
});
T.listen("weather", w => {
  weatherW = w; setState(); const c = w.code, t = w.temp; if (Date.now() - (weatherW._said || 0) < 36e5 * 3) return;
  const key = [51, 53, 55, 61, 63, 65, 80, 81, 82, 95, 96, 99].includes(c) ? "rain" : [71, 73, 75, 77, 85, 86].includes(c) ? "snow" : t >= 28 ? "hot" : t <= -10 ? "cold" : "";
  if (key) { weatherW._said = Date.now(); setTimeout(() => sayCtx(key, key === "rain" ? "sad" : key === "snow" ? "wow" : key === "hot" ? "sick" : "shy", 4), 4000); }
});
T.listen("cfg", c => { cfg = c; applyCfgSounds(c); applyStyle(); setState(); });

T.listen("preview", m => { wakeUp(true); if (open) closeIsl(); const mv = rnd([null, "bob", "sway", "wave", "jelly", "hop"]); setMood(m, mv); fx(MOOD_FX[m], 5); anim(rnd(ISL_ANIMS)); clearTimeout(window.__pv); window.__pv = setTimeout(() => setMood("neutral"), 3500); });

/* ── игры ── */
T.listen("game", g => {
  if (g.on) {
    game = { name: g.name }; wakeUp(true);
    if (g.nag) { setTimeout(() => say(pickPh("nag"), "angry", 4, { force: true, anim: "a-wiggle" }), 500); Snd.play("alert"); }
    else if (chatty() > 0 && Array.isArray(g.phrases) && g.phrases.length) setTimeout(() => say(rnd(g.phrases), rnd(["cool", "glasses", "excited", "focus"]), 4, { force: true, anim: "a-pulse" }), 500);
  } else { game = null; setTimeout(() => sayCtx("gameEnd", rnd(["happy", "tired", "proud", "wink"]), 4, { anim: "a-hop" }), 800); }
  setState();
});

/* ── учёба ── */
T.listen("study", s => {
  if (s.on) { study = { end: Date.now() + s.left * 1000 }; setTimeout(() => sayCtx("study", "focus", 3.5), 3500); } else { study = null; if (s.natural) { anim("a-jelly"); fx("🎓🎉✨", 6); } }
  setState(); paintStudy();
});
setInterval(() => { if (study) setState(); }, 1000);
setInterval(() => { if (study && !quiet() && chatty() > 0 && Math.random() < .25) say(pickPh("study"), "focus", 3.5); }, 5 * 60 * 1000);
function paintStudy() { $("bStudyT").textContent = study ? "Стоп учёбу" : "Учёба"; $("bStudy").classList.toggle("on", !!study); }
$("bStudy").addEventListener("click", e => { e.stopPropagation(); T.invoke(study ? "study_stop" : "study_start"); });

/* ── фокус и заметки ── */
function paintFocus() {
  const on = focusUntil > Date.now(); $("bFocusX").style.display = on ? "" : "none";
  $("bFocusT").textContent = on ? "Фокус: " + Math.ceil((focusUntil - Date.now()) / 60000) + " мин" : "Фокус " + (cfg.focus_min || 45) + " мин"; $("bFocus").classList.toggle("on", on);
}
setInterval(paintFocus, 5000);
$("bFocus").addEventListener("click", async e => { e.stopPropagation(); if (focusUntil > Date.now()) return; await T.invoke("focus_start", { minutes: cfg.focus_min || 45 }); });
$("bFocusX").addEventListener("click", e => { e.stopPropagation(); T.invoke("focus_cancel"); });
T.listen("focus", f => {
  if (f.on) { focusUntil = f.until; paintFocus(); toast({ icon: "🎯", color: "#a78bfa", title: "Фокус включён", sub: "Мелкие уведомления подожду", mood: "focus", snd: "pop", ms: 3000 }); }
  else {
    focusUntil = 0; paintFocus(); const n = (f.missed || []).length;
    if (f.done) toast({ icon: "✅", color: "#4ade80", title: "Фокус закончился", sub: n ? "Пока тебя не трогали: " + f.missed.slice(0, 3).join(", ") + (n > 3 ? " и ещё " + (n - 3) : "") : "Пока ты работал, ничего важного не было", mood: "proud", snd: "ding", ms: 6500 });
  }
});
T.listen("hotkey", async n => {
  if (n === "expand") { open ? closeIsl() : openIsl(); }
  if (n === "note") { await openIsl(); panelGo(3); setTimeout(() => $("noteIn").focus(), 600); }
  if (n === "focus") { if (focusUntil > Date.now()) T.invoke("focus_cancel"); else T.invoke("focus_start", { minutes: cfg.focus_min || 45 }); }
  if (n === "study") T.invoke(study ? "study_stop" : "study_start");
});

/* ── случайные реплики ── */
function chatterLoop() {
  const f = chatty(); const base = f === 0 ? 0 : f === 1 ? 14 : f === 2 ? 5 : 2.5;
  if (!base) { setTimeout(chatterLoop, 60000); return; }
  setTimeout(() => {
    if (!quiet() && !game) {
      const h = new Date().getHours(), pools = ["chatter", "chatter", "chatter"];
      if (h >= 23 || h < 5) pools.push("night", "night"); if (h >= 6 && h < 10) pools.push("morning"); if (music.playing) pools.push("music"); if (study) pools.push("study", "study");
      say(pickPh(rnd(pools)), rnd(["happy", "wink", "think", "cat", "sparkle", "zen", "proud", "cool", "joy"]), 3.5, { anim: Math.random() < .4 ? rnd(ISL_ANIMS) : null });
    }
    chatterLoop();
  }, (base * 60 + Math.random() * base * 60) * 1000);
}
setTimeout(chatterLoop, 3 * 60 * 1000);

/* ── часы ── */
setInterval(() => { const n = new Date(); $("clock").textContent = pad(n.getHours()) + ":" + pad(n.getMinutes()); }, 1000);
{ const n = new Date(); $("clock").textContent = pad(n.getHours()) + ":" + pad(n.getMinutes()); }

/* ── «Что нового» и золотой распад ── */
const NEWS = [
  ["😎", "33 настроения", "Влюблён, в шоке, хитрый, в очках, робот, кот и другие"],
  ["💃", "Новые танцы", "Под музыку он меняет танец и настроение каждые несколько секунд"],
  ["💬", "Сотни реплик", "Шутки по времени суток, погоде, играм и учёбе. ПКМ по острову: потыкать"],
  ["😴", "Сон и обиды", "Засыпает без дела, просыпается рад, злится, если тыкать часто"],
  ["📔", "Дневник, учёба, игры", "Всё вернулось: дневник с паролем, режим учёбы, игровой режим"],
  ["🎤", "Тексты песен и голос", "Строки на острове в такт трека и озвучка уведомлений"],
];
const VER = "3.1";
function showNews() {
  const n = $("news"); n.textContent = "";
  const h = el("div", "nh"); h.append(el("span", "nb", "NEW"), el("h2", "", "Островок " + VER)); n.append(h, el("div", "ns", "Большое обновление"));
  NEWS.forEach(([e, t, s]) => { const r = el("div", "ni"), x = el("div"); x.append(el("b", "", t), el("span", "", s)); r.append(el("div", "e", e), x); n.appendChild(r); });
  const b = el("button", "", "Хорошо, понятно"); n.appendChild(b); requestAnimationFrame(() => requestAnimationFrame(() => n.classList.add("in")));
  b.addEventListener("click", () => dissolve(n));
}
function dissolve(n) {
  const r = n.getBoundingClientRect(), tr = isl.getBoundingClientRect(), tx = tr.left + tr.width / 2, ty = tr.top + tr.height / 2;
  const S = 34, cols = Math.ceil(r.width / S), rows = Math.ceil(r.height / S), tiles = [];
  Snd.play("whoosh");
  for (let y = 0; y < rows; y++) for (let x = 0; x < cols; x++) {
    const t = el("div", "tile"), inner = n.cloneNode(true); inner.removeAttribute("id"); inner.className = ""; inner.style.cssText = `position:absolute;left:${-x * S}px;top:${-y * S}px;width:${r.width}px;transform:none;opacity:1;right:auto;background:#12121a;border:0;border-radius:0;box-shadow:none`;
    t.style.cssText = `left:${r.left + x * S}px;top:${r.top + y * S}px;width:${S}px;height:${S}px`; const w = el("div"); w.style.cssText = "inset:0"; w.appendChild(inner); const gl = el("div", "gl"); t.append(w, gl); t._gl = gl; document.body.appendChild(t); tiles.push([t, x, y]);
  }
  n.style.visibility = "hidden"; n.classList.remove("in");
  let done = 0;
  tiles.forEach(([t, x, y]) => {
    const dx = tx - (r.left + x * S + S / 2), dy = ty - (r.top + y * S + S / 2), delay = (x / cols) * 450 + (1 - y / rows) * 250 + Math.random() * 200, dur = 900 + Math.random() * 500;
    const mx = dx * .5 + (Math.random() - .5) * 320, my = dy * .5 + (Math.random() - .8) * 260;
    const a = t.animate([{ transform: "translate(0,0) scale(1)", opacity: 1 }, { transform: `translate(${mx}px,${my}px) scale(.8)`, opacity: 1, offset: .5 }, { transform: `translate(${dx}px,${dy}px) scale(.15)`, opacity: .2 }], { delay, duration: dur, easing: "cubic-bezier(.5,0,.6,1)", fill: "both" });
    t._gl.animate([{ opacity: 0 }, { opacity: .15, offset: .25 }, { opacity: .95, offset: .55 }, { opacity: 1 }], { delay, duration: dur, fill: "both" });
    a.onfinish = () => { t.remove(); if (++done === tiles.length) arrive(); };
  });
  setTimeout(() => Snd.play("sparkle"), 1100);
}
function arrive() {
  isl.classList.add("gold", "shake"); setMood("joy"); fx("✨⭐🎉", 10); Snd.play("sparkle");
  setTimeout(() => { isl.classList.remove("gold", "shake"); $("news").remove(); toast({ icon: "✨", color: "#ffd34d", title: "Ням! Теперь я " + VER, sub: "Спасибо за обновку", mood: "party", snd: null, ms: 3500 }); }, 1400);
  T.invoke("set_cfg", { patch: { seen_version: VER } });
}

/* ── старт ── */
(async () => {
  cfg = await T.invoke("get_cfg") || {}; applyCfgSounds(cfg); applyStyle(); setState(); sendRects();
  const ver = await T.invoke("app_ver"); const st = await T.invoke("study_state"); if (st && st.on) study = { end: Date.now() + st.left * 1000 };
  if (cfg.seen_version !== ver) setTimeout(showNews, 900);
  else {
    const hw = (await T.invoke("hw_view") || []).filter(h => h.today && !h.done);
    setTimeout(async () => {
      const ex = await T.invoke("exam_info");
      if (hw.length && cfg.hw_show_on_start !== false) toast({ icon: "📚", color: "#38bdf8", title: "Здравствуй! Есть задание на сегодня", sub: (hw[0].subject || "Задание") + ": " + hw[0].body + (hw.length > 1 ? " и ещё " + (hw.length - 1) : ""), mood: "wow", snd: "hw", ms: 6000 });
      else if (ex) toast({ icon: "🎓", color: "#ffd34d", title: ex.text, sub: "Время работает на тебя", mood: "proud", snd: "pop", ms: 4500 });
      else say(pickPh(new Date().getHours() < 10 ? "morning" : "wake"), "joy", 3, { force: true, anim: "a-jelly" });
    }, 900);
  }
})();
window.__demo = { toast, openIsl, closeIsl, showNews, say, setMood, fx, anim, poke, setMusic: m => { music = m; renderMini(); setState(); }, setGame: g => { game = g; setState(); }, sleep: () => { sleeping = true; setMood("sleepy"); setState(); } };
