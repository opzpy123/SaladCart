use crate::error::{AppError, AppResult};
use crate::models::ProviderProfile;
use crate::providers;
use base64::Engine;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct VideoRequest {
    pub model: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub aspect_ratio: String,
    pub duration_ms: i64,
    pub seed: Option<i64>,
    pub resolution: Option<String>,
    pub start_frame: Option<PathBuf>,
    pub references: Vec<PathBuf>,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct VideoTask {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub output_url: Option<String>,
    pub error: Option<String>,
    pub raw: Value,
}

pub async fn submit(
    profile: &ProviderProfile,
    secret: Option<&str>,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    match profile.id.as_str() {
        "kling-video" => kling_submit(profile, require(secret, profile)?, request).await,
        "luma-video" => luma_submit(profile, require(secret, profile)?, request).await,
        "runway-video" => runway_submit(profile, require(secret, profile)?, request).await,
        "minimax-video" => minimax_submit(profile, require(secret, profile)?, request).await,
        "veo-video" => veo_submit(profile, require(secret, profile)?, request).await,
        "replicate-video" => replicate_submit(profile, require(secret, profile)?, request).await,
        "fal-video" => fal_submit(profile, require(secret, profile)?, request).await,
        other => Err(AppError::Provider(format!(
            "video adapter not implemented for {other}"
        ))),
    }
}

pub async fn poll(
    profile: &ProviderProfile,
    secret: Option<&str>,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    match profile.id.as_str() {
        "kling-video" => kling_poll(profile, require(secret, profile)?, task).await,
        "luma-video" => luma_poll(profile, require(secret, profile)?, task).await,
        "runway-video" => runway_poll(profile, require(secret, profile)?, task).await,
        "minimax-video" => minimax_poll(profile, require(secret, profile)?, task).await,
        "veo-video" => veo_poll(profile, require(secret, profile)?, task).await,
        "replicate-video" => replicate_poll(profile, require(secret, profile)?, task).await,
        "fal-video" => fal_poll(profile, require(secret, profile)?, task).await,
        other => Err(AppError::Provider(format!(
            "video adapter not implemented for {other}"
        ))),
    }
}

fn require<'a>(secret: Option<&'a str>, profile: &ProviderProfile) -> AppResult<&'a str> {
    secret
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::MissingCredential(format!("{} API key", profile.label)))
}

fn prompt_with_negative(request: &VideoRequest) -> String {
    if request.negative_prompt.trim().is_empty() {
        request.prompt.clone()
    } else {
        format!(
            "{}\nAvoid: {}",
            request.prompt, request.negative_prompt
        )
    }
}

fn seconds(request: &VideoRequest) -> i64 {
    (request.duration_ms / 1000).clamp(2, 15)
}

