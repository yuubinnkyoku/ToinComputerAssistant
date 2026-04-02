use std::{
    str::FromStr,
    sync::atomic::Ordering,
};

use serde_json::json;
use serenity::all::{
    ButtonStyle, ChannelId, CreateActionRow, CreateButton, CreateInputText, CreateMessage,
    CreateModal, InputTextStyle,
};

use crate::lmclient::LMTool;

pub const MODAL_TRIGGER_PREFIX: &str = "modal_builder:open:";
pub const MODAL_SUBMIT_PREFIX: &str = "modal_builder:submit:";

#[derive(Clone, Debug)]
pub struct ModalInputSpec {
    pub label: String,
    pub custom_id: String,
    pub style: InputTextStyle,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub required: bool,
    pub min_length: Option<u16>,
    pub max_length: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct ModalSpec {
    pub title: String,
    pub logical_custom_id: String,
    pub inputs: Vec<ModalInputSpec>,
}

#[derive(Clone, Debug)]
pub struct PendingModalSpec {
    pub modal: ModalSpec,
    pub submit_custom_id: String,
}

pub struct ModalBuilderTool;

impl ModalBuilderTool {
    pub fn new() -> ModalBuilderTool {
        ModalBuilderTool {}
    }

    fn get_str_arg<'a>(args: &'a serde_json::Value, key: &'a str) -> Result<&'a str, String> {
        args.get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Missing or invalid '{key}' parameter"))
    }

    fn parse_modal_spec(args: &serde_json::Value) -> Result<ModalSpec, String> {
        let title = Self::get_str_arg(args, "title")?.to_string();
        let logical_custom_id = Self::get_str_arg(args, "custom_id")?.to_string();
        let inputs = args
            .get("inputs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Missing or invalid 'inputs' parameter".to_string())?;

        if inputs.is_empty() {
            return Err("'inputs' must contain at least one field".to_string());
        }

        let mut parsed_inputs = Vec::with_capacity(inputs.len());
        for (idx, input) in inputs.iter().enumerate() {
            let label = input
                .get("label")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("inputs[{idx}].label is required"))?
                .to_string();
            let input_custom_id = input
                .get("custom_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("inputs[{idx}].custom_id is required"))?
                .to_string();

            let style = match input
                .get("style")
                .and_then(|v| v.as_str())
                .unwrap_or("short")
            {
                "short" => InputTextStyle::Short,
                "paragraph" => InputTextStyle::Paragraph,
                other => {
                    return Err(format!(
                        "inputs[{idx}].style must be 'short' or 'paragraph', got '{other}'"
                    ));
                }
            };

            let min_length = input
                .get("min_length")
                .and_then(|v| v.as_u64())
                .map(|v| {
                    u16::try_from(v)
                        .map_err(|_| format!("inputs[{idx}].min_length must be <= 65535"))
                })
                .transpose()?;

            let max_length = input
                .get("max_length")
                .and_then(|v| v.as_u64())
                .map(|v| {
                    u16::try_from(v)
                        .map_err(|_| format!("inputs[{idx}].max_length must be <= 65535"))
                })
                .transpose()?;

            parsed_inputs.push(ModalInputSpec {
                label,
                custom_id: input_custom_id,
                style,
                placeholder: input
                    .get("placeholder")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                value: input
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                required: input
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                min_length,
                max_length,
            });
        }

        Ok(ModalSpec {
            title,
            logical_custom_id,
            inputs: parsed_inputs,
        })
    }

    fn build_modal_payload(spec: &ModalSpec, effective_custom_id: &str) -> serde_json::Value {
        let components = spec
            .inputs
            .iter()
            .map(|input| {
                json!({
                    "type": 1,
                    "components": [{
                        "type": 4,
                        "custom_id": input.custom_id,
                        "label": input.label,
                        "style": match input.style {
                            InputTextStyle::Paragraph => 2,
                            _ => 1,
                        },
                        "required": input.required,
                        "placeholder": input.placeholder,
                        "value": input.value,
                        "min_length": input.min_length,
                        "max_length": input.max_length,
                    }],
                })
            })
            .collect::<Vec<serde_json::Value>>();

        json!({
            "title": spec.title,
            "custom_id": effective_custom_id,
            "logical_custom_id": spec.logical_custom_id,
            "components": components,
        })
    }
}

impl Default for ModalBuilderTool {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_create_modal(spec: &ModalSpec, effective_custom_id: &str) -> CreateModal {
    let rows = spec
        .inputs
        .iter()
        .map(|input| {
            let mut text_input = CreateInputText::new(input.style, &input.label, &input.custom_id)
                .required(input.required);

            if let Some(placeholder) = &input.placeholder {
                text_input = text_input.placeholder(placeholder);
            }
            if let Some(value) = &input.value {
                text_input = text_input.value(value);
            }
            if let Some(min_length) = input.min_length {
                text_input = text_input.min_length(min_length);
            }
            if let Some(max_length) = input.max_length {
                text_input = text_input.max_length(max_length);
            }

            CreateActionRow::InputText(text_input)
        })
        .collect::<Vec<CreateActionRow>>();

