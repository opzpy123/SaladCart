use crate::error::{AppError, AppResult};
use crate::models::{ProviderDefaults, Settings};
use crate::paths::AppPaths;
use crate::secrets;
use serde_json::json;
use std::path::Path;

pub fn default_settings() -> Settings {
    let mut providers = Vec::new();
    for (id, label, base, model) in [
        (
            "mock-llm",
            "Mock 编剧 (离线)",
            "local://mock",
            "saladcart-mock-llm",
        ),
        (
            "openai-llm",
            "OpenAI / 兼容接口",
            "https://api.openai.com/v1",
            "gpt-4.1-mini",
        ),
        (
            "deepseek-llm",
            "DeepSeek",
            "https://api.deepseek.com/v1",
            "deepseek-chat",
        ),
        (
            "qwen-llm",
            "通义千问 / DashScope",
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "qwen-plus",
        ),
        (
            "ollama-llm",
            "Ollama (本地)",
            "http://127.0.0.1:11434/v1",
            "qwen2.5:7b",
        ),
    ] {
        providers.push(crate::models::ProviderProfile {
            id: id.into(),
            kind: "llm".into(),
            label: label.into(),
            vendor: id.split('-').next().unwrap_or(id).into(),
            enabled: id == "mock-llm",
            builtin: true,
            api_base: base.into(),
            default_model: model.into(),
            models: vec![model.into()],
            api_key_set: false,
            env_var: match id {
                "openai-llm" => "OPENAI_API_KEY",
                "deepseek-llm" => "DEEPSEEK_API_KEY",
                "qwen-llm" => "DASHSCOPE_API_KEY",
                _ => "",
            }
            .into(),
            options: json!({"temperature": 0.8}),
            capabilities: crate::models::ProviderCapabilities {
                structured_output: true,
                ..Default::default()
            },
        });
    }
    for (id, label, base, model, env, models) in [
        (
            "mock-image",
            "Mock 画师 (离线)",
            "local://mock",
            "saladcart-mock-image",
            "",
            vec!["saladcart-mock-image"],
        ),
        (
            "openai-image",
            "OpenAI Images",
            "https://api.openai.com/v1",
            "gpt-image-1",
            "OPENAI_API_KEY",
            vec!["gpt-image-1"],
        ),
        (
            "gemini-image",
            "Gemini Image",
            "https://generativelanguage.googleapis.com/v1beta",
            "gemini-2.5-flash-image",
            "GEMINI_API_KEY",
            vec!["gemini-2.5-flash-image", "imagen-3.0-generate-002"],
        ),
        (
            "replicate-image",
            "Replicate",
            "https://api.replicate.com/v1",
            "black-forest-labs/flux-schnell",
            "REPLICATE_API_TOKEN",
            vec!["black-forest-labs/flux-schnell"],
        ),
        (
            "comfyui-image",
            "ComfyUI (本地)",
            "http://127.0.0.1:8188",
            "default",
            "",
            vec!["default"],
        ),
        (
            "stability-image",
            "Stability AI",
            "https://api.stability.ai",
            "core",
            "STABILITY_API_KEY",
            vec!["core", "sd3", "ultra"],
        ),
        (
            "fal-image",
            "fal.ai",
            "https://queue.fal.run",
            "fal-ai/flux/schnell",
            "FAL_KEY",
            vec!["fal-ai/flux/schnell", "fal-ai/nano-banana"],
        ),
        (
            "ark-image",
            "火山方舟 Seedream",
            "https://ark.cn-beijing.volces.com/api/v3",
            "doubao-seedream-5-0-flash-260915",
            "ARK_API_KEY",
            vec![
                "doubao-seedream-5-0-flash-260915",
                "doubao-seedream-5-0-pro-260628",
                "doubao-seedream-5-0-260128",
            ],
        ),
    ] {
        providers.push(crate::models::ProviderProfile {
            id: id.into(),
            kind: "image".into(),
            label: label.into(),
            vendor: id.split('-').next().unwrap_or(id).into(),
            enabled: id == "mock-image",
            builtin: true,
            api_base: base.into(),
            default_model: model.into(),
            models: models.into_iter().map(String::from).collect(),
            api_key_set: false,
            env_var: env.into(),
            options: json!({}),
            capabilities: crate::models::ProviderCapabilities {
                aspect_ratios: vec![
                    "9:16".into(),
                    "16:9".into(),
                    "1:1".into(),
                    "3:4".into(),
                    "4:3".into(),
                ],
                reference_image: true,
                max_reference_images: 4,
                ..Default::default()
            },
        });
    }
    for (id, label, base, model, env, caps) in [
        (
            "mock-video",
            "Mock 动画 (离线)",
            "local://mock",
            "saladcart-mock-video",
            "",
            (vec!["9:16", "16:9", "1:1"], vec![3, 5, 8], true, true),
        ),
        (
            "kling-video",
            "可灵 Kling",
            "https://api-singapore.klingai.com",
            "kling-v2-1",
            "KLING_API_KEY",
            (vec!["9:16", "16:9", "1:1"], vec![5, 10], true, true),
        ),
        (
            "luma-video",
            "Luma Dream Machine",
            "https://api.lumalabs.ai",
            "ray-2",
            "LUMA_API_KEY",
            (vec!["9:16", "16:9", "1:1"], vec![5, 9], true, true),
        ),
        (
            "runway-video",
            "Runway",
            "https://api.dev.runwayml.com",
            "gen4_turbo",
            "RUNWAY_API_KEY",
            (vec!["9:16", "16:9", "1:1"], vec![5, 10], true, true),
        ),
        (
            "minimax-video",
            "MiniMax 海螺",
            "https://api.minimax.chat",
            "MiniMax-Hailuo-02",
            "MINIMAX_API_KEY",
            (vec!["9:16", "16:9", "1:1"], vec![6, 10], true, true),
        ),
        (
            "replicate-video",
            "Replicate Video",
            "https://api.replicate.com/v1",
            "minimax/hailuo-02",
            "REPLICATE_API_TOKEN",
            (vec!["9:16", "16:9", "1:1"], vec![5, 10], true, true),
        ),
        (
            "veo-video",
            "Google Veo",
            "https://generativelanguage.googleapis.com/v1beta",
            "veo-3.1-generate-preview",
            "GEMINI_API_KEY",
            (vec!["9:16", "16:9"], vec![4, 6, 8], true, true),
        ),
        (
            "fal-video",
            "fal.ai Video",
            "https://queue.fal.run",
            "fal-ai/kling-video/v2.5-turbo/pro/image-to-video",
            "FAL_KEY",
            (vec!["9:16", "16:9", "1:1"], vec![5, 10], true, true),
        ),
        (
            "ark-video",
            "火山方舟 Seedance",
            "https://ark.cn-beijing.volces.com/api/v3",
            "doubao-seedance-1-0-pro-fast-251015",
            "ARK_API_KEY",
            (vec!["9:16", "16:9", "1:1"], vec![5, 10], true, true),
        ),
    ] {
        providers.push(crate::models::ProviderProfile {
            id: id.into(),
            kind: "video".into(),
            label: label.into(),
            vendor: id.split('-').next().unwrap_or(id).into(),
            enabled: id == "mock-video",
            builtin: true,
            api_base: base.into(),
            default_model: model.into(),
            models: vec![model.into()],
            api_key_set: false,
            env_var: env.into(),
            options: json!({"poll_interval_sec": 5, "timeout_sec": 900}),
            capabilities: crate::models::ProviderCapabilities {
                aspect_ratios: caps.0.into_iter().map(String::from).collect(),
                durations_sec: caps.1,
                text_to_video: caps.2,
                image_to_video: caps.3,
                reference_image: true,
                max_reference_images: 1,
                ..Default::default()
            },
        });
    }

    for (id, label, base, model, env) in [
        (
            "mock-music",
            "Mock 配乐 (离线)",
            "local://mock",
            "saladcart-mock-music",
            "",
        ),
        (
            "jamendo-music",
            "Jamendo 音乐库",
            "https://api.jamendo.com/v3.0",
            "search",
            "JAMENDO_CLIENT_ID",
        ),
        (
            "stability-audio",
            "Stability Audio",
            "https://api.stability.ai",
            "stable-audio-3",
            "STABILITY_API_KEY",
        ),
        (
            "elevenlabs-music",
            "ElevenLabs Music",
            "https://api.elevenlabs.io",
            "music",
            "ELEVENLABS_API_KEY",
        ),
        (
            "volc-music",
            "火山引擎 纯音乐",
            "https://open.volcengineapi.com",
            "v5.0",
            "VOLC_ACCESSKEY_SECRET",
        ),
    ] {
        providers.push(crate::models::ProviderProfile {
            id: id.into(),
            kind: "music".into(),
            label: label.into(),
            vendor: id.split('-').next().unwrap_or(id).into(),
            enabled: true,
            builtin: true,
            api_base: base.into(),
            default_model: model.into(),
            models: vec![model.into()],
            api_key_set: false,
            env_var: env.into(),
            options: json!({"instrumental": true}),
            capabilities: crate::models::ProviderCapabilities::default(),
        });
    }

    Settings {
        providers,
        defaults: ProviderDefaults {
            llm: "mock-llm".into(),
            image: "mock-image".into(),
            video: "mock-video".into(),
        },
        concurrency: 2,
        ffmpeg_path: None,
        storage_dir: None,
        ui_language: "zh-CN".into(),
        keychain_available: None,
        trend_source_url: None,
        trend_token_set: false,
    }
}

