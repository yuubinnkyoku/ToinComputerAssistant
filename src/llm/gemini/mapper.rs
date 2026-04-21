use async_openai::types::responses::{
    EasyInputContent, InputContent, InputItem, InputRole, Item, MessageItem,
};
use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;

use crate::llm::{
    client::{LMContext, Role},
    gemini::types::{Content, InlineData, Part},
};

pub async fn lm_context_to_contents(http: &Client, lm_context: &LMContext) -> Vec<Content> {
    let mut out = Vec::new();

    for item in &lm_context.buf {
        match item {
            InputItem::EasyMessage(msg) => {
                let role = match msg.role {
                    Role::Assistant => "model",
                    _ => "user",
                }
                .to_string();

                let mut parts = Vec::new();
                match &msg.content {
                    EasyInputContent::Text(text) => {
                        if !text.is_empty() {
                            parts.push(Part {
                                text: Some(text.clone()),
                                inline_data: None,
                                function_call: None,
                                function_response: None,
                            });
                        }
                    }
                    EasyInputContent::ContentList(list) => {
                        parts.extend(convert_input_contents(http, list).await);
                    }
                }

                if !parts.is_empty() {
                    out.push(Content { role, parts });
                }
            }
            InputItem::Item(Item::Message(message_item)) => {
                if let MessageItem::Input(input) = message_item {
                    let role = match input.role {
                        InputRole::Assistant => "model",
                        _ => "user",
                    }
                    .to_string();

                    let parts = convert_input_contents(http, &input.content).await;
                    if !parts.is_empty() {
                        out.push(Content { role, parts });
                    }
                }
            }
            _ => {}
        }
    }

    out
}

async fn convert_input_contents(http: &Client, contents: &[InputContent]) -> Vec<Part> {
    let mut parts = Vec::new();

    for content in contents {
        match content {
            InputContent::InputText(text) => {
                if !text.text.is_empty() {
                    parts.push(Part {
                        text: Some(text.text.clone()),
                        inline_data: None,
                        function_call: None,
                        function_response: None,
                    });
                }
            }
            InputContent::InputImage(image) => {
                if let Some(url) = &image.image_url
                    && let Ok(Some(inline_data)) = fetch_inline_image(http, url).await
                {
                    parts.push(Part {
                        text: None,
                        inline_data: Some(inline_data),
                        function_call: None,
                        function_response: None,
                    });
                }
            }
            _ => {}
        }
    }

    parts
}

async fn fetch_inline_image(
    http: &Client,
    image_url: &str,
) -> Result<Option<InlineData>, reqwest::Error> {
    let response = http.get(image_url).send().await?;
    if !response.status().is_success() {
        return Ok(None);
    }

    let mime_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap_or("image/jpeg").to_string())
        .unwrap_or_else(|| infer_image_mime(image_url));

    let bytes = response.bytes().await?;
    let data = general_purpose::STANDARD.encode(bytes);

    Ok(Some(InlineData { mime_type, data }))
}

fn infer_image_mime(image_url: &str) -> String {
    let lower = image_url.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png".to_string()
    } else if lower.ends_with(".gif") {
        "image/gif".to_string()
    } else if lower.ends_with(".webp") {
        "image/webp".to_string()
    } else {
        "image/jpeg".to_string()
    }
}
