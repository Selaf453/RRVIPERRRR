//! Игры и приложения: определение по имени процесса и заголовку, как в Python-версии.
use serde_json::{json, Value};

pub fn default_apps() -> Value {
    json!([
        {"name": "Counter-Strike 2", "exe": ["cs2.exe", "csgo.exe"], "title_kw": [], "game": true, "phrases": ["Опа, CS2! Давай без тильта", "Хедшоты сами себя не сделают", "Удачи в катке, я болею за тебя", "Смотри радар, ну пожалуйста", "Не забывай про гранаты"]},
        {"name": "Valorant", "exe": ["VALORANT-Win64-Shipping.exe"], "title_kw": [], "game": true, "phrases": ["Валорант! Покажи им тактику", "Спайк сам себя не поставит"]},
        {"name": "Dota 2", "exe": ["dota2.exe"], "title_kw": [], "game": true, "phrases": ["Дота... держись там", "Вард сам себя не поставит"]},
        {"name": "Minecraft", "exe": ["Minecraft.Windows.exe", "javaw.exe"], "title_kw": ["Minecraft"], "game": true, "phrases": ["Майнкрафт! Построишь мне домик?"]},
        {"name": "Unity", "exe": ["Unity.exe"], "title_kw": [], "game": false, "phrases": ["Unity открыл? Ну что, опять баги ловить", "Не забудь сохранить сцену!"]},
        {"name": "VS Code", "exe": ["Code.exe"], "title_kw": [], "game": false, "phrases": ["Кодим? Я подержу кофе", "Работает — не трогай"]},
        {"name": "Discord", "exe": ["Discord.exe"], "title_kw": [], "game": false, "phrases": ["Дискорд? Передавай всем привет"]}
    ])
}

/// Приложение из списка, которому соответствует процесс (exe) или заголовок окна.
pub fn match_app<'a>(apps: &'a [Value], exe: &str, title: &str) -> Option<&'a Value> {
    let (e, t) = (exe.to_lowercase(), title.to_lowercase());
    apps.iter().find(|a| {
        let exes = a["exe"].as_array().map(|v| v.iter().filter_map(|x| x.as_str()).any(|x| !x.is_empty() && x.to_lowercase() == e)).unwrap_or(false);
        let kws = a["title_kw"].as_array().map(|v| v.iter().filter_map(|x| x.as_str()).any(|x| !x.is_empty() && t.contains(&x.to_lowercase()))).unwrap_or(false);
        (!e.is_empty() && exes) || kws
    })
}

/// Среди запущенных процессов находит первую игру из списка (только с game=true).
pub fn running_game<'a>(apps: &'a [Value], procs: &[String]) -> Option<&'a Value> {
    apps.iter().filter(|a| a["game"].as_bool().unwrap_or(false)).find(|a| {
        a["exe"].as_array().map(|v| v.iter().filter_map(|x| x.as_str()).any(|x| procs.iter().any(|p| p.eq_ignore_ascii_case(x)))).unwrap_or(false)
    })
}

/// Игровой режим: какие процессы уже запущены.
pub fn any_running(procs: &[String], names: &[&str]) -> bool { names.iter().any(|n| procs.iter().any(|p| p.eq_ignore_ascii_case(n))) }

pub fn short_name(full: &str) -> String {
    match full { "Counter-Strike 2" => "CS2".into(), other => other.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matching() {
        let apps = default_apps(); let apps = apps.as_array().unwrap();
        assert_eq!(match_app(apps, "CS2.EXE", "").unwrap()["name"], "Counter-Strike 2");
        assert_eq!(match_app(apps, "javaw.exe", "Minecraft 1.20").unwrap()["name"], "Minecraft");
        assert_eq!(match_app(apps, "", "Minecraft").unwrap()["name"], "Minecraft");
        assert!(match_app(apps, "notepad.exe", "Блокнот").is_none());
    }
    #[test]
    fn running() {
        let apps = default_apps(); let apps = apps.as_array().unwrap();
        let procs = vec!["explorer.exe".to_string(), "Code.exe".to_string(), "cs2.exe".to_string()];
        assert_eq!(running_game(apps, &procs).unwrap()["name"], "Counter-Strike 2");
        assert!(running_game(apps, &["Code.exe".to_string()]).is_none()); // VS Code не игра
        assert!(any_running(&procs, &["FACEITClient.exe", "cs2.exe"]));
        assert_eq!(short_name("Counter-Strike 2"), "CS2");
    }
}
