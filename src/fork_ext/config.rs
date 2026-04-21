#[derive(Debug, Clone, Default)]
pub struct ForkExtConfig {
    pub enabled: bool,
    pub text_pipeline_enabled: bool,
}

impl ForkExtConfig {
    pub fn from_env() -> Self {
        let enabled = parse_env_bool("FORK_EXT_ENABLED", false);
        let text_pipeline_enabled = parse_env_bool("FORK_EXT_TEXT_PIPELINE_ENABLED", false);

        Self {
            enabled,
            text_pipeline_enabled,
        }
    }
}

fn parse_env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            let normalized = v.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(default)
}
