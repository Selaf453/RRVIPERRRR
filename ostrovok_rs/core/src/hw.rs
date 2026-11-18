//! Домашка: определение предмета и срока из свободного текста (порт функций Python-версии).
use chrono::{Datelike, Duration, Local, NaiveDate};
use fancy_regex::Regex;
use serde_json::{json, Value};

pub struct Subject { pub name: &'static str, pub pat: &'static str, pub color: &'static str, pub emoji: &'static str }

pub const SUBJECTS: &[Subject] = &[
    Subject { name: "Алгебра", pat: "алгебр", color: "#60a5fa", emoji: "📐" },
    Subject { name: "Геометрия", pat: "геометр", color: "#38bdf8", emoji: "📐" },
    Subject { name: "Математика", pat: "матем|матан", color: "#60a5fa", emoji: "📐" },
    Subject { name: "Русский", pat: r"русск|рус\b", color: "#f87171", emoji: "📝" },
    Subject { name: "Литература", pat: "литер|сочинен", color: "#fb923c", emoji: "📖" },
    Subject { name: "Английский", pat: "англ|english", color: "#a78bfa", emoji: "🇬🇧" },
    Subject { name: "Немецкий", pat: "немец", color: "#c084fc", emoji: "🇩🇪" },
    Subject { name: "Французский", pat: "француз", color: "#c084fc", emoji: "🇫🇷" },
    Subject { name: "Физкультура", pat: "физкульт|физра", color: "#fb7185", emoji: "🏃" },
    Subject { name: "Физика", pat: "физик", color: "#2dd4bf", emoji: "⚛️" },
    Subject { name: "Химия", pat: "хими", color: "#4ade80", emoji: "🧪" },
    Subject { name: "Биология", pat: "биолог", color: "#84cc16", emoji: "🧬" },
    Subject { name: "Информатика", pat: "информат|програм|питон|python", color: "#22d3ee", emoji: "💻" },
    Subject { name: "История", pat: "истори", color: "#fbbf24", emoji: "📜" },
    Subject { name: "Обществознание", pat: "обществ", color: "#f59e0b", emoji: "🏛" },
    Subject { name: "География", pat: "географ", color: "#34d399", emoji: "🌍" },
    Subject { name: "ОБЖ", pat: r"обж\b", color: "#f472b6", emoji: "🦺" },
    Subject { name: "Музыка", pat: "музык", color: "#e879f9", emoji: "🎵" },
    Subject { name: "ИЗО", pat: r"изо\b|рисован", color: "#f9a8d4", emoji: "🎨" },
    Subject { name: "Технология", pat: "технолог", color: "#94a3b8", emoji: "🔧" },
];
const WD_SHORT: [&str; 7] = ["пн", "вт", "ср", "чт", "пт", "сб", "вс"];
const WD_PATTERNS: [(&str, i64); 7] = [("понедельник|пн", 0), ("вторник|вт", 1), ("сред|ср", 2), ("четверг|чт", 3), ("пятниц|пт", 4), ("суббот|сб", 5), ("воскресень|вс", 6)];

fn re(p: &str) -> Regex { Regex::new(p).expect("regex") }

pub fn subject_emoji(name: &str) -> &'static str { SUBJECTS.iter().find(|s| s.name == name).map_or("📚", |s| s.emoji) }
pub fn subject_color(name: &str) -> &'static str { SUBJECTS.iter().find(|s| s.name == name).map_or("#c9ccdb", |s| s.color) }

pub fn detect_subject(text: &str) -> String {
    let t = text.to_lowercase();
    for s in SUBJECTS {
        if re(&format!("(?<![а-яёa-z0-9])(?:{})", s.pat)).is_match(&t).unwrap_or(false) { return s.name.to_string(); }
    }
    String::new()
}

/// Срок сдачи, только если он написан явно: «на завтра», «к пятнице», «до 20.11».
pub fn parse_due(text: &str, today: NaiveDate) -> Option<NaiveDate> {
    let t = format!(" {} ", text.to_lowercase());
    let m = |p: &str| re(p).is_match(&t).unwrap_or(false);
    if m("(?<![а-яё])послезавтра(?![а-яё])") { return Some(today + Duration::days(2)); }
    if m("(?<![а-яё])завтра(?![а-яё])") { return Some(today + Duration::days(1)); }
    if m(r"(?<![а-яё])(?:на|к|до)\s+сегодня(?![а-яё])") { return Some(today); }
    if let Ok(Some(c)) = re(r"(?<![а-яё])(?:на|к|до)\s+(\d{1,2})[./](\d{1,2})(?:[./](\d{2,4}))?(?!\d)").captures(&t) {
        let d: u32 = c[1].parse().ok()?;
        let mo: u32 = c[2].parse().ok()?;
        let has_year = c.get(3).is_some();
        let mut y: i32 = c.get(3).map_or(today.year(), |g| g.as_str().parse().unwrap_or(today.year()));
        if y < 100 { y += 2000; }
        if let Some(dd) = NaiveDate::from_ymd_opt(y, mo, d) {
            if !has_year && dd < today { return NaiveDate::from_ymd_opt(y + 1, mo, d).or(Some(dd)); }
            return Some(dd);
        }
    }
    for (pat, wd) in WD_PATTERNS {
        if m(&format!(r"(?<![а-яё])(?:на|к|до)\s+(?:{pat})(?![а-яё]{{3,}})")) {
            let diff = (wd - today.weekday().num_days_from_monday() as i64).rem_euclid(7);
            return Some(today + Duration::days(if diff == 0 { 7 } else { diff }));
        }
    }
    None
}

