//! Дневник. Пароль и формат записей совместимы с Python-версией (pbkdf2-sha256, 100 000 итераций).
//! Важно: пароль закрывает экран дневника, а сами записи лежат в config.json открытым текстом (как и раньше).
use chrono::{Duration, NaiveDate};
use rand::Rng;
use serde_json::{json, Map, Value};
use sha2::Sha256;

pub fn hash_pwd(pwd: &str, salt: &str) -> String {
    let mut out = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha256>(pwd.as_bytes(), salt.as_bytes(), 100_000, &mut out);
    out.iter().map(|b| format!("{b:02x}")).collect()
}
fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
pub fn new_salt() -> String {
    const A: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut r = rand::thread_rng();
    (0..16).map(|_| A[r.gen_range(0..A.len())] as char).collect()
}
pub fn has_password(cfg: &Map<String, Value>) -> bool { cfg.get("diary_hash").and_then(|v| v.as_str()).map_or(false, |s| !s.is_empty()) }

/// Создаёт пароль (первый вход). Возвращает текст ошибки, если не подходит.
pub fn create_password(cfg: &mut Map<String, Value>, p1: &str, p2: &str) -> Result<(), String> {
    if p1.is_empty() { return Err("Введи пароль".into()); }
    if p1.chars().count() < 3 { return Err("Пароль короче 3 символов".into()); }
    if p1 != p2 { return Err("Пароли не совпадают".into()); }
    let salt = new_salt();
    cfg.insert("diary_hash".into(), json!(hash_pwd(p1, &salt)));
    cfg.insert("diary_salt".into(), json!(salt));
    Ok(())
}
pub fn verify(cfg: &Map<String, Value>, pwd: &str) -> bool {
    let salt = cfg.get("diary_salt").and_then(|v| v.as_str()).unwrap_or("");
    let hash = cfg.get("diary_hash").and_then(|v| v.as_str()).unwrap_or("");
    !hash.is_empty() && ct_eq(&hash_pwd(pwd, salt), hash)
}

pub fn has_content(e: &Value) -> bool {
    ["highlight", "text"].iter().any(|k| e[*k].as_str().map_or(false, |s| !s.trim().is_empty()))
}

/// Записи по убыванию даты (только непустые).
pub fn list(cfg: &Map<String, Value>) -> Vec<Value> {
    let mut v: Vec<(String, Value)> = cfg.get("diary_entries").and_then(|e| e.as_object()).map(|m| m.iter().filter(|(_, e)| has_content(e)).map(|(k, e)| (k.clone(), e.clone())).collect()).unwrap_or_default();
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v.into_iter().map(|(d, mut e)| { e["date"] = json!(d); e }).collect()
}
pub fn get(cfg: &Map<String, Value>, date: &str) -> Value {
    cfg.get("diary_entries").and_then(|e| e.get(date)).cloned().unwrap_or_else(|| json!({"mood": "😊", "highlight": "", "text": "", "energy": 2}))
}
pub fn save_entry(cfg: &mut Map<String, Value>, date: &str, mood: &str, highlight: &str, text: &str, energy: i64) -> Result<(), String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| "Неверная дата".to_string())?;
    let entries = cfg.entry("diary_entries").or_insert_with(|| json!({}));
    let m = entries.as_object_mut().ok_or("Повреждённые записи")?;
    let e = json!({"mood": mood, "highlight": highlight.trim(), "text": text.trim(), "energy": energy.clamp(0, 4)});
    if has_content(&e) { m.insert(date.to_string(), e); } else { m.remove(date); }
    Ok(())
}
pub fn delete(cfg: &mut Map<String, Value>, date: &str) {
    if let Some(m) = cfg.get_mut("diary_entries").and_then(|e| e.as_object_mut()) { m.remove(date); }
}
/// Серия дней подряд с записью (если сегодня ещё пусто, считаем со вчера).
pub fn streak(cfg: &Map<String, Value>, today: NaiveDate) -> u32 {
    let ents = cfg.get("diary_entries").and_then(|e| e.as_object());
    let has = |d: NaiveDate| ents.and_then(|m| m.get(&d.to_string())).map_or(false, has_content);
    let mut d = today;
    if !has(d) { d -= Duration::days(1); }
    let mut n = 0;
    while has(d) { n += 1; d -= Duration::days(1); }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::merge_defaults;
    #[test]
    fn hash_matches_python() {
        // Значение посчитано hashlib.pbkdf2_hmac("sha256", "пароль123", "abc123xyz0987654", 100000).hex()
        assert_eq!(hash_pwd("пароль123", "abc123xyz0987654"), "81e1b9e018b1d341c9bb82eb6528bb065556b1d2cfe9c0983b2f8e8ce7f7e236");
    }
    #[test]
    fn password_flow() {
        let mut c = merge_defaults(Map::new());
        assert!(!has_password(&c));
        assert!(create_password(&mut c, "ab", "ab").is_err());
        assert!(create_password(&mut c, "abc", "abd").is_err());
        create_password(&mut c, "секрет", "секрет").unwrap();
        assert!(has_password(&c));
        assert!(verify(&c, "секрет"));
        assert!(!verify(&c, "секре"));
        assert!(!verify(&c, ""));
    }
    #[test]
    fn entries_and_streak() {
        let mut c = merge_defaults(Map::new());
        let t = NaiveDate::from_ymd_opt(2026, 11, 18).unwrap();
        save_entry(&mut c, "2026-11-17", "🔥", "Хороший день", "", 3).unwrap();
        save_entry(&mut c, "2026-11-16", "😊", "", "текст", 1).unwrap();
        save_entry(&mut c, "2026-11-14", "😴", "давно", "", 0).unwrap();
        assert_eq!(streak(&c, t), 2); // сегодня пусто, вчера и позавчера есть
        save_entry(&mut c, "2026-11-18", "😊", "сегодня", "", 2).unwrap();
        assert_eq!(streak(&c, t), 3);
        let l = list(&c);
        assert_eq!(l.len(), 4);
        assert_eq!(l[0]["date"], "2026-11-18");
        save_entry(&mut c, "2026-11-18", "😊", "  ", " ", 2).unwrap(); // пустая запись удаляется
        assert_eq!(list(&c).len(), 3);
        assert!(save_entry(&mut c, "не дата", "😊", "x", "", 1).is_err());
        assert_eq!(get(&c, "2026-11-17")["mood"], "🔥");
        assert_eq!(get(&c, "2030-01-01")["text"], "");
        delete(&mut c, "2026-11-17");
        assert_eq!(list(&c).len(), 2);
    }
    #[test]
    fn python_entries_are_readable() { // запись в формате старой версии
        let mut c = merge_defaults(Map::new());
        c.insert("diary_entries".into(), json!({"2026-11-01": {"mood": "🖤", "highlight": "Вайб", "text": "…", "energy": 4}}));
        assert_eq!(list(&c)[0]["mood"], "🖤");
    }
}
