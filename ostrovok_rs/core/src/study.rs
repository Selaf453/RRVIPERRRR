//! Режим учёбы: таймер на N минут, итог в конце.
#[derive(Debug, Clone, Copy)]
pub struct Study { pub start_ms: i64, pub end_ms: i64 }
impl Study {
    pub fn new(now_ms: i64, minutes: u32) -> Self { Study { start_ms: now_ms, end_ms: now_ms + minutes as i64 * 60_000 } }
    pub fn left_s(&self, now_ms: i64) -> i64 { ((self.end_ms - now_ms) / 1000).max(0) }
    pub fn done(&self, now_ms: i64) -> bool { now_ms >= self.end_ms }
    pub fn studied_min(&self, now_ms: i64) -> i64 { (now_ms.min(self.end_ms) - self.start_ms) / 60_000 }
}
pub fn fmt_mmss(s: i64) -> String { format!("{:02}:{:02}", s / 60, s % 60) }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn study_timer() {
        let s = Study::new(1_000, 50);
        assert_eq!(s.left_s(1_000), 3000);
        assert!(!s.done(1_000 + 49 * 60_000));
        assert!(s.done(1_000 + 50 * 60_000));
        assert_eq!(s.studied_min(1_000 + 20 * 60_000 + 5), 20);
        assert_eq!(s.studied_min(1_000 + 999 * 60_000), 50);
        assert_eq!(fmt_mmss(3000), "50:00");
        assert_eq!(fmt_mmss(65), "01:05");
    }
}
