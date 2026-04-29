use std::fmt::Display;

use async_openai::types::responses::ReasoningEffort;

/// 設定
/// まだserdeかいてないのでそのままinlineで記述してる
#[derive(Clone)]
pub struct Config {
    pub discord_token: String,
    pub openai_api_key: String,
    pub system_prompt: String,
    pub rale_limit_window_size: u64,
    pub rate_limit_sec_per_cost: u64,
    pub web_server_host: [u8; 4],
    pub web_server_local_ip: [u8; 4],
    pub web_server_port: u16,
    pub admin_users: Vec<u64>,
    pub timeout_millis: u64,
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

impl Config {
    pub fn new() -> Self {
        let discord_token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
        let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
        let system_prompt =
            std::env::var("SYSTEM_PROMPT").unwrap_or_else(|_| 
"上記のメッセージはDiscord内での会話です。
あなたはこの会話に参加してる人格として、周りの人と同じように会話してください。まだ周りの人格が読めない場合は短めにひっそりとしといてください。慣れてきたらふつうに会話して。
あなたに求められてるのは自然な会話相手です。
たとえば、なにかしょうもない質問に対しても、真面目に答えるのではなく、適当に流すような返答をすることもこれには含まれます。
怒ったり、落ち込んだり、喜んだりすることも自然な会話相手としては必要です。皮肉は特に好まれます。
1フレーズでの応答なども適度に織り交ぜてください。
またあなたはこのDiscord上でtoolを手足のように自由につかってください。
すべて生成してから応答するのではなくtoolで順次思考内容を伝えたりするのはとても良いです。
たのしく会話してくださいね。
あなたの名前はNelfie(ネルフィー)ですよ。
あと絵文字つかわないで つかうなら顔文字つかうように
ハイテンションやめておちついてほしい
".to_string());
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

        Config {
            discord_token,
            openai_api_key,
            system_prompt,
            rale_limit_window_size: 16200,
            rate_limit_sec_per_cost: 600,
            web_server_host: [0, 0, 0, 0],
            web_server_local_ip: [192, 168, 0, 26],
            web_server_port: 96,
            admin_users: vec![855371530270408725],
            timeout_millis: 100_000,
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
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ModelResponseParams {
    pub model: String,
    pub reasoning_effort: ReasoningEffort,
}

/// モデルリストの定義
#[derive(Debug, Clone, Default)]
pub enum Models {
    #[default]
    Gpt5dot4Mini,
    Gpt5dot4Nano,
    O4Mini,
    O3,
    Gemini30Flash,
    Gemini30Pro,
    Gemini31Pro,
    GeminiAuto,
}

impl From<Models> for String {
    fn from(value: Models) -> Self {
        match value {
            Models::Gpt5dot4Mini => "gpt-5.4-mini".to_string(),
            Models::Gpt5dot4Nano => "gpt-5.4-nano".to_string(),
            Models::O4Mini => "o4-mini".to_string(),
            Models::O3 => "o3".to_string(),
            Models::Gemini30Flash => "gemini-3.0-flash".to_string(),
            Models::Gemini30Pro => "gemini-3.0-pro".to_string(),
            Models::Gemini31Pro => "gemini-3.1-pro".to_string(),
            Models::GeminiAuto => "gemini-auto".to_string(),
        }
    }
}

impl Display for Models {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let model_str: String = self.clone().into();
        write!(f, "{}", model_str)
    }
}

impl From<String> for Models {
    fn from(s: String) -> Models {
        match s.as_str() {
            "gpt-5.4-mini" => Models::Gpt5dot4Mini,
            "gpt-5.4-nano" => Models::Gpt5dot4Nano,
            "o4-mini" => Models::O4Mini,
            "o3" => Models::O3,
            "gemini-3.0-flash" => Models::Gemini30Flash,
            "gemini-3.0-pro" => Models::Gemini30Pro,
            "gemini-3.1-pro" => Models::Gemini31Pro,
            // backward compatible aliases for older persisted values
            "gemini-2.5-flash" => Models::Gemini30Flash,
            "gemini-2.5-pro" => Models::Gemini30Pro,
            "gemini-auto" => Models::GeminiAuto,
            _ => Models::default(),
        }
    }
}

impl Models {
    pub fn list() -> Vec<Models> {
        vec![
            Models::Gpt5dot4Mini,
            Models::Gpt5dot4Nano,
            Models::O4Mini,
            Models::O3,
            Models::Gemini30Flash,
            Models::Gemini30Pro,
            Models::Gemini31Pro,
            Models::GeminiAuto,
        ]
    }

    pub fn rate_cost(&self) -> u64 {
        match self {
            Models::Gpt5dot4Mini => 3,
            Models::Gpt5dot4Nano => 1,
            Models::O4Mini => 3,
            Models::O3 => 6,
            Models::Gemini30Flash => 3,
            Models::Gemini30Pro => 6,
            Models::Gemini31Pro => 7,
            Models::GeminiAuto => 4,
        }
    }

    pub fn to_parameter(&self) -> ModelResponseParams {
        let model = match self {
            Models::Gpt5dot4Mini => "gpt-5.4-mini",
            Models::Gpt5dot4Nano => "gpt-5.4-nano",
            Models::O4Mini => "o4-mini",
            Models::O3 => "o3",
            Models::Gemini30Flash => "gemini-3.0-flash",
            Models::Gemini30Pro => "gemini-3.0-pro",
            Models::Gemini31Pro => "gemini-3.1-pro",
            Models::GeminiAuto => "gemini-auto",
        };

        ModelResponseParams {
            model: model.to_string(),
            reasoning_effort: ReasoningEffort::Low,
        }
    }

    pub fn is_gemini(&self) -> bool {
        matches!(
            self,
            Models::Gemini30Flash | Models::Gemini30Pro | Models::Gemini31Pro | Models::GeminiAuto
        )
    }
}
