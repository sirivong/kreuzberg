//! Structured data extraction using LLM-based schema-guided generation.
//!
//! Uses liter-llm's JSON schema response format to extract structured data
//! from document content. The LLM is constrained to produce output conforming
//! to the caller's JSON schema.

use crate::core::config::llm::StructuredExtractionConfig;

/// Extract structured data from document content using an LLM with JSON schema.
///
/// Sends the document content to the configured LLM with a JSON schema constraint,
/// returning structured data that conforms to the schema.
///
/// # Arguments
///
/// * `content` - The extracted document text to send to the LLM.
/// * `config` - Structured extraction configuration including schema and LLM settings.
///
/// # Returns
///
/// A `serde_json::Value` conforming to the provided JSON schema.
///
/// # Errors
///
/// Returns an error if:
/// - The LLM client cannot be created (invalid provider/credentials).
/// - The LLM request fails (network, rate-limit, etc.).
/// - The LLM response cannot be parsed as valid JSON.
pub async fn extract_structured(
    content: &str,
    config: &StructuredExtractionConfig,
) -> crate::Result<serde_json::Value> {
    use liter_llm::LlmClient;

    let client = super::client::create_client(&config.llm)?;

    // Build prompt from custom Jinja2 template or default
    let template = config
        .prompt
        .as_deref()
        .unwrap_or(super::prompts::STRUCTURED_EXTRACTION_TEMPLATE);

    let schema_json = serde_json::to_string_pretty(&config.schema)
        .map_err(|e| crate::KreuzbergError::validation(format!("Failed to serialize schema for prompt: {e}")))?;

    let ctx = minijinja::context! {
        content => content,
        schema => schema_json,
        schema_name => &config.schema_name,
        schema_description => config.schema_description.as_deref().unwrap_or(""),
    };

    let prompt = super::prompts::render_template(template, &ctx)?;

    // Build chat request with JSON schema response format.
    // Use field assignment because `stream` is pub(crate) in liter-llm.
    let mut request = liter_llm::ChatCompletionRequest::default();
    request.model = config.llm.model.clone();
    request.messages = vec![liter_llm::Message::User(liter_llm::UserMessage {
        content: liter_llm::UserContent::Text(prompt),
        name: None,
    })];
    request.temperature = config.llm.temperature;
    request.max_tokens = config.llm.max_tokens;
    request.response_format = Some(liter_llm::ResponseFormat::JsonSchema {
        json_schema: liter_llm::JsonSchemaFormat {
            name: config.schema_name.clone(),
            description: config.schema_description.clone(),
            schema: config.schema.clone(), // Clone required: liter-llm takes owned Value
            strict: Some(config.strict),
        },
    });

    let response = client
        .chat(request)
        .await
        .map_err(|e| crate::KreuzbergError::parsing(format!("LLM structured extraction request failed: {e}")))?;

    // Extract text content from the first choice
    let text = response
        .choices
        .first()
        .and_then(|c| c.message.content.as_deref())
        .ok_or_else(|| crate::KreuzbergError::parsing("LLM structured extraction returned no content"))?;

    // Parse the response as JSON
    serde_json::from_str(text)
        .map_err(|e| crate::KreuzbergError::parsing(format!("LLM structured extraction returned invalid JSON: {e}")))
}
