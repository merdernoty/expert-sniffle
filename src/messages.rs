use std::env;

pub const START_MESSAGE_PREFIX: &str = "\
👋 Добро пожаловать!
Чтобы получить Гайд « 7 завтраков для похудения», подпишись на канал";
pub const START_MESSAGE_SUFFIX: &str = "После подписки нажмите кнопку \"Готово\".";

pub const DEFAULT_GUIDE_MESSAGE: &str = "\
Спасибо! Вот инструкция 👇

🎥 Видео-гайд:
https://youtube.com/...";

pub const DONE_BUTTON_TEXT: &str = "Готово";
pub const DONE_CALLBACK_DATA: &str = "done";

pub const NOT_SUBSCRIBED_MESSAGE: &str = "\
Вы ещё не подписались. Подпишитесь на каналы и нажмите \"Готово\".";

pub const REQUIRED_CHANNELS: &[&str] = &[];

pub const CHECK_FAILED_MESSAGE: &str = "\
Не могу проверить подписку. Добавьте бота администратором каналов и попробуйте ещё раз.";

pub fn guide_message() -> String {
    env::var("GUIDE_MESSAGE")
        .map(|val| val.replace("\\n", "\n"))
        .unwrap_or_else(|_| DEFAULT_GUIDE_MESSAGE.to_string())
}

pub fn required_channels() -> Vec<String> {
    if let Ok(val) = env::var("REQUIRED_CHANNELS") {
        let channels: Vec<String> = val
            .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
            .map(|c| {
                c.trim_matches(|ch| ch == '\"' || ch == '\'' || ch == '\r' || ch == '\u{feff}')
                    .trim()
            })
            .filter(|c| !c.is_empty())
            .map(|c| {
                if c.starts_with('@') {
                    c.to_string()
                } else {
                    format!("@{c}")
                }
            })
            .collect();
        if !channels.is_empty() {
            return channels;
        }
    }
    REQUIRED_CHANNELS.iter().map(|c| c.to_string()).collect()
}

pub fn start_message() -> String {
    let channels = required_channels();
    let list = if channels.is_empty() {
        String::from("• (каналы не заданы)")
    } else {
        channels
            .iter()
            .map(|c| format!("https://t.me/{}", c.trim_start_matches('@')))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!("{START_MESSAGE_PREFIX}\n{list}\n{START_MESSAGE_SUFFIX}")
}
