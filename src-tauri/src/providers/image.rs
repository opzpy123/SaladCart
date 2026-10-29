use crate::error::{AppError, AppResult};
use crate::models::ProviderProfile;
use crate::providers;
use base64::Engine;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

pub struct ImageRef {
    pub path: PathBuf,
    pub mime: String,
}

pub struct ImageRequest {
    pub model: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub aspect_ratio: String,
    pub width: i64,
    pub height: i64,
    pub count: u32,
    pub seed: Option<i64>,
    pub references: Vec<ImageRef>,
    pub title: String,
}

pub struct ImageResult {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub extension: String,
    pub cost: f64,
    pub raw: Value,
}

pub async fn generate(
    profile: &ProviderProfile,
    secret: Option<&str>,
    request: &ImageRequest,
) -> AppResult<ImageResult> {
    if profile.id.starts_with("mock-") {
        return crate::providers::mock::image(request);
    }
    match profile.id.as_str() {
        "openai-image" => openai(profile, require(secret, profile)?, request).await,
        "gemini-image" => gemini(profile, require(secret, profile)?, request).await,
        "replicate-image" => replicate(profile, require(secret, profile)?, request).await,
        "stability-image" => stability(profile, require(secret, profile)?, request).await,
        "fal-image" => fal(profile, require(secret, profile)?, request).await,
        "ark-image" => ark(profile, require(secret, profile)?, request).await,
        "comfyui-image" => comfyui(profile, request).await,
        other => Err(AppError::Provider(format!(
            "image adapter not implemented for {other}"
        ))),
    }
}

fn require<'a>(secret: Option<&'a str>, profile: &ProviderProfile) -> AppResult<&'a str> {
    secret
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::MissingCredential(format!("{} API key", profile.label)))
}

fn full_prompt(request: &ImageRequest) -> String {
    if request.negative_prompt.trim().is_empty() {
        request.prompt.clone()
    } else {
        format!(
            "{}\n\nStrictly avoid: {}",
            request.prompt, request.negative_prompt
        )
    }
}

fn openai_size(aspect: &str) -> &'static str {
    match aspect {
        "16:9" => "1536x1024",
        "1:1" => "1024x1024",
        _ => "1024x1536",
    }
}

async fn openai(
    profile: &ProviderProfile,
    secret: &str,
    request: &ImageRequest,
) -> AppResult<ImageResult> {
    let client = providers::client()?;
    let size = openai_size(&request.aspect_ratio);
    let prompt = full_prompt(request);
    let response = if request.references.is_empty() {
        let url = providers::endpoint(profile, "images/generations");
        client
            .post(url)
            .bearer_auth(secret)
            .json(&json!({
                "model": request.model,
                "prompt": prompt,
                "n": request.count.clamp(1, 4),
                "size": size
            }))
            .send()
            .await?
    } else {
        let url = providers::endpoint(profile, "images/edits");
        let mut form = reqwest::multipart::Form::new()
            .text("model", request.model.clone())
            .text("prompt", prompt)
            .text("n", request.count.clamp(1, 4).to_string())
            .text("size", size.to_string());
        for (index, reference) in request.references.iter().take(4).enumerate() {
            let bytes = std::fs::read(&reference.path)?;
            let part = reqwest::multipart::Part::bytes(bytes)
                .file_name(format!("reference-{index}.png"))
                .mime_str(&reference.mime)
                .map_err(|e| AppError::Message(e.to_string()))?;
            form = form.part("image[]", part);
        }
        client
            .post(url)
            .bearer_auth(secret)
            .multipart(form)
            .send()
            .await?
    };
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let value: Value = response.json().await?;
    let data = value
        .get("data")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .ok_or_else(|| AppError::Provider("OpenAI Images returned no data".into()))?;
    let bytes = if let Some(b64) = data.get("b64_json").and_then(|v| v.as_str()) {
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| AppError::Provider(format!("invalid base64 image: {e}")))?
    } else if let Some(url) = data.get("url").and_then(|v| v.as_str()) {
        providers::client()?
            .get(url)
            .send()
            .await?
            .bytes()
            .await?
            .to_vec()
    } else {
        return Err(AppError::Provider(
            "OpenAI Images returned neither b64_json nor url".into(),
        ));
    };
    let mime = providers::sniff_image_mime(&bytes).to_string();
    Ok(ImageResult {
        extension: providers::output_ext(&mime).to_string(),
        mime,
        cost: providers::estimate_cost(&profile.id, "image", 0, request.count),
        bytes,
        raw: value,
    })
}