pub fn load(paths: &AppPaths) -> Settings {
    let path = &paths.settings_file;
    let mut settings = if path.exists() {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
            .unwrap_or_else(default_settings)
    } else {
        default_settings()
    };
    // Built-in providers are additive: a new release can introduce providers
    // without wiping the user's saved configuration.
    for builtin in default_settings().providers {
        if !settings.providers.iter().any(|p| p.id == builtin.id) {
            settings.providers.push(builtin);
        }
    }
    let keychain_available = secrets::probe_keychain();
    settings.keychain_available = Some(keychain_available);
    settings.trend_token_set =
        keychain_available && secrets::has_secret("douyin-trend").unwrap_or(false);
    for provider in &mut settings.providers {
        provider.api_key_set =
            provider.enabled && secret_available(provider, keychain_available);
    }
    settings
}

pub fn save(paths: &AppPaths, settings: &Settings) -> AppResult<()> {
    let mut clean = settings.clone();
    for provider in &mut clean.providers {
        provider.api_key_set = false;
    }
    clean.trend_token_set = false;
    let body = serde_json::to_string_pretty(&clean)?;
    std::fs::write(&paths.settings_file, body)?;
    Ok(())
}

fn secret_available(provider: &crate::models::ProviderProfile, keychain_available: bool) -> bool {
    let local = provider.id.starts_with("mock-")
        || provider.vendor == "ollama"
        || provider.id == "comfyui-image";
    if local {
        return false;
    }
    (keychain_available && secrets::has_secret(&provider.id).unwrap_or(false))
        || (!provider.env_var.is_empty() && std::env::var(&provider.env_var).is_ok())
}

