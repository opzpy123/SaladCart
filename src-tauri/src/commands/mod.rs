use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::services::{context, jobs, media, music as music_service};
use crate::settings;
use crate::state::AppState;
use crate::store::{self, ProjectStore};
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};

fn open(state: &AppState, project_id: &str) -> AppResult<ProjectStore> {
    ProjectStore::open(&state.paths, project_id)
}

// ---------------------------------------------------------------- projects

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> AppResult<Vec<ProjectSummary>> {
    store::list_summaries(&state.paths)
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    input: CreateProjectInput,
) -> AppResult<ProjectSnapshot> {
    let store = ProjectStore::create(&state.paths, &input)?;
    store.snapshot()
}

#[tauri::command]
pub fn open_project(state: State<'_, AppState>, project_id: String) -> AppResult<ProjectSnapshot> {
    open(&state, &project_id)?.snapshot()
}

#[tauri::command]
pub fn update_project(
    state: State<'_, AppState>,
    project_id: String,
    input: UpdateProjectInput,
) -> AppResult<Project> {
    open(&state, &project_id)?.update_project(input)
}

#[tauri::command]
pub fn archive_project(state: State<'_, AppState>, project_id: String) -> AppResult<()> {
    store::archive_project(&state.paths, &project_id)
}

#[tauri::command]
pub async fn generate_shot_voice(
    state: State<'_, AppState>,
    project_id: String,
    shot_id: String,
    voice: Option<String>,
) -> AppResult<Asset> {
    let paths = state.paths.clone();
    crate::services::voice::generate(&paths, &project_id, &shot_id, voice).await
}

