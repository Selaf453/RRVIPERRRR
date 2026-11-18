"use strict";
let cfg = {}, page = "general";
const PAGES = [["general", "Общие"], ["style", "Вид острова"], ["char", "Характер и эмоции"], ["music", "Музыка и тексты"], ["sounds", "Звуки"], ["pc", "ПК и система"], ["tg", "Telegram и ДЗ"], ["study", "Учёба"], ["games", "Игры"], ["exam", "ЕГЭ и заметки"], ["focus", "Фокус"]];
const save = patch => { Object.assign(cfg, patch); return T.invoke("set_cfg", { patch }); };

function row(title, sub, ctl) { const r = el("div", "row"), l = el("div", "l"); l.append(el("b", "", title)); if (sub) l.append(el("span", "", sub)); r.append(l, ctl); return r; }
function sw(key, onChange) { const s = el("button", "sw" + (cfg[key] ? " on" : "")); s.addEventListener("click", () => { const v = !cfg[key]; s.classList.toggle("on", v); save({ [key]: v }); if (onChange) onChange(v); }); return s; }
function num(key, min, max, step) { const n = el("input"); n.type = "number"; n.min = min; n.max = max; n.step = step || 1; n.value = cfg[key]; n.addEventListener("change", () => { const v = Math.max(min, Math.min(max, +n.value || min)); n.value = v; save({ [key]: v }); }); return n; }
function txt(key, ph) { const n = el("input"); n.type = "text"; n.value = cfg[key] || ""; n.placeholder = ph || ""; n.addEventListener("change", () => save({ [key]: n.value.trim() })); n.style.width = "220px"; return n; }
function card(title) { const c = el("div", "card"); if (title) c.append(el("h3", "", title)); return c; }
function head(m, t, s) { m.append(el("h2", "", t)); if (s) m.append(el("p", "sub", s)); }
function btn(t, cls, fn) { const b = el("button", "btn " + (cls || ""), t); b.addEventListener("click", fn); return b; }

function nav() {
  const s = $("side"); s.textContent = ""; const h = el("h1"); h.append("🏝 Островок"); s.append(h);
  PAGES.forEach(([k, t]) => { const b = el("button", "nav" + (page === k ? " on" : ""), t); b.addEventListener("click", () => { page = k; render(); }); s.append(b); });
  s.append(el("div", "ver", "Версия 3.1 · Rust"));
}
async function render() {
  nav(); const m = $("main"); m.textContent = ""; await ({ general, style, char, music, sounds, pc, tg, study, games, exam, focus })[page](m);
}

