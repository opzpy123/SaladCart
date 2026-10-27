use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::paths::AppPaths;
use crate::providers::image::{ImageRef, ImageRequest};
use crate::providers::music::{self as music_provider, MusicRequest as ProviderMusicRequest};
use crate::providers::video::VideoRequest;
use crate::providers::{self, image, video};
use crate::services::{context, media, script};
use crate::settings;
use crate::state::AppState;
use crate::store::assets::NewAsset;
use crate::store::jobs::FinishJob;
use crate::store::{dimensions_for, ProjectStore};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub struct JobSpec {
    pub kind: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub provider_id: String,
    pub provider_label: String,
    pub model: String,
    pub cost_estimate: f64,
    pub request_summary: String,
    pub request: Value,
}

pub fn enqueue_script(
    app: &AppHandle,
    state: &AppState,
    project_id: &str,
    mut request: GenerateScriptRequest,
) -> AppResult<GenerationJob> {
    let settings = state.settings();
    let provider_id = request
        .provider_id
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| settings.defaults.llm.clone());
    let profile = settings::find_provider(&settings, &provider_id)?.clone();
    if !profile.enabled {
        return Err(AppError::Invalid(format!(
            "{} is disabled. Enable it in Settings first.",
            profile.label
        )));
    }
    let model = request
        .model
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| profile.default_model.clone());
    request.provider_id = Some(profile.id.clone());
    request.model = Some(model.clone());
    let store = ProjectStore::open(&state.paths, project_id)?;
    let summary = format!("剧本 · {} 集 · {}", request.episodes, profile.label);
    let job = new_job(
        project_id,
        JobSpec {
            kind: "script".into(),
            target_type: "project".into(),
            target_id: Some(project_id.to_string()),
            provider_id: profile.id.clone(),
            provider_label: profile.label.clone(),
            model,
            cost_estimate: 0.0,
            request_summary: summary,
            request: serde_json::to_value(&request)?,
        },
    );
    store.insert_job(&job)?;
    drop(store);
    spawn_job(app.clone(), state, &job);
    Ok(job)
}

pub fn enqueue_generation(
    app: &AppHandle,
    state: &AppState,
    project_id: &str,
    request: GenerationRequest,
) -> AppResult<GenerationJob> {
    let settings = state.settings();
    let store = ProjectStore::open(&state.paths, project_id)?;
    let compiled = context::compile(&store, &settings, &request)?;
    let summary = format!("{} · {}", compiled.title, compiled.provider_label);
    let mut request = request;
    request.compiled = Some(compiled.clone());
    let job = new_job(
        project_id,
        JobSpec {
            kind: request.kind.clone(),
            target_type: request.target_type.clone(),
            target_id: request.target_id.clone(),
            provider_id: compiled.provider_id.clone(),
            provider_label: compiled.provider_label.clone(),
            model: compiled.model.clone(),
            cost_estimate: compiled.estimated_cost,
            request_summary: summary,
            request: serde_json::to_value(&request)?,
        },
    );
    store.insert_job(&job)?;
    drop(store);
    spawn_job(app.clone(), state, &job);
    Ok(job)
}

pub fn enqueue_export(
    app: &AppHandle,
    state: &AppState,
    project_id: &str,
    options: ExportOptions,
) -> AppResult<GenerationJob> {
    let store = ProjectStore::open(&state.paths, project_id)?;
    let timeline = store.timeline()?;
    let summary = format!(
        "导出 · {}×{} · {}fps",
        options.width, options.height, options.fps
    );
    let job = new_job(
        project_id,
        JobSpec {
            kind: "export".into(),
            target_type: "timeline".into(),
            target_id: Some(timeline.id),
            provider_id: "ffmpeg".into(),
            provider_label: "本地 FFmpeg".into(),
            model: "ffmpeg".into(),
            cost_estimate: 0.0,
            request_summary: summary,
            request: serde_json::to_value(&options)?,
        },
    );
    store.insert_job(&job)?;
    drop(store);
    spawn_job(app.clone(), state, &job);
    Ok(job)
}

pub fn enqueue_music(
    app: &AppHandle,
    state: &AppState,
    project_id: &str,
    request: MusicGenerationRequest,
) -> AppResult<GenerationJob> {
    let settings = state.settings();
    let provider_id = request
        .provider_id
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "mock-music".into());
    let profile = settings::find_provider(&settings, &provider_id)?.clone();
    if !profile.enabled {
        return Err(AppError::Invalid(format!("{} is disabled", profile.label)));
    }
    if profile.id == "jamendo-music" {
        return Err(AppError::Invalid(
            "Jamendo is for searching licensed music, not generating it. Choose Mock, Stability or ElevenLabs."
                .into(),
        ));
    }
    let model = request
        .model
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| profile.default_model.clone());
    let duration = request.duration_ms.clamp(8_000, 120_000);
    let cost = providers::estimate_cost(&profile.id, "music", duration, 1);
    let store = ProjectStore::open(&state.paths, project_id)?;
    let job = new_job(
        project_id,
        JobSpec {
            kind: "music".into(),
            target_type: "project".into(),
            target_id: Some(project_id.to_string()),
            provider_id: profile.id.clone(),
            provider_label: profile.label.clone(),
            model,
            cost_estimate: cost,
            request_summary: format!("AI 配乐 · {}", truncate(&request.prompt, 30)),
            request: serde_json::to_value(&request)?,
        },
    );
    store.insert_job(&job)?;
    drop(store);
    spawn_job(app.clone(), state, &job);
    Ok(job)
}

