//! Telegram: привязка по коду, запись и список домашки. Сеть (ureq) отделена от разбора, который проверяется тестами.
use serde_json::{json, Value};
use chrono::NaiveDate;
use crate::hw::{due_label, hw_body, subject_emoji};

pub fn code_matches(user_text: &str, stored: &str) -> bool {
    let up = |s: &str| s.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase();
    let u = up(user_text);
    let raw: String = up(stored).chars().skip(3).collect();
    raw.len() >= 8 && !u.is_empty() && (u == raw || u == format!("OST{raw}"))
}

pub fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }

/// Токен бота = полный доступ к боту; убираем его из любого текста ошибки.
pub fn scrub(text: &str, token: &str) -> String {
    let mut t = text.to_string();
    if !token.is_empty() { t = t.replace(token, "***"); }
    let re = fancy_regex::Regex::new(r"bot\d{6,}:[A-Za-z0-9_-]{20,}").unwrap();
    re.replace_all(&t, "bot***").to_string()
}

pub fn format_list(items: &[Value], today: NaiveDate) -> String {
    let mut lines = vec!["📚 <b>Что задали</b>".to_string()];
    for (i, h) in items.iter().take(40).enumerate() {
        let subj = h["subject"].as_str().unwrap_or("");
        let head = if subj.is_empty() { String::new() } else { format!("<b>{}</b>: ", esc(subj)) };
        let due = due_label(h["due"].as_str().unwrap_or(""), today);
        let tail = if due.is_empty() { String::new() } else { format!("  <i>· {}</i>", esc(&due)) };
        lines.push(format!("{}. {} {}{}{}", i + 1, subject_emoji(subj), head, esc(&hw_body(h)), tail));
    }
    if items.len() > 40 { lines.push(format!("…и ещё {}", items.len() - 40)); }
    lines.join("\n")
}

#[derive(Debug, PartialEq)]
pub enum Cmd { Pair(String), Add(String), List, Unlink, Help, Ignore }

/// Что сделать с входящим сообщением. `paired` — привязан ли уже этот чат.
pub fn classify(text: &str, paired: bool) -> Cmd {
    let t = text.trim();
    if t.is_empty() { return Cmd::Ignore; }
    if !paired { return Cmd::Pair(t.to_string()); }
    match t {
        "/start" | "/help" => Cmd::Help,
        "/unlink" => Cmd::Unlink,
        "/list" | "📋 Что задали" => Cmd::List,
        "✍️ Записать" => Cmd::Help,
        _ if t.starts_with('/') => Cmd::Ignore,
        _ => Cmd::Add(t.to_string()),
    }
}

pub fn api(token: &str, method: &str, body: &Value, timeout_s: u64) -> Result<Value, String> {
    let url = format!("https://api.telegram.org/bot{token}/{method}");
    let r = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(timeout_s)).build()
        .post(&url).send_json(body.clone()).map_err(|e| scrub(&e.to_string(), token))?;
    let v: Value = r.into_json().map_err(|e| scrub(&e.to_string(), token))?;
    if v["ok"].as_bool() == Some(true) { Ok(v["result"].clone()) } else { Err(scrub(&v["description"].to_string(), token)) }
}
pub fn send(token: &str, chat: i64, html: &str) -> Result<(), String> {
    api(token, "sendMessage", &json!({"chat_id": chat, "text": html, "parse_mode": "HTML"}), 15).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codes() {
        assert!(code_matches("ost-abcd-2345", "OST-ABCD-2345"));
        assert!(code_matches("ABCD2345", "OST-ABCD-2345"));
        assert!(!code_matches("OST-ABCD-2346", "OST-ABCD-2345"));
        assert!(!code_matches("", "OST-ABCD-2345"));
        assert!(!code_matches("x", ""));
    }
    #[test]
    fn scrubbing() {
        let tok = "123456789:AAAbbbCCCdddEEEfffGGGhhhIIIjjjKKK";
        let s = scrub(&format!("error https://api.telegram.org/bot{tok}/getMe fail {tok}"), tok);
        assert!(!s.contains("AAAbbb"), "{s}");
    }
    #[test]
    fn commands() {
        assert_eq!(classify("OST-1", false), Cmd::Pair("OST-1".into()));
        assert_eq!(classify("русский стр 3", true), Cmd::Add("русский стр 3".into()));
        assert_eq!(classify("/unlink", true), Cmd::Unlink);
        assert_eq!(classify("📋 Что задали", true), Cmd::List);
        assert_eq!(classify("  ", true), Cmd::Ignore);
    }
    #[test]
    fn list_format() {
        let t = NaiveDate::from_ymd_opt(2026, 11, 18).unwrap();
        let items = vec![json!({"text": "Русский, стр. 34 на завтра", "subject": "Русский", "due": "2026-11-19"})];
        let s = format_list(&items, t);
        assert!(s.contains("<b>Русский</b>: стр. 34") && s.contains("завтра"), "{s}");
        assert_eq!(esc("a<b>&"), "a&lt;b&gt;&amp;");
    }
}
