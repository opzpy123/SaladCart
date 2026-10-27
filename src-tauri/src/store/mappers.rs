use crate::models::*;
use chrono::{SecondsFormat, Utc};
use rusqlite::Row;
use serde_json::Value;
use std::path::Path;

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn parse_json<T: serde::de::DeserializeOwned + Default>(raw: &str) -> T {
    serde_json::from_str(raw).unwrap_or_default()
}

pub fn map_project(row: &Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get("id")?,
        name: row.get("name")?,
        premise: row.get("premise")?,
        logline: row.get("logline")?,
        genre: row.get("genre")?,
        tone: row.get("tone")?,
        language: row.get("language")?,
        aspect_ratio: row.get("aspect_ratio")?,
        target_episode_seconds: row.get("target_episode_seconds")?,
        budget_cap: row.get("budget_cap")?,
        style_bible: parse_json(&row.get::<_, String>("style_json")?),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn map_character(row: &Row<'_>) -> rusqlite::Result<Character> {
    Ok(Character {
        id: row.get("id")?,
        name: row.get("name")?,
        role: row.get("role")?,
        description: row.get("description")?,
        appearance: row.get("appearance")?,
        personality: row.get("personality")?,
        voice: row.get("voice")?,
        negative_prompt: row.get("negative_prompt")?,
        locked_traits: parse_json(&row.get::<_, String>("locked_traits")?),
        reference_asset_id: row.get("reference_asset_id")?,
        sheet_asset_id: row.get("sheet_asset_id")?,
        seed: row.get("seed")?,
        provider_id: row.get("provider_id")?,
        model: row.get("model")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn map_scene(row: &Row<'_>) -> rusqlite::Result<Scene> {
    Ok(Scene {
        id: row.get("id")?,
        episode_id: row.get("episode_id")?,
        title: row.get("title")?,
        slugline: row.get("slugline")?,
        location: row.get("location")?,
        time_of_day: row.get("time_of_day")?,
        synopsis: row.get("synopsis")?,
        script_text: row.get("script_text")?,
        order_index: row.get("order_index")?,
        status: row.get("status")?,
        shots: Vec::new(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn map_shot(row: &Row<'_>) -> rusqlite::Result<Shot> {
    Ok(Shot {
        id: row.get("id")?,
        scene_id: row.get("scene_id")?,
        order_index: row.get("order_index")?,
        title: row.get("title")?,
        description: row.get("description")?,
        prompt: row.get("prompt")?,
        negative_prompt: row.get("negative_prompt")?,
        camera: row.get("camera")?,
        lens: row.get("lens")?,
        duration_ms: row.get("duration_ms")?,
        dialogue: row.get("dialogue")?,
        sound: row.get("sound")?,
        character_ids: Vec::new(),
        continuity_from_shot_id: row.get("continuity_from_shot_id")?,
        start_frame_asset_id: row.get("start_frame_asset_id")?,
        image_asset_id: row.get("image_asset_id")?,
        video_asset_id: row.get("video_asset_id")?,
        voice_asset_id: row.get("voice_asset_id")?,
        status: row.get("status")?,
        provider_id: row.get("provider_id")?,
        model: row.get("model")?,
        seed: row.get("seed")?,
        version: row.get("version")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn map_asset(row: &Row<'_>, project_dir: &Path) -> rusqlite::Result<Asset> {
    let rel_path: String = row.get("rel_path")?;
    let thumb_rel_path: Option<String> = row.get("thumb_rel_path")?;
    Ok(Asset {
        id: row.get("id")?,
        kind: row.get("kind")?,
        name: row.get("name")?,
        abs_path: project_dir.join(&rel_path).to_string_lossy().to_string(),
        rel_path,
        thumb_path: thumb_rel_path.map(|p| project_dir.join(p).to_string_lossy().to_string()),
        mime: row.get("mime")?,
        width: row.get("width")?,
        height: row.get("height")?,
        duration_ms: row.get("duration_ms")?,
        size_bytes: row.get("size_bytes")?,
        source: row.get("source")?,
        provider_id: row.get("provider_id")?,
        model: row.get("model")?,
        prompt: row.get("prompt")?,
        negative_prompt: row.get("negative_prompt")?,
        seed: row.get("seed")?,
        version: row.get("version")?,
        parent_asset_id: row.get("parent_asset_id")?,
        license: row.get("license")?,
        license_url: row.get("license_url")?,
        attribution: row.get("attribution")?,
        source_url: row.get("source_url")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
    })
}

pub fn map_job(row: &Row<'_>) -> rusqlite::Result<GenerationJob> {
    let request: String = row.get("request_json")?;
    let result: Option<String> = row.get("result_json")?;
    Ok(GenerationJob {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        kind: row.get("kind")?,
        target_type: row.get("target_type")?,
        target_id: row.get("target_id")?,
        provider_id: row.get("provider_id")?,
        provider_label: row.get("provider_label")?,
        model: row.get("model")?,
        status: row.get("status")?,
        progress: row.get("progress")?,
        stage: row.get("stage")?,
        error: row.get("error")?,
        cost_estimate: row.get("cost_estimate")?,
        cost_actual: row.get("cost_actual")?,
        request_summary: row.get("request_summary")?,
        request: serde_json::from_str(&request).unwrap_or(Value::Null),
        result: result.and_then(|r| serde_json::from_str(&r).ok()),
        result_asset_id: row.get("result_asset_id")?,
        created_at: row.get("created_at")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
    })
}

pub fn map_timeline(row: &Row<'_>) -> rusqlite::Result<Timeline> {
    Ok(Timeline {
        id: row.get("id")?,
        name: row.get("name")?,
        aspect_ratio: row.get("aspect_ratio")?,
        width: row.get("width")?,
        height: row.get("height")?,
        fps: row.get("fps")?,
        clips: Vec::new(),
        subtitles: Vec::new(),
        music_asset_id: row.get("music_asset_id")?,
        music_volume: row.get("music_volume")?,
        bpm: row.get("bpm")?,
        beat_grid_ms: parse_json(&row.get::<_, String>("beat_grid_json")?),
        beat_offset_ms: row.get("beat_offset_ms")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn map_clip(row: &Row<'_>) -> rusqlite::Result<TimelineClip> {
    Ok(TimelineClip {
        id: row.get("id")?,
        timeline_id: row.get("timeline_id")?,
        shot_id: row.get("shot_id")?,
        asset_id: row.get("asset_id")?,
        order_index: row.get("order_index")?,
        in_ms: row.get("in_ms")?,
        out_ms: row.get("out_ms")?,
        duration_ms: row.get("duration_ms")?,
        label: row.get("label")?,
        muted: row.get::<_, i64>("muted")? != 0,
        transition: row.get("transition")?,
    })
}

pub fn map_subtitle(row: &Row<'_>) -> rusqlite::Result<Subtitle> {
    Ok(Subtitle {
        id: row.get("id")?,
        timeline_id: row.get("timeline_id")?,
        start_ms: row.get("start_ms")?,
        end_ms: row.get("end_ms")?,
        text: row.get("text")?,
    })
}