pub fn ensure_ready(provider: &crate::models::ProviderProfile) -> AppResult<()> {
    if provider.id.starts_with("mock-")
        || provider.vendor == "ollama"
        || provider.id == "comfyui-image"
    {
        return Ok(());
    }
    resolve_secret(provider).map(|_| ())
}

pub fn resolve_secret(provider: &crate::models::ProviderProfile) -> AppResult<String> {
    if let Ok(value) = secrets::get_secret(&provider.id) {
        if !value.is_empty() {
            return Ok(value);
        }
    }
    if !provider.env_var.is_empty() {
        if let Ok(value) = std::env::var(&provider.env_var) {
            if !value.is_empty() {
                return Ok(value);
            }
        }
    }
    Err(AppError::MissingCredential(format!(
        "{} needs an API key",
        provider.label
    )))
}

pub fn find_provider<'a>(
    settings: &'a Settings,
    id: &str,
) -> AppResult<&'a crate::models::ProviderProfile> {
    settings
        .providers
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("provider {id}")))
}

pub fn update_persisted(
    paths: &AppPaths,
    update: impl FnOnce(&mut Settings),
) -> AppResult<Settings> {
    let mut settings = load(paths);
    update(&mut settings);
    save(paths, &settings)?;
    Ok(load(paths))
}

pub fn is_mock(id: &str) -> bool {
    id.starts_with("mock-")
}

pub fn ensure_ffmpeg_configured(path: &Path) -> AppResult<()> {
    if !path.exists() {
        return Err(AppError::FfmpegUnavailable);
    }
    Ok(())
}