fn truncate(value: &str, limit: usize) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "原创配乐".into()
    } else {
        trimmed.chars().take(limit).collect()
    }
}

pub fn retry_job(
    app: &AppHandle,
    state: &AppState,
    project_id: &str,
    job_id: &str,
) -> AppResult<GenerationJob> {
    let store = ProjectStore::open(&state.paths, project_id)?;
    let job = store.get_job(job_id)?;
    drop(store);
    match job.kind.as_str() {
        "script" => {
            let request: GenerateScriptRequest = serde_json::from_value(job.request)
                .map_err(|e| AppError::Invalid(format!("bad script request: {e}")))?;
            enqueue_script(app, state, project_id, request)
        }
        "image" | "video" => {
            let request: GenerationRequest = serde_json::from_value(job.request)
                .map_err(|e| AppError::Invalid(format!("bad generation request: {e}")))?;
            enqueue_generation(app, state, project_id, request)
        }
        "export" => {
            let options: ExportOptions = serde_json::from_value(job.request)
                .map_err(|e| AppError::Invalid(format!("bad export request: {e}")))?;
            enqueue_export(app, state, project_id, options)
        }
        "music" => {
            let request: MusicGenerationRequest = serde_json::from_value(job.request)
                .map_err(|e| AppError::Invalid(format!("bad music request: {e}")))?;
            enqueue_music(app, state, project_id, request)
        }
        other => Err(AppError::Invalid(format!(
            "retry is not supported for {other} jobs"
        ))),
    }
}

fn new_job(project_id: &str, spec: JobSpec) -> GenerationJob {
    GenerationJob {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: project_id.to_string(),
        kind: spec.kind,
        target_type: spec.target_type,
        target_id: spec.target_id,
        provider_id: spec.provider_id,
        provider_label: spec.provider_label,
        model: spec.model,
        status: "queued".into(),
        progress: 0,
        stage: "queued".into(),
        error: None,
        cost_estimate: spec.cost_estimate,
        cost_actual: 0.0,
        request_summary: spec.request_summary,
        request: spec.request,
        result: None,
        result_asset_id: None,
        created_at: crate::store::now_iso(),
        started_at: None,
        finished_at: None,
    }
}

fn spawn_job(app: AppHandle, state: &AppState, job: &GenerationJob) {
    let runtime = state.jobs.clone();
    let paths = state.paths.clone();
    let job_id = job.id.clone();
    let project_id = job.project_id.clone();
    let kind = job.kind.clone();
    let task_job_id = job_id.clone();
    let task_runtime = runtime.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let permit = task_runtime.semaphore.clone().acquire_owned().await;
        match permit {
            Ok(_permit) => {
                if let Err(error) = execute(&app, &paths, &task_job_id, &kind).await {
                    mark_failed(&app, &paths, &project_id, &task_job_id, &error);
                }
            }
            Err(_) => {
                mark_failed(
                    &app,
                    &paths,
                    &project_id,
                    &task_job_id,
                    &AppError::Message("job queue was closed".into()),
                );
            }
        }
        task_runtime.finish(&task_job_id);
    });
    runtime.track(job_id, handle);
}

async fn execute(app: &AppHandle, paths: &AppPaths, job_id: &str, kind: &str) -> AppResult<()> {
    match kind {
        "script" => run_script(app, paths, job_id).await,
        "image" | "video" => run_generation(app, paths, job_id).await,
        "music" => run_music(app, paths, job_id).await,
        "export" => run_export(app, paths, job_id).await,
        other => Err(AppError::Invalid(format!("unknown job kind: {other}"))),
    }
}

fn emit_job(app: &AppHandle, job: &GenerationJob) {
    let _ = app.emit("job:update", job);
}

fn progress(
    app: &AppHandle,
    paths: &AppPaths,
    project_id: &str,
    job_id: &str,
    pct: i64,
    stage: &str,
) -> AppResult<()> {
    let store = ProjectStore::open(paths, project_id)?;
    let job = store.update_job_progress(job_id, pct, stage)?;
    emit_job(app, &job);
    Ok(())
}

