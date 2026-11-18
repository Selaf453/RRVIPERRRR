#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod platform;

use ostrovok_core::{config, diary, games, hw, logic, lyrics, study, telegram};
use serde_json::{json, Map, Value};
use std::{
    path::PathBuf,
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessesToUpdate, System};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder,
};

const APP_VER: &str = "3.1";

struct Shared {
    dir: PathBuf,
    cfg: Mutex<Map<String, Value>>,
    focus: Mutex<logic::Focus>,
    last_hw_ms: Mutex<i64>,
    rects: Mutex<Vec<[f64; 4]>>,
    diary_ok: AtomicBool,
    study: Mutex<Option<study::Study>>,
    game: Mutex<Option<String>>,
}
type S<'a> = State<'a, Arc<Shared>>;

fn now_ms() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0) }
fn today() -> chrono::NaiveDate { chrono::Local::now().date_naive() }

impl Shared {
    fn save(&self) { let c = self.cfg.lock().unwrap(); let _ = config::save(&self.dir, &c); }
    fn get_bool(&self, k: &str) -> bool { self.cfg.lock().unwrap().get(k).and_then(|v| v.as_bool()).unwrap_or(false) }
    fn get_f(&self, k: &str, d: f64) -> f64 { self.cfg.lock().unwrap().get(k).and_then(|v| v.as_f64()).unwrap_or(d) }
    fn homework(&self) -> Vec<Value> { self.cfg.lock().unwrap().get("homework").and_then(|v| v.as_array().cloned()).unwrap_or_default() }
    fn set_homework(&self, v: Vec<Value>) { self.cfg.lock().unwrap().insert("homework".into(), Value::Array(v)); self.save(); }
}

/// Единая точка показа уведомления: учитывает режим фокуса.
fn notify(app: &AppHandle, sh: &Shared, mut t: Value, urgent: bool) {
    let title = t["title"].as_str().unwrap_or("").to_string();
    let allowed = sh.focus.lock().unwrap().allow(now_ms(), &title, urgent);
    if !allowed { return; }
    if sh.get_bool("speak") && !title.is_empty() {
        let sub = t["sub"].as_str().unwrap_or("");
        platform::speak(&format!("{title}. {sub}"));
    }
    t["sounds"] = json!(sh.get_bool("sounds"));
    t["vol"] = json!(sh.get_f("sound_vol", 60.0));
    let _ = app.emit("toast", t);
}
fn toast(icon: &str, color: &str, title: &str, sub: &str, mood: &str, snd: &str) -> Value {
    json!({"icon": icon, "color": color, "title": title, "sub": sub, "mood": mood, "snd": snd})
}

// ───────── команды для интерфейса ─────────
#[tauri::command]
fn get_cfg(sh: S) -> Value { Value::Object(sh.cfg.lock().unwrap().clone()) }
#[tauri::command]
fn app_ver() -> &'static str { APP_VER }

