use async_openai::{Client as OpenAIClient, config::OpenAIConfig};

use crate::{app::config::NimConfig, llm::client::LMClient};

/// Fork-local NVIDIA NIM adapter.
///
/// NIM exposes OpenAI-compatible endpoints, but OpenAI-only built-in tools and
/// Responses reasoning fields are not portable across all NIM deployments.
pub fn build_lm_client(config: &NimConfig) -> Result<LMClient, String> {
    let base_url = config
        .base_url
        .as_deref()
        .ok_or_else(|| "NIM_BASE_URL is not set".to_string())?;
    let api_key = config
        .api_key
        .clone()
        .ok_or_else(|| "NIM_API_KEY is not set".to_string())?;
    let openai_config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(base_url.trim_end_matches('/'));

    Ok(LMClient::new_with_options(
        OpenAIClient::with_config(openai_config),
        false,
        false,
    ))
}