function general(m) {
  head(m, "Общие", "Основное");
  const c = card(); c.append(
    row("Запускать вместе с Windows", "Островок сам появится после включения ПК", sw("autostart", v => T.invoke("autostart_set", { on: v }))),
    row("Город для погоды", "Температура справа на острове", txt("city", "Например, Челябинск")),
    row("Танцы под музыку", "Глаза пляшут, пока играет трек", sw("dance")),
    row("Показывать ДЗ при запуске", "Если есть задание на сегодня", sw("hw_show_on_start")));
  m.append(c);
  const d = card("Дневник"); d.append(row("Личный дневник", "Записи защищены паролем на экране входа", btn("Открыть", "main", () => T.invoke("open_diary")))); m.append(d);
  const h = card("Горячие клавиши"); h.append(el("div", "hint", "Ctrl+Alt+I раскрыть остров · Ctrl+Alt+N быстрая заметка · Ctrl+Alt+F фокус. ПКМ по острову: потыкать.")); m.append(h);
}
function style(m) {
  head(m, "Вид острова", "Выбери форму, она применяется сразу");
  const c = card(), g = el("div", "styles");
  [["pill", "Пилюля"], ["notch", "Чёлка"], ["rounded", "Скруглённый"], ["glass", "Стекло"], ["neon", "Неон"]].forEach(([k, t]) => {
    const b = el("button", cfg.island_style === k ? "on" : ""); b.append(el("i", k), t); b.addEventListener("click", () => { save({ island_style: k }); render(); }); g.append(b);
  });
  c.append(g); m.append(c);
}
function sounds(m) {
  head(m, "Звуки", "Мягкие сигналы острова и «вибро» для перерыва");
  const c = card(); const vol = el("input"); vol.type = "range"; vol.min = 0; vol.max = 100; vol.value = cfg.sound_vol ?? 60; vol.addEventListener("change", () => { save({ sound_vol: +vol.value }); applyCfgSounds(cfg); });
  c.append(row("Звуки включены", "", sw("sounds", () => applyCfgSounds(cfg))), row("Громкость", "", vol));
  const b = el("div", "btns"); [["ding", "Уведомление"], ["buzz", "Вибро (перерыв)"], ["hw", "Домашка"], ["shutter", "Скриншот"], ["alert", "Тревога"]].forEach(([k, t]) => b.append(btn(t, "", () => { applyCfgSounds({ ...cfg, sounds: true }); Snd.play(k); })));
  c.append(b); m.append(c);
}
function pc(m) {
  head(m, "ПК и система", "О чём остров будет сообщать");
  const c = card("События"); c.append(
    row("Флешки и диски", "Подключили или вынули", sw("pc_usb")), row("Интернет", "Пропал или вернулся", sw("pc_net")),
    row("Раскладка клавиатуры", "RU / EN при переключении", sw("pc_layout")), row("Caps Lock", "Включён или выключен", sw("pc_caps")),
    row("Скриншоты", "Новый файл в Снимках экрана", sw("pc_shots")), row("Батарея и зарядка", "Зарядка, 20% и 10% (на ноутбуках)", sw("battery_watch")));
  m.append(c);
  const d = card("Напоминания"); d.append(
    row("Пора на перерыв", "Гудит, если долго сидишь без пауз", sw("pc_break")), row("Перерыв через, минут", "", num("pc_break_min", 5, 240)),
    row("Программы-обжоры", "Если процесс занял много памяти", sw("pc_hog")), row("Порог памяти, ГБ", "", num("pc_hog_gb", 0.5, 64, 0.5)));
  m.append(d);
}
async function tg(m) {
  head(m, "Telegram и ДЗ", "Пиши задания боту, а остров покажет их на ПК");
  const c = card("Бот"); const linked = cfg.tg_chat_id != null, has = !!cfg.tg_token;
  if (!has) {
    const f = el("div", "hwform"), t = el("input", "w"); t.type = "password"; t.placeholder = "Токен бота от @BotFather"; t.autocomplete = "off"; const st = el("div", "hint");
    f.append(t, btn("Подключить", "main", async () => { st.textContent = "Проверяю…"; st.className = "hint"; try { const u = await T.invoke("tg_connect", { token: t.value }); cfg = await T.invoke("get_cfg"); render(); } catch (e) { st.textContent = String(e); st.className = "hint err"; } }));
    c.append(f, st);
    const o = el("ol", "steps"); ["Открой в Telegram @BotFather и отправь /newbot", "Придумай имя, скопируй токен и вставь выше", "Нажми «Подключить», потом отправь боту личный код"].forEach(s => o.append(el("li", "", s))); c.append(o);
  } else {
    c.append(row("Бот @" + (cfg.tg_bot_username || "…"), linked ? "Чат привязан" + (cfg.tg_user_name ? ": " + cfg.tg_user_name : "") : "Отправь боту личный код ниже", el("span", linked ? "ok" : "err", linked ? "Подключён" : "Ждёт код")));
    if (!linked) { const k = el("div"); k.append(el("div", "code", cfg.tg_code), el("div", "hint", "Отправь этот код боту в Telegram, и чат привяжется.")); c.append(k); }
    c.append(el("div", "btns")); c.lastChild.append(btn("Отвязать", "red", async () => { await T.invoke("tg_unlink"); cfg = await T.invoke("get_cfg"); render(); }));
  }
  m.append(c);
  const opt = card(); opt.append(row("Напоминать о ДЗ раз в час", "Только если есть задание на сегодня или просрочено", sw("hw_hourly"))); m.append(opt);

  // домашка
  const h = card("Домашка"); const f = el("div", "hwform"), inp = el("input", "w"); inp.type = "text"; inp.placeholder = "Например: Русский, стр. 34 упр. 5 на завтра"; inp.maxLength = 300;
  const add = async () => { const v = inp.value.trim(); if (!v) return; await T.invoke("hw_add", { text: v }); inp.value = ""; Snd.play("done"); fill(); };
  inp.addEventListener("keydown", e => { if (e.key === "Enter") add(); }); f.append(inp, btn("Записать", "main", add)); h.append(f, el("div", "hint", "Предмет и срок («на завтра», «к пятнице», «до 20.11») определятся сами."));
  const list = el("div"); h.append(list); m.append(h);
  async function fill() {
    const items = await T.invoke("hw_view") || []; list.textContent = "";
    if (!items.length) { list.append(el("div", "empty", "Пока пусто. Напиши задание сюда или боту в Telegram.")); return; }
    const sec = (t, arr) => { if (!arr.length) return; list.append(el("div", "sech", t)); arr.forEach(h => list.append(hwCard(h, fill))); };
    sec("Нужно сделать", items.filter(x => !x.done).sort((a, b) => (a.due || "9").localeCompare(b.due || "9"))); const done = items.filter(x => x.done); sec("Готово", done);
    if (done.length) { const b = el("div", "btns"); b.append(btn("Убрать сделанное", "", async () => { await T.invoke("hw_clear_done"); fill(); })); list.append(b); }
  }
  fill();
}
function hwCard(h, fill) {
  const r = el("div", "hw" + (h.done ? " done" : "")), bar = el("div", "bar"); bar.style.background = h.color;
  const mid = el("div", "m"), b = el("b", "", h.subject || "Задание"); b.style.color = h.color; mid.append(b, el("div", "", h.body));
  r.append(bar, mid);
  if (h.label) r.append(el("span", "pill " + (h.label === "просрочено" ? "late" : h.label === "сегодня" ? "today" : ""), h.label));
  const ok = el("button", "a ok", h.done ? "↺" : "✓"), del = el("button", "a del", "✕");
  ok.addEventListener("click", async () => { await T.invoke("hw_toggle", { id: h.id }); if (!h.done) Snd.play("done"); fill(); });
  del.addEventListener("click", async () => { await T.invoke("hw_delete", { id: h.id }); fill(); });
  r.append(ok, del); return r;
}
function daysTo(d) { return Math.round((new Date(d + "T00:00:00") - new Date(new Date().toISOString().slice(0, 10) + "T00:00:00")) / 864e5); }
function plural(n) { const a = n % 10, b = n % 100; return b >= 11 && b <= 14 ? "дней" : a === 1 ? "день" : a >= 2 && a <= 4 ? "дня" : "дней"; }
function exam(m) {
  head(m, "ЕГЭ и заметки", "Остров считает дни и напоминает о событии");
  const c = card("Отсчёт до события"), list = el("div");
  const paint = () => {
    list.textContent = ""; const ex = [...(cfg.exams || [])].map((e, i) => ({ ...e, i })).sort((a, b) => a.date.localeCompare(b.date));
    if (!ex.length) list.append(el("div", "empty", "Добавь ЕГЭ или контрольную, и остров покажет, сколько дней осталось."));
    ex.forEach(e => { const n = daysTo(e.date), r = el("div", "ex"), t = el("div", "n"); t.append(el("b", "", e.name), el("span", "", e.date));
      const d = el("div", "big", n > 0 ? n + " " + plural(n) : n === 0 ? "сегодня" : "прошло"); const x = el("button", "btn red", "✕"); x.addEventListener("click", () => { const a = (cfg.exams || []).filter((_, k) => k !== e.i); save({ exams: a }); paint(); }); r.append(t, d, x); list.append(r); });
  };
  const f = el("div", "hwform"), nm = el("input", "w"), dt = el("input"); nm.type = "text"; nm.placeholder = "Название, например ЕГЭ по информатике"; dt.type = "date"; f.style.gridTemplateColumns = "1fr auto auto";
  f.append(nm, dt, btn("Добавить", "main", () => { if (!nm.value.trim() || !dt.value) return; save({ exams: [...(cfg.exams || []), { name: nm.value.trim(), date: dt.value }] }); nm.value = ""; paint(); }));
  c.append(f, list); m.append(c); paint();
  const n = card("Заметки (Ctrl+Alt+N)"), nl = el("div");
  const paintN = () => { nl.textContent = ""; const a = cfg.notes || []; if (!a.length) nl.append(el("div", "empty", "Пока нет заметок. Нажми Ctrl+Alt+N на острове."));
    a.forEach((x, i) => { const r = el("div", "ex"), t = el("div", "n"); t.append(el("b", "", x.t), el("span", "", x.d)); const del = el("button", "btn red", "✕"); del.addEventListener("click", () => { save({ notes: (cfg.notes || []).filter((_, k) => k !== i) }); paintN(); }); r.append(t, del); nl.append(r); }); };
  n.append(nl); m.append(n); paintN();
}
function focus(m) {
  head(m, "Фокус", "На время работы остров молчит и прячет мелкие уведомления");
  const c = card(); c.append(row("Длительность по умолчанию, минут", "Тревога про батарею 10% всё равно прозвучит", num("focus_min", 5, 240)));
  const b = el("div", "btns"); [25, 45, 60].forEach(x => b.append(btn("Начать на " + x + " мин", x === 45 ? "main" : "", () => T.invoke("focus_start", { minutes: x })))); b.append(btn("Остановить", "red", () => T.invoke("focus_cancel"))); c.append(b);
  m.append(c); const d = card(); d.append(el("div", "hint", "В конце фокуса остров коротко скажет, что ты пропустил. Включить можно и с клавиатуры: Ctrl+Alt+F.")); m.append(d);
}

