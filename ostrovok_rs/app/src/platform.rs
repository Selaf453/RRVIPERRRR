//! Всё, что зависит от Windows. На других системах возвращает «нет данных», чтобы код собирался везде.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Music { pub title: String, pub artist: String, pub playing: bool, pub source: String, pub pos: f64, pub dur: f64 }

#[cfg(windows)]
mod imp {
    use super::Music;
    #[repr(C)]
    struct LastInputInfo { cb_size: u32, dw_time: u32 }
    #[repr(C)]
    struct PowerStatus { ac: u8, flag: u8, percent: u8, saver: u8, life: u32, full_life: u32 }
    #[link(name = "user32")]
    extern "system" {
        fn GetLastInputInfo(p: *mut LastInputInfo) -> i32;
        fn GetKeyState(vk: i32) -> i16;
        fn GetForegroundWindow() -> isize;
        fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        fn GetKeyboardLayout(thread: u32) -> isize;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetTickCount() -> u32;
        fn GetLogicalDrives() -> u32;
        fn GetDriveTypeW(root: *const u16) -> u32;
        fn GetSystemPowerStatus(p: *mut PowerStatus) -> i32;
    }

    pub fn idle_seconds() -> f64 {
        unsafe {
            let mut li = LastInputInfo { cb_size: std::mem::size_of::<LastInputInfo>() as u32, dw_time: 0 };
            if GetLastInputInfo(&mut li) == 0 { return 0.0; }
            GetTickCount().wrapping_sub(li.dw_time) as f64 / 1000.0
        }
    }
    pub fn caps_on() -> bool { unsafe { GetKeyState(0x14) & 1 != 0 } }
    pub fn layout() -> String {
        unsafe {
            let hwnd = GetForegroundWindow();
            let tid = if hwnd != 0 { GetWindowThreadProcessId(hwnd, std::ptr::null_mut()) } else { 0 };
            let lang = (GetKeyboardLayout(tid) as usize & 0x3FF) as u32;
            match lang { 0x19 => "RU", 0x09 => "EN", 0x22 => "UA", _ => "" }.to_string()
        }
    }
    pub fn removable_drives() -> Vec<String> {
        let mut out = vec![];
        unsafe {
            let mask = GetLogicalDrives();
            for i in 0..26u32 {
                if mask & (1 << i) == 0 { continue; }
                let letter = (b'A' + i as u8) as char;
                let root: Vec<u16> = format!("{letter}:\\").encode_utf16().chain(std::iter::once(0)).collect();
                if GetDriveTypeW(root.as_ptr()) == 2 { out.push(format!("{letter}:")); }
            }
        }
        out
    }
    /// (процент, подключена ли зарядка); None на ПК без батареи.
    pub fn battery() -> Option<(u8, bool)> {
        unsafe {
            let mut s = PowerStatus { ac: 0, flag: 0, percent: 255, saver: 0, life: 0, full_life: 0 };
            if GetSystemPowerStatus(&mut s) == 0 || s.percent > 100 || s.flag & 128 != 0 { return None; }
            Some((s.percent, s.ac == 1))
        }
    }
    pub fn music() -> Option<Music> {
        use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager as M, GlobalSystemMediaTransportControlsSessionPlaybackStatus as PS};
        let mgr = M::RequestAsync().ok()?.get().ok()?;
        let ses = mgr.GetCurrentSession().ok()?;
        let props = ses.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
        let playing = ses.GetPlaybackInfo().ok()?.PlaybackStatus().ok()? == PS::Playing;
        let title = props.Title().ok()?.to_string();
        if title.is_empty() { return None; }
        let (pos, dur) = ses.GetTimelineProperties().ok()
            .map(|tl| (tl.Position().map(|t| t.Duration as f64 / 1e7).unwrap_or(0.0), tl.EndTime().map(|t| t.Duration as f64 / 1e7).unwrap_or(0.0)))
            .unwrap_or((0.0, 0.0));
        Some(Music { title, artist: props.Artist().map(|a| a.to_string()).unwrap_or_default(), playing,
                     source: ses.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default(), pos, dur })
    }

    /// Управление плеером: prev / next / что угодно другое = пауза-плей.
    pub fn media(cmd: &str) {
        use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as M;
        let Some(mgr) = M::RequestAsync().ok().and_then(|a| a.get().ok()) else { return };
        let Ok(ses) = mgr.GetCurrentSession() else { return };
        let _ = match cmd {
            "prev" => ses.TrySkipPreviousAsync().and_then(|a| a.get()),
            "next" => ses.TrySkipNextAsync().and_then(|a| a.get()),
            _ => ses.TryTogglePlayPauseAsync().and_then(|a| a.get()),
        };
    }

    use std::os::windows::process::CommandExt;
    const NO_WINDOW: u32 = 0x08000000;
    /// Озвучка через встроенный синтезатор Windows (русский голос, если установлен). Текст идёт через переменную окружения, а не в команде.
    pub fn speak(text: &str) {
        let script = "Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; \
            try{ $s.SelectVoiceByHints('NotSet','NotSet',0,[System.Globalization.CultureInfo]'ru-RU') }catch{}; $s.Speak($env:OSTR_SAY)";
        let _ = std::process::Command::new("powershell").args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script])
            .env("OSTR_SAY", text).creation_flags(NO_WINDOW).spawn();
    }
    /// Открывает ссылку в браузере по умолчанию (без оболочки cmd, чтобы в адресе не сработали команды).
    pub fn open_url(url: &str) -> bool {
        if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("steam://")) { return false; }
        std::process::Command::new("explorer.exe").arg(url).creation_flags(NO_WINDOW).spawn().is_ok()
    }
    pub fn launch(path: &str) -> bool { std::process::Command::new(path).creation_flags(NO_WINDOW).spawn().is_ok() }
}

#[cfg(not(windows))]
mod imp {
    use super::Music;
    pub fn idle_seconds() -> f64 { 0.0 }
    pub fn caps_on() -> bool { false }
    pub fn layout() -> String { String::new() }
    pub fn removable_drives() -> Vec<String> { vec![] }
    pub fn battery() -> Option<(u8, bool)> { None }
    pub fn music() -> Option<Music> { None }
    pub fn speak(_t: &str) {}
    pub fn media(_c: &str) {}
    pub fn open_url(_u: &str) -> bool { false }
    pub fn launch(_p: &str) -> bool { false }
}
pub use imp::*;
