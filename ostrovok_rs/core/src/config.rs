//! Конфиг в том же формате, что у Python-версии (%APPDATA%\Ostrovok\config.json),
//! поэтому домашка, дневник и привязка Telegram переезжают без потерь.
use rand::Rng;
use serde_json::{json, Map, Value};
use std::{fs, io, path::{Path, PathBuf}};

pub fn defaults() -> Map<String, Value> {
    let v = json!({
        "name": "Глазастик", "island_style": "pill", "accent": "#ff8a3d", "scale": 1.0, "opacity": 0.97, "offset_y": 6,
        "autostart": false, "chatty": 2, "hide_fullscreen": true, "expand_on_hover": true, "follow_cursor": true,
        "show_lyrics": true, "dance": true, "beat_sensitivity": 115, "show_stats_in_games": true,
        "city": "Челябинск", "pomodoro_min": 25, "break_min": 5, "reminder_sound": true,
        "reminders": [], "hotkey_note": "ctrl+alt+n", "hotkey_expand": "ctrl+alt+i",
        "diary_hash": "", "diary_salt": "", "diary_entries": {},
        "onboarded": false, "user_name": "", "last_greet_date": "", "seen_version": "",
        "tg_token": "", "tg_bot_username": "", "tg_chat_id": null, "tg_user_name": "", "tg_offset": 0, "tg_code": "",
        "homework": [], "hw_show_on_start": true, "hw_hourly": true,
        "pc_usb": true, "pc_net": true, "pc_layout": true, "pc_caps": true,
        "pc_break": true, "pc_break_min": 45, "pc_hog": true, "pc_hog_gb": 3.0, "pc_shots": true,
        "sounds": true, "sound_vol": 60,
        // новое в 3.0
        "exams": [], "notes": [], "battery_watch": true, "focus_min": 45,
        "speak": false, "sleep_after_min": 5, "lyrics_offset": 0.0,
        "study_min": 50, "study_playlist": 0, "study_nag": true, "playlists": [{"name": "Мой плейлист для учёбы", "url": "https://music.yandex.ru/"}],
        "gaming_faceit": true, "gaming_cs2": true, "gaming_music": true, "custom_faceit_path": ""
    });
    v.as_object().unwrap().clone()
}

pub fn gen_pair_code() -> String {
    const A: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut r = rand::thread_rng();
    let s: String = (0..8).map(|_| A[r.gen_range(0..A.len())] as char).collect();
    format!("OST-{}-{}", &s[..4], &s[4..])
}

pub fn config_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Ostrovok")
}

/// Дописывает недостающие ключи значениями по умолчанию; чужие ключи не трогает.
pub fn merge_defaults(mut cfg: Map<String, Value>) -> Map<String, Value> {
    for (k, v) in defaults() { cfg.entry(k).or_insert(v); }
    let ok = cfg.get("tg_code").and_then(|v| v.as_str()).map_or(false, |s| s.starts_with("OST-"));
    if !ok { cfg.insert("tg_code".into(), Value::String(gen_pair_code())); }
    cfg
}

pub fn load(dir: &Path) -> Map<String, Value> {
    let p = dir.join("config.json");
    let parsed = fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
    match parsed {
        Some(Value::Object(m)) => merge_defaults(m),
        _ => {
            if p.exists() { // битый файл не затираем молча
                let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                let _ = fs::copy(&p, dir.join(format!("config.broken-{n}.json")));
            }
            merge_defaults(Map::new())
        }
    }
}

pub fn save(dir: &Path, cfg: &Map<String, Value>) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("config.{}.tmp", std::process::id()));
    fs::write(&tmp, serde_json::to_string_pretty(cfg)?)?;
    let dst = dir.join("config.json");
    for i in 0..5 {
        match fs::rename(&tmp, &dst) {
            Ok(()) => return Ok(()),
            Err(_) if i < 4 => std::thread::sleep(std::time::Duration::from_millis(50 * (i + 1))),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merge_keeps_user_keys_and_adds_new() {
        let mut m = Map::new();
        m.insert("city".into(), json!("Омск"));
        m.insert("diary_entries".into(), json!({"2026-11-01": "x"}));
        let c = merge_defaults(m);
        assert_eq!(c["city"], "Омск");
        assert_eq!(c["diary_entries"]["2026-11-01"], "x");
        assert_eq!(c["sound_vol"], 60);
        assert!(c["tg_code"].as_str().unwrap().starts_with("OST-"));
        assert!(c["exams"].is_array());
    }
    #[test]
    fn roundtrip_and_broken_file() {
        let d = std::env::temp_dir().join(format!("ostr_t_{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        let mut c = load(&d);
        c.insert("city".into(), json!("Тюмень"));
        save(&d, &c).unwrap();
        assert_eq!(load(&d)["city"], "Тюмень");
        fs::write(d.join("config.json"), "{битый").unwrap();
        let c2 = load(&d);
        assert_eq!(c2["city"], "Челябинск");
        assert!(fs::read_dir(&d).unwrap().any(|e| e.unwrap().file_name().to_string_lossy().starts_with("config.broken-")));
        let _ = fs::remove_dir_all(&d);
    }
    #[test]
    fn pair_code_shape() {
        let c = gen_pair_code();
        assert_eq!(c.len(), 13);
        assert!(c.starts_with("OST-"));
    }
}