#[tauri::command]
fn set_cfg(app: AppHandle, sh: S, patch: Map<String, Value>) {
    {
        let mut c = sh.cfg.lock().unwrap();
        for (k, v) in patch { c.insert(k, v); }
    }
    sh.save();
    let _ = app.emit("cfg", Value::Object(sh.cfg.lock().unwrap().clone()));
}
#[tauri::command]
fn autostart_set(app: AppHandle, on: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let m = app.autolaunch();
    (if on { m.enable() } else { m.disable() }).map_err(|e| e.to_string())
}
#[tauri::command]
fn hw_add(app: AppHandle, sh: S, text: String) -> Value {
    let mut list = sh.homework();
    let item = hw::make_hw(&text, &list);
    list.push(item.clone());
    sh.set_homework(list);
    let _ = app.emit("hw", ());
    item
}
#[tauri::command]
fn hw_toggle(app: AppHandle, sh: S, id: i64) {
    let mut list = sh.homework();
    for h in list.iter_mut() { if h["id"].as_i64() == Some(id) { let d = h["done"].as_bool().unwrap_or(false); h["done"] = json!(!d); } }
    sh.set_homework(list);
    let _ = app.emit("hw", ());
}
#[tauri::command]
fn hw_delete(app: AppHandle, sh: S, id: i64) {
    let list: Vec<Value> = sh.homework().into_iter().filter(|h| h["id"].as_i64() != Some(id)).collect();
    sh.set_homework(list);
    let _ = app.emit("hw", ());
}
#[tauri::command]
fn hw_clear_done(app: AppHandle, sh: S) {
    let list: Vec<Value> = sh.homework().into_iter().filter(|h| !h["done"].as_bool().unwrap_or(false)).collect();
    sh.set_homework(list);
    let _ = app.emit("hw", ());
}
#[tauri::command]
fn hw_view(sh: S) -> Value {
    let t = today();
    let items: Vec<Value> = sh.homework().into_iter().map(|mut h| {
        let subj = h["subject"].as_str().unwrap_or("").to_string();
        h["body"] = json!(hw::hw_body(&h));
        h["label"] = json!(hw::due_label(h["due"].as_str().unwrap_or(""), t));
        h["color"] = json!(hw::subject_color(&subj));
        h["late"] = json!(h["due"].as_str().map_or(false, |d| !d.is_empty() && h["done"] != json!(true) && hw::is_due_today_or_late(&h, t) && h["label"] == json!("просрочено")));
        h["today"] = json!(hw::is_due_today_or_late(&h, t));
        h
    }).collect();
    json!(items)
}
#[tauri::command]
fn tg_connect(app: AppHandle, sh: S, token: String) -> Result<String, String> {
    let token = token.trim().to_string();
    let me = telegram::api(&token, "getMe", &json!({}), 15).map_err(|e| if e.contains("Unauthorized") { "Неверный токен".to_string() } else { e })?;
    let uname = me["username"].as_str().unwrap_or("").to_string();
    {
        let mut c = sh.cfg.lock().unwrap();
        c.insert("tg_token".into(), json!(token));
        c.insert("tg_bot_username".into(), json!(uname));
        c.insert("tg_chat_id".into(), Value::Null);
        c.insert("tg_offset".into(), json!(0));
    }
    sh.save();
    let _ = app.emit("cfg", Value::Object(sh.cfg.lock().unwrap().clone()));
    Ok(uname)
}
#[tauri::command]
fn tg_unlink(app: AppHandle, sh: S) {
    {
        let mut c = sh.cfg.lock().unwrap();
        for k in ["tg_token", "tg_bot_username", "tg_user_name"] { c.insert(k.into(), json!("")); }
        c.insert("tg_chat_id".into(), Value::Null);
    }
    sh.save();
    let _ = app.emit("cfg", Value::Object(sh.cfg.lock().unwrap().clone()));
}
#[tauri::command]
fn focus_start(app: AppHandle, sh: S, minutes: u32) {
    sh.focus.lock().unwrap().start(now_ms(), minutes);
    let _ = app.emit("focus", json!({"on": true, "until": now_ms() + minutes as i64 * 60_000}));
}
#[tauri::command]
fn focus_cancel(app: AppHandle, sh: S) {
    let missed = sh.focus.lock().unwrap().cancel();
    let _ = app.emit("focus", json!({"on": false, "missed": missed}));
}
#[tauri::command]
fn exam_info(sh: S) -> Value {
    let ex = sh.cfg.lock().unwrap().get("exams").and_then(|v| v.as_array().cloned()).unwrap_or_default();
    match logic::nearest_exam(&ex, today()) { Some((n, t, d)) => json!({"name": n, "text": t, "days": d}), None => Value::Null }
}
#[tauri::command]
fn set_rects(sh: S, rects: Vec<[f64; 4]>) { *sh.rects.lock().unwrap() = rects; }
#[tauri::command]
fn open_settings(app: AppHandle) { show_settings(&app); }
#[tauri::command]
fn quit_app(app: AppHandle) { app.exit(0); }