fn image_b64(path: &PathBuf) -> AppResult<String> {
    let bytes = std::fs::read(path)?;
    if bytes.len() > 25 * 1024 * 1024 {
        return Err(AppError::Invalid("start frame is too large".into()));
    }
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

fn status_normalize(raw: &str) -> String {
    let raw = raw.to_ascii_lowercase();
    if raw.contains("succeed") || raw.contains("success") || raw.contains("complete") || raw == "done" {
        "succeeded".into()
    } else if raw.contains("fail") || raw.contains("error") || raw.contains("cancel") {
        "failed".into()
    } else if raw.contains("queue") || raw.contains("pending") || raw.contains("submit") || raw.contains("wait") {
        "pending".into()
    } else {
        "processing".into()
    }
}

fn first_str(value: &Value, pointers: &[&str]) -> Option<String> {
    for pointer in pointers {
        if let Some(v) = value.pointer(pointer) {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

fn first_status(value: &Value) -> String {
    first_str(
        value,
        &[
            "/status",
            "/state",
            "/task_status",
            "/data/task_status",
            "/data/status",
            "/output/status",
        ],
    )
    .map(|s| status_normalize(&s))
    .unwrap_or_else(|| "processing".into())
}

async fn kling_submit(
    profile: &ProviderProfile,
    secret: &str,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    let kind = if request.start_frame.is_some() {
        "image2video"
    } else {
        "text2video"
    };
    let default_path = format!("/v1/videos/{kind}");
    let path = profile
        .options
        .get(format!("submitPath{}", if kind == "image2video" { "Image" } else { "Text" }))
        .and_then(|v| v.as_str())
        .unwrap_or(&default_path);
    let mut body = json!({
        "model_name": request.model,
        "prompt": prompt_with_negative(request),
        "negative_prompt": request.negative_prompt,
        "duration": seconds(request).to_string(),
        "aspect_ratio": request.aspect_ratio,
        "mode": "std"
    });
    if let Some(frame) = &request.start_frame {
        let field = profile
            .options
            .get("imageField")
            .and_then(|v| v.as_str())
            .unwrap_or("image");
        body[field] = json!(image_b64(frame)?);
    }
    let auth = kling_auth(secret)?;
    let client = providers::client()?;
    let response = client
        .post(providers::endpoint(profile, path))
        .bearer_auth(auth)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let id = first_str(
        &raw,
        &["/data/task_id", "/task_id", "/data/id", "/id", "/data/taskId"],
    )
    .ok_or_else(|| AppError::Provider("Kling returned no task id".into()))?;
    let status = first_status(&raw);
    Ok(VideoTask {
        id,
        kind: kind.into(),
        status,
        output_url: kling_output(&raw),
        error: None,
        raw,
    })
}

async fn kling_poll(
    profile: &ProviderProfile,
    secret: &str,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    let default_path = format!("/v1/videos/{}/{}", task.kind, task.id);
    let path = profile
        .options
        .get("pollPath")
        .and_then(|v| v.as_str())
        .map(|p| p.replace("{kind}", &task.kind).replace("{id}", &task.id))
        .unwrap_or(default_path);
    let auth = kling_auth(secret)?;
    let client = providers::client()?;
    let response = client
        .get(providers::endpoint(profile, &path))
        .bearer_auth(auth)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let status = first_status(&raw);
    let error = first_str(&raw, &["/data/task_status_msg", "/message", "/error"]);
    Ok(VideoTask {
        id: task.id.clone(),
        kind: task.kind.clone(),
        output_url: kling_output(&raw),
        status,
        error,
        raw,
    })
}

fn kling_output(raw: &Value) -> Option<String> {
    first_str(
        raw,
        &[
            "/data/task_result/videos/0/url",
            "/data/result/videos/0/url",
            "/task_result/videos/0/url",
            "/data/videos/0/url",
            "/video_url",
            "/data/video_url",
        ],
    )
}

fn kling_auth(secret: &str) -> AppResult<String> {
    if let Some((ak, sk)) = secret.split_once(':') {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;
        let claims = json!({"iss": ak, "exp": now + 1800, "nbf": now - 5});
        return encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(sk.as_bytes()),
        )
        .map_err(|e| AppError::Provider(format!("could not sign Kling JWT: {e}")));
    }
    Ok(secret.to_string())
}

fn luma_ratio(aspect: &str) -> &'static str {
    match aspect {
        "16:9" => "16:9",
        "1:1" => "1:1",
        "4:5" => "4:5",
        "9:16" => "9:16",
        _ => "9:16",
    }
}

async fn luma_submit(
    profile: &ProviderProfile,
    secret: &str,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let mut body = json!({
        "prompt": prompt_with_negative(request),
        "model": request.model,
        "aspect_ratio": luma_ratio(&request.aspect_ratio),
        "duration": format!("{}s", seconds(request)),
        "resolution": request.resolution.clone().unwrap_or_else(|| "720p".into())
    });
    if let Some(frame) = &request.start_frame {
        body["keyframes"] = json!({
            "frame0": {"type": "image", "url": providers::data_uri(frame)?}
        });
    }
    let path = profile
        .options
        .get("submitPath")
        .and_then(|v| v.as_str())
        .unwrap_or("/dream-machine/v1/generations");
    let response = client
        .post(providers::endpoint(profile, path))
        .bearer_auth(secret)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let id = first_str(&raw, &["/id", "/generation/id"])
        .ok_or_else(|| AppError::Provider("Luma returned no generation id".into()))?;
    let status = first_status(&raw);
    let output_url = first_str(
        &raw,
        &["/assets/video", "/assets/video/url", "/output/0/url", "/video/url"],
    );
    Ok(VideoTask {
        id,
        kind: "generation".into(),
        status,
        output_url,
        error: None,
        raw,
    })
}

async fn luma_poll(
    profile: &ProviderProfile,
    secret: &str,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    let path = format!("/dream-machine/v1/generations/{}", task.id);
    let client = providers::client()?;
    let response = client
        .get(providers::endpoint(profile, &path))
        .bearer_auth(secret)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let status = first_status(&raw);
    let output_url = first_str(
        &raw,
        &["/assets/video", "/assets/video/url", "/output/0/url", "/video/url"],
    );
    let error = first_str(&raw, &["/failure_reason", "/error", "/message"]);
    Ok(VideoTask {
        id: task.id.clone(),
        kind: task.kind.clone(),
        status,
        output_url,
        error,
        raw,
    })
}

fn runway_ratio(aspect: &str) -> &'static str {
    match aspect {
        "16:9" => "1280:720",
        "1:1" => "960:960",
        "9:16" => "720:1280",
        _ => "720:1280",
    }
}

async fn runway_submit(
    profile: &ProviderProfile,
    secret: &str,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let has_image = request.start_frame.is_some();
    let path = if has_image {
        "/v1/image_to_video"
    } else {
        "/v1/text_to_video"
    };
    let mut body = json!({
        "model": request.model,
        "promptText": prompt_with_negative(request),
        "duration": seconds(request),
        "ratio": runway_ratio(&request.aspect_ratio)
    });
    if let Some(frame) = &request.start_frame {
        body["promptImage"] = json!(providers::data_uri(frame)?);
    }
    let response = client
        .post(providers::endpoint(profile, path))
        .bearer_auth(secret)
        .header("X-Runway-Version", "2024-11-06")
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let id = first_str(&raw, &["/id", "/task/id"])
        .ok_or_else(|| AppError::Provider("Runway returned no task id".into()))?;
    let status = first_status(&raw);
    Ok(VideoTask {
        id,
        kind: if has_image { "image_to_video" } else { "text_to_video" }.into(),
        status,
        output_url: runway_output(&raw),
        error: first_str(&raw, &["/failure", "/error"]),
        raw,
    })
}

async fn runway_poll(
    profile: &ProviderProfile,
    secret: &str,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let response = client
        .get(providers::endpoint(profile, &format!("/v1/tasks/{}", task.id)))
        .bearer_auth(secret)
        .header("X-Runway-Version", "2024-11-06")
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    Ok(VideoTask {
        id: task.id.clone(),
        kind: task.kind.clone(),
        status: first_status(&raw),
        output_url: runway_output(&raw),
        error: first_str(&raw, &["/failure", "/error"]),
        raw,
    })
}

fn runway_output(raw: &Value) -> Option<String> {
    first_str(raw, &["/output/0", "/output/0/url", "/result/0", "/video_url"])
}

async fn minimax_submit(
    profile: &ProviderProfile,
    secret: &str,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let group_id = profile
        .options
        .get("groupId")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let path = if group_id.is_empty() {
        "/v1/video_generation".to_string()
    } else {
        format!("/v1/video_generation?GroupId={group_id}")
    };
    let mut body = json!({
        "model": request.model,
        "prompt": prompt_with_negative(request),
        "duration": seconds(request).max(6),
        "resolution": request.resolution.clone().unwrap_or_else(|| "768P".into())
    });
    if let Some(frame) = &request.start_frame {
        body["first_frame_image"] = json!(providers::data_uri(frame)?);
    }
    let response = client
        .post(providers::endpoint(profile, &path))
        .bearer_auth(secret)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let id = first_str(&raw, &["/task_id", "/data/task_id", "/id"])
        .ok_or_else(|| AppError::Provider("MiniMax returned no task id".into()))?;
    Ok(VideoTask {
        id,
        kind: "video_generation".into(),
        status: first_status(&raw),
        output_url: None,
        error: None,
        raw,
    })
}

async fn minimax_poll(
    profile: &ProviderProfile,
    secret: &str,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let group_id = profile
        .options
        .get("groupId")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let mut query = format!("/v1/query/video_generation?task_id={}", task.id);
    if !group_id.is_empty() {
        query.push_str(&format!("&GroupId={group_id}"));
    }
    let response = client
        .get(providers::endpoint(profile, &query))
        .bearer_auth(secret)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let mut raw: Value = response.json().await?;
    let mut status = first_status(&raw);
    let mut output_url = first_str(
        &raw,
        &["/file/download_url", "/data/file/download_url", "/video_url"],
    );
    if status == "succeeded" && output_url.is_none() {
        if let Some(file_id) = first_str(&raw, &["/file_id", "/data/file_id"]) {
            let path = if group_id.is_empty() {
                format!("/v1/files/retrieve?file_id={file_id}")
            } else {
                format!("/v1/files/retrieve?file_id={file_id}&GroupId={group_id}")
            };
            let file_response = client
                .get(providers::endpoint(profile, &path))
                .bearer_auth(secret)
                .send()
                .await?;
            if file_response.status().is_success() {
                if let Ok(file_json) = file_response.json::<Value>().await {
                    output_url = first_str(
                        &file_json,
                        &["/file/download_url", "/data/file/download_url", "/download_url"],
                    );
                    raw["retrieved_file"] = file_json;
                }
            }
        }
        if output_url.is_none() {
            status = "processing".into();
        }
    }
    Ok(VideoTask {
        id: task.id.clone(),
        kind: task.kind.clone(),
        status,
        output_url,
        error: first_str(&raw, &["/base_resp/status_msg", "/error", "/message"]),
        raw,
    })
}

async fn veo_submit(
    profile: &ProviderProfile,
    secret: &str,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let mut instance = json!({"prompt": prompt_with_negative(request)});
    if let Some(frame) = &request.start_frame {
        instance["image"] = json!({
            "bytesBase64Encoded": image_b64(frame)?,
            "mimeType": providers::image_mime(frame)
        });
    }
    let body = json!({
        "instances": [instance],
        "parameters": {
            "aspectRatio": request.aspect_ratio,
            "durationSeconds": seconds(request).clamp(4, 8),
            "resolution": request.resolution.clone().unwrap_or_else(|| "1080p".into()),
            "sampleCount": 1
        }
    });
    let path = format!("models/{}:predictLongRunning", request.model);
    let response = client
        .post(providers::endpoint(profile, &path))
        .header("x-goog-api-key", secret)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let id = first_str(&raw, &["/name", "/operation/name"])
        .ok_or_else(|| AppError::Provider("Veo returned no operation name".into()))?;
    Ok(VideoTask {
        id,
        kind: "operation".into(),
        status: "pending".into(),
        output_url: None,
        error: None,
        raw,
    })
}

async fn veo_poll(
    profile: &ProviderProfile,
    secret: &str,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let url = if task.id.starts_with("http") {
        task.id.clone()
    } else {
        format!(
            "{}/{}",
            providers::base(profile),
            task.id.trim_start_matches('/')
        )
    };
    let response = client
        .get(url)
        .header("x-goog-api-key", secret)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let done = raw.get("done").and_then(|v| v.as_bool()).unwrap_or(false);
    let output_url = first_str(
        &raw,
        &[
            "/response/generateVideoResponse/generatedSamples/0/video/uri",
            "/response/generatedVideos/0/video/uri",
            "/response/videos/0/uri",
            "/response/generatedVideos/0/videoUrl",
            "/response/predictions/0/video/uri",
        ],
    );
    let status = if raw.get("error").is_some() {
        "failed"
    } else if done && output_url.is_some() {
        "succeeded"
    } else if done {
        "failed"
    } else {
        "processing"
    };
    Ok(VideoTask {
        id: task.id.clone(),
        kind: task.kind.clone(),
        status: status.into(),
        output_url,
        error: first_str(&raw, &["/error/message", "/error"]),
        raw,
    })
}

async fn replicate_submit(
    profile: &ProviderProfile,
    secret: &str,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let mut input = json!({
        "prompt": prompt_with_negative(request),
        "aspect_ratio": request.aspect_ratio,
        "duration": seconds(request)
    });
    if let Some(seed) = request.seed {
        input["seed"] = json!(seed);
    }
    if let Some(frame) = &request.start_frame {
        let field = if request.model.contains("hailuo") || request.model.contains("minimax") {
            "first_frame_image"
        } else if request.model.contains("kling") {
            "start_image"
        } else if request.model.contains("wan") {
            "image"
        } else {
            "start_image"
        };
        input[field] = json!(providers::data_uri(frame)?);
    }
    let path = format!("models/{}/predictions", request.model);
    let response = client
        .post(providers::endpoint(profile, &path))
        .bearer_auth(secret)
        .json(&json!({"input": input}))
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let id = first_str(&raw, &["/id"])
        .ok_or_else(|| AppError::Provider("Replicate returned no prediction id".into()))?;
    Ok(VideoTask {
        id,
        kind: "prediction".into(),
        status: first_status(&raw),
        output_url: None,
        error: None,
        raw,
    })
}

async fn replicate_poll(
    profile: &ProviderProfile,
    secret: &str,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let response = client
        .get(providers::endpoint(
            profile,
            &format!("predictions/{}", task.id),
        ))
        .bearer_auth(secret)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let output = raw.get("output").cloned().unwrap_or(Value::Null);
    let output_url = output
        .as_str()
        .map(str::to_string)
        .or_else(|| {
            output
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            first_str(
                &raw,
                &[
                    "/output/0/url",
                    "/output/video",
                    "/output/video/url",
                    "/output/video_url",
                ],
            )
        });
    Ok(VideoTask {
        id: task.id.clone(),
        kind: task.kind.clone(),
        status: first_status(&raw),
        output_url,
        error: first_str(&raw, &["/error", "/error/message"]),
        raw,
    })
}

async fn fal_submit(
    profile: &ProviderProfile,
    secret: &str,
    request: &VideoRequest,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let path = request.model.trim_start_matches('/');
    let mut input = json!({
        "prompt": prompt_with_negative(request),
        "aspect_ratio": request.aspect_ratio,
        "duration": seconds(request)
    });
    if let Some(frame) = &request.start_frame {
        input["image_url"] = json!(providers::data_uri(frame)?);
    }
    let response = client
        .post(providers::endpoint(profile, path))
        .header("Authorization", format!("Key {secret}"))
        .json(&json!({"input": input}))
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let raw: Value = response.json().await?;
    let id = first_str(&raw, &["/request_id"])
        .ok_or_else(|| AppError::Provider("fal returned no request_id".into()))?;
    Ok(VideoTask {
        id,
        kind: "request".into(),
        status: first_status(&raw),
        output_url: None,
        error: None,
        raw,
    })
}

async fn fal_poll(
    profile: &ProviderProfile,
    secret: &str,
    task: &VideoTask,
) -> AppResult<VideoTask> {
    let client = providers::client()?;
    let status_url = first_str(&task.raw, &["/status_url"]).unwrap_or_else(|| {
        format!(
            "{}/requests/{}/status",
            providers::base(profile),
            task.id
        )
    });
    let response = client
        .get(status_url)
        .header("Authorization", format!("Key {secret}"))
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let mut raw: Value = response.json().await?;
    let status = first_status(&raw);
    if status == "succeeded" {
        if let Some(response_url) = first_str(&task.raw, &["/response_url"]) {
            let result = client
                .get(response_url)
                .header("Authorization", format!("Key {secret}"))
                .send()
                .await?;
            if result.status().is_success() {
                if let Ok(json) = result.json::<Value>().await {
                    raw["result"] = json;
                }
            }
        }
    }
    let output_url = first_str(
        &raw,
        &[
            "/result/video/url",
            "/result/videos/0/url",
            "/result/output/0/url",
            "/video/url",
        ],
    );
    Ok(VideoTask {
        id: task.id.clone(),
        kind: task.kind.clone(),
        status,
        output_url,
        error: first_str(&raw, &["/detail", "/error"]),
        raw,
    })
}

pub async fn poll_delay(profile: &ProviderProfile) {
    let seconds = profile
        .options
        .get("poll_interval_sec")
        .and_then(|v| v.as_u64())
        .unwrap_or(5)
        .clamp(2, 30);
    tokio::time::sleep(Duration::from_secs(seconds)).await;
}

pub fn timeout(profile: &ProviderProfile) -> Duration {
    Duration::from_secs(
        profile
            .options
            .get("timeout_sec")
            .and_then(|v| v.as_u64())
            .unwrap_or(900),
    )
}
