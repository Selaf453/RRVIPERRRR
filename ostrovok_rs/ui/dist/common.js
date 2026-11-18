"use strict";
/* Мост к Rust. В обычном браузере (для проверки вида) подставляется заглушка с примерами данных. */
const IN_TAURI = !!(window.__TAURI__ && window.__TAURI__.core);
const $ = id => document.getElementById(id);
function el(tag, cls, txt) { const e = document.createElement(tag); if (cls) e.className = cls; if (txt !== undefined) e.textContent = txt; return e; }

const mock = (() => {
  const ymd = n => { const d = new Date(); d.setDate(d.getDate() + n); return d.getFullYear() + "-" + String(d.getMonth() + 1).padStart(2, "0") + "-" + String(d.getDate()).padStart(2, "0"); };
  const cfg = { chatty: 2, sleep_after_min: 5, speak: false, show_lyrics: true, lyrics_offset: 0, study_min: 50, study_playlist: 0, study_nag: true, playlists: [{ name: "Мой плейлист для учёбы", url: "https://music.yandex.ru/" }], gaming_faceit: true, gaming_cs2: true, gaming_music: true, custom_faceit_path: "", island_style: "pill", accent: "#ff8a3d", sounds: true, sound_vol: 60, city: "Челябинск", pc_usb: true, pc_net: true, pc_layout: true, pc_caps: true, pc_break: true, pc_break_min: 45, pc_hog: true, pc_hog_gb: 3, pc_shots: true, hw_hourly: true, hw_show_on_start: true, battery_watch: true, focus_min: 45, autostart: false, dance: true, seen_version: "3.0", tg_token: "", tg_bot_username: "", tg_chat_id: null, tg_code: "OST-K7QM-4XWD",
    exams: [{ name: "ЕГЭ информатика", date: ymd(205) }, { name: "Контрольная по алгебре", date: ymd(3) }], notes: [{ t: "Купить тетрадь", d: ymd(0) }] };
  let hw = [
    { id: 1, subject: "Русский", text: "Русский, стр. 34 упр. 5 на завтра", due: ymd(1), done: false },
    { id: 2, subject: "Алгебра", text: "алгебра №12, 14 на сегодня", due: ymd(0), done: false },
    { id: 3, subject: "Физика", text: "физика §3, вопросы к пятнице", due: ymd(4), done: false }];
  const col = { Русский: "#f87171", Алгебра: "#60a5fa", Физика: "#2dd4bf" };
  const lab = d => { const n = Math.round((new Date(d + "T00:00:00") - new Date(ymd(0) + "T00:00:00")) / 864e5); return n < 0 ? "просрочено" : n === 0 ? "сегодня" : n === 1 ? "завтра" : d.slice(8) + "." + d.slice(5, 7); };
  const body = h => h.text.replace(/^[^,\s]+[,\s]+/, "").replace(/\s*(на|к)\s+(завтра|сегодня|пятнице)\s*$/i, "");
  const view = () => hw.map(h => ({ ...h, body: body(h), label: lab(h.due), color: col[h.subject] || "#c9ccdb", today: h.due <= ymd(0) && !h.done, late: false }));
  const handlers = {
    get_cfg: () => JSON.parse(JSON.stringify(cfg)), app_ver: () => "3.0", hw_view: () => view(),
    set_cfg: a => { Object.assign(cfg, a.patch); }, hw_add: a => { hw.push({ id: Date.now(), subject: "", text: a.text, due: "", done: false }); },
    hw_toggle: a => { const h = hw.find(x => x.id === a.id); if (h) h.done = !h.done; }, hw_delete: a => { hw = hw.filter(x => x.id !== a.id); }, hw_clear_done: () => { hw = hw.filter(x => !x.done); },
    exam_info: () => ({ name: "Контрольная по алгебре", text: "Контрольная по алгебре: 3 дня", days: 3 }),
    diary_state: () => ({ has_pwd: true, open: window.__dopen || false }), diary_auth: a => { if (a.pwd !== "123") return Promise.reject("Неверный пароль"); window.__dopen = true; },
    diary_list: () => ({ items: window.__de, streak: 4 }), diary_get: a => (window.__de.find(x => x.date === a.date) || { mood: "😊", highlight: "", text: "", energy: 2 }), diary_save: () => {}, diary_delete: () => {}, diary_lock: () => { window.__dopen = false; },
    media_cmd: () => {}, clip_get: () => null, clip_set: () => {}, study_state: () => ({ on: false }), study_start: () => {}, study_stop: () => {}, launch_gaming: () => {}, open_diary: () => {}, say_text: () => {},
    tg_connect: () => "ostrovok_bot", autostart_set: () => {}, set_rects: () => {}, focus_start: () => {}, focus_cancel: () => {}, open_settings: () => {}, tg_unlink: () => {}, quit_app: () => {} };
  return { cfg, invoke: (c, a) => Promise.resolve(handlers[c] ? handlers[c](a || {}) : null) };
})();
window.__de = [{ date: ymdG(0), mood: "🔥", highlight: "Сделал дневник на Rust", text: "Сегодня переписали Островок.\nКрутой день!", energy: 1 }, { date: ymdG(-1), mood: "🖤", highlight: "Вайб", text: "Слушал 9mice весь вечер", energy: 2 }, { date: ymdG(-2), mood: "😴", highlight: "Устал после школы", text: "", energy: 4 }];
function ymdG(n) { const d = new Date(); d.setDate(d.getDate() + n); return d.getFullYear() + "-" + String(d.getMonth() + 1).padStart(2, "0") + "-" + String(d.getDate()).padStart(2, "0"); }
const T = {
  invoke: (c, a) => IN_TAURI ? window.__TAURI__.core.invoke(c, a || {}) : mock.invoke(c, a),
  listen: (e, cb) => { if (IN_TAURI) window.__TAURI__.event.listen(e, ev => cb(ev.payload)); },
};