async fn gemini(
    profile: &ProviderProfile,
    secret: &str,
    request: &ImageRequest,
) -> AppResult<ImageResult> {
    let client = providers::client()?;
    let prompt = full_prompt(request);
    let url = if request.model.starts_with("imagen") {
        providers::endpoint(profile, &format!("models/{}:predict", request.model))
    } else {
        providers::endpoint(
            profile,
            &format!("models/{}:generateContent", request.model),
        )
    };
    let body = if request.model.starts_with("imagen") {
        json!({
            "instances": [{"prompt": prompt}],
            "parameters": {
                "sampleCount": request.count.clamp(1, 4),
                "aspectRatio": request.aspect_ratio
            }
        })
    } else {
        let mut parts = vec![json!({"text": prompt})];
        for reference in &request.references {
            let bytes = std::fs::read(&reference.path)?;
            parts.push(json!({
                "inlineData": {
                    "mimeType": reference.mime.clone(),
                    "data": base64::engine::general_purpose::STANDARD.encode(bytes)
                }
            }));
        }
        json!({
            "contents": [{"role": "user", "parts": parts}],
            "generationConfig": {
                "responseModalities": ["TEXT", "IMAGE"],
                "imageConfig": {"aspectRatio": request.aspect_ratio}
            }
        })
    };
    let response = client
        .post(url)
        .header("x-goog-api-key", secret)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let value: Value = response.json().await?;
    let b64 = if request.model.starts_with("imagen") {
        value
            .pointer("/predictions/0/bytesBase64Encoded")
            .and_then(|v| v.as_str())
    } else {
        value
            .pointer("/candidates/0/content/parts")
            .and_then(|v| v.as_array())
            .and_then(|parts| {
                parts.iter().find_map(|part| {
                    part.pointer("/inlineData/data")
                        .and_then(|v| v.as_str())
                        .or_else(|| {
                            part.pointer("/inline_data/data")
                                .and_then(|v| v.as_str())
                        })
                })
            })
    }
    .ok_or_else(|| {
        AppError::Provider(
            "Gemini returned no image. The prompt may have been blocked, or this model may not support image output.".into(),
        )
    })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| AppError::Provider(format!("invalid base64 image: {e}")))?;
    let mime = providers::sniff_image_mime(&bytes).to_string();
    Ok(ImageResult {
        extension: providers::output_ext(&mime).to_string(),
        mime,
        cost: providers::estimate_cost(&profile.id, "image", 0, request.count),
        bytes,
        raw: value,
    })
}

