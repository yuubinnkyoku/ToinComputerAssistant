use std::sync::Arc;

use super::{config::ForkExtConfig, text_pipeline::preprocess_tts_text};

pub trait TtsTextPreprocessor: Send + Sync {
    fn preprocess(&self, input: &str) -> String;
}

#[derive(Default)]
pub struct NoopTtsTextPreprocessor;

impl TtsTextPreprocessor for NoopTtsTextPreprocessor {
    fn preprocess(&self, input: &str) -> String {
        input.to_string()
    }
}

#[derive(Default)]
pub struct ForkTtsTextPreprocessor;

impl TtsTextPreprocessor for ForkTtsTextPreprocessor {
    fn preprocess(&self, input: &str) -> String {
        preprocess_tts_text(input)
    }
}

pub fn build_tts_text_preprocessor(config: &ForkExtConfig) -> Arc<dyn TtsTextPreprocessor> {
    if config.enabled && config.text_pipeline_enabled {
        Arc::new(ForkTtsTextPreprocessor)
    } else {
        Arc::new(NoopTtsTextPreprocessor)
    }
}

#[cfg(test)]
mod tests {
    use super::{ForkTtsTextPreprocessor, NoopTtsTextPreprocessor, TtsTextPreprocessor};

    #[test]
    fn noop_keeps_text() {
        let p = NoopTtsTextPreprocessor;
        assert_eq!(p.preprocess("abc"), "abc");
    }

    #[test]
    fn fork_preprocessor_applies_pipeline() {
        let p = ForkTtsTextPreprocessor;
        assert_eq!(p.preprocess("草草"), "くさ");
    }
}
