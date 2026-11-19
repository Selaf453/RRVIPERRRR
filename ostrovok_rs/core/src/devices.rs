//! Заряд беспроводных устройств: протокол мышек ATK/VXE по 2.4G-донглу.
//! Кадр 17 байт, report id 0x08, команда 0x04 = «данные питания»,
//! ответ: заряд в байте 6, флаг «на кабеле» в байте 7, мВ в байтах 8-9, контрольная сумма в байте 16.
//! (по открытым проектам HaloBattery / python-pulsar-mouse-tool; на реальной R1 Pro не проверено)

pub const FRAME_LEN: usize = 17;
pub const REPORT_ID: u8 = 0x08;
pub const CMD_POWER: u8 = 0x04;

#[derive(Debug, Clone, PartialEq)]
pub struct Battery { pub percent: u8, pub charging: bool, pub mv: u16 }

pub fn checksum(frame: &[u8]) -> u8 {
    0x55u8.wrapping_sub(frame.iter().take(FRAME_LEN - 1).fold(0u8, |a, b| a.wrapping_add(*b)))
}

/// Кадр-запрос «данные питания».
pub fn power_request() -> [u8; FRAME_LEN] {
    let mut f = [0u8; FRAME_LEN];
    f[0] = REPORT_ID; f[1] = CMD_POWER;
    f[FRAME_LEN - 1] = checksum(&f);
    f
}

/// Разбор ответа. None — это не наш ответ (другое событие, битая сумма, невозможный заряд).
pub fn parse_power(buf: &[u8]) -> Option<Battery> {
    if buf.len() < FRAME_LEN || buf[0] != REPORT_ID || buf[1] != CMD_POWER { return None; }
    if buf[FRAME_LEN - 1] != checksum(buf) { return None; }
    let p = buf[6];
    if p == 0 || p > 100 { return None; } // 0 = мышь спит, а не пустая
    Some(Battery { percent: p, charging: buf[7] != 0, mv: u16::from_be_bytes([buf[8], buf[9]]) })
}

pub fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect::<Vec<_>>().join(" ") }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_is_valid() {
        let f = power_request();
        assert_eq!(f[0], 0x08); assert_eq!(f[1], 0x04);
        assert_eq!(f.iter().fold(0u8, |a, b| a.wrapping_add(*b)), 0x55);
    }
    #[test]
    fn parses_good_reply() {
        let mut r = [0u8; FRAME_LEN]; r[0] = 0x08; r[1] = 0x04; r[6] = 87; r[7] = 1; r[8] = 0x0f; r[9] = 0x30; r[16] = checksum(&r);
        assert_eq!(parse_power(&r), Some(Battery { percent: 87, charging: true, mv: 3888 }));
    }
    #[test]
    fn refuses_bad_replies() {
        let mut r = [0u8; FRAME_LEN]; r[0] = 0x08; r[1] = 0x04; r[6] = 50; r[16] = checksum(&r);
        r[16] ^= 1; assert!(parse_power(&r).is_none());           // битая сумма
        let mut e = [0u8; FRAME_LEN]; e[0] = 0x08; e[1] = 0x0a; e[16] = checksum(&e); assert!(parse_power(&e).is_none()); // событие, не ответ
        let mut z = [0u8; FRAME_LEN]; z[0] = 0x08; z[1] = 0x04; z[6] = 0; z[16] = checksum(&z); assert!(parse_power(&z).is_none()); // спит
        let mut h = [0u8; FRAME_LEN]; h[0] = 0x08; h[1] = 0x04; h[6] = 150; h[16] = checksum(&h); assert!(parse_power(&h).is_none());
    }
}
