use crate::error::{AppError, AppResult};
use crate::models::ProviderProfile;
use crate::providers;
use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
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
        let mock = providers::mock::music(&request.prompt, request.duration_ms, request.seed)?;
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
        "volc-music" => volcengine(profile, secret, request).await,
        other => Err(AppError::Provider(format!(
            "music adapter not implemented for {other}"
        ))),
    }
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

struct VolcSigned {
    date: String,
    payload_hash: String,
    authorization: String,
}

fn volc_sign(access_key: &str, secret_key: &str, action: &str, version: &str, body: &[u8], host: &str) -> VolcSigned {
    const REGION: &str = "cn-north-1";
    const SERVICE: &str = "sami";
    let date = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let auth_date = &date[..8];
    let payload_hash = sha256_hex(body);
    let query = format!("Action={action}&Version={version}");
    let canonical_headers = format!(
        "content-type:application/json\nhost:{host}\nx-content-sha256:{payload_hash}\nx-date:{date}\n"
    );
    let signed_headers = "content-type;host;x-content-sha256;x-date";
    let canonical_request = format!(
        "POST\n/\n{query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
    );
    let scope = format!("{auth_date}/{REGION}/{SERVICE}/request");
    let string_to_sign = format!(
        "HMAC-SHA256\n{date}\n{scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );
    let k_date = hmac_sha256(secret_key.as_bytes(), auth_date.as_bytes());
    let k_region = hmac_sha256(&k_date, REGION.as_bytes());
    let k_service = hmac_sha256(&k_region, SERVICE.as_bytes());
    let k_signing = hmac_sha256(&k_service, b"request");
    let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));
    let authorization = format!(
        "HMAC-SHA256 Credential={access_key}/{scope}, SignedHeaders={signed_headers}, Signature={signature}"
    );
    VolcSigned {
        date,
        payload_hash,
        authorization,
    }
}

async fn volc_call(
    profile: &ProviderProfile,
    access_key: &str,
    secret_key: &str,
    action: &str,
    body: Value,
) -> AppResult<Value> {
    let client = providers::client()?;
    let host = providers::base(profile)
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string();
    let bytes = serde_json::to_vec(&body)?;
    let signed = volc_sign(access_key, secret_key, action, "2024-08-12", &bytes, &host);
    let url = format!(
        "{}/?Action={action}&Version=2024-08-12",
        providers::base(profile).trim_end_matches('/')
    );
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Date", signed.date)
        .header("X-Content-Sha256", signed.payload_hash)
        .header("Authorization", signed.authorization)
        .body(bytes)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        let message = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|value| {
                value
                    .pointer("/ResponseMetadata/Error/Message")
                    .and_then(|v| v.as_str())
                    .or_else(|| value.pointer("/ResponseMetadata/Error/Code").and_then(|v| v.as_str()))
                    .map(str::to_string)
            })
            .unwrap_or_else(|| text.chars().take(500).collect());
        return Err(AppError::Provider(format!("{}: {message}", status.as_u16())));
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn volc_result_url(value: &Value) -> Option<String> {
    value
        .pointer("/ResultUrl")
        .and_then(|v| v.as_str())
        .or_else(|| value.pointer("/Result/AudioUrl").and_then(|v| v.as_str()))
        .or_else(|| value.pointer("/Result/Url").and_then(|v| v.as_str()))
        .map(str::to_string)
}

fn volc_task_id(value: &Value) -> Option<String> {
    value
        .pointer("/TaskId")
        .and_then(|v| v.as_str())
        .or_else(|| value.pointer("/Result/TaskId").and_then(|v| v.as_str()))
        .map(str::to_string)
}

async fn volcengine(
    profile: &ProviderProfile,
    secret: &str,
    request: &MusicRequest,
) -> AppResult<MusicResult> {
    let (access_key, secret_key) = secret.split_once(':').ok_or_else(|| {
        AppError::MissingCredential(
            "火山引擎纯音乐需要 AccessKey:SecretKey（在火山引擎访问控制创建）".into(),
        )
    })?;
    let seconds = (request.duration_ms / 1000).clamp(10, 120);
    let submitted = volc_call(
        profile,
        access_key,
        secret_key,
        "GenBGM",
        json!({
            "Text": request.prompt.clone(),
            "Duration": seconds,
            "Version": request.model.clone(),
            "EnableInputRewrite": true
        }),
    )
    .await?;
    let task_id = volc_task_id(&submitted)
        .ok_or_else(|| AppError::Provider("火山引擎纯音乐没有返回 TaskId".into()))?;
    let mut result = submitted;
    let mut audio_url = volc_result_url(&result);
    for _ in 0..60 {
        if audio_url.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
        result = volc_call(
            profile,
            access_key,
            secret_key,
            "QueryTask",
            json!({"TaskId": task_id}),
        )
        .await?;
        let status = result
            .get("Status")
            .and_then(|v| v.as_str())
            .unwrap_or("Processing");
        if status.eq_ignore_ascii_case("fail") || status.eq_ignore_ascii_case("failed") {
            return Err(AppError::Provider(format!(
                "火山引擎纯音乐生成失败: {}",
                result
                    .pointer("/Message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
            )));
        }
        audio_url = volc_result_url(&result);
    }
    let audio_url = audio_url.ok_or_else(|| {
        AppError::Provider("火山引擎纯音乐超时，未返回 ResultUrl".into())
    })?;
    let client = providers::client()?;
    let response = client.get(&audio_url).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let mime = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mpeg")
        .to_string();
    let bytes = response.bytes().await?.to_vec();
    Ok(MusicResult {
        extension: if mime.contains("wav") {
            "wav"
        } else if mime.contains("mp4") || mime.contains("aac") {
            "m4a"
        } else {
            "mp3"
        }
        .into(),
        mime,
        bytes,
        cost: 0.002 * seconds as f64,
        raw: json!({"provider": "volc-music", "taskId": task_id, "result": result}),
    })
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
                extension: if content_type.contains("wav") {
                    "wav"
                } else {
                    "mp3"
                }
                .into(),
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
        extension: if content_type.contains("wav") {
            "wav"
        } else {
            "mp3"
        }
        .into(),
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
