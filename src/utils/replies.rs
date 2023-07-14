use super::emojis::RikaMoji;

pub fn cool_text(emoji: RikaMoji, text: &str) -> String {
    format!("{} | {}", emoji, text)
}

