"use strict";
/* Панель острова: точки-вкладки, страницы, анимации */
const IC = {
  home: '<path d="M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"/>',
  music: '<path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>',
  check: '<path d="m9 11 3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>',
  note: '<path d="M15 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M15 3v5h5"/>',
  cal: '<rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/>',
  clip: '<rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>',
  cpu: '<rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"/>',
  gear: '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M2 12h3M19 12h3M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1"/>',
  up: '<path d="m18 15-6-6-6 6"/>',
  prev: '<path d="M19 20 9 12l10-8z"/><path d="M5 19V5"/>', next: '<path d="m5 4 10 8-10 8z"/><path d="M19 5v14"/>',
  play: '<path d="m7 3 13 9-13 9z"/>', pause: '<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>',
  target: '<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>',
  cap: '<path d="M22 10 12 5 2 10l10 5z"/><path d="M6 12v5c3 2 9 2 12 0v-5"/>',
  pad: '<path d="M6 12h4M8 10v4"/><path d="M15 13h.01M18 11h.01"/><rect x="2" y="6" width="20" height="12" rx="6"/>',
  book: '<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V3H6.5A2.5 2.5 0 0 0 4 5.5z"/><path d="M4 19.5V22h16"/>',
  trash: '<path d="M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14"/>', x: '<path d="M18 6 6 18M6 6l12 12"/>',
  pin: '<path d="M12 17v5"/><path d="M9 3h6l-1 7 3 3H7l3-3z"/>', bell: '<path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.7 21a2 2 0 0 1-3.4 0"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>',
  mouse: '<rect x="6" y="2" width="12" height="20" rx="6"/><path d="M12 2v7"/>', head: '<path d="M4 14v-2a8 8 0 0 1 16 0v2"/><rect x="2" y="14" width="5" height="7" rx="2"/><rect x="17" y="14" width="5" height="7" rx="2"/>',
  tick: '<path d="m5 12 5 5L20 7"/>',
};
const svg = n => '<svg class="ic" viewBox="0 0 24 24">' + IC[n] + '</svg>';
const paintIcons = root => (root || document).querySelectorAll("[data-ic]").forEach(e => { e.innerHTML = svg(e.dataset.ic); });
paintIcons();

