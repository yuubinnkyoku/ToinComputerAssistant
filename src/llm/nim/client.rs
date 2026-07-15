use std::{collections::HashMap, io, sync::Arc};

use async_openai::types::responses::{
    EasyInputContent, InputContent, InputItem, InputRole, Item, MessageItem, OutputMessageContent,
};
use reqwest::{Client as HttpClient, Url};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    app::{config::NimConfig, context::NelfieContext},
    llm::client::{LMContext, LMTool, Role},
};

#[derive(Clone)]
pub struct NimClient {
    http: HttpClient,
    config: NimConfig,
}

impl NimClient {
    pub fn new(config: NimConfig) -> Self {
        Self {
            http: HttpClient::new(),
            config,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn generate_response_with_model(
        &self,
        model: &str,
        _ob_ctx: NelfieContext,
        lm_context: &LMContext,
        max_tokens: Option<u32>,
        _tools: Option<Arc<HashMap<String, Box<dyn LMTool>>>>,
        state_mpsc: Option<mpsc::Sender<String>>,
        delta_mpsc: Option<mpsc::Sender<String>>,
    ) -> Result<LMContext, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = self
            .config
            .base_url
            .as_deref()
            .ok_or_else(|| io::Error::other("NIM_BASE_URL is not set"))?;
        let api_key = self
            .config
            .api_key
            .as_deref()
            .ok_or_else(|| io::Error::other("NIM_API_KEY is not set"))?;

        let state_send = |s: String| {
            if let Some(tx) = state_mpsc.as_ref() {
                let _ = tx.clone().try_send(s);
            }
        };

        let delta_send = |s: String| {
            if let Some(tx) = delta_mpsc.as_ref() {
                let _ = tx.clone().try_send(s);
            }
        };

        let messages = lm_context_to_chat_messages(lm_context);
        if messages.is_empty() {
            return Err(Box::new(io::Error::other("NIM request has no messages")));
        }

        state_send("NIM chat completion in progress...".to_string());
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            max_tokens,
            stream: false,
        };

        let response = self
            .http
            .post(chat_completions_url(base_url)?)
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Box::new(io::Error::other(format!(
                "NIM API error {}: {}",
                status, body
            ))));
        }

        let response = response.json::<ChatCompletionResponse>().await?;
        let text = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_default();
        if text.is_empty() {
            return Err(Box::new(io::Error::other(
                "NIM response did not include message content",
            )));
        }

        let mut delta_context = LMContext::new();
        delta_context.add_text(text.clone(), Role::Assistant);
        delta_send(text);
        Ok(delta_context)
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

fn chat_completions_url(base_url: &str) -> Result<Url, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Url::parse(&format!(
        "{}/chat/completions",
        base_url.trim_end_matches('/')
    ))?)
}

fn lm_context_to_chat_messages(lm_context: &LMContext) -> Vec<ChatMessage> {
    let mut messages = Vec::new();

    for item in &lm_context.buf {
        match item {
            InputItem::EasyMessage(msg) => {
                let content = easy_content_text(&msg.content);
                if content.is_empty() {
                    continue;
                }

                let role = if matches!(msg.role, Role::System | Role::Developer) {
                    "system"
                } else if matches!(msg.role, Role::Assistant) {
                    "assistant"
                } else {
                    "user"
                };
                push_chat_message(&mut messages, role, content);
            }
            InputItem::Item(Item::Message(message_item)) => match message_item {
                MessageItem::Input(input) => {
                    let content = input_contents_text(&input.content);
                    if content.is_empty() {
                        continue;
                    }

                    let role = if matches!(input.role, InputRole::System | InputRole::Developer) {
                        "system"
                    } else {
                        "user"
                    };
                    push_chat_message(&mut messages, role, content);
                }
                MessageItem::Output(output) => {
                    let content = output_message_text(&output.content);
                    if !content.is_empty() {
                        push_chat_message(&mut messages, "assistant", content);
                    }
                }
            },
            _ => {}
        }
    }

    messages
}

fn push_chat_message(messages: &mut Vec<ChatMessage>, role: &str, content: String) {
    if let Some(last) = messages.last_mut()
        && last.role == role
    {
        last.content.push('\n');
        last.content.push_str(&content);
        return;
    }

    messages.push(ChatMessage {
        role: role.to_string(),
        content,
    });
}

fn easy_content_text(content: &EasyInputContent) -> String {
    match content {
        EasyInputContent::Text(text) => text.clone(),
        EasyInputContent::ContentList(list) => input_contents_text(list),
    }
}

fn input_contents_text(contents: &[InputContent]) -> String {
    contents
        .iter()
        .filter_map(|content| match content {
            InputContent::InputText(text) => Some(text.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn output_message_text(contents: &[OutputMessageContent]) -> String {
    contents
        .iter()
        .filter_map(|content| match content {
            OutputMessageContent::OutputText(text) => Some(text.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