    CreateModal::new(effective_custom_id, &spec.title).components(rows)
}

#[async_trait::async_trait]
impl LMTool for ModalBuilderTool {
    fn name(&self) -> String {
        "modal-builder-tool".to_string()
    }

    fn description(&self) -> String {
        "Build Discord modal definitions. Use build_modal to only generate payload JSON, or build_and_send_trigger to post a button in Discord that opens the modal when clicked. If the user expects a visible action in Discord, prefer build_and_send_trigger.".to_string()
    }

    fn json_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "Operation to run. Use build_and_send_trigger for actual Discord-side modal opening flow.",
                    "enum": ["build_modal", "build_and_send_trigger"]
                },
                "channel_id": {
                    "type": "string",
                    "description": "Target channel ID. Required for build_and_send_trigger."
                },
                "title": {
                    "type": "string",
                    "description": "Modal title. Required for build_modal."
                },
                "custom_id": {
                    "type": "string",
                    "description": "Modal custom ID. Required for build_modal."
                },
                "inputs": {
                    "type": "array",
                    "description": "List of text input definitions.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "label": {
                                "type": "string",
                                "description": "Input label."
                            },
                            "custom_id": {
                                "type": "string",
                                "description": "Input custom ID."
                            },
                            "style": {
                                "type": "string",
                                "description": "Input style. 'short' or 'paragraph'. Defaults to 'short'.",
                                "enum": ["short", "paragraph"]
                            },
                            "placeholder": {
                                "type": "string",
                                "description": "Optional placeholder text."
                            },
                            "value": {
                                "type": "string",
                                "description": "Optional prefilled value."
                            },
                            "required": {
                                "type": "boolean",
                                "description": "Whether this input is required. Defaults to true."
                            },
                            "min_length": {
                                "type": "integer",
                                "description": "Optional minimum length."
                            },
                            "max_length": {
                                "type": "integer",
                                "description": "Optional maximum length."
                            }
                        },
                        "required": ["label", "custom_id"]
                    }
                },
                "trigger_label": {
                    "type": "string",
                    "description": "Button label used by build_and_send_trigger. Defaults to 'Open modal'."
                },
                "trigger_message": {
                    "type": "string",
                    "description": "Message body posted with the trigger button. Defaults to a short guide text."
                }
            },
            "required": ["operation", "title", "custom_id", "inputs"]
        })
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        ob_ctx: crate::context::NhelvContext,
    ) -> Result<String, String> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'operation' parameter".to_string())?;

        match operation {
            "build_modal" => {
                let spec = Self::parse_modal_spec(&args)?;
                let payload = Self::build_modal_payload(&spec, &spec.logical_custom_id);

                let result = json!({
                    "status": "ok",
                    "operation": operation,
                    "modal": payload,
                    "note": "Payload generated only. To open it in Discord, use build_and_send_trigger.",
                });

                Ok(result.to_string())
            }
            "build_and_send_trigger" => {
                let spec = Self::parse_modal_spec(&args)?;
                let channel_id = Self::get_str_arg(&args, "channel_id")?;
                let channel_id = ChannelId::from_str(channel_id)
                    .map_err(|e| format!("Invalid 'channel_id': {e}"))?;

                let seq = ob_ctx.response_seq.fetch_add(1, Ordering::Relaxed);
                let trigger_custom_id = format!("{}{}", MODAL_TRIGGER_PREFIX, seq);
                let submit_custom_id = format!("{}{}", MODAL_SUBMIT_PREFIX, seq);

                let pending = PendingModalSpec {
                    modal: spec.clone(),
                    submit_custom_id: submit_custom_id.clone(),
                };
                ob_ctx.pending_modals.insert(trigger_custom_id.clone(), pending);

                let trigger_label = args
                    .get("trigger_label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Open modal");
                let trigger_message = args
                    .get("trigger_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("モーダルを開くには下のボタンを押してね。");

                let builder = CreateMessage::new()
                    .content(trigger_message)
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(trigger_custom_id.clone())
                            .style(ButtonStyle::Primary)
                            .label(trigger_label),
                    ])]);

                let sent = channel_id
                    .send_message(ob_ctx.discord_client.open().http.clone(), builder)
                    .await
                    .map_err(|e| format!("Failed to send modal trigger message: {e}"))?;

                let result = json!({
                    "status": "ok",
                    "operation": operation,
                    "channel_id": channel_id.to_string(),
                    "trigger_message_id": sent.id.to_string(),
                    "trigger_custom_id": trigger_custom_id,
                    "modal": Self::build_modal_payload(&spec, &submit_custom_id),
                    "note": "When a user clicks the button, the modal is opened via interaction response.",
                });

                Ok(result.to_string())
            }
            other => Err(format!(
                "Unsupported 'operation': {other}. Use: build_modal or build_and_send_trigger."
            )),
        }
    }
}
