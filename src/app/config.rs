use crate::llm::prompt::DEFAULT_SYSTEM_PROMPT;

/// 設定
/// まだserdeかいてないのでそのままinlineで記述してる
#[derive(Clone)]
pub struct Config {
    pub discord_token: String,
    pub openai_api_key: String,
    pub system_prompt: String,
    pub rate_limit_window_size: u64,
    pub rate_limit_sec_per_cost: u64,
    pub cron_rate_limit_window_size: u64,
    pub cron_rate_limit_sec_per_run: u64,
    pub admin_users: Vec<u64>,
    pub timeout_millis: u64,
    pub context_compaction_token_limit: u32,
    pub context_summary_max_output_tokens: u32,
    pub voicevox_default_speaker: u32,
    pub voicevox_core_acceleration: String,
    pub voicevox_core_cpu_threads: u16,
    pub voicevox_core_load_all_models: bool,
    pub voicevox_output_sampling_rate: u32,
    pub voicevox_preload_on_startup: bool,
    pub voicevox_open_jtalk_dict_dir: String,
    pub voicevox_vvm_dir: String,
    pub voicevox_onnxruntime_filename: String,
    pub gemini: GeminiConfig,
    pub nim: NimConfig,
    pub monitor_tcp_bind: String,
}

#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub default_model: String,
    pub auto_models: Vec<String>,
    pub enable_google_search: bool,
    pub max_tool_loops: usize,
}

#[derive(Debug, Clone)]
pub struct NimConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
}

impl Config {
    pub fn new() -> Self {
        let discord_token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
        let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
        let system_prompt =
            std::env::var("SYSTEM_PROMPT").unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_owned());
        let context_compaction_token_limit = std::env::var("CONTEXT_COMPACTION_TOKEN_LIMIT")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(64_000);
        let context_summary_max_output_tokens = std::env::var("CONTEXT_SUMMARY_MAX_OUTPUT_TOKENS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(4_096);
        let voicevox_default_speaker = std::env::var("VOICEVOX_DEFAULT_SPEAKER")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3);
        let voicevox_core_acceleration =
            std::env::var("VOICEVOX_CORE_ACCELERATION").unwrap_or_else(|_| "auto".to_string());
        let voicevox_core_cpu_threads = std::env::var("VOICEVOX_CORE_CPU_THREADS")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(0);
        let voicevox_core_load_all_models = std::env::var("VOICEVOX_CORE_LOAD_ALL_MODELS")
            .ok()
            .map(|v| {
                let normalized = v.trim().to_ascii_lowercase();
                matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
            })
            .unwrap_or(false);
        // 24000 の倍数で、8000以上96000以下の値じゃないと怒られるっぽい (VOICEVOXの仕様)
        let voicevox_output_sampling_rate = std::env::var("VOICEVOX_OUTPUT_SAMPLING_RATE")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v >= 8_000 && *v <= 96_000 && *v % 24_000 == 0)
            .unwrap_or(48_000);
        let voicevox_preload_on_startup = std::env::var("VOICEVOX_PRELOAD_ON_STARTUP")
            .ok()
            .map(|v| {
                let normalized = v.trim().to_ascii_lowercase();
                matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
            })
            .unwrap_or(true);
        let voicevox_open_jtalk_dict_dir = std::env::var("VOICEVOX_OPEN_JTALK_DICT_DIR")
            .unwrap_or_else(|_| "voicevox_core/dict/open_jtalk_dic_utf_8-1.11".to_string());
        let voicevox_vvm_dir = std::env::var("VOICEVOX_VVM_DIR")
            .unwrap_or_else(|_| "voicevox_core/models/vvms".to_string());
        let voicevox_onnxruntime_filename =
            std::env::var("VOICEVOX_ONNXRUNTIME_FILENAME").unwrap_or_else(|_| "".to_string());
        let gemini_api_key = std::env::var("GEMINI_API_KEY").ok();
        let gemini_base_url = std::env::var("GEMINI_BASE_URL")
            .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let gemini_default_model =
            std::env::var("GEMINI_DEFAULT_MODEL").unwrap_or_else(|_| "gemini-3.0-flash".to_string());
        let gemini_auto_models = std::env::var("GEMINI_AUTO_MODELS")
            .ok()
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|list| !list.is_empty())
            .unwrap_or_else(|| {
                vec![
                    // fallback order (best quality first)
                    "gemini-3.1-pro".to_string(),
                    "gemini-3.0-pro".to_string(),
                    "gemini-3.0-flash".to_string(),
                    "gemini-2.5-pro".to_string(),
                    "gemini-3.1-flash-lite".to_string(),
                    "gemma-4-31b".to_string(),
                    "gemma-4-26b-a4b".to_string(),
                    "gemini-2.5-flash".to_string(),
                    "gemini-2.5-flash-lite".to_string(),
                    "gemma-4-e4b".to_string(),
                    "gemini-2.0-flash-lite".to_string(),
                    "gemma-4-e2b".to_string(),
                    "gemma-3-27b-it".to_string(),
                    "gemma-3-12b-it".to_string(),
                    "gemma-3-4b-it".to_string(),
                    "gemma-3-1b-it".to_string(),
                ]
            });
        let gemini_enable_google_search = std::env::var("GEMINI_ENABLE_GOOGLE_SEARCH")
            .ok()
            .map(|v| {
                let normalized = v.trim().to_ascii_lowercase();
                matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
            })
            .unwrap_or(false);
        let gemini_max_tool_loops = std::env::var("GEMINI_MAX_TOOL_LOOPS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .map(|v| v.clamp(1, 20))
            .unwrap_or(10);
        let nim_api_key = std::env::var("NIM_API_KEY").ok();
        let nim_base_url = std::env::var("NIM_BASE_URL").ok();
        let nim_default_model = std::env::var("NIM_DEFAULT_MODEL")
            .unwrap_or_else(|_| "nvidia/llama-3.1-nemotron-nano-8b-v1".to_string());
        let monitor_tcp_bind =
            std::env::var("MONITOR_TCP_BIND").unwrap_or_else(|_| "127.0.0.1:9791".to_string());

        Config {
            discord_token,
            openai_api_key,
            system_prompt,
            rate_limit_window_size: 16200,
            rate_limit_sec_per_cost: 600,
            cron_rate_limit_window_size: 3600,
            cron_rate_limit_sec_per_run: 3600,
            admin_users: vec![855371530270408725],
            timeout_millis: 100_000,
            context_compaction_token_limit,
            context_summary_max_output_tokens,
            voicevox_default_speaker,
            voicevox_core_acceleration,
            voicevox_core_cpu_threads,
            voicevox_core_load_all_models,
            voicevox_output_sampling_rate,
            voicevox_preload_on_startup,
            voicevox_open_jtalk_dict_dir,
            voicevox_vvm_dir,
            voicevox_onnxruntime_filename,
            gemini: GeminiConfig {
                api_key: gemini_api_key,
                base_url: gemini_base_url,
                default_model: gemini_default_model,
                auto_models: gemini_auto_models,
                enable_google_search: gemini_enable_google_search,
                max_tool_loops: gemini_max_tool_loops,
            },
            nim: NimConfig {
                api_key: nim_api_key,
                base_url: nim_base_url,
                default_model: nim_default_model,
            },
            monitor_tcp_bind,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