fn mark_failed(
    app: &AppHandle,
    paths: &AppPaths,
    project_id: &str,
    job_id: &str,
    error: &AppError,
) {
    if let Ok(store) = ProjectStore::open(paths, project_id) {
        if let Ok(job) = store.finish_job(
            job_id,
            FinishJob {
                status: "failed".into(),
                error: Some(error.to_string()),
                result: None,
                result_asset_id: None,
                cost_actual: 0.0,
            },
        ) {
            emit_job(app, &job);
        }
    }
}

fn secret_for(profile: &ProviderProfile) -> AppResult<Option<String>> {
    if profile.id.starts_with("mock-")
        || profile.id == "comfyui-image"
        || profile.vendor == "ollama"
    {
        Ok(None)
    } else {
        settings::resolve_secret(profile).map(Some)
    }
}

async fn run_script(app: &AppHandle, paths: &AppPaths, job_id: &str) -> AppResult<()> {
    let (job, request, profile, project) = {
        let store = ProjectStore::open_by_job(paths, job_id)?;
        let job = store.start_job(job_id)?;
        emit_job(app, &job);
        let request: GenerateScriptRequest = serde_json::from_value(job.request.clone())
            .map_err(|e| AppError::Invalid(format!("bad script request: {e}")))?;
        let settings = settings::load(paths);
        let profile = settings::find_provider(&settings, &job.provider_id)?.clone();
        let project = store.project()?;
        (job, request, profile, project)
    };
    let secret = secret_for(&profile)?;
    progress(
        app,
        paths,
        &job.project_id,
        job_id,
        12,
        "building story bible",
    )?;
    let generated = script::generate(&profile, secret.as_deref(), &request, &project).await?;
    progress(
        app,
        paths,
        &job.project_id,
        job_id,
        72,
        "saving episodes and shots",
    )?;
    let mut store = ProjectStore::open_by_job(paths, job_id)?;
    let stats = store.apply_generated_script(generated, &request)?;
    let job = store.finish_job(
        job_id,
        FinishJob {
            status: "succeeded".into(),
            error: None,
            result: Some(json!({
                "charactersCreated": stats.characters_created,
                "episodes": stats.episodes,
                "scenes": stats.scenes,
                "shots": stats.shots,
                "provider": profile.label,
                "model": job.model,
            })),
            result_asset_id: None,
            cost_actual: 0.0,
        },
    )?;
    emit_job(app, &job);
    Ok(())
}

enum StartFrame {
    Image(PathBuf),
    Video(PathBuf, Option<i64>),
}

