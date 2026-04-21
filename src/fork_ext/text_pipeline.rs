use once_cell::sync::Lazy;
use regex::{Captures, Regex};

static RE_TRAILING_W: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(?P<stem>\S+?)w{2,}(?=\s|$)").expect("valid trailing-w regex"));

static RE_REPEAT_GRASS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"草{2,}").expect("valid repeat grass regex"));

/// fork 固有の軽量TTS前処理。
///
/// upstream 既定動作への影響を避けるため、明示的な opt-in 時のみ呼び出す。
pub fn preprocess_tts_text(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let normalized = RE_TRAILING_W.replace_all(input, |caps: &Captures<'_>| {
        format!("{} わら", &caps["stem"])
    });

    RE_REPEAT_GRASS
        .replace_all(&normalized, "くさ")
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::preprocess_tts_text;

    #[test]
    fn trailing_w_is_normalized() {
        assert_eq!(preprocess_tts_text("それなwww"), "それな わら");
    }

    #[test]
    fn repeated_grass_is_normalized() {
        assert_eq!(preprocess_tts_text("草草草"), "くさ");
    }
}
