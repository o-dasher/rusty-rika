use crate::utils::markdown::bold;

use super::emojis::RikaMoji;

pub fn cool_text(emoji: RikaMoji, text: &str) -> String {
    bold(format!("{} | {}", emoji, text))
}

