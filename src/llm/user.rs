use std::{fs, path::PathBuf};

use dashmap::DashMap;
use log::{error, warn};
use serde_json::{Value, json};
use serenity::all::UserId;

use crate::app::config::Models;

const USER_CONTEXTS_STORE_PATH: &str = "data/runtime/user_contexts.json";

/// ユーザー情報のプール
pub struct UserContexts {
    pub contexts: DashMap<UserId, UserContext>,
    store_path: PathBuf,
}

/// ユーザー情報の構造体
#[derive(Clone)]
pub struct UserContext {
    pub user_id: UserId,
    pub main_model: Models,
    pub rate_line: u64,
    pub voice_speaker: Option<u32>,
    pub voice_speed_scale: Option<f32>,
    pub voice_pitch_scale: Option<f32>,
    pub voice_pan: Option<f32>,
}

impl UserContext {
    pub fn new(user_id: UserId) -> UserContext {
        UserContext {
            user_id,
            main_model: Models::default(),
            rate_line: 1,
            voice_speaker: None,
            voice_speed_scale: None,
            voice_pitch_scale: None,
            voice_pan: None,
        }
    }
}

impl UserContexts {
    pub fn new() -> UserContexts {
        let mut user_contexts = UserContexts {
            contexts: DashMap::new(),
            store_path: PathBuf::from(USER_CONTEXTS_STORE_PATH),
        };
        user_contexts.load_from_disk();
        user_contexts
    }

    pub fn get_or_create(&self, user_id: UserId) -> UserContext {
        match self.contexts.entry(user_id) {
            dashmap::mapref::entry::Entry::Occupied(entry) => entry.get().clone(),
            dashmap::mapref::entry::Entry::Vacant(vacant) => {
                let ctx = UserContext::new(user_id);
                let out = ctx.clone();
                vacant.insert(ctx);
                out
            }
        }
    }

    pub fn set_model(&self, user_id: UserId, model: Models) {
        self.contexts
            .entry(user_id)
            .or_insert_with(|| UserContext::new(user_id))
            .main_model = model;
        self.save_to_disk();
    }

    pub fn set_rate_line(&self, user_id: UserId, rate_line: u64) {
        let old_rate_line;
        {
            let mut entry = self
                .contexts
                .entry(user_id)
                .or_insert_with(|| UserContext::new(user_id));

            old_rate_line = entry.rate_line;
            entry.rate_line = rate_line;
        }

        if old_rate_line == 0 || rate_line == 0 {
            self.save_to_disk();
        }
    }

    pub fn set_voice_speaker(&self, user_id: UserId, speaker: Option<u32>) {
        {
            let mut entry = self
                .contexts
                .entry(user_id)
                .or_insert_with(|| UserContext::new(user_id));
            entry.voice_speaker = speaker;
        }
        self.save_to_disk();
    }

    pub fn set_voice_speed_scale(&self, user_id: UserId, speed_scale: Option<f32>) {
        {
            let mut entry = self
                .contexts
                .entry(user_id)
                .or_insert_with(|| UserContext::new(user_id));
            entry.voice_speed_scale = speed_scale;
        }
        self.save_to_disk();
    }

    pub fn set_voice_pitch_scale(&self, user_id: UserId, pitch_scale: Option<f32>) {
        {
            let mut entry = self
                .contexts
                .entry(user_id)
                .or_insert_with(|| UserContext::new(user_id));
            entry.voice_pitch_scale = pitch_scale;
        }
        self.save_to_disk();
    }

    pub fn set_voice_pan(&self, user_id: UserId, pan: Option<f32>) {
        {
            let mut entry = self
                .contexts
                .entry(user_id)
                .or_insert_with(|| UserContext::new(user_id));
            entry.voice_pan = pan;
        }
        self.save_to_disk();
    }

    fn save_to_disk(&self) {
        let default_model_name = Models::default().to_string();
        let entries = self
            .contexts
            .iter()
            .filter_map(|entry| {
                let value = entry.value();
                let model_name = value.main_model.to_string();
                if model_name == default_model_name
                    && value.rate_line != 0
                    && value.voice_speaker.is_none()
                    && value.voice_speed_scale.is_none()
                    && value.voice_pitch_scale.is_none()
                    && value.voice_pan.is_none()
                {
                    return None;
                }

                let mut obj = json!({
                    "user_id": value.user_id.get().to_string(),
                    "main_model": model_name,
                });

                if value.rate_line == 0 {
                    obj["rate_line"] = json!(0u64);
                }

                if let Some(speaker) = value.voice_speaker {
                    obj["voice_speaker"] = json!(speaker);
                }

                if let Some(speed_scale) = value.voice_speed_scale {
                    obj["voice_speed_scale"] = json!(speed_scale);
                }

                if let Some(pitch_scale) = value.voice_pitch_scale {
                    obj["voice_pitch_scale"] = json!(pitch_scale);
                }

                if let Some(pan) = value.voice_pan {
                    obj["voice_pan"] = json!(pan);
                }

                Some(obj)
            })
            .collect::<Vec<Value>>();

        let doc = json!({
            "version": 1,
            "contexts": entries,
        });

        if let Some(parent) = self.store_path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            error!("failed to create user context directory: {}", e);
            return;
        }

        let body = match serde_json::to_string_pretty(&doc) {
            Ok(v) => v,
            Err(e) => {
                error!("failed to serialize user contexts: {}", e);
                return;
            }
        };

        if let Err(e) = fs::write(&self.store_path, body) {
            error!("failed to write user contexts: {}", e);
        }
    }

    fn load_from_disk(&mut self) {
        let text = match fs::read_to_string(&self.store_path) {
            Ok(v) => v,
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    warn!("failed to read user contexts file: {}", e);
                }
                return;
            }
        };

        let doc: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                warn!("failed to parse user contexts file: {}", e);
                return;
            }
        };

        let Some(contexts) = doc.get("contexts").and_then(Value::as_array) else {
            return;
        };

        for ctx in contexts {
            let Some(user_id_raw) = ctx.get("user_id") else {
                continue;
            };
            let Some(user_id_num) = parse_u64(user_id_raw) else {
                continue;
            };

            let model = ctx
                .get("main_model")
                .and_then(Value::as_str)
                .map(|s| Models::from(s.to_string()))
                .unwrap_or_default();
            let rate_line = ctx.get("rate_line").and_then(Value::as_u64).unwrap_or(1);
            let voice_speaker = ctx
                .get("voice_speaker")
                .and_then(parse_u64)
                .and_then(|v| u32::try_from(v).ok());
            let voice_speed_scale = ctx
                .get("voice_speed_scale")
                .and_then(Value::as_f64)
                .map(|v| v as f32);
            let voice_pitch_scale = ctx
                .get("voice_pitch_scale")
                .and_then(Value::as_f64)
                .map(|v| v as f32);
            let voice_pan = ctx
                .get("voice_pan")
                .and_then(Value::as_f64)
                .map(|v| v as f32);

            self.contexts.insert(
                UserId::new(user_id_num),
                UserContext {
                    user_id: UserId::new(user_id_num),
                    main_model: model,
                    rate_line,
                    voice_speaker,
                    voice_speed_scale,
                    voice_pitch_scale,
                    voice_pan,
                },
            );
        }
    }
}

impl Default for UserContexts {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse::<u64>().ok()))
}
