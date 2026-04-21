use std::{collections::HashMap, io, sync::Arc};

use anyhow::anyhow;
use log::{debug, info, warn};
use reqwest::Client as HttpClient;
use tokio::sync::mpsc;

use crate::{
    app::config::GeminiConfig,
    app::context::NelfieContext,
    llm::{
        client::{LMContext, LMTool, Role},
        gemini::{
            mapper::lm_context_to_contents,
            types::{Content, FunctionDeclaration, GenerateContentRequest, GenerateContentResponse, GoogleSearch, Tool},
        },
    },
};

#[derive(Clone)]
pub struct GeminiClient {
    http: HttpClient,
    config: GeminiConfig,
}

impl GeminiClient {
    pub fn new(config: GeminiConfig) -> Self {
        Self {
            http: HttpClient::new(),
            config,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn generate_response_with_model(
        &self,
        model: &str,
        ob_ctx: NelfieContext,
        lm_context: &LMContext,
        tools: Option<Arc<HashMap<String, Box<dyn LMTool>>>>,
        state_mpsc: Option<mpsc::Sender<String>>,
        delta_mpsc: Option<mpsc::Sender<String>>,
    ) -> Result<LMContext, Box<dyn std::error::Error + Send + Sync>> {
        let api_key = self
            .config
            .api_key
            .clone()
            .ok_or_else(|| anyhow!("GEMINI_API_KEY is not set"))?;

        let tools_map = tools.unwrap_or_default();
        let mut delta_context = LMContext::new();

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

        for i in 0..self.config.max_tool_loops {
            let mut current_lm = lm_context.clone();
            current_lm.extend(&delta_context);
            let contents = lm_context_to_contents(&self.http, &current_lm).await;
            let request = self.build_request(contents, &tools_map);

            state_send(format!("Gemini generating... (loop {})", i + 1));
            let response = self
                .call_generate_content(model, &api_key, &request)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            let Some(candidate) = response.candidates.first() else {
                let block_reason = response
                    .prompt_feedback
                    .and_then(|f| f.block_reason)
                    .unwrap_or_else(|| "unknown".to_string());
                return Err(Box::new(io::Error::other(format!(
                    "Gemini returned no candidates: {}",
                    block_reason
                ))));
            };

            let finish_reason = candidate
                .finish_reason
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            debug!("Gemini finish reason: {}", finish_reason);

            let Some(content) = &candidate.content else {
                return Err(Box::new(io::Error::other("Gemini candidate had no content")));
            };

            let mut has_function_call = false;
            for part in &content.parts {
                if let Some(text) = &part.text
                    && !text.is_empty()
                {
                    delta_context.add_text(text.clone(), Role::Assistant);
                    delta_send(text.clone());
                }

                if let Some(function_call) = &part.function_call {
                    has_function_call = true;
                    state_send(format!("Executing tool: {}", function_call.name));

                    let output_text = if let Some(tool) = tools_map.get(&function_call.name) {
                        match tool.execute(function_call.args.clone(), ob_ctx.clone()).await {
                            Ok(v) => v,
                            Err(e) => format!("Error: {}", e),
                        }
                    } else {
                        format!("Error: tool '{}' is not defined", function_call.name)
                    };

                    delta_context.add_text(
                        format!("Tool '{}' result: {}", function_call.name, output_text),
                        Role::User,
                    );
                }
            }

            if !has_function_call {
                info!("Gemini response completed without pending tool calls");
                break;
            }
        }

        Ok(delta_context)
    }

    pub async fn generate_response_with_fallback(
        &self,
        models: &[String],
        ob_ctx: NelfieContext,
        lm_context: &LMContext,
        tools: Option<Arc<HashMap<String, Box<dyn LMTool>>>>,
        state_mpsc: Option<mpsc::Sender<String>>,
        delta_mpsc: Option<mpsc::Sender<String>>,
    ) -> Result<LMContext, Box<dyn std::error::Error + Send + Sync>> {
        let mut errors = Vec::new();

        for model in models {
            let state = state_mpsc.clone();
            if let Some(tx) = state {
                let _ = tx.try_send(format!("gemini-auto trying model: {}", model));
            }

            match self
                .generate_response_with_model(
                    model,
                    ob_ctx.clone(),
                    lm_context,
                    tools.clone(),
                    state_mpsc.clone(),
                    delta_mpsc.clone(),
                )
                .await
            {
                Ok(ctx) => return Ok(ctx),
                Err(e) => {
                    warn!("gemini-auto failed on {}: {}", model, e);
                    errors.push(format!("{}: {}", model, e));
                }
            }
        }

        Err(Box::new(io::Error::other(format!(
            "all gemini-auto models failed: {}",
            errors.join(" | ")
        ))))
    }

    fn build_request(
        &self,
        contents: Vec<Content>,
        tools: &HashMap<String, Box<dyn LMTool>>,
    ) -> GenerateContentRequest {
        let mut tool_defs = Vec::new();

        let function_declarations = tools
            .values()
            .map(|tool| FunctionDeclaration {
                name: tool.name(),
                description: tool.description(),
                parameters: tool.json_schema(),
            })
            .collect::<Vec<_>>();

        if !function_declarations.is_empty() {
            tool_defs.push(Tool {
                function_declarations: Some(function_declarations),
                google_search: None,
            });
        }

        if self.config.enable_google_search {
            tool_defs.push(Tool {
                function_declarations: None,
                google_search: Some(GoogleSearch {}),
            });
        }

        GenerateContentRequest {
            contents,
            tools: if tool_defs.is_empty() {
                None
            } else {
                Some(tool_defs)
            },
        }
    }

    async fn call_generate_content(
        &self,
        model: &str,
        api_key: &str,
        request: &GenerateContentRequest,
    ) -> Result<GenerateContentResponse, anyhow::Error> {
        let base = self.config.base_url.trim_end_matches('/');
        let url = format!("{}/models/{}:generateContent", base, model);

        let response = self
            .http
            .post(url)
            .query(&[("key", api_key)])
            .json(request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Gemini API error {}: {}", status, body));
        }

        Ok(response.json::<GenerateContentResponse>().await?)
    }
}