/* ── звуки (Web Audio, включается после первого клика) ── */
const Snd = (() => {
  let actx = null, on = true, vol = .6;
  const ctx = () => { if (!actx) { try { actx = new (window.AudioContext || window.webkitAudioContext)(); } catch (e) {} } if (actx && actx.state === "suspended") actx.resume(); return actx; };
  const tone = (f, d, type, v, w, flat) => {
    if (!actx || !on) return; const t = actx.currentTime + (w || 0), o = actx.createOscillator(), g = actx.createGain(), a = (v || .2) * vol * 1.6;
    o.type = type || "sine"; o.frequency.value = f; g.gain.setValueAtTime(0, t); g.gain.linearRampToValueAtTime(a, t + .006);
    if (flat) { g.gain.setValueAtTime(a, t + d - .03); g.gain.linearRampToValueAtTime(0, t + d); } else g.gain.exponentialRampToValueAtTime(.0001, t + d);
    o.connect(g).connect(actx.destination); o.start(t); o.stop(t + d + .02);
  };
  const noise = (d, v, w, hp) => {
    if (!actx || !on) return; const n = actx.sampleRate * d, b = actx.createBuffer(1, n, actx.sampleRate), c = b.getChannelData(0);
    for (let i = 0; i < n; i++) c[i] = (Math.random() * 2 - 1) * Math.exp(-i / n * 8);
    const s = actx.createBufferSource(), f = actx.createBiquadFilter(), g = actx.createGain(); f.type = "highpass"; f.frequency.value = hp || 1500; g.gain.value = v * vol * 1.6;
    s.buffer = b; s.connect(f).connect(g).connect(actx.destination); s.start(actx.currentTime + (w || 0));
  };
  const K = {
    pop() { tone(520, .09, "sine", .18); tone(780, .09, "sine", .14, .05); },
    ding() { tone(988, .14, "sine", .2); tone(1319, .35, "sine", .22, .12); },
    dong() { tone(784, .14, "sine", .2); tone(587, .38, "sine", .22, .12); },
    tick() { tone(1200, .05, "triangle", .1); },
    alert() { tone(880, .13, "square", .07); tone(880, .13, "square", .07, .2); },
    shutter() { noise(.04, .3); noise(.07, .22, .07, 800); },
    buzz() { for (let i = 0; i < 3; i++) { tone(168, .2, "sawtooth", .13, i * .27, true); tone(336, .2, "triangle", .06, i * .27, true); } },
    hw() { [659, 784, 988].forEach((f, i) => tone(f, .2, "sine", .15, i * .15)); tone(1319, .4, "sine", .17, .5); },
    done() { tone(784, .1, "sine", .16); tone(1175, .25, "sine", .18, .09); },
    whoosh() { noise(.7, .16, 0, 400); }, sparkle() { [1568, 1976, 2349, 2637].forEach((f, i) => tone(f, .35, "sine", .1, i * .07)); },
  };
  return { play(k) { ctx(); if (K[k]) K[k](); }, enable(v) { on = !!v; }, volume(p) { vol = Math.max(0, Math.min(1, p / 100)); }, unlock: ctx };
})();
addEventListener("pointerdown", () => Snd.unlock(), { once: true });

function applyCfgSounds(c) { Snd.enable(c.sounds !== false); Snd.volume(c.sound_vol ?? 60); }