// ───────── дневник ─────────
fn need_diary(sh: &Shared) -> Result<(), String> { if sh.diary_ok.load(Ordering::SeqCst) { Ok(()) } else { Err("Дневник закрыт".into()) } }
#[tauri::command]
fn diary_state(sh: S) -> Value { json!({"has_pwd": diary::has_password(&sh.cfg.lock().unwrap()), "open": sh.diary_ok.load(Ordering::SeqCst)}) }
#[tauri::command]
fn diary_auth(sh: S, pwd: String, pwd2: String) -> Result<(), String> {
    {
        let mut c = sh.cfg.lock().unwrap();
        if diary::has_password(&c) {
            if !diary::verify(&c, &pwd) { return Err("Неверный пароль".into()); }
        } else {
            diary::create_password(&mut c, &pwd, &pwd2)?;
        }
    }
    sh.save();
    sh.diary_ok.store(true, Ordering::SeqCst);
    Ok(())
}
#[tauri::command]
fn diary_lock(sh: S) { sh.diary_ok.store(false, Ordering::SeqCst); }
#[tauri::command]
fn diary_list(sh: S) -> Result<Value, String> {
    need_diary(&sh)?;
    let c = sh.cfg.lock().unwrap();
    Ok(json!({"items": diary::list(&c), "streak": diary::streak(&c, today())}))
}
#[tauri::command]
fn diary_get(sh: S, date: String) -> Result<Value, String> { need_diary(&sh)?; Ok(diary::get(&sh.cfg.lock().unwrap(), &date)) }
#[tauri::command]
fn diary_save(sh: S, date: String, mood: String, highlight: String, text: String, energy: i64) -> Result<(), String> {
    need_diary(&sh)?;
    diary::save_entry(&mut sh.cfg.lock().unwrap(), &date, &mood, &highlight, &text, energy)?;
    sh.save();
    Ok(())
}
#[tauri::command]
fn diary_delete(sh: S, date: String) -> Result<(), String> { need_diary(&sh)?; diary::delete(&mut sh.cfg.lock().unwrap(), &date); sh.save(); Ok(()) }
#[tauri::command]
fn say_text(sh: S, text: String) { if sh.get_bool("speak") { platform::speak(&text.chars().take(200).collect::<String>()); } }
#[tauri::command]
fn media_cmd(cmd: String) { platform::media(&cmd); }
#[tauri::command]
fn clip_get() -> Option<String> { arboard::Clipboard::new().ok()?.get_text().ok().filter(|t| !t.trim().is_empty() && t.len() < 5000) }
#[tauri::command]
fn clip_set(text: String) { if let Ok(mut c) = arboard::Clipboard::new() { let _ = c.set_text(text); } }
#[tauri::command]
fn preview_mood(app: AppHandle, mood: String) { let _ = app.emit("preview", mood); }
#[tauri::command]
fn open_diary(app: AppHandle) { show_window(&app, "diary", "diary.html", "Личный дневник: Островок", 1000.0, 680.0); }

