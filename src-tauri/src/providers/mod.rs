pub mod image;
pub mod llm;
pub mod mock;
pub mod music;
pub mod video;

use crate::error::{AppError, AppResult};
use crate::models::ProviderProfile;
use base64::Engine;
use std::path::Path;
use std::time::Duration;

pub fn client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("SaladCart/0.1")
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(20))
        .build()
        .map_err(Into::into)
}

pub fn base(profile: &ProviderProfile) -> String {
    profile.api_base.trim_end_matches('/').to_string()
}

pub fn endpoint(profile: &ProviderProfile, path: &str) -> String {
    format!("{}/{}", base(profile), path.trim_start_matches('/'))
}

pub fn error_from_response(status: reqwest::StatusCode, body: &str) -> AppError {
    let message = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            v.get("error")
                .and_then(|e| {
                    e.get("message")
                        .and_then(|m| m.as_str())
                        .or_else(|| e.as_str())
                })
                .map(str::to_string)
                .or_else(|| v.get("message").and_then(|m| m.as_str()).map(str::to_string))
        })
        .unwrap_or_else(|| {
            let trimmed = body.trim();
            if trimmed.is_empty() {
                format!("HTTP {}", status.as_u16())
            } else {
                trimmed.chars().take(500).collect()
            }
        });
    AppError::Provider(format!("{}: {}", status.as_u16(), message))
}

pub fn data_uri(path: &Path) -> AppResult<String> {
    let bytes = std::fs::read(path)?;
    if bytes.len() > 25 * 1024 * 1024 {
        return Err(AppError::Invalid(format!(
            "reference image is too large: {}",
            path.display()
        )));
    }
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
}

pub fn image_mime(path: &Path) -> String {
    mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string()
}

pub async fn download(url: &str, dest: &Path) -> AppResult<()> {
    let client = client()?;
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(error_from_response(status, &body));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let bytes = response.bytes().await?;
    std::fs::write(dest, bytes)?;
    Ok(())
}

pub fn sanitize_filename(name: &str, fallback: &str) -> String {
    let name = name.trim().replace(['/', '\\', ':'], "-");
    let name = name.trim_matches(['.', ' ']);
    if name.is_empty() {
        fallback.to_string()
    } else {
        name.chars().take(120).collect()
    }
}

pub fn estimate_cost(provider_id: &str, kind: &str, duration_ms: i64, count: u32) -> f64 {
    if provider_id.starts_with("mock-") || provider_id == "comfyui-image" {
        return 0.0;
    }
    let count = count.max(1) as f64;
    let seconds = (duration_ms.max(0) as f64 / 1000.0).max(1.0);
    match (kind, provider_id) {
        ("image", "openai-image") => 0.04 * count,
        ("image", "gemini-image") => 0.039 * count,
        ("image", "replicate-image") => 0.01 * count,
        ("video", "kling-video") => 0.10 * (seconds / 5.0).ceil(),
        ("video", "luma-video") => 0.08 * (seconds / 5.0).ceil(),
        ("video", "runway-video") => 0.12 * (seconds / 5.0).ceil(),
        ("video", "minimax-video") => 0.06 * (seconds / 6.0).ceil(),
        ("video", "replicate-video") => 0.08 * (seconds / 5.0).ceil(),
        ("video", "veo-video") => 0.40 * seconds,
        ("video", "fal-video") => 0.07 * seconds,
        ("image", "stability-image") => 0.03 * count,
        ("image", "fal-image") => 0.01 * count,
        ("music", "stability-audio") => 0.08,
        ("music", "elevenlabs-music") => 0.10,
        ("music", "mock-music") => 0.0,
        (_, id) if id.contains("image") => 0.04 * count,
        _ => 0.10 * (seconds / 5.0).ceil(),
    }
}

pub async fn download_with_auth(
    profile: &ProviderProfile,
    secret: Option<&str>,
    url: &str,
    dest: &Path,
) -> AppResult<()> {
    let client = client()?;
    let mut request = client.get(url);
    if profile.id == "veo-video" {
        if let Some(secret) = secret {
            request = request.header("x-goog-api-key", secret);
        }
    }
    let response = request.send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(error_from_response(status, &body));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let bytes = response.bytes().await?;
    std::fs::write(dest, bytes)?;
    Ok(())
}

pub fn output_ext(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        _ => "png",
    }
}

pub fn sniff_image_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/png"
    }
}