async fn run_generation(app: &AppHandle, paths: &AppPaths, job_id: &str) -> AppResult<()> {
    let (
        job,
        request,
        compiled,
        profile,
        settings,
        start_frame,
        refs,
        project_dir,
        aspect,
        project_width,
        project_height,
    ) = {
        let store = ProjectStore::open_by_job(paths, job_id)?;
        let job = store.start_job(job_id)?;
        emit_job(app, &job);
        let request: GenerationRequest = serde_json::from_value(job.request.clone())
            .map_err(|e| AppError::Invalid(format!("bad generation request: {e}")))?;
        let settings = settings::load(paths);
        let compiled = match request.compiled.clone() {
            Some(compiled) => compiled,
            None => context::compile(&store, &settings, &request)?,
        };
        let profile = settings::find_provider(&settings, &compiled.provider_id)?.clone();
        let refs = compiled
            .character_refs
            .iter()
            .filter_map(|reference| {
                let path = PathBuf::from(&reference.abs_path);
                if path.exists() {
                    Some(ImageRef {
                        mime: providers::image_mime(&path),
                        path,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let start_frame = resolve_start_frame(&store, &request, &compiled)?;
        let aspect = compiled.aspect_ratio.clone();
        let (w, h) = dimensions_for(&aspect);
        (
            job,
            request,
            compiled,
            profile,
            settings,
            start_frame,
            refs,
            store.dir.clone(),
            aspect,
            w,
            h,
        )
    };
    let secret = secret_for(&profile)?;
    if request.kind == "video" {
        run_video_job(
            app,
            paths,
            &job,
            &request,
            &compiled,
            &profile,
            secret.as_deref(),
            &settings,
            start_frame,
            refs,
            &project_dir,
            &aspect,
            project_width,
            project_height,
        )
        .await
    } else {
        run_image_job(
            app,
            paths,
            &job,
            &request,
            &compiled,
            &profile,
            secret.as_deref(),
            &project_dir,
            &aspect,
            project_width,
            project_height,
            refs,
        )
        .await
    }
}

fn resolve_start_frame(
    store: &ProjectStore,
    request: &GenerationRequest,
    compiled: &ContextPreview,
) -> AppResult<Option<StartFrame>> {
    if request.target_type != "shot" {
        return Ok(None);
    }
    let Some(shot_id) = &request.target_id else {
        return Ok(None);
    };
    let shot = store.get_shot(shot_id)?;
    if let Some(asset_id) = &shot.start_frame_asset_id {
        if let Ok(asset) = store.get_asset(asset_id) {
            return Ok(Some(StartFrame::Image(PathBuf::from(asset.abs_path))));
        }
    }
    // A generated keyframe is the canonical start frame for its shot; without
    // this the video job silently falls back to the previous shot or a portrait.
    if let Some(asset_id) = &shot.image_asset_id {
        if let Ok(asset) = store.get_asset(asset_id) {
            if asset.kind == "image" {
                return Ok(Some(StartFrame::Image(PathBuf::from(asset.abs_path))));
            }
        }
    }
    if request.options.use_continuity {
        if let Some(previous_id) = &compiled.continuity_from_shot_id {
            if let Ok(previous) = store.get_shot(previous_id) {
                if let Some(asset_id) = &previous.video_asset_id {
                    if let Ok(asset) = store.get_asset(asset_id) {
                        return Ok(Some(StartFrame::Video(
                            PathBuf::from(asset.abs_path),
                            asset.duration_ms,
                        )));
                    }
                }
                if let Some(asset_id) = &previous.image_asset_id {
                    if let Ok(asset) = store.get_asset(asset_id) {
                        return Ok(Some(StartFrame::Image(PathBuf::from(asset.abs_path))));
                    }
                }
            }
        }
    }
    if let Some(reference) = compiled.character_refs.first() {
        let path = PathBuf::from(&reference.abs_path);
        if path.exists() {
            return Ok(Some(StartFrame::Image(path)));
        }
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
async fn run_image_job(
    app: &AppHandle,
    paths: &AppPaths,
    job: &GenerationJob,
    request: &GenerationRequest,
    compiled: &ContextPreview,
    profile: &ProviderProfile,
    secret: Option<&str>,
    project_dir: &Path,
    aspect: &str,
    width: i64,
    height: i64,
    refs: Vec<ImageRef>,
) -> AppResult<()> {
    progress(
        app,
        paths,
        &job.project_id,
        &job.id,
        12,
        "compiling generation context",
    )?;
    let image_request = ImageRequest {
        model: compiled.model.clone(),
        prompt: compiled.user_prompt.clone(),
        negative_prompt: compiled.negative_prompt.clone(),
        aspect_ratio: aspect.to_string(),
        width,
        height,
        count: 1,
        seed: request.options.seed,
        references: refs,
        title: compiled.title.clone(),
    };
    progress(app, paths, &job.project_id, &job.id, 24, "generating image")?;
    let result = image::generate(profile, secret, &image_request).await?;
    let image_id = uuid::Uuid::new_v4().to_string();
    let rel = format!("assets/images/{image_id}.{}", result.extension);
    let abs = project_dir.join(&rel);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, &result.bytes)?;
    let dims = ::image::image_dimensions(&abs).unwrap_or((width as u32, height as u32));
    let thumb_rel = format!("thumbs/{image_id}.jpg");
    let thumb_abs = project_dir.join(&thumb_rel);
    let thumb = media::image_thumbnail(&abs, &thumb_abs, 640)
        .ok()
        .map(|_| thumb_rel.clone());

    progress(
        app,
        paths,
        &job.project_id,
        &job.id,
        82,
        "saving asset and linking to project",
    )?;
    let mut store = ProjectStore::open_by_job(paths, &job.id)?;
    let is_sheet = request
        .options
        .variant
        .as_deref()
        .map(|v| v == "sheet")
        .unwrap_or(false);
    let name = if request.target_type == "character" {
        if is_sheet {
            format!("{} · 角色三视图", compiled.title)
        } else {
            format!("{} · 角色定妆照", compiled.title)
        }
    } else {
        format!("{} · 画面", compiled.title)
    };
    let asset = store.insert_asset(NewAsset {
        kind: "image".into(),
        name,
        rel_path: rel,
        thumb_rel_path: thumb,
        mime: result.mime.clone(),
        width: Some(dims.0 as i64),
        height: Some(dims.1 as i64),
        duration_ms: None,
        source: "generated".into(),
        provider_id: Some(profile.id.clone()),
        model: Some(compiled.model.clone()),
        prompt: Some(compiled.user_prompt.clone()),
        negative_prompt: Some(compiled.negative_prompt.clone()),
        seed: request.options.seed,
        version: 1,
        parent_asset_id: None,
        license: None,
        license_url: None,
        attribution: None,
        source_url: None,
        status: "ready".into(),
    })?;
    if request.target_type == "character" {
        let character_id = request
            .target_id
            .clone()
            .ok_or_else(|| AppError::Invalid("missing character target".into()))?;
        store.attach_character_reference(&character_id, &asset.id, is_sheet)?;
    } else if request.target_type == "shot" {
        let shot_id = request
            .target_id
            .clone()
            .ok_or_else(|| AppError::Invalid("missing shot target".into()))?;
        store.attach_shot_asset(
            &shot_id,
            &asset.id,
            "image",
            &profile.id,
            &compiled.model,
            request.options.seed,
        )?;
        store.set_shot_status(&shot_id, "needs_review")?;
    }
    let cost = compiled.estimated_cost;
    let finished = store.finish_job(
        &job.id,
        FinishJob {
            status: "succeeded".into(),
            error: None,
            result: Some(json!({
                "assetId": asset.id,
                "absPath": asset.abs_path,
                "provider": profile.label,
                "model": compiled.model,
                "costEstimate": cost,
                "raw": result.raw,
                "warnings": compiled.warnings,
            })),
            result_asset_id: Some(asset.id.clone()),
            cost_actual: cost,
        },
    )?;
    emit_job(app, &finished);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_video_job(
    app: &AppHandle,
    paths: &AppPaths,
    job: &GenerationJob,
    request: &GenerationRequest,
    compiled: &ContextPreview,
    profile: &ProviderProfile,
    secret: Option<&str>,
    settings: &Settings,
    start_frame: Option<StartFrame>,
    refs: Vec<ImageRef>,
    project_dir: &Path,
    aspect: &str,
    width: i64,
    height: i64,
) -> AppResult<()> {
    let shot_id = request
        .target_id
        .clone()
        .ok_or_else(|| AppError::Invalid("video generation needs a shot target".into()))?;
    let duration_ms = compiled.duration_ms.max(2000);
    let mut continuity_temp: Option<PathBuf> = None;
    let ffmpeg = media::ensure_ffmpeg(settings)?;
    let frame_path: Option<PathBuf> = match &start_frame {
        Some(StartFrame::Image(path)) => Some(path.clone()),
        Some(StartFrame::Video(path, asset_duration)) => {
            let temp = project_dir.join(format!("thumbs/continuity-{}.png", job.id));
            let at = asset_duration.unwrap_or(5000).saturating_sub(300);
            if ffmpeg.extract_frame(path, at, &temp).await.is_ok() {
                continuity_temp = Some(temp.clone());
                Some(temp)
            } else {
                None
            }
        }
        None => None,
    };

    let raw_result: Value;
    let provider_cost = if profile.id.starts_with("mock-") {
        0.0
    } else {
        compiled.estimated_cost
    };
    let (video_path, video_id) = if profile.id.starts_with("mock-") {
        progress(
            app,
            paths,
            &job.project_id,
            &job.id,
            18,
            "rendering offline preview",
        )?;
        let placeholder = image::generate(
            profile,
            None,
            &ImageRequest {
                model: compiled.model.clone(),
                prompt: compiled.user_prompt.clone(),
                negative_prompt: compiled.negative_prompt.clone(),
                aspect_ratio: aspect.to_string(),
                width,
                height,
                count: 1,
                seed: request.options.seed,
                references: Vec::new(),
                title: compiled.title.clone(),
            },
        )
        .await?;
        let id = uuid::Uuid::new_v4().to_string();
        let rel = format!("assets/videos/{id}.mp4");
        let abs = project_dir.join(&rel);
        let (w, h) = mock_video_dims(aspect);
        ffmpeg
            .write_placeholder_video(&placeholder.bytes, &abs, w, h, duration_ms)
            .await?;
        raw_result = json!({"mock": true, "bpmHint": null});
        (abs, id)
    } else {
        progress(
            app,
            paths,
            &job.project_id,
            &job.id,
            14,
            "submitting to video provider",
        )?;
        let video_request = VideoRequest {
            model: compiled.model.clone(),
            prompt: compiled.user_prompt.clone(),
            negative_prompt: compiled.negative_prompt.clone(),
            aspect_ratio: aspect.to_string(),
            duration_ms,
            seed: request.options.seed,
            resolution: request.options.resolution.clone(),
            start_frame: frame_path.clone(),
            references: refs
                .iter()
                .map(|reference| reference.path.clone())
                .filter(|path| path.exists())
                .collect(),
            title: compiled.title.clone(),
        };
        let mut task = video::submit(profile, secret, &video_request).await?;
        let deadline = Instant::now() + video::timeout(profile);
        let mut pct = 20;
        loop {
            if task.status == "succeeded" {
                break;
            }
            if task.status == "failed" {
                return Err(AppError::Provider(format!(
                    "{} rejected the job: {}",
                    profile.label,
                    task.error.clone().unwrap_or_else(|| "unknown error".into())
                )));
            }
            if Instant::now() > deadline {
                return Err(AppError::Provider(format!(
                    "{} timed out before rendering the shot",
                    profile.label
                )));
            }
            tokio::time::sleep(Duration::from_secs(
                profile
                    .options
                    .get("poll_interval_sec")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5)
                    .clamp(2, 30),
            ))
            .await;
            task = video::poll(profile, secret, &task).await?;
            pct = (pct + 4).min(86);
            progress(
                app,
                paths,
                &job.project_id,
                &job.id,
                pct,
                "provider is rendering the shot",
            )?;
        }
        let url = task.output_url.clone().ok_or_else(|| {
            AppError::Provider(format!("{} finished without a video URL", profile.label))
        })?;
        progress(
            app,
            paths,
            &job.project_id,
            &job.id,
            88,
            "downloading video",
        )?;
        let id = uuid::Uuid::new_v4().to_string();
        let temp = project_dir.join(format!("assets/videos/{id}.download"));
        providers::download_with_auth(profile, secret, &url, &temp).await?;
        let extension = media::detect_video_extension(&temp);
        let rel = format!("assets/videos/{id}.{extension}");
        let abs = project_dir.join(&rel);
        std::fs::rename(&temp, &abs)?;
        raw_result = json!({
            "providerTask": task.raw,
            "providerTaskId": task.id,
            "warnings": compiled.warnings
        });
        (abs, id)
    };

    if let Some(temp) = continuity_temp {
        let _ = std::fs::remove_file(temp);
    }
    let probe = ffmpeg.probe(&video_path);
    let duration = probe.duration_ms.unwrap_or(duration_ms);
    let final_width = probe.width.unwrap_or(width);
    let final_height = probe.height.unwrap_or(height);
    let thumb_rel = format!("thumbs/{video_id}.jpg");
    let thumb_abs = project_dir.join(&thumb_rel);
    let frame_temp = project_dir.join(format!("thumbs/{video_id}-frame.png"));
    let thumb = if ffmpeg
        .extract_frame(&video_path, 800.min(duration / 3), &frame_temp)
        .await
        .is_ok()
    {
        let ok = media::image_thumbnail(&frame_temp, &thumb_abs, 640).is_ok();
        let _ = std::fs::remove_file(&frame_temp);
        if ok {
            Some(thumb_rel.clone())
        } else {
            None
        }
    } else {
        None
    };
    let extension = video_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4");
    let mime = match extension {
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        _ => "video/mp4",
    };
    let rel = video_path
        .strip_prefix(project_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    progress(
        app,
        paths,
        &job.project_id,
        &job.id,
        93,
        "linking take to shot",
    )?;
    let mut store = ProjectStore::open_by_job(paths, &job.id)?;
    let previous_video = store.get_shot(&shot_id)?.video_asset_id;
    let parent_version = store.get_shot(&shot_id)?.version;
    let asset = store.insert_asset(NewAsset {
        kind: "video".into(),
        name: format!("{} · 镜头成片", compiled.title),
        rel_path: rel,
        thumb_rel_path: thumb,
        mime: mime.into(),
        width: Some(final_width),
        height: Some(final_height),
        duration_ms: Some(duration),
        source: "generated".into(),
        provider_id: Some(profile.id.clone()),
        model: Some(compiled.model.clone()),
        prompt: Some(compiled.user_prompt.clone()),
        negative_prompt: Some(compiled.negative_prompt.clone()),
        seed: request.options.seed,
        version: parent_version + 1,
        parent_asset_id: previous_video,
        license: None,
        license_url: None,
        attribution: None,
        source_url: None,
        status: "ready".into(),
    })?;
    store.attach_shot_asset(
        &shot_id,
        &asset.id,
        "video",
        &profile.id,
        &compiled.model,
        request.options.seed,
    )?;
    store.set_shot_status(&shot_id, "needs_review")?;
    let finished = store.finish_job(
        &job.id,
        FinishJob {
            status: "succeeded".into(),
            error: None,
            result: Some(json!({
                "assetId": asset.id,
                "absPath": asset.abs_path,
                "durationMs": duration,
                "width": final_width,
                "height": final_height,
                "provider": profile.label,
                "model": compiled.model,
                "costEstimate": provider_cost,
                "startFrame": frame_path.map(|p| p.to_string_lossy().to_string()),
                "details": raw_result,
            })),
            result_asset_id: Some(asset.id.clone()),
            cost_actual: provider_cost,
        },
    )?;
    emit_job(app, &finished);
    Ok(())
}

fn mock_video_dims(aspect: &str) -> (i64, i64) {
    match aspect {
        "16:9" => (1280, 720),
        "1:1" => (960, 960),
        "4:5" => (900, 1125),
        _ => (720, 1280),
    }
}

async fn run_music(app: &AppHandle, paths: &AppPaths, job_id: &str) -> AppResult<()> {
    let (job, request, profile, settings) = {
        let store = ProjectStore::open_by_job(paths, job_id)?;
        let job = store.start_job(job_id)?;
        emit_job(app, &job);
        let request: MusicGenerationRequest = serde_json::from_value(job.request.clone())
            .map_err(|e| AppError::Invalid(format!("bad music request: {e}")))?;
        let settings = settings::load(paths);
        let profile = settings::find_provider(&settings, &job.provider_id)?.clone();
        (job, request, profile, settings)
    };
    let secret = secret_for(&profile)?;
    progress(app, paths, &job.project_id, &job.id, 12, "composing motif")?;
    let result = music_provider::generate(
        &profile,
        secret.as_deref(),
        &ProviderMusicRequest {
            model: job.model.clone(),
            prompt: request.prompt.clone(),
            duration_ms: request.duration_ms,
            instrumental: request.instrumental,
            seed: request.seed,
        },
    )
    .await?;
    progress(app, paths, &job.project_id, &job.id, 76, "saving track")?;
    let project_dir = ProjectStore::open_by_job(paths, job_id)?.dir.clone();
    let id = uuid::Uuid::new_v4().to_string();
    let rel = format!("assets/audio/{id}.{}", result.extension);
    let abs = project_dir.join(&rel);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, &result.bytes)?;
    let duration = media::Ffmpeg::discover(&settings)
        .and_then(|ffmpeg| ffmpeg.probe(&abs).duration_ms)
        .unwrap_or(request.duration_ms);
    let bpm = result.raw.get("bpm").and_then(|v| v.as_f64());
    let mut store = ProjectStore::open_by_job(paths, job_id)?;
    let asset = store.insert_asset(NewAsset {
        kind: "audio".into(),
        name: format!("{} · AI 配乐", truncate(&request.prompt, 24)),
        rel_path: rel,
        thumb_rel_path: None,
        mime: result.mime,
        width: None,
        height: None,
        duration_ms: Some(duration),
        source: "generated".into(),
        provider_id: Some(profile.id.clone()),
        model: Some(job.model.clone()),
        prompt: Some(request.prompt.clone()),
        negative_prompt: None,
        seed: request.seed,
        version: 1,
        parent_asset_id: None,
        license: Some(format!(
            "{} generated audio — check provider terms",
            profile.label
        )),
        license_url: None,
        attribution: None,
        source_url: None,
        status: "ready".into(),
    })?;
    if request.attach_to_timeline {
        let beats = bpm
            .map(|value| {
                let interval = (60_000.0 / value) as i64;
                let mut grid = Vec::new();
                let mut time = 0;
                while time <= duration {
                    grid.push(time);
                    time += interval.max(1);
                }
                grid
            })
            .unwrap_or_default();
        let current_volume = store.timeline()?.music_volume;
        store.update_timeline(TimelinePatch {
            music_asset_id: Some(asset.id.clone()),
            music_volume: Some(current_volume),
            bpm,
            beat_grid_ms: Some(beats),
            beat_offset_ms: Some(0),
            ..Default::default()
        })?;
    }
    let finished = store.finish_job(
        &job.id,
        FinishJob {
            status: "succeeded".into(),
            error: None,
            result: Some(json!({
                "assetId": asset.id,
                "bpm": bpm,
                "provider": profile.label,
                "raw": result.raw,
            })),
            result_asset_id: Some(asset.id.clone()),
            cost_actual: result.cost,
        },
    )?;
    emit_job(app, &finished);
    Ok(())
}

async fn run_export(app: &AppHandle, paths: &AppPaths, job_id: &str) -> AppResult<()> {
    let (job, options, settings, ffmpeg, plan) = {
        let store = ProjectStore::open_by_job(paths, job_id)?;
        let job = store.start_job(job_id)?;
        emit_job(app, &job);
        let options: ExportOptions = serde_json::from_value(job.request.clone())
            .map_err(|e| AppError::Invalid(format!("bad export request: {e}")))?;
        let settings = settings::load(paths);
        let ffmpeg = media::ensure_ffmpeg(&settings)?;
        let plan = media::build_render_plan(&store, &settings, &options)?;
        (job, options, settings, ffmpeg, plan)
    };

    let last_update = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10)));
    let progress_app = app.clone();
    let progress_paths = paths.clone();
    let progress_project = job.project_id.clone();
    let progress_job = job.id.clone();
    let callback = move |pct: i64| {
        let now = Instant::now();
        if let Ok(mut last) = last_update.lock() {
            if now.duration_since(*last) < Duration::from_millis(700) && pct < 100 {
                return;
            }
            *last = now;
        }
        if let Ok(store) = ProjectStore::open(&progress_paths, &progress_project) {
            if let Ok(job) = store.update_job_progress(&progress_job, pct, "rendering with FFmpeg")
            {
                let _ = progress_app.emit("job:update", job);
            }
        }
    };
    let render = ffmpeg.render(&plan, callback).await?;

    let mut store = ProjectStore::open_by_job(paths, &job.id)?;
    let rel = Path::new(&render.path)
        .strip_prefix(&store.dir)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let size = std::fs::metadata(&render.path)
        .map(|m| m.len() as i64)
        .unwrap_or(render.size_bytes);
    let file_name = Path::new(&render.path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("SaladCart export.mp4")
        .to_string();
    let asset = store.insert_asset(NewAsset {
        kind: "video".into(),
        name: file_name,
        rel_path: rel,
        thumb_rel_path: None,
        mime: "video/mp4".into(),
        width: Some(render.width),
        height: Some(render.height),
        duration_ms: Some(render.duration_ms),
        source: "export".into(),
        provider_id: Some("ffmpeg".into()),
        model: Some("ffmpeg".into()),
        prompt: None,
        negative_prompt: None,
        seed: None,
        version: 1,
        parent_asset_id: None,
        license: None,
        license_url: None,
        attribution: None,
        source_url: None,
        status: "ready".into(),
    })?;
    let provenance_path = crate::services::provenance::write(&store)
        .ok()
        .map(|path| path.to_string_lossy().to_string());
    let _ = options;
    let _ = settings;
    let finished = store.finish_job(
        &job.id,
        FinishJob {
            status: "succeeded".into(),
            error: None,
            result: Some(json!({
                "path": render.path,
                "assetId": asset.id,
                "durationMs": render.duration_ms,
                "width": render.width,
                "height": render.height,
                "sizeBytes": size,
                "provenancePath": provenance_path,
            })),
            result_asset_id: Some(asset.id.clone()),
            cost_actual: 0.0,
        },
    )?;
    emit_job(app, &finished);
    Ok(())
}

impl ProjectStore {
    pub fn open_by_job(paths: &AppPaths, job_id: &str) -> AppResult<Self> {
        let mut found = None;
        for entry in std::fs::read_dir(&paths.projects_dir)? {
            let Ok(entry) = entry else { continue };
            let Some(project_id) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if let Ok(store) = ProjectStore::open(paths, &project_id) {
                if store.get_job(job_id).is_ok() {
                    found = Some(store);
                    break;
                }
            }
        }
        found.ok_or_else(|| AppError::NotFound(format!("job {job_id}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::assets::NewAsset;
    use serde_json::json;

    #[test]
    fn generated_keyframe_is_the_video_start_frame() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(temp.path().to_path_buf()).unwrap();
        let mut store = ProjectStore::create(
            &paths,
            &CreateProjectInput {
                name: "keyframe".into(),
                premise: "test".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let generated: GeneratedScript = serde_json::from_value(json!({
            "title": "keyframe",
            "logline": "",
            "synopsis": "",
            "characters": [],
            "episodes": [{
                "title": "episode",
                "scenes": [{
                    "title": "scene",
                    "shots": [{"title": "one"}, {"title": "two"}]
                }]
            }]
        }))
        .unwrap();
        store
            .apply_generated_script(generated, &GenerateScriptRequest::default())
            .unwrap();
        let snapshot = store.snapshot().unwrap();
        let shots = &snapshot.episodes[0].scenes[0].shots;
        assert_eq!(shots.len(), 2);
        let target_id = shots[1].id.clone();

        let rel = "assets/images/keyframe.png";
        let abs = store.dir.join(rel);
        std::fs::write(&abs, b"png").unwrap();
        let asset = store
            .insert_asset(NewAsset {
                kind: "image".into(),
                name: "keyframe".into(),
                rel_path: rel.into(),
                thumb_rel_path: None,
                mime: "image/png".into(),
                width: Some(128),
                height: Some(128),
                duration_ms: None,
                source: "mock".into(),
                provider_id: Some("mock-image".into()),
                model: Some("mock".into()),
                prompt: Some("keyframe".into()),
                negative_prompt: None,
                seed: None,
                version: 1,
                parent_asset_id: None,
                license: None,
                license_url: None,
                attribution: None,
                source_url: None,
                status: "ready".into(),
            })
            .unwrap();
        store
            .attach_shot_asset(&target_id, &asset.id, "image", "mock-image", "mock", None)
            .unwrap();

        let request = GenerationRequest {
            kind: "video".into(),
            target_type: "shot".into(),
            target_id: Some(target_id),
            provider_id: Some("mock-video".into()),
            model: None,
            options: GenerationOptions::default(),
            compiled: None,
        };
        let settings = settings::default_settings();
        let compiled = context::compile(&store, &settings, &request).unwrap();
        let start = resolve_start_frame(&store, &request, &compiled)
            .unwrap()
            .expect("a start frame");
        match start {
            StartFrame::Image(path) => assert_eq!(path, abs),
            StartFrame::Video(..) => panic!("keyframe should be used instead of a video frame"),
        }
    }
}