/* ── страницы ── */
const pages = [...document.querySelectorAll(".pg")], dotsBox = $("dots");
let cur = 0, titleEl = null;
pages.forEach((p, i) => { [...p.children].forEach((c, j) => c.style.setProperty("--i", j)); const d = el("button", "dot"); d.title = p.dataset.title; d.addEventListener("click", e => { e.stopPropagation(); panelGo(i); }); dotsBox.appendChild(d); });
function setTitle(t, first) {
  const box = $("pTitle"); const old = titleEl;
  if (old) { old.className = "out"; setTimeout(() => old.remove(), 300); }
  titleEl = el("span", first ? "" : "in", t); box.appendChild(titleEl);
}
var panelGo = function (i, silent) {
  i = (i + pages.length) % pages.length; const first = titleEl === null;
  if (i === cur && !first) return;
  pages.forEach((p, k) => { p.classList.toggle("on", k === i); p.classList.toggle("left", k < i); });
  [...dotsBox.children].forEach((d, k) => d.classList.toggle("on", k === i));
  const p = pages[i]; cur = i;
  isl.style.setProperty("--oh", p.dataset.h + "px"); isl.style.setProperty("--tc", p.dataset.tc); $("panel").style.setProperty("--tc", p.dataset.tc);
  setTitle(p.dataset.title, first);
  if (!silent && !first) Snd.play("tick");
  if (i === 2) panelHw(); if (i === 3) renderNotes(); if (i === 4) renderCal(); if (i === 5) renderClip(); if (i === 1) paintMusic();
};
function panelOpen() { panelGo(0, true); $("pClock").textContent = new Date().toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" }); }
document.addEventListener("click", e => { const g = e.target.closest("[data-go]"); if (g) { e.stopPropagation(); panelGo(+g.dataset.go); } });
let wheelT = 0;
$("panel").addEventListener("wheel", e => { if (e.target.closest(".scroll")) return; const n = Date.now(); if (n - wheelT < 450) return; wheelT = n; panelGo(cur + (e.deltaY > 0 || e.deltaX > 0 ? 1 : -1)); }, { passive: true });
window.addEventListener("keydown", e => { if (!open || e.target.tagName === "INPUT") return; if (e.key === "ArrowRight") panelGo(cur + 1); if (e.key === "ArrowLeft") panelGo(cur - 1); if (e.key === "Escape") closeIsl(); });
let sx = null; $("panel").addEventListener("pointerdown", e => { if (!e.target.closest("button,input,.scroll")) sx = e.clientX; });
window.addEventListener("pointerup", e => { if (sx === null) return; const d = e.clientX - sx; sx = null; if (Math.abs(d) > 60) panelGo(cur + (d < 0 ? 1 : -1)); });
setInterval(() => { $("pClock").textContent = new Date().toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" }); }, 15000);

/* ── главная: погода ── */
const WX = c => c === 0 ? "ясно" : c <= 3 ? "облачно" : c <= 48 ? "туман" : c <= 57 ? "морось" : c <= 67 ? "дождь" : c <= 77 ? "снег" : c <= 82 ? "ливень" : c <= 86 ? "снегопад" : "гроза";
function panelWeather(w) { if (!w || w.temp === undefined) return; $("wTemp").textContent = Math.round(w.temp) + "°"; $("wDesc").textContent = WX(w.code) + (cfg.city ? " · " + cfg.city : ""); }
T.listen("weather", panelWeather);

/* ── музыка ── */
const fmt = s => { s = Math.max(0, Math.round(s || 0)); return Math.floor(s / 60) + ":" + String(s % 60).padStart(2, "0"); };
function paintMusic() {
  const m = music || {}, pl = !!m.playing, t = m.title || "Ничего не играет", a = m.title ? (m.artist || "") + (pl ? "" : " · пауза") : "Включи трек, я услышу";
  $("mTitle").textContent = t; $("mArt").textContent = a; $("mTitle2").textContent = t; $("mArt2").textContent = a; $("artM").classList.toggle("pl", pl); $("artH").textContent = pl ? "♫" : "♪";
  $("bPlay").innerHTML = svg(pl ? "pause" : "play"); $("bPlayH").innerHTML = svg(pl ? "pause" : "play"); document.querySelectorAll(".eq.big b").forEach(b => b.style.animationPlayState = pl ? "running" : "paused");
  const pos = mpos.playing ? mpos.pos + (Date.now() - mpos.at) / 1000 : mpos.pos, dur = m.dur || 0;
  $("mProg").style.width = dur ? Math.min(100, pos / dur * 100) + "%" : "0"; $("mPos").textContent = fmt(pos); $("mDur").textContent = fmt(dur);
  const ly = cfg.show_lyrics !== false && typeof currentLyric === "function" && currentLyric(); $("mLyr").textContent = ly || "";
}
T.listen("music", () => setTimeout(paintMusic, 30));
setInterval(() => { if (open && cur === 1) paintMusic(); }, 1000); paintMusic();
const mc = c => e => { e.stopPropagation(); T.invoke("media_cmd", { cmd: c }); };
$("bPlay").addEventListener("click", mc("toggle")); $("bPlayH").addEventListener("click", mc("toggle")); $("bPrev").addEventListener("click", mc("prev")); $("bNext").addEventListener("click", mc("next"));
$("bPlayH").addEventListener("click", e => e.stopPropagation(), true);

/* ── домашка ── */
async function panelHw() {
  const all = (await T.invoke("hw_view") || []).sort((a, b) => (a.done - b.done) || (a.due || "9").localeCompare(b.due || "9"));
  const box = $("hwList"); box.textContent = "";
  if (!all.length) { const e = el("div", "empty"); e.innerHTML = "<big>🎉</big>Ничего не задано, отдыхай"; box.appendChild(e); return; }
  all.forEach((h, i) => {
    const r = el("div", "hwrow" + (h.done ? " dn" : "")); r.style.animationDelay = i * 40 + "ms"; r.style.setProperty("--c", h.color || "#c9ccdb");
    const c = el("button", "chk" + (h.done ? " on" : "")); c.innerHTML = svg("tick");
    c.addEventListener("click", async e => { e.stopPropagation(); c.classList.toggle("on"); r.classList.toggle("dn"); if (!h.done) { Snd.play("done"); fx("✨", 3); } await T.invoke("hw_toggle", { id: h.id }); setTimeout(panelHw, 450); });
    const sub = el("b", "", h.subject || "Задание"); sub.style.color = h.color;
    r.append(c, sub, el("span", "tx", h.body)); if (h.label && !h.done) r.append(el("span", "pill " + (h.label === "просрочено" ? "late" : h.label === "сегодня" ? "today" : ""), h.label));
    const x = el("button", "x"); x.innerHTML = svg("x"); x.addEventListener("click", async e => { e.stopPropagation(); r.style.opacity = 0; r.style.transform = "translateX(30px)"; r.style.transition = ".25s"; await T.invoke("hw_delete", { id: h.id }); setTimeout(panelHw, 260); }); r.append(x);
    box.appendChild(r);
  });
}
async function addHw() { const i = $("hwIn"), t = i.value.trim(); if (!t) return; i.value = ""; await T.invoke("hw_add", { text: t }); Snd.play("done"); setMood("love"); fx("📚", 3); setTimeout(() => setMood("happy"), 1200); panelHw(); }
$("hwAdd").addEventListener("click", e => { e.stopPropagation(); addHw(); }); $("hwIn").addEventListener("keydown", e => { e.stopPropagation(); if (e.key === "Enter") addHw(); });

/* ── заметки ── */
function renderNotes() {
  const box = $("noteList"); box.textContent = ""; const n = cfg.notes || [];
  if (!n.length) { const e = el("div", "empty"); e.innerHTML = "<big>📝</big>Пока пусто. Напиши что-нибудь ниже"; box.appendChild(e); return; }
  [...n].map((x, i) => [x, i]).reverse().forEach(([x, i], k) => {
    const r = el("div", "nrow"); r.style.animationDelay = k * 40 + "ms"; r.append(el("span", "tx", x.t), el("span", "pill", (x.d || "").slice(8) + "." + (x.d || "").slice(5, 7)));
    const d = el("button", "x"); d.innerHTML = svg("trash"); d.addEventListener("click", async e => { e.stopPropagation(); const l = (cfg.notes || []).filter((_, j) => j !== i); cfg.notes = l; await T.invoke("set_cfg", { patch: { notes: l } }); renderNotes(); }); r.append(d); box.appendChild(r);
  });
}
async function addNote() { const i = $("noteIn"), t = i.value.trim(); if (!t) return; i.value = ""; const l = [...(cfg.notes || []), { t, d: new Date().toISOString().slice(0, 10) }]; cfg.notes = l; await T.invoke("set_cfg", { patch: { notes: l } }); Snd.play("done"); setMood("love"); fx("💗", 4); setTimeout(() => setMood("happy"), 1200); renderNotes(); }
$("noteAdd").addEventListener("click", e => { e.stopPropagation(); addNote(); }); $("noteIn").addEventListener("keydown", e => { e.stopPropagation(); if (e.key === "Enter") addNote(); });

/* ── календарь и напоминания ── */
function parseWhen(s) {
  s = (s || "").trim().toLowerCase(); let m;
  if ((m = s.match(/^через\s+(\d+)\s*(мин|м|час|ч)/))) return Date.now() + +m[1] * (m[2].startsWith("ч") ? 3600e3 : 60e3);
  const d = new Date(); let hh = null, mm = 0;
  if ((m = s.match(/(\d{1,2})[:.](\d{2})\s*$/))) { hh = +m[1]; mm = +m[2]; }
  if ((m = s.match(/(\d{1,2})\.(\d{1,2})(?:\.(\d{2,4}))?\s+\d/))) { d.setMonth(+m[2] - 1, +m[1]); if (m[3]) d.setFullYear(+m[3] < 100 ? 2000 + +m[3] : +m[3]); }
  else if (/послезавтра/.test(s)) d.setDate(d.getDate() + 2); else if (/завтра/.test(s)) d.setDate(d.getDate() + 1);
  if (hh === null || hh > 23 || mm > 59) return null; d.setHours(hh, mm, 0, 0);
  if (d.getTime() < Date.now() && !/завтра|\d{1,2}\.\d{1,2}\s/.test(s)) d.setDate(d.getDate() + 1); return d.getTime();
}
const whenLabel = ms => { const d = new Date(ms), t = d.toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" }), n = Math.round((new Date(d).setHours(0, 0, 0, 0) - new Date().setHours(0, 0, 0, 0)) / 864e5); return (n <= 0 ? "Сегодня" : n === 1 ? "Завтра" : d.toLocaleDateString("ru-RU", { weekday: "short", day: "numeric", month: "short" })) + " " + t; };
function renderCal() {
  const box = $("calList"); box.textContent = ""; const ev = [];
  (cfg.reminders || []).forEach((r, i) => ev.push({ at: r.at, t: r.t, i, rem: true }));
  (cfg.exams || []).forEach(x => { const at = new Date(x.date + "T09:00:00").getTime(); if (at > Date.now() - 864e5) ev.push({ at, t: x.name, exam: true }); });
  ev.sort((a, b) => a.at - b.at);
  if (!ev.length) { const e = el("div", "dim", "Ничего не запланировано"); box.appendChild(e); return; }
  ev.slice(0, 5).forEach((x, k) => {
    const r = el("div", "ev"); r.style.animationDelay = k * 45 + "ms"; r.append(el("span", "w", x.exam ? new Date(x.at).toLocaleDateString("ru-RU", { day: "numeric", month: "short" }) : whenLabel(x.at)));
    const b = el("span", "bell"); b.innerHTML = svg(x.exam ? "target" : "bell"); r.append(b, el("span", "tx", x.t));
    if (x.rem) { const d = el("button", "x"); d.style.opacity = 1; d.innerHTML = svg("trash"); d.addEventListener("click", async e => { e.stopPropagation(); const l = (cfg.reminders || []).filter((_, j) => j !== x.i); cfg.reminders = l; await T.invoke("set_cfg", { patch: { reminders: l } }); renderCal(); }); r.append(d); }
    box.appendChild(r);
  });
}
async function addRem() {
  const t = $("remT").value.trim(), at = parseWhen($("remW").value); if (!t) return;
  if (!at) { $("remW").style.borderColor = "#f87171"; setTimeout(() => $("remW").style.borderColor = "", 900); anim("a-wiggle", 600); return; }
  const l = [...(cfg.reminders || []), { t, at }]; cfg.reminders = l; await T.invoke("set_cfg", { patch: { reminders: l } }); $("remT").value = ""; $("remW").value = ""; Snd.play("done"); fx("🔔", 3); renderCal();
}
$("remAdd").addEventListener("click", e => { e.stopPropagation(); addRem(); });
["remT", "remW"].forEach(id => $(id).addEventListener("keydown", e => { e.stopPropagation(); if (e.key === "Enter") addRem(); }));
setInterval(async () => {
  const l = cfg.reminders || [], due = l.filter(r => r.at <= Date.now()); if (!due.length) return;
  cfg.reminders = l.filter(r => r.at > Date.now()); await T.invoke("set_cfg", { patch: { reminders: cfg.reminders } });
  due.forEach((r, i) => setTimeout(() => toast({ icon: "🔔", color: "#fb923c", title: r.t, sub: "Напоминание", mood: "excited", snd: "ding", ms: 7000 }), i * 800)); if (open && cur === 4) renderCal();
}, 15000);

/* ── буфер обмена ── */
let clipHist = [], clipTab = "hist", lastClip = null; const clipShelf = () => cfg.shelf || [];
async function pollClip() {
  if (cfg.clip_on === false) return; const t = await T.invoke("clip_get"); if (!t || t === lastClip) return; lastClip = t;
  clipHist = [{ t, at: Date.now() }, ...clipHist.filter(x => x.t !== t)].slice(0, 30); if (open && cur === 5 && clipTab === "hist") renderClip();
}
setInterval(pollClip, 1500);
function renderClip() {
  const box = $("clList"); box.textContent = ""; const items = clipTab === "hist" ? clipHist : clipShelf().map(t => ({ t }));
  if (!items.length) { const e = el("div", "empty"); e.innerHTML = clipTab === "hist" ? "<big>📋</big>Скопируй что-нибудь, и оно появится здесь" : "<big>📌</big>Закрепи нужное звёздочкой, оно останется навсегда"; box.appendChild(e); return; }
  items.forEach((x, k) => {
    const r = el("div", "crow"); r.style.animationDelay = Math.min(k, 8) * 35 + "ms"; r.append(el("span", "tx", x.t.replace(/\s+/g, " ")));
    if (x.at) r.append(el("span", "tm", new Date(x.at).toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" })));
    const pinned = clipShelf().includes(x.t), p = el("button", "pn" + (pinned ? " on" : "")); p.innerHTML = svg("pin");
    p.addEventListener("click", async e => { e.stopPropagation(); const s = clipShelf().includes(x.t) ? clipShelf().filter(v => v !== x.t) : [x.t, ...clipShelf()].slice(0, 20); cfg.shelf = s; await T.invoke("set_cfg", { patch: { shelf: s } }); renderClip(); }); r.append(p);
    r.addEventListener("click", async () => { lastClip = x.t; await T.invoke("clip_set", { text: x.t }); r.classList.add("cp"); Snd.play("done"); setTimeout(() => r.classList.remove("cp"), 700); });
    box.appendChild(r);
  });
}
document.querySelectorAll("[data-cl]").forEach(b => b.addEventListener("click", e => { e.stopPropagation(); clipTab = b.dataset.cl; document.querySelectorAll("[data-cl]").forEach(x => x.classList.toggle("on", x === b)); renderClip(); }));
$("clClear").addEventListener("click", async e => { e.stopPropagation(); if (clipTab === "hist") clipHist = []; else { cfg.shelf = []; await T.invoke("set_cfg", { patch: { shelf: [] } }); } renderClip(); });
if (!IN_TAURI) { clipHist = [{ t: "https://github.com/", at: Date.now() - 4e5 }, { t: "Встречаемся в 19:00 у входа в парк", at: Date.now() - 9e5 }, { t: "cargo build --release -p ostrovok", at: Date.now() - 18e5 }]; }

/* ── таймер ── */
const CIRC = 414.7; let tm = { total: 600, left: 600, end: 0, run: false };
function paintTimer() {
  const left = tm.run ? Math.max(0, Math.ceil((tm.end - Date.now()) / 1000)) : tm.left;
  $("tmBig").textContent = fmt(left); $("tmFg").style.strokeDashoffset = CIRC * (1 - (tm.total ? left / tm.total : 0)); $("tmRing").classList.toggle("run", tm.run); $("tmGo").textContent = tm.run ? "Пауза" : (tm.left < tm.total ? "Дальше" : "Старт");
  if (tm.run && left <= 0) { tm.run = false; tm.left = tm.total; paintTimer(); toast({ icon: "⏰", color: "#22d3ee", title: "Таймер сработал", sub: Math.round(tm.total / 60) + " мин прошло", mood: "excited", snd: "ding", ms: 7000 }); fx("⏰✨", 6); }
}
function setTimerMin(m) { tm = { total: m * 60, left: m * 60, end: 0, run: false }; document.querySelectorAll(".presets .btn").forEach(b => b.classList.toggle("on", +b.dataset.m === m)); paintTimer(); }
document.querySelectorAll(".presets .btn").forEach(b => b.addEventListener("click", e => { e.stopPropagation(); setTimerMin(+b.dataset.m); Snd.play("tick"); }));
$("tmGo").addEventListener("click", e => { e.stopPropagation(); const v = parseInt($("tmIn").value); if (v > 0 && !tm.run && tm.left === tm.total) { setTimerMin(Math.min(v, 600)); $("tmIn").value = ""; }
  if (tm.run) { tm.left = Math.max(0, Math.ceil((tm.end - Date.now()) / 1000)); tm.run = false; } else { tm.end = Date.now() + tm.left * 1000; tm.run = true; Snd.play("pop"); } paintTimer(); });
$("tmReset").addEventListener("click", e => { e.stopPropagation(); tm.run = false; tm.left = tm.total; paintTimer(); });
$("tmIn").addEventListener("keydown", e => { e.stopPropagation(); if (e.key === "Enter") $("tmGo").click(); });
setInterval(paintTimer, 500); paintTimer();
/* таймер идёт в фоне: в закрытом острове показываем в чипе */

/* ── остров тянется к курсору ── */
let tx = 1, ty = 1, cx = 1, cy = 1, curX = -999, curY = -999;
function nearCursor(x, y) { curX = x; curY = y; }
window.addEventListener("pointermove", e => nearCursor(e.clientX, e.clientY), { passive: true });
T.listen("cur", c => nearCursor(c.x, c.y));
(function loop() {
  if (!open && !toasting) {
    const r = isl.getBoundingClientRect(), dx = Math.max(r.left - curX, 0, curX - r.right), dy = Math.max(r.top - curY, 0, curY - r.bottom), d = Math.hypot(dx, dy), k = Math.max(0, 1 - d / 130);
    const inside = d === 0 ? 1 : k;
    tx = 1 + .06 * inside * inside * (3 - 2 * inside) * .9 + .0; ty = 1 + .13 * inside;
  } else { tx = 1; ty = 1; }
  cx += (tx - cx) * .14; cy += (ty - cy) * .14;
  isl.style.setProperty("--sx", cx.toFixed(4)); isl.style.setProperty("--sy", cy.toFixed(4));
  requestAnimationFrame(loop);
})();

/* ── пока остров открыт, он живой: настроение под вкладку и иногда меняется ── */
const PAGE_MOODS = [["happy", "joy", "wink", "proud"], ["joy", "love", "star", "cool"], ["focus", "think", "proud", "glasses"], ["love", "shy", "happy", "wink"], ["wink", "think", "excited", "happy"], ["glasses", "cool", "sly", "robot"], ["robot", "focus", "cat", "cool"], ["focus", "excited", "zen", "think"]];
const MOVES = [null, "bob", "sway", "wave", "hop", "jelly"];
function pageMood(first) { if (!open) return; const m = rnd(PAGE_MOODS[cur] || PAGE_MOODS[0]); setMood(m, rnd(MOVES)); if (!first && Math.random() < .5) fx(MOOD_FX[m] || "✨", 3); }
const _go = panelGo; panelGo = function (i, silent) { const before = cur; _go(i, silent); if (cur !== before) pageMood(false); };
setInterval(() => { if (open && Math.random() < .8) pageMood(false); }, 5500);


/* ── заряд мышки (2.4G-донгл ATK/VXE) ── */
let mouseLow = 101;
async function pollMouse() {
  if (cfg.mouse_battery === false) return; const r = await T.invoke("mouse_battery"); const v = $("vMouse"), b = $("bMouse");
  if (!r || !r.ok) { v.className = "v err"; v.textContent = IN_TAURI ? "нет данных" : "—"; b.style.width = "0"; return; }
  v.className = "v"; v.textContent = (r.charging ? "⚡ " : "") + r.percent + "%"; b.style.width = r.percent + "%"; b.style.background = r.percent <= 15 ? "#f87171" : r.percent <= 40 ? "#fbbf24" : "#4ade80";
  if (!r.charging && r.percent <= 15 && r.percent < mouseLow - 4) { mouseLow = r.percent; toast({ icon: "🖱️", color: "#f87171", title: "Мышка разряжается: " + r.percent + "%", sub: "Пора заряжать", mood: "sad", snd: "ding", ms: 6000 }); }
  if (r.charging || r.percent > 25) mouseLow = 101;
}
setTimeout(pollMouse, 6000); setInterval(pollMouse, 90000);
