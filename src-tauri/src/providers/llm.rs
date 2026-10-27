use crate::error::{AppError, AppResult};
use crate::models::ProviderProfile;
use crate::providers;
use serde_json::{json, Value};

pub struct LlmRequest {
    pub model: String,
    pub system: String,
    pub user: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub json: bool,
}

pub struct LlmResult {
    pub text: String,
    pub usage: Value,
}

pub async fn complete(
    profile: &ProviderProfile,
    secret: Option<&str>,
    request: LlmRequest,
) -> AppResult<LlmResult> {
    let client = providers::client()?;
    let url = providers::endpoint(profile, "chat/completions");
    let mut payload = json!({
        "model": request.model,
        "messages": [
            {"role": "system", "content": request.system},
            {"role": "user", "content": request.user}
        ],
        "temperature": request.temperature,
        "max_tokens": request.max_tokens,
        "stream": false
    });
    if request.json {
        payload["response_format"] = json!({"type": "json_object"});
    }
    let mut builder = client.post(&url).json(&payload);
    if let Some(secret) = secret {
        builder = builder.bearer_auth(secret);
    }
    let response = builder.send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        // Some OpenAI-compatible servers reject response_format. Retry once
        // without it so local/legacy endpoints still work.
        if request.json && status.as_u16() == 400 {
            payload.as_object_mut().map(|o| o.remove("response_format"));
            let retry = client.post(&url).json(&payload);
            let retry = if let Some(secret) = secret {
                retry.bearer_auth(secret)
            } else {
                retry
            };
            let response = retry.send().await?;
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(providers::error_from_response(status, &body));
            }
            return parse_response(response.json::<Value>().await?);
        }
        return Err(providers::error_from_response(status, &body));
    }
    parse_response(response.json::<Value>().await?)
}

fn parse_response(value: Value) -> AppResult<LlmResult> {
    let text = value
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            value
                .pointer("/choices/0/text")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            value
                .pointer("/message/content")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .ok_or_else(|| AppError::Provider("LLM returned no text content".into()))?;
    Ok(LlmResult {
        text,
        usage: value.get("usage").cloned().unwrap_or(Value::Null),
    })
}

pub fn parse_json_loose<T: serde::de::DeserializeOwned>(raw: &str) -> AppResult<T> {
    let trimmed = raw.trim();
    if let Ok(value) = serde_json::from_str::<T>(trimmed) {
        return Ok(value);
    }
    let without_fence = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(trimmed)
        .trim();
    if let Ok(value) = serde_json::from_str::<T>(without_fence) {
        return Ok(value);
    }
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            let slice = &trimmed[start..=end];
            if let Ok(value) = serde_json::from_str::<T>(slice) {
                return Ok(value);
            }
        }
    }
    Err(AppError::Provider(
        "The model did not return valid structured JSON. Try again or switch models.".into(),
    ))
}