async fn replicate(
    profile: &ProviderProfile,
    secret: &str,
    request: &ImageRequest,
) -> AppResult<ImageResult> {
    let client = providers::client()?;
    let mut input = json!({
        "prompt": full_prompt(request),
        "num_outputs": request.count.clamp(1, 4)
    });
    if request.model.contains("flux") {
        input["aspect_ratio"] = json!(request.aspect_ratio);
        input["output_format"] = json!("png");
    } else {
        input["negative_prompt"] = json!(request.negative_prompt);
        input["width"] = json!(request.width);
        input["height"] = json!(request.height);
    }
    if let Some(seed) = request.seed {
        input["seed"] = json!(seed);
    }
    let version = profile
        .options
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let (url, body) = if version.is_empty() {
        (
            providers::endpoint(profile, &format!("models/{}/predictions", request.model)),
            json!({"input": input}),
        )
    } else {
        (
            providers::endpoint(profile, "predictions"),
            json!({"version": version, "input": input}),
        )
    };
    let response = client
        .post(url)
        .bearer_auth(secret)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let mut prediction: Value = response.json().await?;
    let mut attempts = 0;
    loop {
        let status = prediction
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("starting");
        if status == "succeeded" {
            break;
        }
        if status == "failed" || status == "canceled" {
            return Err(AppError::Provider(format!(
                "Replicate image job {status}: {}",
                prediction
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
            )));
        }
        attempts += 1;
        if attempts > 90 {
            return Err(AppError::Provider("Replicate image job timed out".into()));
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        let poll_url = prediction
            .pointer("/urls/get")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| {
                prediction
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| providers::endpoint(profile, &format!("predictions/{id}")))
            })
            .ok_or_else(|| AppError::Provider("Replicate returned no poll URL".into()))?;
        let response = client.get(poll_url).bearer_auth(secret).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(providers::error_from_response(status, &body));
        }
        prediction = response.json().await?;
    }
    let output = prediction.get("output").cloned().unwrap_or(Value::Null);
    let url = output
        .as_str()
        .map(str::to_string)
        .or_else(|| {
            output
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .ok_or_else(|| AppError::Provider("Replicate returned an empty image output".into()))?;
    let bytes = client.get(url).send().await?.bytes().await?.to_vec();
    let mime = providers::sniff_image_mime(&bytes).to_string();
    Ok(ImageResult {
        extension: providers::output_ext(&mime).to_string(),
        mime,
        cost: providers::estimate_cost(&profile.id, "image", 0, request.count),
        bytes,
        raw: prediction,
    })
}

async fn stability(
    profile: &ProviderProfile,
    secret: &str,
    request: &ImageRequest,
) -> AppResult<ImageResult> {
    let client = providers::client()?;
    let model = if request.model.is_empty() {
        "core"
    } else {
        request.model.as_str()
    };
    let form = reqwest::multipart::Form::new()
        .text("prompt", full_prompt(request))
        .text("aspect_ratio", request.aspect_ratio.clone())
        .text("output_format", "png");
    let response = client
        .post(providers::endpoint(
            profile,
            &format!("/v2beta/stable-image/generate/{model}"),
        ))
        .bearer_auth(secret)
        .header("Accept", "image/*")
        .multipart(form)
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
        .unwrap_or("image/png")
        .to_string();
    if content_type.contains("json") {
        let value: Value = response.json().await?;
        return Err(AppError::Provider(format!(
            "Stability returned no image: {}",
            value
                .get("errors")
                .map(|v| v.to_string())
                .unwrap_or_else(|| value.to_string())
        )));
    }
    let bytes = response.bytes().await?.to_vec();
    let mime = providers::sniff_image_mime(&bytes).to_string();
    Ok(ImageResult {
        extension: providers::output_ext(&mime).to_string(),
        mime,
        cost: providers::estimate_cost(&profile.id, "image", 0, request.count),
        bytes,
        raw: json!({"provider": "stability", "model": model, "referencesIgnored": !request.references.is_empty()}),
    })
}

async fn fal(
    profile: &ProviderProfile,
    secret: &str,
    request: &ImageRequest,
) -> AppResult<ImageResult> {
    let client = providers::client()?;
    let path = request.model.trim_start_matches('/');
    let input = json!({
        "prompt": full_prompt(request),
        "aspect_ratio": request.aspect_ratio,
        "num_images": request.count.clamp(1, 4)
    });
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
    let submitted: Value = response.json().await?;
    let request_id = submitted
        .get("request_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Provider("fal returned no request_id".into()))?;
    let status_url = submitted
        .get("status_url")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}/requests/{request_id}/status", providers::base(profile)));
    let response_url = submitted
        .get("response_url")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    for _ in 0..90 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let status_response = client
            .get(&status_url)
            .header("Authorization", format!("Key {secret}"))
            .send()
            .await?;
        if !status_response.status().is_success() {
            continue;
        }
        let status_json: Value = status_response.json().await?;
        let status = status_json
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("IN_PROGRESS");
        if status == "FAILED" {
            return Err(AppError::Provider("fal image generation failed".into()));
        }
        if status == "COMPLETED" {
            let response_url = response_url
                .or_else(|| {
                    status_json
                        .get("response_url")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                })
                .ok_or_else(|| AppError::Provider("fal returned no response URL".into()))?;
            let result: Value = client
                .get(response_url)
                .header("Authorization", format!("Key {secret}"))
                .send()
                .await?
                .json()
                .await?;
            let url = result
                .pointer("/images/0/url")
                .and_then(|v| v.as_str())
                .or_else(|| result.get("image").and_then(|v| v.as_str()))
                .ok_or_else(|| AppError::Provider("fal returned no image URL".into()))?;
            let bytes = client.get(url).send().await?.bytes().await?.to_vec();
            let mime = providers::sniff_image_mime(&bytes).to_string();
            return Ok(ImageResult {
                extension: providers::output_ext(&mime).to_string(),
                mime,
                cost: providers::estimate_cost(&profile.id, "image", 0, request.count),
                bytes,
                raw: json!({"provider": "fal", "requestId": request_id, "response": result}),
            });
        }
    }
    Err(AppError::Provider("fal image generation timed out".into()))
}