#[tauri::command]
pub fn list_project_references(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<ProjectReference>> {
    let summaries = store::list_summaries(&state.paths)?;
    let mut out = Vec::new();
    for summary in summaries {
        if summary.id == project_id {
            continue;
        }
        if let Ok(source) = ProjectStore::open(&state.paths, &summary.id) {
            if let (Ok(project), Ok(characters)) = (source.project(), source.list_characters()) {
                out.push(ProjectReference {
                    project_id: summary.id,
                    project_name: summary.name,
                    characters,
                    style_bible: project.style_bible,
                });
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn import_project_style(
    state: State<'_, AppState>,
    project_id: String,
    source_project_id: String,
) -> AppResult<Project> {
    if project_id == source_project_id {
        return Err(AppError::Invalid(
            "cannot copy a project style onto itself".into(),
        ));
    }
    let source = open(&state, &source_project_id)?;
    let mut target = open(&state, &project_id)?;
    target.import_style_from(&source)
}

#[tauri::command]
pub fn import_character(
    state: State<'_, AppState>,
    project_id: String,
    source_project_id: String,
    character_id: String,
) -> AppResult<Character> {
    if project_id == source_project_id {
        return Err(AppError::Invalid(
            "cannot import a character into the same project".into(),
        ));
    }
    let source = open(&state, &source_project_id)?;
    let mut target = open(&state, &project_id)?;
    target.import_character_from(&source, &character_id)
}

// ------------------------------------------------------------------ script

#[tauri::command]
pub fn update_scene(
    state: State<'_, AppState>,
    project_id: String,
    scene_id: String,
    input: UpdateSceneInput,
) -> AppResult<Scene> {
    open(&state, &project_id)?.update_scene(&scene_id, input)
}

#[tauri::command]
pub fn update_shot(
    state: State<'_, AppState>,
    project_id: String,
    shot_id: String,
    input: UpdateShotInput,
) -> AppResult<Shot> {
    open(&state, &project_id)?.update_shot(&shot_id, input)
}

#[tauri::command]
pub fn set_shot_status(
    state: State<'_, AppState>,
    project_id: String,
    shot_id: String,
    status: String,
) -> AppResult<Shot> {
    open(&state, &project_id)?.set_shot_status(&shot_id, &status)
}

#[tauri::command]
pub fn list_shot_takes(
    state: State<'_, AppState>,
    project_id: String,
    shot_id: String,
) -> AppResult<Vec<ShotTake>> {
    open(&state, &project_id)?.shot_takes(&shot_id)
}

#[tauri::command]
pub fn select_shot_take(
    state: State<'_, AppState>,
    project_id: String,
    shot_id: String,
    asset_id: String,
) -> AppResult<Shot> {
    let mut store = open(&state, &project_id)?;
    let asset = store.get_asset(&asset_id)?;
    if asset.kind != "video" {
        return Err(AppError::Invalid(
            "only video assets can be used as a shot take".into(),
        ));
    }
    store.attach_shot_asset(
        &shot_id,
        &asset_id,
        "video",
        asset.provider_id.as_deref().unwrap_or("imported"),
        asset.model.as_deref().unwrap_or("unknown"),
        asset.seed,
    )?;
    store.set_shot_status(&shot_id, "needs_review")
}

#[tauri::command]
pub fn delete_shot(
    state: State<'_, AppState>,
    project_id: String,
    shot_id: String,
) -> AppResult<()> {
    open(&state, &project_id)?.delete_shot(&shot_id)
}

#[tauri::command]
pub fn delete_scene(
    state: State<'_, AppState>,
    project_id: String,
    scene_id: String,
) -> AppResult<()> {
    open(&state, &project_id)?.delete_scene(&scene_id)
}

// -------------------------------------------------------------- characters

#[tauri::command]
pub fn create_character(
    state: State<'_, AppState>,
    project_id: String,
    input: CreateCharacterInput,
) -> AppResult<Character> {
    open(&state, &project_id)?.create_character(input)
}

#[tauri::command]
pub fn update_character(
    state: State<'_, AppState>,
    project_id: String,
    character_id: String,
    input: UpdateCharacterInput,
) -> AppResult<Character> {
    open(&state, &project_id)?.update_character(&character_id, input)
}

#[tauri::command]
pub fn delete_character(
    state: State<'_, AppState>,
    project_id: String,
    character_id: String,
) -> AppResult<()> {
    open(&state, &project_id)?.delete_character(&character_id)
}

// ------------------------------------------------------------------ assets

#[tauri::command]
pub fn import_asset(
    state: State<'_, AppState>,
    project_id: String,
    input: ImportAssetInput,
) -> AppResult<Asset> {
    open(&state, &project_id)?.import_asset(input)
}

#[tauri::command]
pub fn list_assets(
    state: State<'_, AppState>,
    project_id: String,
    kind: Option<String>,
) -> AppResult<Vec<Asset>> {
    open(&state, &project_id)?.list_assets(kind.as_deref())
}

#[tauri::command]
pub fn set_asset_status(
    state: State<'_, AppState>,
    project_id: String,
    asset_id: String,
    status: String,
) -> AppResult<Asset> {
    open(&state, &project_id)?.set_asset_status(&asset_id, &status)
}

// -------------------------------------------------------------- generation

#[tauri::command]
pub fn get_context_preview(
    state: State<'_, AppState>,
    project_id: String,
    request: GenerationRequest,
) -> AppResult<ContextPreview> {
    let store = open(&state, &project_id)?;
    context::compile(&store, &state.settings(), &request)
}

#[tauri::command]
pub fn generate_script(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    request: GenerateScriptRequest,
) -> AppResult<GenerationJob> {
    jobs::enqueue_script(&app, &state, &project_id, request)
}

#[tauri::command]
pub fn preview_novel(input: NovelInput) -> AppResult<NovelPlan> {
    if input.text.trim().chars().count() < 50 {
        return Err(AppError::Invalid(
            "novel text is too short; paste at least 50 characters".into(),
        ));
    }
    Ok(crate::services::novel::plan(&input.text))
}

#[tauri::command]
pub fn get_novel(state: State<'_, AppState>, project_id: String) -> AppResult<Option<NovelSource>> {
    open(&state, &project_id)?.novel_source()
}

#[tauri::command]
pub fn start_novel_adaptation(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    novel: NovelInput,
    options: NovelAdaptRequest,
) -> AppResult<GenerationJob> {
    jobs::enqueue_novel(&app, &state, &project_id, novel, options)
}

#[tauri::command]
pub fn read_novel_file(path: String) -> AppResult<String> {
    let file = std::path::Path::new(&path);
    let extension = file
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "txt" | "md" | "text") {
        return Err(AppError::Invalid(
            "only .txt, .md or .text novel files are supported".into(),
        ));
    }
    let metadata = std::fs::metadata(file)
        .map_err(|error| AppError::NotFound(format!("novel file: {error}")))?;
    if metadata.len() > 5 * 1024 * 1024 {
        return Err(AppError::Invalid(
            "novel file is over 5 MB; split it into volumes".into(),
        ));
    }
    let bytes = std::fs::read(file)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[tauri::command]
pub fn generate_media(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    request: GenerationRequest,
) -> AppResult<GenerationJob> {
    jobs::enqueue_generation(&app, &state, &project_id, request)
}

#[tauri::command]
pub fn list_jobs(
    state: State<'_, AppState>,
    project_id: String,
    limit: Option<i64>,
) -> AppResult<Vec<GenerationJob>> {
    open(&state, &project_id)?.list_jobs(limit.unwrap_or(80).clamp(1, 500))
}

#[tauri::command]
pub fn cancel_job(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    job_id: String,
) -> AppResult<GenerationJob> {
    let store = open(&state, &project_id)?;
    // Resolve the job inside the requested project before aborting anything,
    // so a mismatched project id cannot cancel another project's work.
    store.get_job(&job_id)?;
    state.jobs.cancel(&job_id);
    let job = store.cancel_job_record(&job_id)?;
    let _ = app.emit("job:update", &job);
    Ok(job)
}

#[tauri::command]
pub fn retry_job(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    job_id: String,
) -> AppResult<GenerationJob> {
    jobs::retry_job(&app, &state, &project_id, &job_id)
}

// ---------------------------------------------------------------- timeline

#[tauri::command]
pub fn get_timeline(state: State<'_, AppState>, project_id: String) -> AppResult<Timeline> {
    open(&state, &project_id)?.timeline()
}

#[tauri::command]
pub fn update_timeline(
    state: State<'_, AppState>,
    project_id: String,
    patch: TimelinePatch,
) -> AppResult<Timeline> {
    open(&state, &project_id)?.update_timeline(patch)
}

#[tauri::command]
pub fn auto_assemble_timeline(
    state: State<'_, AppState>,
    project_id: String,
    approved_only: Option<bool>,
) -> AppResult<Timeline> {
    open(&state, &project_id)?.auto_assemble(approved_only.unwrap_or(false))
}

#[tauri::command]
pub fn export_timeline(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    options: ExportOptions,
) -> AppResult<GenerationJob> {
    jobs::enqueue_export(&app, &state, &project_id, options)
}

// ---------------------------------------------------------------- settings

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderPatch {
    pub enabled: Option<bool>,
    pub api_base: Option<String>,
    pub default_model: Option<String>,
    pub models: Option<Vec<String>>,
    pub options: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferencesPatch {
    pub defaults: Option<ProviderDefaults>,
    pub concurrency: Option<u32>,
    pub ffmpeg_path: Option<String>,
    pub clear_ffmpeg_path: bool,
    pub ui_language: Option<String>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    Ok(settings::load(&state.paths))
}

fn promote_mock_default(settings: &mut Settings, provider_id: &str) -> bool {
    let Some(provider) = settings.providers.iter().find(|p| p.id == provider_id) else {
        return false;
    };
    if provider.kind == "llm"
        && provider.enabled
        && provider.api_key_set
        && settings.defaults.llm.starts_with("mock-")
    {
        settings.defaults.llm = provider.id.clone();
        return true;
    }
    false
}

#[tauri::command]
pub fn update_provider(
    state: State<'_, AppState>,
    provider_id: String,
    patch: ProviderPatch,
) -> AppResult<Settings> {
    let mut next = settings::update_persisted(&state.paths, |settings| {
        if let Some(provider) = settings.providers.iter_mut().find(|p| p.id == provider_id) {
            if let Some(v) = patch.enabled {
                provider.enabled = v;
            }
            if let Some(v) = patch.api_base {
                provider.api_base = v.trim().trim_end_matches('/').to_string();
            }
            if let Some(v) = patch.default_model {
                provider.default_model = v;
            }
            if let Some(v) = patch.models {
                provider.models = v;
            }
            if let Some(v) = patch.options {
                provider.options = v;
            }
        }
    })?;
    if promote_mock_default(&mut next, &provider_id) {
        settings::save(&state.paths, &next)?;
        next = settings::load(&state.paths);
    }
    if let Ok(mut current) = state.settings.write() {
        *current = next.clone();
    }
    Ok(next)
}

#[tauri::command]
pub fn update_preferences(
    state: State<'_, AppState>,
    patch: PreferencesPatch,
) -> AppResult<Settings> {
    let next = settings::update_persisted(&state.paths, |settings| {
        if let Some(defaults) = patch.defaults {
            settings.defaults = defaults;
        }
        if let Some(concurrency) = patch.concurrency {
            settings.concurrency = concurrency.clamp(1, 4);
        }
        if patch.clear_ffmpeg_path {
            settings.ffmpeg_path = None;
        } else if let Some(path) = patch.ffmpeg_path {
            settings.ffmpeg_path = if path.trim().is_empty() {
                None
            } else {
                Some(path.trim().to_string())
            };
        }
        if let Some(language) = patch.ui_language {
            settings.ui_language = language;
        }
    })?;
    if let Ok(mut current) = state.settings.write() {
        *current = next.clone();
    }
    Ok(next)
}

#[tauri::command]
pub fn set_provider_secret(
    state: State<'_, AppState>,
    provider_id: String,
    secret: String,
) -> AppResult<Settings> {
    let current = settings::load(&state.paths);
    settings::find_provider(&current, &provider_id)?;
    crate::secrets::set_secret(&provider_id, &secret)?;
    let mut next = settings::load(&state.paths);
    if promote_mock_default(&mut next, &provider_id) {
        settings::save(&state.paths, &next)?;
        next = settings::load(&state.paths);
    }
    if let Ok(mut guard) = state.settings.write() {
        *guard = next.clone();
    }
    Ok(next)
}

#[tauri::command]
pub fn clear_provider_secret(
    state: State<'_, AppState>,
    provider_id: String,
) -> AppResult<Settings> {
    let current = settings::load(&state.paths);
    let provider = settings::find_provider(&current, &provider_id)?.clone();
    crate::secrets::delete_secret(&provider_id)?;
    let mut next = settings::load(&state.paths);
    if provider.kind == "llm" && next.defaults.llm == provider_id {
        next.defaults.llm = "mock-llm".into();
        settings::save(&state.paths, &next)?;
        next = settings::load(&state.paths);
    }
    if let Ok(mut guard) = state.settings.write() {
        *guard = next.clone();
    }
    Ok(next)
}

#[tauri::command]
pub fn test_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> AppResult<ProviderTestResult> {
    let started = std::time::Instant::now();
    let settings = settings::load(&state.paths);
    let provider = settings::find_provider(&settings, &provider_id)?;
    if provider.id.starts_with("mock-") {
        return Ok(ProviderTestResult {
            ok: true,
            message:
                "Offline mock provider is ready. It produces local placeholder media at zero cost."
                    .into(),
            latency_ms: 1,
        });
    }
    if provider.vendor == "ollama" || provider.id == "comfyui-image" {
        return Ok(ProviderTestResult {
            ok: true,
            message: format!(
                "Local endpoint configured at {}. Make sure the service is running before generating.",
                provider.api_base
            ),
            latency_ms: started.elapsed().as_millis() as u64,
        });
    }
    match settings::resolve_secret(provider) {
        Ok(_) => Ok(ProviderTestResult {
            ok: true,
            message: "API key is stored in the OS keychain. A real generation will verify account access and consume credits.".into(),
            latency_ms: started.elapsed().as_millis() as u64,
        }),
        Err(_) => Ok(ProviderTestResult {
            ok: false,
            message: "No API key found. Paste a key below; it is stored in the OS keychain, never in the project.".into(),
            latency_ms: started.elapsed().as_millis() as u64,
        }),
    }
}

#[tauri::command]
pub fn export_provenance(state: State<'_, AppState>, project_id: String) -> AppResult<String> {
    let store = open(&state, &project_id)?;
    let path = crate::services::provenance::write(&store)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_system_status(state: State<'_, AppState>) -> AppResult<SystemStatus> {
    let (available, path, version) = media::status(&state.settings());
    let projects = store::list_summaries(&state.paths)?.len();
    Ok(SystemStatus {
        ffmpeg_available: available,
        ffmpeg_path: path,
        ffmpeg_version: version,
        platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        storage_dir: state.paths.root.to_string_lossy().to_string(),
        project_count: projects,
        data_dir: state.paths.root.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn open_storage_folder(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(state.paths.root.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| AppError::Message(format!("could not open folder: {e}")))
}

#[tauri::command]
pub fn reveal_project_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let dir = state.paths.project_dir(&project_id)?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| AppError::Message(format!("could not open folder: {e}")))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFromPath {
    pub source_path: String,
    pub kind: String,
    pub name: String,
}

#[tauri::command]
pub async fn import_asset_path(
    state: State<'_, AppState>,
    project_id: String,
    input: ImportFromPath,
) -> AppResult<Asset> {
    let settings = state.settings();
    let mut store = open(&state, &project_id)?;
    let mut asset = store.import_asset(ImportAssetInput {
        source_path: input.source_path,
        kind: input.kind,
        name: input.name,
    })?;
    if asset.kind != "image" {
        if let Ok(ffmpeg) = media::ensure_ffmpeg(&settings) {
            let probe = ffmpeg.probe(std::path::Path::new(&asset.abs_path));
            let mut thumb_rel = None;
            if asset.kind == "video" {
                let candidate = format!("thumbs/{}.jpg", asset.id);
                let thumb_abs = store.dir.join(&candidate);
                if ffmpeg
                    .extract_frame(std::path::Path::new(&asset.abs_path), 0, &thumb_abs)
                    .await
                    .is_ok()
                {
                    thumb_rel = Some(candidate);
                }
            }
            asset = store.set_asset_media(&asset.id, probe.duration_ms, thumb_rel.as_deref())?;
        }
    }
    Ok(asset)
}

#[tauri::command]
pub async fn search_music(
    state: State<'_, AppState>,
    provider_id: String,
    query: String,
    limit: Option<u32>,
) -> AppResult<MusicSearchResult> {
    let paths = state.paths.clone();
    music_service::search(&paths, &provider_id, &query, limit.unwrap_or(8)).await
}

#[tauri::command]
pub async fn import_music_url(
    state: State<'_, AppState>,
    project_id: String,
    track: MusicTrack,
) -> AppResult<Asset> {
    let paths = state.paths.clone();
    music_service::import_track(&paths, &project_id, &track).await
}

#[tauri::command]
pub fn generate_music(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    request: MusicGenerationRequest,
) -> AppResult<GenerationJob> {
    jobs::enqueue_music(&app, &state, &project_id, request)
}

#[tauri::command]
pub async fn analyze_music_beats(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<BeatAnalysis> {
    let paths = state.paths.clone();
    let settings = state.settings();
    music_service::analyze_beats(&paths, &project_id, &settings).await
}

#[tauri::command]
pub fn auto_beat_sync(state: State<'_, AppState>, project_id: String) -> AppResult<BeatSyncResult> {
    let mut store = open(&state, &project_id)?;
    music_service::auto_beat_sync(&mut store)
}

#[tauri::command]
pub fn attach_music(
    state: State<'_, AppState>,
    project_id: String,
    asset_id: String,
    volume: Option<f64>,
) -> AppResult<Timeline> {
    let mut store = open(&state, &project_id)?;
    music_service::attach_music(&mut store, &asset_id, volume)
}

#[tauri::command]
pub async fn create_demo_project(state: State<'_, AppState>) -> AppResult<ProjectSnapshot> {
    let paths = state.paths.clone();
    let settings = state.settings();
    crate::services::demo::create(&paths, &settings).await
}

#[tauri::command]
pub fn health_check() -> Value {
    json!({"ok": true, "app": "SaladCart"})
}