pub fn due_label(due_iso: &str, today: NaiveDate) -> String {
    let Ok(d) = NaiveDate::parse_from_str(due_iso, "%Y-%m-%d") else { return String::new() };
    let delta = (d - today).num_days();
    match delta {
        i64::MIN..=-1 => "просрочено".into(),
        0 => "сегодня".into(),
        1 => "завтра".into(),
        2..=6 => format!("{} {:02}.{:02}", WD_SHORT[d.weekday().num_days_from_monday() as usize], d.day(), d.month()),
        _ => format!("{:02}.{:02}", d.day(), d.month()),
    }
}

pub fn make_hw(text: &str, existing: &[Value]) -> Value {
    let t = re(r"\s*\n\s*").replace_all(text.trim(), " · ").to_string();
    let t = re(r"[ \t]+").replace_all(&t, " ").to_string();
    let t: String = t.chars().take(400).collect();
    let now = Local::now();
    let due = parse_due(&t, now.date_naive());
    let max = existing.iter().filter_map(|h| h.get("id").and_then(|v| v.as_i64())).max().unwrap_or(0);
    let id = std::cmp::max(now.timestamp_millis(), max + 1);
    json!({"id": id, "subject": detect_subject(&t), "text": t, "due": due.map(|d| d.to_string()).unwrap_or_default(),
           "done": false, "created": now.format("%Y-%m-%dT%H:%M:%S").to_string()})
}

/// Текст без названия предмета в начале и без срока в конце.
pub fn hw_body(item: &Value) -> String {
    let mut text = item["text"].as_str().unwrap_or("").to_string();
    let subj = item["subject"].as_str().unwrap_or("");
    if let Some(s) = SUBJECTS.iter().find(|s| s.name == subj) {
        if let Ok(Some(m)) = re(&format!(r"(?i)^\s*(?:{})[а-яёa-z]*[\s,:.\-—–]*", s.pat)).find(&text) {
            if text.len() > m.end() { text = text[m.end()..].trim().to_string(); }
        }
    }
    if item["due"].as_str().map_or(false, |d| !d.is_empty()) {
        let wd = WD_PATTERNS.iter().map(|(p, _)| *p).collect::<Vec<_>>().join("|");
        let tail = format!(r"(?i)[\s,;.\-—–]*(?:(?:на|к|до)\s+(?:сегодня|завтра|послезавтра|(?:{wd})[а-яё]*|\d{{1,2}}[./]\d{{1,2}}(?:[./]\d{{2,4}})?)|послезавтра|завтра)\s*$");
        let cut = re(&tail).replace(&text, "").trim().to_string();
        if !cut.is_empty() { text = cut; }
    }
    text
}

pub fn is_due_today_or_late(item: &Value, today: NaiveDate) -> bool {
    if item["done"].as_bool().unwrap_or(false) { return false; }
    match item["due"].as_str().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()) {
        Some(d) => d <= today,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(y: i32, m: u32, dd: u32) -> NaiveDate { NaiveDate::from_ymd_opt(y, m, dd).unwrap() }
    #[test]
    fn subjects() {
        assert_eq!(detect_subject("Русский, стр. 34 на завтра"), "Русский");
        assert_eq!(detect_subject("алгебра №12"), "Алгебра");
        assert_eq!(detect_subject("прочитать главу"), "");
        assert_eq!(detect_subject("информатика: питон"), "Информатика");
    }
    #[test]
    fn due_parsing() {
        let t = d(2026, 11, 18); // среда
        assert_eq!(parse_due("русский на завтра", t), Some(d(2026, 11, 19)));
        assert_eq!(parse_due("к послезавтра", t), Some(d(2026, 11, 20)));
        assert_eq!(parse_due("физика на сегодня", t), Some(t));
        assert_eq!(parse_due("до 20.11", t), Some(d(2026, 11, 20)));
        assert_eq!(parse_due("к 01.11", t), Some(d(2027, 11, 1)));
        assert_eq!(parse_due("к пятнице", t), Some(d(2026, 11, 20)));
        assert_eq!(parse_due("на среду", t), Some(d(2026, 11, 25)));
        assert_eq!(parse_due("просто текст", t), None);
        assert_eq!(parse_due("на 31.02", t), None);
    }
    #[test]
    fn labels() {
        let t = d(2026, 11, 18);
        assert_eq!(due_label("2026-11-17", t), "просрочено");
        assert_eq!(due_label("2026-11-18", t), "сегодня");
        assert_eq!(due_label("2026-11-19", t), "завтра");
        assert_eq!(due_label("2026-11-20", t), "пт 20.11");
        assert_eq!(due_label("2026-12-05", t), "05.12");
        assert_eq!(due_label("", t), "");
    }
    #[test]
    fn body_cleanup() {
        let h = json!({"text": "Русский, стр. 34 упр. 5 на завтра", "subject": "Русский", "due": "2026-11-19"});
        assert_eq!(hw_body(&h), "стр. 34 упр. 5");
    }
    #[test]
    fn make_and_today() {
        let a = make_hw("алгебра  №12 \n №14", &[]);
        assert_eq!(a["subject"], "Алгебра");
        assert_eq!(a["text"], "алгебра №12 · №14");
        let b = make_hw("физика", &[a.clone()]);
        assert!(b["id"].as_i64() > a["id"].as_i64().map(|x| x - 1));
        let t = d(2026, 11, 18);
        assert!(is_due_today_or_late(&json!({"due": "2026-11-18", "done": false}), t));
        assert!(is_due_today_or_late(&json!({"due": "2026-11-10", "done": false}), t));
        assert!(!is_due_today_or_late(&json!({"due": "2026-11-19", "done": false}), t));
        assert!(!is_due_today_or_late(&json!({"due": "2026-11-18", "done": true}), t));
        assert!(!is_due_today_or_late(&json!({"due": "", "done": false}), t));
    }
}
