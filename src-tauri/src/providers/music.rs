use crate::error::{AppError, AppResult};
use crate::models::ProviderProfile;
use crate::providers;
use base64::Engine;
use serde_json::{json, Value};
use std::time::Duration;

pub struct MusicRequest {
    pub model: String,
    pub prompt: String,
    pub duration_ms: i64,
    pub instrumental: bool,
    pub seed: Option<i64>,
}

pub struct MusicResult {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub extension: String,
    pub cost: f64,
    pub raw: Value,
}

pub async fn generate(
    profile: &ProviderProfile,
    secret: Option<&str>,
    request: &MusicRequest,
) -> AppResult<MusicResult> {
    if profile.id.starts_with("mock-") {
        let mock = providers::mock::music(
            &request.prompt,
            request.duration_ms,
            request.seed,
        )?;
        return Ok(MusicResult {
            bytes: mock.bytes,
            mime: mock.mime,
            extension: mock.extension,
            cost: 0.0,
            raw: mock.raw,
        });
    }
    let secret = secret
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::MissingCredential(format!("{} API key", profile.label)))?;
    match profile.id.as_str() {
        "stability-audio" => stability(profile, secret, request).await,
        "elevenlabs-music" => elevenlabs(profile, secret, request).await,
        other => Err(AppError::Provider(format!(
            "music adapter not implemented for {other}"
        ))),
    }
}

async fn stability(
    profile: &ProviderProfile,
    secret: &str,
    request: &MusicRequest,
) -> AppResult<MusicResult> {
    let client = providers::client()?;
    let duration = (request.duration_ms / 1000).clamp(1, 380);
    let form = reqwest::multipart::Form::new()
        .text("prompt", request.prompt.clone())
        .text("duration", duration.to_string())
        .text("output_format", "mp3")
        .text("steps", "6");
    let response = client
        .post(providers::endpoint(
            profile,
            "/v2beta/audio/stable-audio/text-to-audio",
        ))
        .bearer_auth(secret)
        .header("Accept", "application/json")
        .multipart(form)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    if response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("audio/"))
        .unwrap_or(false)
    {
        let mime = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("audio/mpeg")
            .to_string();
        let bytes = response.bytes().await?.to_vec();
        return Ok(MusicResult {
            bytes,
            extension: if mime.contains("wav") { "wav" } else { "mp3" }.into(),
            mime,
            cost: 0.08,
            raw: json!({"provider": "stability", "async": false}),
        });
    }
    let submitted: Value = response.json().await?;
    let id = submitted
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Provider("Stability Audio returned no generation id".into()))?
        .to_string();
    let poll_url = providers::endpoint(profile, &format!("/v2beta/audio/results/{id}"));
    for _ in 0..120 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let response = client
            .get(&poll_url)
            .bearer_auth(secret)
            .header("Accept", "audio/*")
            .send()
            .await?;
        if !response.status().is_success() {
            if response.status().as_u16() == 404 || response.status().as_u16() == 202 {
                continue;
            }
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(providers::error_from_response(status, &body));
        }
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        if content_type.starts_with("audio/") {
            let bytes = response.bytes().await?.to_vec();
            return Ok(MusicResult {
                extension: if content_type.contains("wav") { "wav" } else { "mp3" }.into(),
                mime: content_type,
                bytes,
                cost: 0.08,
                raw: json!({"provider": "stability", "generationId": id}),
            });
        }
        let value: Value = response.json().await?;
        let status = value
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("processing");
        if status == "failed" || status == "error" {
            return Err(AppError::Provider(format!(
                "Stability Audio failed: {}",
                value
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
            )));
        }
        let audio_url = value
            .get("audio_url")
            .or_else(|| value.get("url"))
            .and_then(|v| v.as_str());
        if let Some(url) = audio_url {
            let bytes = client.get(url).send().await?.bytes().await?.to_vec();
            return Ok(MusicResult {
                bytes,
                mime: "audio/mpeg".into(),
                extension: "mp3".into(),
                cost: 0.08,
                raw: json!({"provider": "stability", "generationId": id, "response": value}),
            });
        }
        if let Some(b64) = value.get("audio").and_then(|v| v.as_str()) {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| AppError::Provider(format!("invalid Stability audio: {e}")))?;
            return Ok(MusicResult {
                bytes,
                mime: "audio/mpeg".into(),
                extension: "mp3".into(),
                cost: 0.08,
                raw: json!({"provider": "stability", "generationId": id}),
            });
        }
    }
    Err(AppError::Provider(
        "Stability Audio did not finish within 4 minutes".into(),
    ))
}

async fn elevenlabs(
    profile: &ProviderProfile,
    secret: &str,
    request: &MusicRequest,
) -> AppResult<MusicResult> {
    let client = providers::client()?;
    let body = json!({
        "prompt": request.prompt,
        "music_length_ms": request.duration_ms.clamp(3_000, 600_000),
        "force_instrumental": request.instrumental
    });
    let response = client
        .post(providers::endpoint(profile, "/v1/music"))
        .header("xi-api-key", secret)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mpeg")
        .to_string();
    let bytes = response.bytes().await?.to_vec();
    Ok(MusicResult {
        extension: if content_type.contains("wav") { "wav" } else { "mp3" }.into(),
        mime: content_type,
        bytes,
        cost: 0.10,
        raw: json!({"provider": "elevenlabs", "instrumental": request.instrumental}),
    })
}

pub fn sniff_audio_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        "audio/wav"
    } else if bytes.starts_with(b"ID3") || bytes.first() == Some(&0xFF) {
        "audio/mpeg"
    } else if bytes.starts_with(b"OggS") {
        "audio/ogg"
    } else if bytes.starts_with(b"fLaC") {
        "audio/flac"
    } else {
        "audio/mpeg"
    }
}
