use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc;

use crate::{
    app::{
        config::{ModelResponseParams, Models},
        context::NelfieContext,
    },
    llm::{
        client::{LMContext, LMTool},
        gemini::client::GeminiClient,
        nim,
    },
};

#[allow(clippy::too_many_arguments)]
pub async fn generate_response_by_model(
    ob_ctx: NelfieContext,
    model: Models,
    lm_context: &LMContext,
    max_tokens: Option<u32>,
    tools: Option<Arc<HashMap<String, Box<dyn LMTool>>>>,
    state_mpsc: Option<mpsc::Sender<String>>,
    delta_mpsc: Option<mpsc::Sender<String>>,
) -> Result<LMContext, Box<dyn std::error::Error + Send + Sync>> {
    match model {
        Models::Gemini30Flash => {
            let gemini = GeminiClient::new(ob_ctx.config.gemini.clone());
            gemini
                .generate_response_with_model(
                    "gemini-3.0-flash",
                    ob_ctx,
                    lm_context,
                    max_tokens,
                    tools,
                    state_mpsc,
                    delta_mpsc,
                )
                .await
        }
        Models::Gemini30Pro => {
            let gemini = GeminiClient::new(ob_ctx.config.gemini.clone());
            gemini
                .generate_response_with_model(
                    "gemini-3.0-pro",
                    ob_ctx,
                    lm_context,
                    max_tokens,
                    tools,
                    state_mpsc,
                    delta_mpsc,
                )
                .await
        }
        Models::Gemini31Pro => {
            let gemini = GeminiClient::new(ob_ctx.config.gemini.clone());
            gemini
                .generate_response_with_model(
                    "gemini-3.1-pro",
                    ob_ctx,
                    lm_context,
                    max_tokens,
                    tools,
                    state_mpsc,
                    delta_mpsc,
                )
                .await
        }
        Models::GeminiAuto => {
            let gemini = GeminiClient::new(ob_ctx.config.gemini.clone());
            let auto_models = if ob_ctx.config.gemini.auto_models.is_empty() {
                vec![ob_ctx.config.gemini.default_model.clone()]
            } else {
                ob_ctx.config.gemini.auto_models.clone()
            };
            gemini
                .generate_response_with_fallback(
                    &auto_models,
                    ob_ctx,
                    lm_context,
                    max_tokens,
                    tools,
                    state_mpsc,
                    delta_mpsc,
                )
                .await
        }
        Models::NimDefault => {
            let nim_client = nim::client::NimClient::new(ob_ctx.config.nim.clone());
            let nim_model = ob_ctx.config.nim.default_model.clone();
            nim_client
                .generate_response_with_model(
                    &nim_model,
                    ob_ctx,
                    lm_context,
                    max_tokens,
                    tools,
                    state_mpsc,
                    delta_mpsc,
                )
                .await
        }
        _ => {
            let params = ModelResponseParams {
                model: model.to_string(),
                ..model.to_parameter()
            };
            ob_ctx
                .lm_client
                .generate_response(
                    ob_ctx.clone(),
                    lm_context,
                    max_tokens,
                    tools,
                    state_mpsc,
                    delta_mpsc,
                    Some(params),
                )
                .await
        }
    }
}