fn ark_size(aspect: &str) -> &'static str {
    match aspect {
        "16:9" => "2560x1440",
        "1:1" => "1920x1920",
        "4:5" => "1728x2160",
        "3:4" => "1664x2218",
        _ => "1440x2560",
    }
}

async fn ark(
    profile: &ProviderProfile,
    secret: &str,
    request: &ImageRequest,
) -> AppResult<ImageResult> {
    let client = providers::client()?;
    let mut body = json!({
        "model": request.model,
        "prompt": full_prompt(request),
        "size": ark_size(&request.aspect_ratio),
        "response_format": "url",
        "watermark": false
    });
    if let Some(seed) = request.seed {
        body["seed"] = json!(seed);
    }
    if !request.references.is_empty() {
        let images: Vec<String> = request
            .references
            .iter()
            .take(4)
            .map(|reference| {
                let bytes = std::fs::read(&reference.path)?;
                Ok(format!(
                    "data:{};base64,{}",
                    reference.mime,
                    base64::engine::general_purpose::STANDARD.encode(bytes)
                ))
            })
            .collect::<AppResult<Vec<_>>>()?;
        body["image"] = if images.len() == 1 {
            json!(images[0])
        } else {
            json!(images)
        };
    }
    let response = client
        .post(providers::endpoint(profile, "images/generations"))
        .bearer_auth(secret)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::ark_error(status, &body));
    }
    let value: Value = response.json().await?;
    let data = value
        .get("data")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .ok_or_else(|| AppError::Provider("Ark Seedream returned no image data".into()))?;
    let bytes = if let Some(b64) = data.get("b64_json").and_then(|v| v.as_str()) {
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| AppError::Provider(format!("invalid Ark base64 image: {e}")))?
    } else if let Some(url) = data.get("url").and_then(|v| v.as_str()) {
        client.get(url).send().await?.bytes().await?.to_vec()
    } else {
        return Err(AppError::Provider(
            "Ark Seedream returned neither url nor b64_json".into(),
        ));
    };
    let mime = providers::sniff_image_mime(&bytes).to_string();
    Ok(ImageResult {
        extension: providers::output_ext(&mime).to_string(),
        mime,
        cost: providers::estimate_cost(&profile.id, "image", 0, request.count),
        bytes,
        raw: value,
    })
}