// ───────── учёба и игры ─────────
#[tauri::command]
fn study_start(app: AppHandle, sh: S) {
    let (mins, pl) = {
        let c = sh.cfg.lock().unwrap();
        let m = c.get("study_min").and_then(|v| v.as_u64()).unwrap_or(50) as u32;
        let i = c.get("study_playlist").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let p = c.get("playlists").and_then(|v| v.as_array()).and_then(|a| a.get(i)).cloned();
        (m, p)
    };
    *sh.study.lock().unwrap() = Some(study::Study::new(now_ms(), mins));
    let mut sub = format!("{mins} мин");
    if let Some(p) = pl {
        if let Some(u) = p["url"].as_str() { if platform::open_url(u) { sub = format!("{mins} мин · плейлист: {}", p["name"].as_str().unwrap_or("")); } }
    }
    notify(&app, &sh, toast("📚", "#a78bfa", "Режим учёбы", &sub, "happy", "pop"), true);
    let _ = app.emit("study", json!({"on": true, "left": mins as i64 * 60}));
}
#[tauri::command]
fn study_stop(app: AppHandle, sh: S) { finish_study(&app, &sh, false); }
#[tauri::command]
fn study_state(sh: S) -> Value { match *sh.study.lock().unwrap() { Some(s) => json!({"on": true, "left": s.left_s(now_ms())}), None => json!({"on": false}) } }
fn finish_study(app: &AppHandle, sh: &Shared, natural: bool) {
    let Some(s) = sh.study.lock().unwrap().take() else { return };
    let m = s.studied_min(now_ms());
    let t = toast("🎓", "#a78bfa", "Учёба окончена", &format!("Ты занимался {m} мин. Ты молодец!"), "happy", "ding");
    notify(app, sh, t, true);
    let _ = app.emit("study", json!({"on": false, "natural": natural}));
}
#[tauri::command]
fn launch_gaming(app: AppHandle, sh: S) {
    let cfg = sh.cfg.lock().unwrap().clone();
    let a = app.clone();
    thread::spawn(move || {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let procs: Vec<String> = sys.processes().values().map(|p| p.name().to_string_lossy().to_string()).collect();
        let on = |k: &str| cfg.get(k).and_then(|v| v.as_bool()).unwrap_or(true);
        let mut res: Vec<String> = vec![];
        if on("gaming_faceit") {
            if games::any_running(&procs, &["faceitclient.exe", "faceitservice.exe", "faceit.exe"]) { res.push("FACEIT AC уже запущен".into()); }
            else {
                let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
                let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".into());
                let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
                let custom = cfg.get("custom_faceit_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let cands = [custom, format!(r"{pf}\FACEIT AC\faceitclient.exe"), format!(r"{pf86}\FACEIT AC\faceitclient.exe"), format!(r"{local}\FACEIT\FACEIT.exe")];
                let ok = cands.iter().any(|p| !p.is_empty() && std::path::Path::new(p).exists() && platform::launch(p));
                res.push(if ok { "FACEIT AC запущен".into() } else { "FACEIT AC не найден".into() });
            }
        }
        if on("gaming_cs2") {
            if games::any_running(&procs, &["cs2.exe"]) { res.push("CS2 уже открыта".into()); }
            else { res.push(if platform::open_url("steam://rungameid/730") { "CS2 запускается".into() } else { "Steam не найден".into() }); }
        }
        if on("gaming_music") {
            if games::any_running(&procs, &["Яндекс Музыка.exe", "YandexMusic.exe", "Spotify.exe", "AIMP.exe", "foobar2000.exe"]) { res.push("Музыка уже открыта".into()); }
            else { platform::open_url("https://music.yandex.ru"); res.push("Музыка включена".into()); }
        }
        let sh = a.state::<Arc<Shared>>().inner().clone();
        let t = toast("🎮", "#f97316", "Игровой режим", &res.join(" · "), "cool", "pop");
        notify(&a, &sh, t, true);
    });
}

fn show_settings(app: &AppHandle) { show_window(app, "settings", "settings.html", "Островок: настройки", 980.0, 700.0); }
fn show_window(app: &AppHandle, label: &str, page: &str, title: &str, w: f64, h: f64) {
    if let Some(win) = app.get_webview_window(label) { let _ = win.show(); let _ = win.unminimize(); let _ = win.set_focus(); return; }
    let _ = WebviewWindowBuilder::new(app, label, WebviewUrl::App(page.into()))
        .title(title).inner_size(w, h).min_inner_size(760.0, 520.0).center().build();
}

// ───────── фоновые циклы ─────────
fn spawn_click_through(app: AppHandle) {
    thread::spawn(move || {
        let mut last = None;
        let mut last_pos = (0.0f64, 0.0f64);
        let mut n: u32 = 0;
        loop {
            thread::sleep(Duration::from_millis(40));
            n = n.wrapping_add(1);
            let Some(w) = app.get_webview_window("island") else { continue };
            let sh = app.state::<Arc<Shared>>();
            let (Ok(cur), Ok(pos)) = (w.cursor_position(), w.outer_position()) else { continue };
            let sf = w.scale_factor().unwrap_or(1.0);
            let (x, y) = ((cur.x - pos.x as f64) / sf, (cur.y - pos.y as f64) / sf);
            let inside = sh.rects.lock().unwrap().iter().any(|r| x >= r[0] && x <= r[0] + r[2] && y >= r[1] && y <= r[1] + r[3]);
            if n % 3 == 0 && ((x - last_pos.0).abs() > 4.0 || (y - last_pos.1).abs() > 4.0) {
                last_pos = (x, y);
                let _ = app.emit("cur", json!({"x": x, "y": y}));
            }
            if last != Some(inside) { let _ = w.set_ignore_cursor_events(!inside); last = Some(inside); }
        }
    });
}

fn place_island(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("island") {
        if let Ok(Some(m)) = w.primary_monitor() {
            let sf = m.scale_factor();
            let lw = m.size().width as f64 / sf;
            let _ = w.set_size(tauri::LogicalSize::new(lw, 660.0));
            let _ = w.set_position(tauri::PhysicalPosition::new(m.position().x, m.position().y));
        }
    }
}

fn spawn_monitor(app: AppHandle) {
    thread::spawn(move || {
        let sh = app.state::<Arc<Shared>>().inner().clone();
        let mut sys = System::new();
        let mut brk = logic::BreakTracker::default();
        let mut net = logic::NetWatch::default();
        let mut bat = logic::BatteryWatch::default();
        let (mut caps, mut lay, mut drives) = (platform::caps_on(), platform::layout(), platform::removable_drives());
        let mut shots_seen = latest_shot();
        let mut hog_told: Vec<String> = vec![];
        let (net_tx, net_rx) = std::sync::mpsc::channel::<bool>();
        thread::spawn(move || loop {
            let ok = std::net::TcpStream::connect_timeout(&"1.1.1.1:53".parse().unwrap(), Duration::from_millis(1500)).is_ok()
                || std::net::TcpStream::connect_timeout(&"8.8.8.8:53".parse().unwrap(), Duration::from_millis(1500)).is_ok();
            let _ = net_tx.send(ok);
            thread::sleep(Duration::from_secs(8));
        });
        let mut tick: u64 = 0;
        loop {
            thread::sleep(Duration::from_secs(1));
            tick += 1;
            let ms = now_ms();

            if tick % 2 == 0 { let _ = app.emit("idle", platform::idle_seconds()); }
            // конец учёбы
            let study_over = sh.study.lock().unwrap().map_or(false, |st| st.done(ms));
            if study_over { finish_study(&app, &sh, true); }
            // игры: процессы смотрим раз в 5 секунд
            if tick % 5 == 0 {
                sys.refresh_processes(ProcessesToUpdate::All, true);
                let procs: Vec<String> = sys.processes().values().map(|p| p.name().to_string_lossy().to_string()).collect();
                let apps = sh.cfg.lock().unwrap().get("apps").and_then(|v| v.as_array().cloned()).unwrap_or_else(|| games::default_apps().as_array().cloned().unwrap());
                let now_game = games::running_game(&apps, &procs).map(|a| (a["name"].as_str().unwrap_or("Игра").to_string(), a["phrases"].clone()));
                let prev = sh.game.lock().unwrap().clone();
                match (&prev, &now_game) {
                    (None, Some((name, phrases))) => {
                        *sh.game.lock().unwrap() = Some(name.clone());
                        let nag = sh.study.lock().unwrap().is_some() && sh.get_bool("study_nag");
                        let _ = app.emit("game", json!({"on": true, "name": games::short_name(name), "phrases": phrases, "nag": nag}));
                    }
                    (Some(_), None) => { *sh.game.lock().unwrap() = None; let _ = app.emit("game", json!({"on": false})); }
                    _ => {}
                }
            }
            // конец фокуса
            if let Some(missed) = sh.focus.lock().unwrap().finish(ms) {
                let _ = app.emit("focus", json!({"on": false, "missed": missed, "done": true}));
            }
            // перерыв
            if sh.get_bool("pc_break") {
                let lim = sh.get_f("pc_break_min", 45.0);
                if brk.step(1.0, platform::idle_seconds(), 300.0, lim) {
                    let mut t = toast("👀", "#a78bfa", "Пора сделать перерыв", &format!("{} мин за ПК. Поморгай, встань, попей воды", lim as i64), "sleepy", "buzz");
                    t["shake"] = json!(true); t["ms"] = json!(7000);
                    notify(&app, &sh, t, false);
                }
            }
            if tick % 2 == 0 {
                if sh.get_bool("pc_caps") { let c = platform::caps_on(); if c != caps { caps = c; notify(&app, &sh, toast("⌨️", "#ffd34d", if c { "Caps Lock включён" } else { "Caps Lock выключен" }, "", "wow", "tick"), false); } }
                if sh.get_bool("pc_layout") { let l = platform::layout(); if !l.is_empty() && l != lay { lay = l.clone(); notify(&app, &sh, toast("⌨️", "#a78bfa", &format!("Раскладка: {l}"), "", "neutral", "tick"), false); } }
            }
            if tick % 3 == 0 && sh.get_bool("pc_usb") {
                let d = platform::removable_drives();
                for x in d.iter().filter(|x| !drives.contains(*x)) { notify(&app, &sh, toast("🔌", "#38bdf8", &format!("Подключён диск {x}"), "Открой в проводнике", "wow", "ding"), false); }
                for x in drives.iter().filter(|x| !d.contains(*x)) { notify(&app, &sh, toast("🔌", "#94a3b8", &format!("Диск {x} отключён"), "", "neutral", "tick"), false); }
                drives = d;
            }
            while let Ok(alive) = net_rx.try_recv() {
                if !sh.get_bool("pc_net") { continue; }
                match net.step(alive, ms) {
                    Some(logic::NetEvent::Lost) => notify(&app, &sh, toast("📡", "#f87171", "Интернет пропал", "Скажу, когда вернётся", "sleepy", "dong"), false),
                    Some(logic::NetEvent::Back { down_s }) => {
                        let sub = if down_s >= 60 { format!("Не было {} мин", down_s / 60) } else { format!("Не было {down_s} с") };
                        notify(&app, &sh, toast("📡", "#4ade80", "Интернет вернулся", &sub, "happy", "ding"), false)
                    }
                    None => {}
                }
            }
            if tick % 5 == 0 {
                sys.refresh_cpu_usage(); sys.refresh_memory();
                let ram = if sys.total_memory() > 0 { sys.used_memory() as f64 * 100.0 / sys.total_memory() as f64 } else { 0.0 };
                let _ = app.emit("stats", json!({"cpu": sys.global_cpu_usage(), "ram": ram}));
            }
            if tick % 10 == 0 && sh.get_bool("pc_shots") {
                let n = latest_shot();
                if n.is_some() && n != shots_seen { shots_seen = n; notify(&app, &sh, toast("📸", "#a78bfa", "Скриншот сохранён", "Лежит в Изображения → Снимки экрана", "happy", "shutter"), false); }
            }
            if tick % 30 == 0 && sh.get_bool("battery_watch") {
                if let Some((p, ac)) = platform::battery() {
                    use logic::BatteryEvent::*;
                    match bat.step(p, ac) {
                        Some(PluggedIn) => notify(&app, &sh, toast("🔌", "#4ade80", "Зарядка подключена", &format!("Заряд {p}%"), "happy", "ding"), false),
                        Some(Unplugged) => notify(&app, &sh, toast("🔋", "#fbbf24", "Зарядку отключили", &format!("Заряд {p}%"), "neutral", "tick"), false),
                        Some(Low20) => notify(&app, &sh, toast("🪫", "#fb923c", "Батарея 20%", "Пора искать зарядку", "wow", "alert"), false),
                        Some(Low10) => notify(&app, &sh, toast("🪫", "#f87171", "Батарея 10%!", "Подключи зарядку, иначе ПК выключится", "sweat", "alert"), true),
                        Some(Full) => notify(&app, &sh, toast("🔋", "#4ade80", "Батарея полная", "Можно отключать", "happy", "ding"), false),
                        None => {}
                    }
                }
            }
            if tick % 60 == 30 && sh.get_bool("pc_hog") {
                let lim = (sh.get_f("pc_hog_gb", 3.0) * 1024.0 * 1024.0 * 1024.0) as u64;
                let mut seen = vec![];
                for p in sys.processes().values() {
                    if p.memory() > lim {
                        let name = p.name().to_string_lossy().to_string();
                        seen.push(name.clone());
                        if !hog_told.contains(&name) {
                            let gb = p.memory() as f64 / 1073741824.0;
                            let mut t = toast("🧠", "#fb923c", &format!("{name} занял {gb:.1} ГБ памяти"), "Если это не ты запускал, проверь диспетчер задач", "sweat", "alert"); t["ms"] = json!(6000);
                            notify(&app, &sh, t, false);
                        }
                    }
                }
                hog_told = seen;
            }
            if tick % 30 == 0 {
                let items = sh.homework();
                let last = *sh.last_hw_ms.lock().unwrap();
                if logic::hourly_hw_due(&items, today(), ms, last, sh.get_bool("hw_hourly")) {
                    *sh.last_hw_ms.lock().unwrap() = ms;
                    notify(&app, &sh, hw_toast(&items), false);
                }
            }
        }
    });
}

fn hw_toast(items: &[Value]) -> Value {
    let t = today();
    let due: Vec<&Value> = items.iter().filter(|h| hw::is_due_today_or_late(h, t)).collect();
    let (title, sub) = match due.first() {
        Some(f) => {
            let subj = f["subject"].as_str().unwrap_or("Задание");
            let more = if due.len() > 1 { format!(" и ещё {}", due.len() - 1) } else { String::new() };
            ("Есть задание на сегодня".to_string(), format!("{}: {}{}", if subj.is_empty() { "Задание" } else { subj }, hw::hw_body(f), more))
        }
        None => ("На сегодня заданий нет".to_string(), "Можно отдыхать".to_string()),
    };
    let mut v = toast("📚", "#38bdf8", &title, &sub, "wow", "hw"); v["ms"] = json!(6500); v
}

fn latest_shot() -> Option<(String, u64)> {
    let dir = PathBuf::from(std::env::var_os("USERPROFILE")?).join("Pictures").join("Screenshots");
    std::fs::read_dir(dir).ok()?.filter_map(|e| e.ok())
        .filter_map(|e| { let m = e.metadata().ok()?.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_secs(); Some((e.file_name().to_string_lossy().to_string(), m)) })
        .max_by_key(|x| x.1)
}

fn spawn_music(app: AppHandle) {
    thread::spawn(move || {
        let mut last = platform::Music::default();
        loop {
            let cur = platform::music().unwrap_or_default();
            let changed = (&cur.title, &cur.artist, cur.playing) != (&last.title, &last.artist, last.playing);
            if changed {
                let _ = app.emit("music", json!({"title": cur.title, "artist": cur.artist, "playing": cur.playing, "source": cur.source}));
                if cur.title != last.title || cur.artist != last.artist {
                    let _ = app.emit("lyrics", json!({"lines": []}));
                    let (a, t, ar, d) = (app.clone(), cur.title.clone(), cur.artist.clone(), cur.dur);
                    if !t.is_empty() && app.state::<Arc<Shared>>().get_bool("show_lyrics") {
                        thread::spawn(move || {
                            let lines = lyrics::fetch(&t, &ar, d);
                            let _ = a.emit("lyrics", json!({"title": t, "lines": lines}));
                        });
                    }
                }
            }
            if cur.playing { let _ = app.emit("mpos", json!({"pos": cur.pos, "dur": cur.dur})); }
            last = cur;
            thread::sleep(Duration::from_secs(2));
        }
    });
}

fn spawn_weather(app: AppHandle) {
    thread::spawn(move || {
        let mut city_done = String::new();
        let mut loc: Option<(f64, f64)> = None;
        loop {
            let city = app.state::<Arc<Shared>>().cfg.lock().unwrap().get("city").and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
            if city != city_done || loc.is_none() {
                city_done = city.clone();
                loc = ureq::get("https://geocoding-api.open-meteo.com/v1/search").query("name", &city).query("count", "1").query("language", "ru")
                    .timeout(Duration::from_secs(10)).call().ok().and_then(|r| r.into_json::<Value>().ok())
                    .and_then(|v| { let r = v["results"].get(0)?.clone(); Some((r["latitude"].as_f64()?, r["longitude"].as_f64()?)) });
            }
            if let Some((la, lo)) = loc {
                if let Some(v) = ureq::get("https://api.open-meteo.com/v1/forecast").query("latitude", &la.to_string()).query("longitude", &lo.to_string())
                    .query("current", "temperature_2m,weather_code,is_day").timeout(Duration::from_secs(10)).call().ok().and_then(|r| r.into_json::<Value>().ok())
                {
                    let c = &v["current"];
                    let _ = app.emit("weather", json!({"temp": c["temperature_2m"], "code": c["weather_code"], "day": c["is_day"], "city": city_done}));
                }
            }
            thread::sleep(Duration::from_secs(900));
        }
    });
}

fn spawn_telegram(app: AppHandle) {
    thread::spawn(move || loop {
        let sh = app.state::<Arc<Shared>>().inner().clone();
        let (token, offset, chat, code) = {
            let c = sh.cfg.lock().unwrap();
            (c["tg_token"].as_str().unwrap_or("").to_string(), c["tg_offset"].as_i64().unwrap_or(0), c["tg_chat_id"].as_i64(), c["tg_code"].as_str().unwrap_or("").to_string())
        };
        if token.is_empty() { thread::sleep(Duration::from_secs(3)); continue; }
        let res = telegram::api(&token, "getUpdates", &json!({"offset": offset, "timeout": 20, "allowed_updates": ["message"]}), 35);
        let ups = match res { Ok(Value::Array(a)) => a, Ok(_) => vec![], Err(_) => { thread::sleep(Duration::from_secs(5)); continue } };
        for u in ups {
            let uid = u["update_id"].as_i64().unwrap_or(0);
            sh.cfg.lock().unwrap().insert("tg_offset".into(), json!(uid + 1));
            let m = &u["message"];
            let (Some(cid), text) = (m["chat"]["id"].as_i64(), m["text"].as_str().unwrap_or("")) else { continue };
            let paired = chat == Some(cid);
            if chat.is_some() && !paired { continue; } // чужой чат игнорируем
            match telegram::classify(text, paired) {
                telegram::Cmd::Pair(t) => {
                    if telegram::code_matches(&t, &code) {
                        {
                            let mut c = sh.cfg.lock().unwrap();
                            c.insert("tg_chat_id".into(), json!(cid));
                            c.insert("tg_user_name".into(), json!(m["from"]["first_name"].as_str().unwrap_or("")));
                        }
                        let _ = telegram::send(&token, cid, "✅ Готово, чат привязан. Пиши задания, я запишу.\nНапример: <i>Русский, стр. 34 на завтра</i>");
                        let _ = app.emit("cfg", Value::Object(sh.cfg.lock().unwrap().clone()));
                    } else {
                        let _ = telegram::send(&token, cid, "Пришли личный код из настроек Островка (вкладка «Telegram и ДЗ»).");
                    }
                }
                telegram::Cmd::Add(t) => {
                    let mut list = sh.homework();
                    let item = hw::make_hw(&t, &list);
                    list.push(item.clone());
                    sh.set_homework(list);
                    let lbl = hw::due_label(item["due"].as_str().unwrap_or(""), today());
                    let _ = telegram::send(&token, cid, &format!("✅ Записал: {}{}", telegram::esc(&hw::hw_body(&item)), if lbl.is_empty() { String::new() } else { format!(" <i>· {lbl}</i>") }));
                    let _ = app.emit("hw", ());
                    let mut tt = toast("📚", "#38bdf8", "Новое задание из Telegram", &format!("{}: {}", item["subject"].as_str().unwrap_or("Задание"), hw::hw_body(&item)), "wow", "hw");
                    tt["ms"] = json!(5500);
                    notify(&app, &sh, tt, false);
                }
                telegram::Cmd::List => {
                    let pending: Vec<Value> = sh.homework().into_iter().filter(|h| !h["done"].as_bool().unwrap_or(false)).collect();
                    let _ = telegram::send(&token, cid, &if pending.is_empty() { "Ничего не задано 🎉".to_string() } else { telegram::format_list(&pending, today()) });
                }
                telegram::Cmd::Unlink => {
                    let _ = telegram::send(&token, cid, "Отвязал этот чат.");
                    sh.cfg.lock().unwrap().insert("tg_chat_id".into(), Value::Null);
                    let _ = app.emit("cfg", Value::Object(sh.cfg.lock().unwrap().clone()));
                }
                telegram::Cmd::Help => { let _ = telegram::send(&token, cid, "🏝 <b>Островок: домашка</b>\nПросто напиши задание, я запишу.\n/list — что задали\n/unlink — отвязать чат\n\n⚠️ Я отвечаю, только когда компьютер включён."); }
                telegram::Cmd::Ignore => {}
            }
            sh.save();
        }
    });
}

fn register_hotkeys(app: &AppHandle) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
    for (key, name) in [("ctrl+alt+n", "note"), ("ctrl+alt+i", "expand"), ("ctrl+alt+f", "focus")] {
        let _ = app.global_shortcut().on_shortcut(key, move |app, _s, ev| {
            if ev.state() == ShortcutState::Pressed { let _ = app.emit("hotkey", name); }
        });
    }
}