function slider(key, min, max, step, fmt) {
  const w = el("div"); w.style.cssText = "display:flex;align-items:center;gap:10px"; const r = el("input"); r.type = "range"; r.min = min; r.max = max; r.step = step || 1; r.value = cfg[key] ?? min; const v = el("span", "", fmt ? fmt(+r.value) : r.value); v.style.cssText = "min-width:70px;color:#9a9db5;font-size:13px";
  r.addEventListener("input", () => { v.textContent = fmt ? fmt(+r.value) : r.value; }); r.addEventListener("change", () => save({ [key]: +r.value })); w.append(r, v); return w;
}
function char(m) {
  head(m, "Характер и эмоции", "33 настроения, танцы и реплики острова");
  const c = card("Поведение"); c.append(
    row("Разговорчивость", "0 молчит, 3 болтает без остановки", slider("chatty", 0, 3, 1, v => ["молчит", "редко", "обычно", "часто"][v])),
    row("Засыпает через", "Если ты не трогаешь ПК", slider("sleep_after_min", 1, 30, 1, v => v + " мин")),
    row("Танцы под музыку", "Меняет танец и настроение каждые несколько секунд", sw("dance")),
    row("Озвучивать реплики", "Голос Windows читает уведомления (нужен русский голос)", sw("speak")));
  m.append(c);
  const g = card("Все настроения (нажми, остров покажет)"); const box = el("div"); box.style.cssText = "display:grid;grid-template-columns:repeat(auto-fill,minmax(130px,1fr));gap:8px";
  MOODS.forEach(([id, name, fxc]) => { const b = el("button", "btn", (fxc || "🙂") + " " + name); b.style.cssText += ";text-align:left;min-height:40px;font-size:13px"; b.addEventListener("click", () => T.invoke("preview_mood", { mood: id })); box.append(b); });
  g.append(box); m.append(g);
}
function music(m) {
  head(m, "Музыка и тексты", "Остров слышит трек и танцует");
  const c = card(); c.append(row("Показывать тексты песен", "Строки в такт, если нашлись на lrclib.net", sw("show_lyrics")), row("Сдвиг текста", "Если строки идут не вовремя", slider("lyrics_offset", -5, 5, .5, v => (v > 0 ? "+" : "") + v + " с")));
  m.append(c); const d = card(); d.append(el("div", "hint", "Название и позицию трека берём из Windows (Яндекс Музыка, Spotify, браузер). Видео и стримы остров тоже может принять за музыку: тогда он просто покачивается.")); m.append(d);
}
function study(m) {
  head(m, "Учёба", "Таймер и плейлист для работы над уроками");
  const c = card(); c.append(row("Длительность, минут", "", num("study_min", 5, 240)), row("Ругаться на игры во время учёбы", "Остров напомнит, что сейчас учёба", sw("study_nag")));
  const b = el("div", "btns"); b.append(btn("Начать учёбу", "main", () => T.invoke("study_start")), btn("Остановить", "red", () => T.invoke("study_stop"))); c.append(b); m.append(c);
  const p = card("Плейлисты"), list = el("div");
  const paint = () => { list.textContent = ""; const pl = cfg.playlists || [];
    if (!pl.length) list.append(el("div", "empty", "Добавь плейлист, он откроется при старте учёбы."));
    pl.forEach((x, i) => { const r = el("div", "ex"), t = el("div", "n"); t.append(el("b", "", x.name), el("span", "", x.url)); const use = el("button", "btn" + ((cfg.study_playlist || 0) === i ? " main" : ""), (cfg.study_playlist || 0) === i ? "Выбран" : "Выбрать");
      use.addEventListener("click", () => { save({ study_playlist: i }); paint(); }); const x2 = el("button", "btn red", "✕"); x2.addEventListener("click", () => { save({ playlists: pl.filter((_, k) => k !== i), study_playlist: 0 }); paint(); }); r.append(t, use, x2); list.append(r); }); };
  const f = el("div", "hwform"), nm = el("input"), u = el("input"); nm.type = "text"; nm.placeholder = "Название"; u.type = "text"; u.placeholder = "Ссылка https://…"; f.style.gridTemplateColumns = "1fr 2fr auto";
  f.append(nm, u, btn("Добавить", "main", () => { if (!nm.value.trim() || !/^https?:\/\//.test(u.value.trim())) return; save({ playlists: [...(cfg.playlists || []), { name: nm.value.trim(), url: u.value.trim() }] }); nm.value = u.value = ""; paint(); }));
  p.append(f, list); m.append(p); paint();
}
function games(m) {
  head(m, "Игры", "Игровой режим и реакция на запуск игр");
  const c = card("Игровой режим"); c.append(row("Запускать FACEIT AC", "", sw("gaming_faceit")), row("Запускать CS2 через Steam", "", sw("gaming_cs2")), row("Открывать музыку", "", sw("gaming_music")), row("Свой путь к FACEIT", "Если автоопределение не нашло", txt("custom_faceit_path", "C:\\...\\faceitclient.exe")));
  const b = el("div", "btns"); b.append(btn("Запустить игровой режим", "main", () => T.invoke("launch_gaming"))); c.append(b); m.append(c);
  const d = card(); d.append(el("div", "hint", "Когда запускается игра из списка (CS2, Valorant, Dota 2, Minecraft…), остров показывает нагрузку процессора и памяти и подбадривает. Мелкие уведомления в игре тоже можно скрыть режимом фокуса.")); m.append(d);
}
T.listen("cfg", c => { cfg = c; });
(async () => { cfg = await T.invoke("get_cfg") || {}; applyCfgSounds(cfg); render(); })();
