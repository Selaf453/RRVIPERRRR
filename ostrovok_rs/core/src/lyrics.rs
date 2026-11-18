//! Тексты песен с синхронизацией по времени (lrclib.net, как в Python-версии).
use serde_json::Value;

pub fn parse_lrc(text: &str) -> Vec<(f64, String)> {
    let mut out = vec![];
    for line in text.lines() {
        let mut rest = line.trim_start();
        let mut stamps = vec![];
        while rest.starts_with('[') {
            let Some(end) = rest.find(']') else { break };
            let tag = &rest[1..end];
            if let Some((m, s)) = tag.split_once(':') {
                if let (Ok(m), Ok(s)) = (m.parse::<f64>(), s.parse::<f64>()) { stamps.push(m * 60.0 + s); }
            }
            rest = &rest[end + 1..];
        }
        let body = rest.trim().to_string();
        for t in stamps { out.push((t, body.clone())); }
    }
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    out
}

/// Строка, которая должна звучать в момент `pos` (с поправкой `offset` секунд).
pub fn current_line(lines: &[(f64, String)], pos: f64, offset: f64) -> Option<&str> {
    let p = pos + offset;
    lines.iter().rev().find(|(t, _)| *t <= p).map(|(_, s)| s.as_str()).filter(|s| !s.is_empty())
}

/// Название без скобок «(feat. …)», исполнитель только первый.
pub fn clean(title: &str, artist: &str) -> (String, String) {
    let mut t = String::new();
    let mut depth = 0;
    for c in title.chars() { match c { '(' | '[' => depth += 1, ')' | ']' if depth > 0 => depth -= 1, _ if depth == 0 => t.push(c), _ => {} } }
    let a = artist.split(|c| c == ',' || c == '&').next().unwrap_or("");
    let a = a.split(" feat").next().unwrap_or(a).split(" ft").next().unwrap_or(a);
    (t.trim().to_string(), a.trim().to_string())
}

pub fn fetch(title: &str, artist: &str, duration: f64) -> Vec<(f64, String)> {
    let (t, a) = clean(title, artist);
    let get = |path: &str, with_dur: bool| -> Option<Value> {
        let mut r = ureq::get(&format!("https://lrclib.net/api/{path}")).set("User-Agent", "Ostrovok/3.0").timeout(std::time::Duration::from_secs(8)).query("track_name", &t).query("artist_name", &a);
        if with_dur && duration > 0.0 { r = r.query("duration", &(duration.round() as i64).to_string()); }
        r.call().ok()?.into_json().ok()
    };
    if let Some(v) = get("get", true) { if let Some(s) = v["syncedLyrics"].as_str() { return parse_lrc(s); } }
    if let Some(Value::Array(a)) = get("search", false) { for it in a { if let Some(s) = it["syncedLyrics"].as_str() { return parse_lrc(s); } } }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lrc() {
        let l = parse_lrc("[ti:x]\n[00:12.50] Первая\n[00:20.00][01:00.10]Повтор\n[00:30.00]\nмусор");
        assert_eq!(l.len(), 4);
        assert_eq!(l[0], (12.5, "Первая".to_string()));
        assert_eq!(current_line(&l, 15.0, 0.0), Some("Первая"));
        assert_eq!(current_line(&l, 21.0, 0.0), Some("Повтор"));
        assert_eq!(current_line(&l, 35.0, 0.0), None); // пустая строка = пауза
        assert_eq!(current_line(&l, 5.0, 0.0), None);
        assert_eq!(current_line(&l, 11.0, 2.0), Some("Первая")); // поправка вперёд
    }
    #[test]
    fn cleaning() {
        assert_eq!(clean("Villain (feat. X) [Remix]", "9mice, Kai Angel"), ("Villain".to_string(), "9mice".to_string()));
        assert_eq!(clean("Song", "A & B"), ("Song".to_string(), "A".to_string()));
        assert_eq!(clean("Song", "A feat. B").1, "A");
    }
}