async fn comfyui(profile: &ProviderProfile, request: &ImageRequest) -> AppResult<ImageResult> {
    let client = crate::providers::client()?;
    let mut reference_name: Option<String> = None;
    if let Some(reference) = request.references.first() {
        let url = providers::endpoint(profile, "upload/image");
        let bytes = std::fs::read(&reference.path)?;
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(format!(
                "saladcart-ref.{}",
                providers::output_ext(&reference.mime)
            ))
            .mime_str(&reference.mime)
            .map_err(|e| AppError::Message(e.to_string()))?;
        let response = client
            .post(url)
            .multipart(reqwest::multipart::Form::new().part("image", part))
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(providers::error_from_response(status, &body));
        }
        let value: Value = response.json().await?;
        reference_name = value
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
    }

    let checkpoint = profile
        .options
        .get("checkpoint")
        .and_then(|v| v.as_str())
        .unwrap_or("sd_xl_base_1.0.safetensors");
    let workflow = profile.options.get("workflow").cloned().unwrap_or_else(|| {
        json!({
            "3": {"class_type":"CheckpointLoaderSimple","inputs":{"ckpt_name":checkpoint}},
            "5": {"class_type":"EmptyLatentImage","inputs":{"width":request.width.max(512),"height":request.height.max(512),"batch_size":1}},
            "6": {"class_type":"CLIPTextEncode","inputs":{"text":request.prompt,"clip":["3",1]}},
            "7": {"class_type":"CLIPTextEncode","inputs":{"text":request.negative_prompt,"clip":["3",1]}},
            "4": {"class_type":"KSampler","inputs":{"seed":request.seed.unwrap_or(42),"steps":28,"cfg":6.5,"sampler_name":"euler","scheduler":"normal","denoise":1.0,"model":["3",0],"positive":["6",0],"negative":["7",0],"latent_image":["5",0]}},
            "8": {"class_type":"VAEDecode","inputs":{"samples":["4",0],"vae":["3",2]}},
            "9": {"class_type":"SaveImage","inputs":{"filename_prefix":"saladcart","images":["8",0]}}
        })
    });
    let mut workflow_text = serde_json::to_string(&workflow)?;
    if workflow_text.contains("{{REF_IMAGE}}") {
        let name = reference_name.clone().unwrap_or_default();
        workflow_text = workflow_text.replace("{{REF_IMAGE}}", &escape_json(&name));
    }
    workflow_text = workflow_text
        .replace("{{PROMPT}}", &escape_json(&request.prompt))
        .replace("{{NEGATIVE}}", &escape_json(&request.negative_prompt))
        .replace("{{SEED}}", &request.seed.unwrap_or(42).to_string())
        .replace("{{WIDTH}}", &request.width.to_string())
        .replace("{{HEIGHT}}", &request.height.to_string());
    let workflow: Value = serde_json::from_str(&workflow_text)?;
    let client_id = uuid::Uuid::new_v4().to_string();
    let response = client
        .post(providers::endpoint(profile, "prompt"))
        .json(&json!({"prompt": workflow, "client_id": client_id}))
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let value: Value = response.json().await?;
    let prompt_id = value
        .get("prompt_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Provider("ComfyUI rejected the workflow".into()))?;
    let mut image_info: Option<Value> = None;
    for _ in 0..180 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let response = client
            .get(providers::endpoint(
                profile,
                &format!("history/{prompt_id}"),
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            continue;
        }
        let history: Value = response.json().await?;
        if let Some(outputs) = history
            .get(prompt_id)
            .and_then(|v| v.get("outputs"))
            .and_then(|v| v.as_object())
        {
            for output in outputs.values() {
                if let Some(images) = output.get("images").and_then(|v| v.as_array()) {
                    if let Some(first) = images.first() {
                        image_info = Some(first.clone());
                        break;
                    }
                }
            }
        }
        if image_info.is_some() {
            break;
        }
    }
    let image = image_info.ok_or_else(|| {
        AppError::Provider("ComfyUI did not return an image within 180 seconds".into())
    })?;
    let filename = image
        .get("filename")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Provider("ComfyUI output has no filename".into()))?;
    let subfolder = image
        .get("subfolder")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let kind = image
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("output");
    let url = format!(
        "{}?filename={}&subfolder={}&type={}",
        providers::endpoint(profile, "view"),
        urlencode(filename),
        urlencode(subfolder),
        urlencode(kind)
    );
    let bytes = client.get(url).send().await?.bytes().await?.to_vec();
    let mime = providers::sniff_image_mime(&bytes).to_string();
    Ok(ImageResult {
        extension: providers::output_ext(&mime).to_string(),
        mime,
        cost: 0.0,
        bytes,
        raw: json!({"promptId": prompt_id, "image": image, "referenceUploaded": reference_name.is_some()}),
    })
}

fn escape_json(value: &str) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "\"\"".into())
        .trim_matches('"')
        .to_string()
}

fn urlencode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".into(),
            other => other
                .to_string()
                .bytes()
                .map(|b| format!("%{b:02X}"))
                .collect(),
        })
        .collect()
}
