//! Решения острова: когда напомнить про ДЗ, про перерыв, про батарею, про ЕГЭ; режим фокуса.
use chrono::NaiveDate;
use serde_json::Value;
use crate::hw::is_due_today_or_late;

/// Напоминание о ДЗ раз в час: только если есть задание на сегодня или просроченное.
pub fn hourly_hw_due(items: &[Value], today: NaiveDate, now_ms: i64, last_ms: i64, enabled: bool) -> bool {
    enabled && now_ms - last_ms >= 3_600_000 && items.iter().any(|h| is_due_today_or_late(h, today))
}

/// Перерыв: сколько минут подряд есть активность. Пауза ввода больше idle_reset_s обнуляет счётчик.
#[derive(Default, Debug)]
pub struct BreakTracker { pub active_s: f64 }
impl BreakTracker {
    pub fn step(&mut self, dt_s: f64, idle_s: f64, idle_reset_s: f64, limit_min: f64) -> bool {
        if idle_s >= idle_reset_s { self.active_s = 0.0; return false; }
        self.active_s += dt_s;
        if self.active_s >= limit_min * 60.0 { self.active_s = 0.0; return true; }
        false
    }
}

// ───────── новое: отсчёт до ЕГЭ / контрольной ─────────
#[derive(Debug, PartialEq)]
pub enum ExamState { Future(i64), Today, Past }
pub fn exam_state(date: &str, today: NaiveDate) -> Option<ExamState> {
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let n = (d - today).num_days();
    Some(if n > 0 { ExamState::Future(n) } else if n == 0 { ExamState::Today } else { ExamState::Past })
}
pub fn plural_days(n: i64) -> &'static str {
    let (a, b) = (n % 10, n % 100);
    if (11..=14).contains(&b) { "дней" } else if a == 1 { "день" } else if (2..=4).contains(&a) { "дня" } else { "дней" }
}
/// Ближайший экзамен: (название, текст для острова). «Сегодня» важнее будущих.
pub fn nearest_exam(exams: &[Value], today: NaiveDate) -> Option<(String, String, i64)> {
    let mut best: Option<(i64, String)> = None;
    for e in exams {
        let (Some(name), Some(date)) = (e["name"].as_str(), e["date"].as_str()) else { continue };
        match exam_state(date, today) {
            Some(ExamState::Future(n)) if best.as_ref().map_or(true, |b| n < b.0) => best = Some((n, name.to_string())),
            Some(ExamState::Today) => best = Some((0, name.to_string())),
            _ => {}
        }
    }
    best.map(|(n, name)| {
        let txt = if n == 0 { format!("{name}: сегодня! Всё получится") } else { format!("{name}: {n} {}", plural_days(n)) };
        (name, txt, n)
    })
}

// ───────── новое: батарея ─────────
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BatteryEvent { PluggedIn, Unplugged, Low20, Low10, Full }
#[derive(Default)]
pub struct BatteryWatch { prev_ac: Option<bool>, warned20: bool, warned10: bool, full_said: bool }
impl BatteryWatch {
    pub fn step(&mut self, percent: u8, ac: bool) -> Option<BatteryEvent> {
        let prev = self.prev_ac.replace(ac);
        if ac { self.warned20 = false; self.warned10 = false; } else { self.full_said = false; }
        if prev.is_some() && prev != Some(ac) { return Some(if ac { BatteryEvent::PluggedIn } else { BatteryEvent::Unplugged }); }
        if !ac && percent <= 10 && !self.warned10 { self.warned10 = true; self.warned20 = true; return Some(BatteryEvent::Low10); }
        if !ac && percent <= 20 && !self.warned20 { self.warned20 = true; return Some(BatteryEvent::Low20); }
        if ac && percent >= 100 && !self.full_said { self.full_said = true; return Some(BatteryEvent::Full); }
        None
    }
}

// ───────── новое: режим фокуса ─────────
#[derive(Default, Debug)]
pub struct Focus { pub until_ms: Option<i64>, pub muted: Vec<String> }
impl Focus {
    pub fn start(&mut self, now_ms: i64, minutes: u32) { self.until_ms = Some(now_ms + minutes as i64 * 60_000); self.muted.clear(); }
    pub fn active(&self, now_ms: i64) -> bool { self.until_ms.map_or(false, |u| now_ms < u) }
    pub fn left_s(&self, now_ms: i64) -> i64 { self.until_ms.map_or(0, |u| ((u - now_ms) / 1000).max(0)) }
    /// true = показать сейчас; false = придержать до конца фокуса.
    pub fn allow(&mut self, now_ms: i64, title: &str, urgent: bool) -> bool {
        if !self.active(now_ms) || urgent { return true; }
        if self.muted.len() < 30 { self.muted.push(title.to_string()); }
        false
    }
    /// Вызвать, когда время вышло: вернёт список пропущенного и выключит фокус.
    pub fn finish(&mut self, now_ms: i64) -> Option<Vec<String>> {
        match self.until_ms { Some(u) if now_ms >= u => { self.until_ms = None; Some(std::mem::take(&mut self.muted)) } _ => None }
    }
    pub fn cancel(&mut self) -> Vec<String> { self.until_ms = None; std::mem::take(&mut self.muted) }
}

