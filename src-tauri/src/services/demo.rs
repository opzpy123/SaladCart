use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::paths::AppPaths;
use crate::providers::image::ImageRequest;
use crate::providers::mock;
use crate::services::{media, music as music_service, script};
use crate::store::assets::NewAsset;
use crate::store::ProjectStore;
use std::collections::HashMap;

const DEMO_NAME: &str = "雾港来信 · 演示";
const DEMO_PREMISE: &str =
    "在常年落雨的海临城，一个能触碰旧物读取记忆的女孩，发现一枚铜钥匙里存着自己的声音，而每次读取都会让她失去一段记忆。";

pub async fn create(paths: &AppPaths, settings: &Settings) -> AppResult<ProjectSnapshot> {
    if let Ok(summaries) = crate::store::list_summaries(paths) {
        if let Some(existing) = summaries.into_iter().find(|p| p.name == DEMO_NAME) {
            return ProjectStore::open(paths, &existing.id)?.snapshot();
        }
    }
    let input = CreateProjectInput {
        name: DEMO_NAME.into(),
        premise: DEMO_PREMISE.into(),
        genre: "悬疑 / 都市奇幻".into(),
        tone: "冷冽、诗意、克制".into(),
        language: "zh-CN".into(),
        aspect_ratio: "9:16".into(),
        target_episode_seconds: 90,
        style_preset: Some("anime-cel".into()),
        style_bible: None,
    };
    let store = ProjectStore::create(paths, &input)?;
    let project_id = store.project_id.clone();
    let project = store.project()?;
    let profile = settings
        .providers
        .iter()
        .find(|p| p.id == "mock-llm")
        .cloned()
        .ok_or_else(|| AppError::Message("Mock LLM provider is missing".into()))?;
    drop(store);

    let script_request = GenerateScriptRequest {
        premise: input.premise.clone(),
        genre: input.genre.clone(),
        tone: input.tone.clone(),
        language: "zh-CN".into(),
        episodes: 1,
        scenes_per_episode: 2,
        shots_per_scene: 3,
        style_notes: "雨夜、霓虹倒影、克制的镜头运动".into(),
        provider_id: Some("mock-llm".into()),
        model: None,
        replace_existing: false,
        seed: Some(21),
    };
    let generated = script::generate(&profile, None, &script_request, &project).await?;
    let mut store = ProjectStore::open(paths, &project_id)?;
    store.apply_generated_script(generated, &script_request)?;
    let snapshot = store.snapshot()?;
    let project_dir = store.dir.clone();
    drop(store);

    let characters = snapshot.characters.clone();
    let shots: Vec<Shot> = snapshot
        .episodes
        .iter()
        .flat_map(|episode| episode.scenes.iter())
        .flat_map(|scene| scene.shots.iter().cloned())
        .collect();
    let (image_width, image_height) = (900_i64, 1600_i64);
    let mut character_images: Vec<(String, NewAsset)> = Vec::new();
    let mut shot_images: HashMap<String, (NewAsset, Vec<u8>)> = HashMap::new();
    let mut shot_videos: Vec<(String, NewAsset)> = Vec::new();

    for (index, character) in characters.iter().enumerate() {
        let result = mock::image(&ImageRequest {
            model: "saladcart-mock-image".into(),
            prompt: format!(
                "cinematic character portrait, {} {}",
                character.appearance, character.description
            ),
            negative_prompt: character.negative_prompt.clone(),
            aspect_ratio: "9:16".into(),
            width: image_width as i64,
            height: image_height as i64,
            count: 1,
            seed: Some(70 + index as i64),
            references: vec![],
            title: format!("CHARACTER {:02}", index + 1),
        })?;
        let file_id = uuid::Uuid::new_v4().to_string();
        let rel = format!("assets/images/{file_id}.png");
        let abs = project_dir.join(&rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&abs, &result.bytes)?;
        let thumb_rel = format!("thumbs/{file_id}.jpg");
        let thumb = media::image_thumbnail(&abs, &project_dir.join(&thumb_rel), 640)
            .ok()
            .map(|_| thumb_rel);
        character_images.push((
            character.id.clone(),
            NewAsset {
                kind: "image".into(),
                name: format!("{} · 定妆照", character.name),
                rel_path: rel,
                thumb_rel_path: thumb,
                mime: result.mime,
                width: Some(image_width),
                height: Some(image_height),
                duration_ms: None,
                source: "generated".into(),
                provider_id: Some("mock-image".into()),
                model: Some("saladcart-mock-image".into()),
                prompt: Some(character.appearance.clone()),
                negative_prompt: None,
                seed: Some(70 + index as i64),
                version: 1,
                parent_asset_id: None,
                license: Some("Mock generated artwork".into()),
                license_url: None,
                attribution: None,
                source_url: None,
                status: "approved".into(),
            },
        ));
    }

    for (index, shot) in shots.iter().enumerate() {
        let result = mock::image(&ImageRequest {
            model: "saladcart-mock-image".into(),
            prompt: shot.prompt.clone(),
            negative_prompt: shot.negative_prompt.clone(),
            aspect_ratio: "9:16".into(),
            width: image_width as i64,
            height: image_height as i64,
            count: 1,
            seed: Some(100 + index as i64),
            references: vec![],
            title: format!("SHOT {:02}", index + 1),
        })?;
        let file_id = uuid::Uuid::new_v4().to_string();
        let rel = format!("assets/images/{file_id}.png");
        let abs = project_dir.join(&rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&abs, &result.bytes)?;
        let thumb_rel = format!("thumbs/{file_id}.jpg");
        let thumb = media::image_thumbnail(&abs, &project_dir.join(&thumb_rel), 640)
            .ok()
            .map(|_| thumb_rel);
        shot_images.insert(
            shot.id.clone(),
            (
                NewAsset {
                    kind: "image".into(),
                    name: format!("{} · 关键画面", shot.title),
                    rel_path: rel,
                    thumb_rel_path: thumb.clone(),
                    mime: result.mime.clone(),
                    width: Some(image_width),
                    height: Some(image_height),
                    duration_ms: None,
                    source: "generated".into(),
                    provider_id: Some("mock-image".into()),
                    model: Some("saladcart-mock-image".into()),
                    prompt: Some(shot.prompt.clone()),
                    negative_prompt: Some(shot.negative_prompt.clone()),
                    seed: Some(100 + index as i64),
                    version: 1,
                    parent_asset_id: None,
                    license: Some("Mock generated artwork".into()),
                    license_url: None,
                    attribution: None,
                    source_url: None,
                    status: "approved".into(),
                },
                result.bytes,
            ),
        );
    }

    if let Some(ffmpeg) = media::Ffmpeg::discover(settings) {
        for (index, shot) in shots.iter().take(4).enumerate() {
            let Some((_, image_bytes)) = shot_images.get(&shot.id) else {
                continue;
            };
            let file_id = uuid::Uuid::new_v4().to_string();
            let rel = format!("assets/videos/{file_id}.mp4");
            let abs = project_dir.join(&rel);
            let duration = shot.duration_ms.clamp(3_000, 6_000);
            if ffmpeg
                .write_placeholder_video(image_bytes, &abs, 720, 1280, duration)
                .await
                .is_ok()
            {
                let thumb = shot_images
                    .get(&shot.id)
                    .and_then(|(asset, _)| asset.thumb_rel_path.clone());
                shot_videos.push((
                    shot.id.clone(),
                    NewAsset {
                        kind: "video".into(),
                        name: format!("{} · 演示成片", shot.title),
                        rel_path: rel,
                        thumb_rel_path: thumb,
                        mime: "video/mp4".into(),
                        width: Some(720),
                        height: Some(1280),
                        duration_ms: Some(duration),
                        source: "generated".into(),
                        provider_id: Some("mock-video".into()),
                        model: Some("saladcart-mock-video".into()),
                        prompt: Some(shot.prompt.clone()),
                        negative_prompt: Some(shot.negative_prompt.clone()),
                        seed: Some(100 + index as i64),
                        version: 1,
                        parent_asset_id: None,
                        license: Some("Mock generated footage".into()),
                        license_url: None,
                        attribution: None,
                        source_url: None,
                        status: "needs_review".into(),
                    },
                ));
            }
        }
    }

    let music = mock::music(
        "rain-soaked mystery, restrained piano and low pulse",
        24_000,
        Some(9),
    )?;
    let music_id = uuid::Uuid::new_v4().to_string();
    let music_rel = format!("assets/audio/{music_id}.wav");
    let music_abs = project_dir.join(&music_rel);
    if let Some(parent) = music_abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&music_abs, &music.bytes)?;
    let music_asset = NewAsset {
        kind: "audio".into(),
        name: "雨幕之下 · 演示配乐".into(),
        rel_path: music_rel,
        thumb_rel_path: None,
        mime: music.mime,
        width: None,
        height: None,
        duration_ms: Some(music.duration_ms),
        source: "generated".into(),
        provider_id: Some("mock-music".into()),
        model: Some("saladcart-mock-music".into()),
        prompt: Some("rain-soaked mystery".into()),
        negative_prompt: None,
        seed: Some(9),
        version: 1,
        parent_asset_id: None,
        license: Some("Mock generated music — demo use".into()),
        license_url: None,
        attribution: None,
        source_url: None,
        status: "approved".into(),
    };

    let mut store = ProjectStore::open(paths, &project_id)?;
    for (character_id, asset) in character_images {
        let asset = store.insert_asset(asset)?;
        store.attach_character_reference(&character_id, &asset.id, false)?;
    }
    for (shot_id, (asset, _)) in shot_images {
        let asset = store.insert_asset(asset)?;
        store.attach_shot_asset(
            &shot_id,
            &asset.id,
            "image",
            "mock-image",
            "saladcart-mock-image",
            Some(100),
        )?;
    }
    for (shot_id, asset) in shot_videos {
        let asset = store.insert_asset(asset)?;
        store.attach_shot_asset(
            &shot_id,
            &asset.id,
            "video",
            "mock-video",
            "saladcart-mock-video",
            Some(100),
        )?;
        store.set_shot_status(&shot_id, "needs_review")?;
    }
    let music_asset = store.insert_asset(music_asset)?;
    store.auto_assemble(false)?;
    let bpm = music.bpm;
    let interval = (60_000.0 / bpm) as i64;
    let mut beats = Vec::new();
    let mut time = 0;
    while time <= music.duration_ms {
        beats.push(time);
        time += interval.max(1);
    }
    let volume = store.timeline()?.music_volume;
    store.update_timeline(TimelinePatch {
        music_asset_id: Some(music_asset.id),
        music_volume: Some(volume),
        bpm: Some(bpm),
        beat_grid_ms: Some(beats),
        beat_offset_ms: Some(0),
        ..Default::default()
    })?;
    let _ = music_service::auto_beat_sync(&mut store);
    store.snapshot()
}
