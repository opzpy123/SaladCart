use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STATUS_DRAFT: &str = "draft";
pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_SUCCEEDED: &str = "succeeded";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_CANCELED: &str = "canceled";
pub const STATUS_READY: &str = "ready";
pub const STATUS_APPROVED: &str = "approved";
pub const STATUS_NEEDS_REVIEW: &str = "needs_review";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct StyleBible {
    pub visual_style: String,
    pub palette: String,
    pub lighting: String,
    pub camera_language: String,
    pub negative_prompt: String,
    pub reference_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub premise: String,
    pub logline: String,
    pub genre: String,
    pub tone: String,
    pub language: String,
    pub aspect_ratio: String,
    pub target_episode_seconds: i64,
    pub style_bible: StyleBible,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectCounts {
    pub characters: i64,
    pub episodes: i64,
    pub scenes: i64,
    pub shots: i64,
    pub videos: i64,
    pub images: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub logline: String,
    pub premise: String,
    pub genre: String,
    pub aspect_ratio: String,
    pub updated_at: String,
    pub cover_path: Option<String>,
    pub storage_path: String,
    pub counts: ProjectCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub id: String,
    pub name: String,
    pub role: String,
    pub description: String,
    pub appearance: String,
    pub personality: String,
    pub voice: String,
    pub negative_prompt: String,
    pub locked_traits: Vec<String>,
    pub reference_asset_id: Option<String>,
    pub sheet_asset_id: Option<String>,
    pub seed: Option<i64>,
    pub provider_id: String,
    pub model: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub order_index: i64,
    pub scenes: Vec<Scene>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: String,
    pub episode_id: String,
    pub title: String,
    pub slugline: String,
    pub location: String,
    pub time_of_day: String,
    pub synopsis: String,
    pub script_text: String,
    pub order_index: i64,
    pub status: String,
    pub shots: Vec<Shot>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shot {
    pub id: String,
    pub scene_id: String,
    pub order_index: i64,
    pub title: String,
    pub description: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub camera: String,
    pub lens: String,
    pub duration_ms: i64,
    pub dialogue: String,
    pub sound: String,
    pub character_ids: Vec<String>,
    pub continuity_from_shot_id: Option<String>,
    pub start_frame_asset_id: Option<String>,
    pub image_asset_id: Option<String>,
    pub video_asset_id: Option<String>,
    pub voice_asset_id: Option<String>,
    pub status: String,
    pub provider_id: String,
    pub model: String,
    pub seed: Option<i64>,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub rel_path: String,
    pub abs_path: String,
    pub thumb_path: Option<String>,
    pub mime: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration_ms: Option<i64>,
    pub size_bytes: Option<i64>,
    pub source: String,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub seed: Option<i64>,
    pub version: i64,
    pub parent_asset_id: Option<String>,
    pub license: Option<String>,
    pub license_url: Option<String>,
    pub attribution: Option<String>,
    pub source_url: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationJob {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub provider_id: String,
    pub provider_label: String,
    pub model: String,
    pub status: String,
    pub progress: i64,
    pub stage: String,
    pub error: Option<String>,
    pub cost_estimate: f64,
    pub cost_actual: f64,
    pub request_summary: String,
    pub request: Value,
    pub result: Option<Value>,
    pub result_asset_id: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineClip {
    pub id: String,
    pub timeline_id: String,
    pub shot_id: Option<String>,
    pub asset_id: Option<String>,
    pub order_index: i64,
    pub in_ms: i64,
    pub out_ms: i64,
    pub duration_ms: i64,
    pub label: String,
    pub muted: bool,
    pub transition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtitle {
    pub id: String,
    pub timeline_id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub id: String,
    pub name: String,
    pub aspect_ratio: String,
    pub width: i64,
    pub height: i64,
    pub fps: i64,
    pub clips: Vec<TimelineClip>,
    pub subtitles: Vec<Subtitle>,
    pub music_asset_id: Option<String>,
    pub music_volume: f64,
    pub bpm: Option<f64>,
    pub beat_grid_ms: Vec<i64>,
    pub beat_offset_ms: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub project: Project,
    pub characters: Vec<Character>,
    pub episodes: Vec<Episode>,
    pub assets: Vec<Asset>,
    pub jobs: Vec<GenerationJob>,
    pub timeline: Timeline,
    pub storage_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderCapabilities {
    pub aspect_ratios: Vec<String>,
    pub durations_sec: Vec<u32>,
    pub text_to_video: bool,
    pub image_to_video: bool,
    pub reference_image: bool,
    pub max_reference_images: u32,
    pub structured_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProfile {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub vendor: String,
    pub enabled: bool,
    pub builtin: bool,
    pub api_base: String,
    pub default_model: String,
    pub models: Vec<String>,
    pub api_key_set: bool,
    pub env_var: String,
    pub options: Value,
    pub capabilities: ProviderCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderDefaults {
    pub llm: String,
    pub image: String,
    pub video: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub providers: Vec<ProviderProfile>,
    pub defaults: ProviderDefaults,
    pub concurrency: u32,
    pub ffmpeg_path: Option<String>,
    pub storage_dir: Option<String>,
    pub ui_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateProjectInput {
    pub name: String,
    pub premise: String,
    pub genre: String,
    pub tone: String,
    pub language: String,
    pub aspect_ratio: String,
    pub target_episode_seconds: i64,
    pub style_preset: Option<String>,
    pub style_bible: Option<StyleBible>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub premise: Option<String>,
    pub logline: Option<String>,
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub language: Option<String>,
    pub aspect_ratio: Option<String>,
    pub target_episode_seconds: Option<i64>,
    pub style_bible: Option<StyleBible>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateCharacterInput {
    pub name: String,
    pub role: String,
    pub description: String,
    pub appearance: String,
    pub personality: String,
    pub voice: String,
    pub locked_traits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateCharacterInput {
    pub name: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub appearance: Option<String>,
    pub personality: Option<String>,
    pub voice: Option<String>,
    pub negative_prompt: Option<String>,
    pub locked_traits: Option<Vec<String>>,
    pub reference_asset_id: Option<String>,
    pub sheet_asset_id: Option<String>,
    pub seed: Option<i64>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub status: Option<String>,
    pub clear_reference_asset: bool,
    pub clear_sheet_asset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateSceneInput {
    pub title: Option<String>,
    pub slugline: Option<String>,
    pub location: Option<String>,
    pub time_of_day: Option<String>,
    pub synopsis: Option<String>,
    pub script_text: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateShotInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub duration_ms: Option<i64>,
    pub dialogue: Option<String>,
    pub sound: Option<String>,
    pub character_ids: Option<Vec<String>>,
    pub continuity_from_shot_id: Option<String>,
    pub clear_continuity: bool,
    pub start_frame_asset_id: Option<String>,
    pub status: Option<String>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub seed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GenerateScriptRequest {
    pub premise: String,
    pub genre: String,
    pub tone: String,
    pub language: String,
    pub episodes: u32,
    pub scenes_per_episode: u32,
    pub shots_per_scene: u32,
    pub style_notes: String,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub replace_existing: bool,
    pub seed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedCharacter {
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub appearance: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub voice: String,
    #[serde(default)]
    pub locked_traits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedShot {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub camera: String,
    #[serde(default)]
    pub lens: String,
    #[serde(default = "default_shot_seconds")]
    pub duration_sec: i64,
    #[serde(default)]
    pub dialogue: String,
    #[serde(default)]
    pub sound: String,
    #[serde(default)]
    pub characters: Vec<String>,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub negative_prompt: String,
}

fn default_shot_seconds() -> i64 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedScene {
    pub title: String,
    #[serde(default)]
    pub slugline: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub time_of_day: String,
    #[serde(default)]
    pub synopsis: String,
    #[serde(default)]
    pub script_text: String,
    #[serde(default)]
    pub shots: Vec<GeneratedShot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedEpisode {
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub scenes: Vec<GeneratedScene>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneratedScript {
    pub title: String,
    pub logline: String,
    pub synopsis: String,
    pub style_bible: StyleBible,
    pub characters: Vec<GeneratedCharacter>,
    pub episodes: Vec<GeneratedEpisode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GenerationOptions {
    pub duration_ms: Option<i64>,
    pub aspect_ratio: Option<String>,
    pub seed: Option<i64>,
    pub variant: Option<String>,
    pub count: Option<u32>,
    pub extra_prompt: String,
    pub use_continuity: bool,
    pub reference_asset_ids: Vec<String>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationRequest {
    pub kind: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub options: GenerationOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextAssetRef {
    pub id: String,
    pub name: String,
    pub abs_path: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPreview {
    pub target_type: String,
    pub target_id: String,
    pub title: String,
    pub provider_id: String,
    pub provider_label: String,
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub negative_prompt: String,
    pub character_refs: Vec<ContextAssetRef>,
    pub style_bible: StyleBible,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
    pub continuity_from_shot_id: Option<String>,
    pub continuity_note: Option<String>,
    pub estimated_cost: f64,
    pub aspect_ratio: String,
    pub duration_ms: i64,
    pub token_estimate: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TimelineClipInput {
    pub shot_id: Option<String>,
    pub asset_id: Option<String>,
    pub in_ms: i64,
    pub out_ms: i64,
    pub label: String,
    pub muted: bool,
    pub transition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SubtitleInput {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TimelinePatch {
    pub name: Option<String>,
    pub aspect_ratio: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub fps: Option<i64>,
    pub clips: Option<Vec<TimelineClipInput>>,
    pub subtitles: Option<Vec<SubtitleInput>>,
    pub music_asset_id: Option<String>,
    pub clear_music: bool,
    pub music_volume: Option<f64>,
    pub bpm: Option<f64>,
    pub beat_grid_ms: Option<Vec<i64>>,
    pub beat_offset_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    pub width: i64,
    pub height: i64,
    pub fps: i64,
    pub burn_subtitles: bool,
    pub music_asset_id: Option<String>,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReference {
    pub project_id: String,
    pub project_name: String,
    pub characters: Vec<Character>,
    pub style_bible: StyleBible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub ffmpeg_available: bool,
    pub ffmpeg_path: Option<String>,
    pub ffmpeg_version: Option<String>,
    pub platform: String,
    pub app_version: String,
    pub storage_dir: String,
    pub project_count: usize,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTestResult {
    pub ok: bool,
    pub message: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedAsset {
    pub asset: Asset,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ImportAssetInput {
    pub source_path: String,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub provider_id: String,
    pub provider_label: String,
    pub preview_url: String,
    pub download_url: String,
    pub duration_ms: i64,
    pub license: String,
    pub license_url: String,
    pub requires_attribution: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicSearchResult {
    pub tracks: Vec<MusicTrack>,
    pub provider_label: String,
    pub query: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeatAnalysis {
    pub asset_id: String,
    pub bpm: f64,
    pub confidence: f64,
    pub offset_ms: i64,
    pub beat_grid_ms: Vec<i64>,
    pub waveform: Vec<f32>,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeatSyncResult {
    pub aligned_cuts: usize,
    pub total_cuts: usize,
    pub average_offset_ms: i64,
    pub timeline: Timeline,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MusicGenerationRequest {
    pub prompt: String,
    pub duration_ms: i64,
    pub instrumental: bool,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub attach_to_timeline: bool,
    pub seed: Option<i64>,
}