// ───────── события ПК ─────────
/// Интернет: сообщаем о смене состояния только после двух одинаковых проверок подряд (без дребезга).
#[derive(Default)]
pub struct NetWatch { state: Option<bool>, pending: Option<bool>, down_since_ms: Option<i64> }
#[derive(Debug, PartialEq)]
pub enum NetEvent { Lost, Back { down_s: i64 } }
impl NetWatch {
    pub fn step(&mut self, alive: bool, now_ms: i64) -> Option<NetEvent> {
        let Some(cur) = self.state else { self.state = Some(alive); return None };
        if alive == cur { self.pending = None; return None; }
        if self.pending != Some(alive) { self.pending = Some(alive); return None; }
        self.pending = None; self.state = Some(alive);
        if !alive { self.down_since_ms = Some(now_ms); Some(NetEvent::Lost) }
        else { let s = self.down_since_ms.take().map_or(0, |t| (now_ms - t) / 1000); Some(NetEvent::Back { down_s: s }) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn d(y: i32, m: u32, dd: u32) -> NaiveDate { NaiveDate::from_ymd_opt(y, m, dd).unwrap() }
    const H: i64 = 3_600_000;
    #[test]
    fn hourly_only_when_homework_today() {
        let t = d(2026, 11, 18);
        let today = vec![json!({"due": "2026-11-18", "done": false})];
        let tomorrow = vec![json!({"due": "2026-11-19", "done": false})];
        assert!(hourly_hw_due(&today, t, 2 * H, 0, true));
        assert!(!hourly_hw_due(&today, t, H - 1, 0, true));
        assert!(!hourly_hw_due(&tomorrow, t, 5 * H, 0, true));
        assert!(!hourly_hw_due(&today, t, 5 * H, 0, false));
        assert!(!hourly_hw_due(&[], t, 5 * H, 0, true));
    }
    #[test]
    fn break_tracker() {
        let mut b = BreakTracker::default();
        let mut fired = 0;
        for _ in 0..(45 * 60) { if b.step(1.0, 0.0, 300.0, 45.0) { fired += 1; } }
        assert_eq!(fired, 1);
        for _ in 0..(40 * 60) { b.step(1.0, 0.0, 300.0, 45.0); }
        assert!(!b.step(1.0, 400.0, 300.0, 45.0)); // отошёл от ПК: счётчик сброшен
        assert_eq!(b.active_s, 0.0);
    }
    #[test]
    fn exams() {
        let t = d(2026, 11, 18);
        assert_eq!(exam_state("2026-11-18", t), Some(ExamState::Today));
        assert_eq!(exam_state("2026-11-21", t), Some(ExamState::Future(3)));
        assert_eq!(exam_state("2026-11-01", t), Some(ExamState::Past));
        assert_eq!(exam_state("мусор", t), None);
        assert_eq!((plural_days(1), plural_days(2), plural_days(5), plural_days(11), plural_days(21), plural_days(22)), ("день", "дня", "дней", "дней", "день", "дня"));
        let ex = vec![json!({"name": "ЕГЭ информатика", "date": "2027-06-10"}), json!({"name": "Контрольная", "date": "2026-11-21"}), json!({"name": "Старое", "date": "2026-01-01"})];
        let (n, txt, days) = nearest_exam(&ex, t).unwrap();
        assert_eq!((n.as_str(), txt.as_str(), days), ("Контрольная", "Контрольная: 3 дня", 3));
        let ex2 = vec![json!({"name": "Диктант", "date": "2026-11-18"})];
        assert!(nearest_exam(&ex2, t).unwrap().1.contains("сегодня"));
        assert!(nearest_exam(&[], t).is_none());
    }
    #[test]
    fn battery() {
        let mut w = BatteryWatch::default();
        assert_eq!(w.step(80, false), None);
        assert_eq!(w.step(50, false), None);
        assert_eq!(w.step(20, false), Some(BatteryEvent::Low20));
        assert_eq!(w.step(19, false), None);
        assert_eq!(w.step(10, false), Some(BatteryEvent::Low10));
        assert_eq!(w.step(9, false), None);
        assert_eq!(w.step(9, true), Some(BatteryEvent::PluggedIn));
        assert_eq!(w.step(100, true), Some(BatteryEvent::Full));
        assert_eq!(w.step(100, true), None);
        assert_eq!(w.step(100, false), Some(BatteryEvent::Unplugged));
        assert_eq!(w.step(15, false), Some(BatteryEvent::Low20)); // после зарядки предупреждения снова работают
    }
    #[test]
    fn focus() {
        let mut f = Focus::default();
        assert!(f.allow(0, "x", false));
        f.start(1000, 25);
        assert!(f.active(2000));
        assert!(!f.allow(2000, "Скриншот сохранён", false));
        assert!(f.allow(2000, "Пожар!", true));
        assert!(!f.allow(3000, "Раскладка: EN", false));
        assert_eq!(f.left_s(1000 + 25 * 60_000 - 5000), 5);
        assert_eq!(f.finish(500), None);
        let missed = f.finish(1000 + 25 * 60_000).unwrap();
        assert_eq!(missed, vec!["Скриншот сохранён", "Раскладка: EN"]);
        assert!(!f.active(1000 + 26 * 60_000));
    }
    #[test]
    fn net_debounce() {
        let mut n = NetWatch::default();
        assert_eq!(n.step(true, 0), None);
        assert_eq!(n.step(false, 1000), None); // один сбой не считается
        assert_eq!(n.step(true, 2000), None);
        assert_eq!(n.step(false, 3000), None);
        assert_eq!(n.step(false, 4000), Some(NetEvent::Lost));
        assert_eq!(n.step(true, 100_000), None);
        assert_eq!(n.step(true, 101_000), Some(NetEvent::Back { down_s: 97 }));
    }
}