fn main() {
    let dir = config::config_dir();
    let cfg = config::load(&dir);
    let _ = config::save(&dir, &cfg);
    let shared = Arc::new(Shared { dir, cfg: Mutex::new(cfg), focus: Mutex::new(logic::Focus::default()), last_hw_ms: Mutex::new(now_ms()), rects: Mutex::new(vec![]), diary_ok: AtomicBool::new(false), study: Mutex::new(None), game: Mutex::new(None) });

    tauri::Builder::default()
        // Второй запуск не плодит копии, а будит уже работающий Островок (раньше было «уже запущен»).
        .plugin(tauri_plugin_single_instance::init(|app, _a, _c| {
            if let Some(w) = app.get_webview_window("island") { let _ = w.show(); }
            let _ = app.emit("toast", json!({"icon": "👋", "color": "#ffd34d", "title": "Я уже здесь", "sub": "Островок уже запущен", "mood": "happy", "snd": "pop"}));
        }))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(shared.clone())
        .invoke_handler(tauri::generate_handler![
            get_cfg, app_ver, set_cfg, autostart_set, hw_add, hw_toggle, hw_delete, hw_clear_done, hw_view,
            tg_connect, tg_unlink, focus_start, focus_cancel, exam_info, set_rects, open_settings, quit_app,
            diary_state, diary_auth, diary_lock, diary_list, diary_get, diary_save, diary_delete, open_diary, say_text, preview_mood,
            study_start, study_stop, study_state, launch_gaming, media_cmd, clip_get, clip_set
        ])
        .setup(move |app| {
            let h = app.handle().clone();
            place_island(&h);
            let menu = Menu::with_items(app, &[
                &MenuItem::with_id(app, "settings", "Настройки", true, None::<&str>)?,
                &MenuItem::with_id(app, "diary", "Дневник", true, None::<&str>)?,
                &MenuItem::with_id(app, "study", "Режим учёбы", true, None::<&str>)?,
                &MenuItem::with_id(app, "game", "Игровой режим", true, None::<&str>)?,
                &MenuItem::with_id(app, "focus", "Фокус на 45 минут", true, None::<&str>)?,
                &MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?,
            ])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Островок")
                .menu(&menu)
                .on_menu_event(|app, ev| match ev.id.as_ref() {
                    "settings" => show_settings(app),
                    "diary" => show_window(app, "diary", "diary.html", "Личный дневник: Островок", 1000.0, 680.0),
                    "study" => { let _ = app.emit("hotkey", "study"); }
                    "game" => { launch_gaming(app.clone(), app.state()); }
                    "focus" => { let _ = app.emit("hotkey", "focus"); }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            register_hotkeys(&h);
            spawn_click_through(h.clone());
            spawn_monitor(h.clone());
            spawn_music(h.clone());
            spawn_weather(h.clone());
            spawn_telegram(h);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("ошибка запуска Островка");
}
