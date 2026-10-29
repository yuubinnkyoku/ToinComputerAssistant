use crate::{
    app::context::NelfieContext,
    llm::{
        client::{GenerateResponseOptions, GenerateResponseResult},
        context::LMContext,
        gemini::client::GeminiClient,
        models::Models,
        nim,
        tool::ToolExecutionContext,
    },
};

/// Model-based backend routing (fork-specific).
///
/// The selected model is read from `options.parameters`, mirroring how the
/// upstream OpenAI client is invoked from `discord::responses`. Gemini and
/// NIM models are served by their isolated clients; every other model falls
/// through to the upstream OpenAI client unchanged.
pub async fn generate_response_by_model(
    ob_ctx: NelfieContext,
    lm_context: &LMContext,
    options: GenerateResponseOptions,
) -> Result<GenerateResponseResult, Box<dyn std::error::Error + Send + Sync>> {
    let model = Models::from_name(&options.parameters.model).unwrap_or_default();

    if !model.is_gemini() && !matches!(model, Models::NimDefault) {
        let lm_client = ob_ctx.lm_client.clone();
        return lm_client
            .generate_response(ob_ctx, lm_context, options)
            .await;
    }

    let GenerateResponseOptions {
        max_output_tokens,
        tools,
        state_sender,
        parameters: _,
        user_context_scope,
        allow_mainline_model_change,
        // Gemini/NIM have no OpenAI prompt-cache key or per-token rate usage
        // reporting; dropping the sender simply closes that channel.
        context_cache_key: _,
        rate_usage_sender: _,
    } = options;

    let tool_context = ToolExecutionContext {
        user_context_scope,
        allow_mainline_model_change,
        image_generation_enabled: false,
    };
    let max_tokens = Some(max_output_tokens);

    let context = match model {
        Models::Gemini30Flash | Models::Gemini30Pro | Models::Gemini31Pro => {
            let gemini = GeminiClient::new(ob_ctx.config.gemini.clone());
            gemini
                .generate_response_with_model(
                    &model.to_string(),
                    ob_ctx,
                    lm_context,
                    max_tokens,
                    Some(tools),
                    state_sender,
                    tool_context,
                )
                .await?
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
                    Some(tools),
                    state_sender,
                    tool_context,
                )
                .await?
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
                    Some(tools),
                    state_sender,
                )
                .await?
        }
        _ => unreachable!("non-OpenAI models are matched above"),
    };

    Ok(GenerateResponseResult {
        context,
        generated_images: Vec::new(),
        final_model: model,
    })
}
