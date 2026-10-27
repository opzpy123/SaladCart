use crate::error::AppResult;
use crate::store::ProjectStore;
use serde_json::json;
use std::path::PathBuf;

pub fn build(store: &ProjectStore) -> AppResult<serde_json::Value> {
    let snapshot = store.snapshot()?;
    let assets = snapshot
        .assets
        .iter()
        .map(|asset| {
            json!({
                "id": asset.id,
                "kind": asset.kind,
                "name": asset.name,
                "source": asset.source,
                "providerId": asset.provider_id,
                "model": asset.model,
                "prompt": asset.prompt,
                "negativePrompt": asset.negative_prompt,
                "seed": asset.seed,
                "version": asset.version,
                "parentAssetId": asset.parent_asset_id,
                "license": asset.license,
                "licenseUrl": asset.license_url,
                "attribution": asset.attribution,
                "sourceUrl": asset.source_url,
                "createdAt": asset.created_at,
                "status": asset.status,
            })
        })
        .collect::<Vec<_>>();
    let shots = snapshot
        .episodes
        .iter()
        .flat_map(|episode| episode.scenes.iter().map(move |scene| (episode, scene)))
        .flat_map(|(episode, scene)| {
            scene.shots.iter().map(move |shot| {
                json!({
                    "episode": episode.title,
                    "scene": scene.title,
                    "title": shot.title,
                    "description": shot.description,
                    "prompt": shot.prompt,
                    "characters": shot.character_ids,
                    "providerId": shot.provider_id,
                    "model": shot.model,
                    "seed": shot.seed,
                    "version": shot.version,
                    "status": shot.status,
                    "imageAssetId": shot.image_asset_id,
                    "videoAssetId": shot.video_asset_id,
                    "voiceAssetId": shot.voice_asset_id,
                    "continuityFromShotId": shot.continuity_from_shot_id,
                })
            })
        })
        .collect::<Vec<_>>();
    let jobs = snapshot
        .jobs
        .iter()
        .map(|job| {
            json!({
                "id": job.id,
                "kind": job.kind,
                "targetType": job.target_type,
                "targetId": job.target_id,
                "provider": job.provider_label,
                "model": job.model,
                "status": job.status,
                "costEstimate": job.cost_estimate,
                "costActual": job.cost_actual,
                "createdAt": job.created_at,
                "finishedAt": job.finished_at,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "schemaVersion": 1,
        "app": "SaladCart",
        "appVersion": env!("CARGO_PKG_VERSION"),
        "exportedAt": crate::store::now_iso(),
        "notice": "AI-generated production dossier. Review provider and music licences before publishing.",
        "project": {
            "id": snapshot.project.id,
            "name": snapshot.project.name,
            "logline": snapshot.project.logline,
            "genre": snapshot.project.genre,
            "aspectRatio": snapshot.project.aspect_ratio,
            "styleBible": snapshot.project.style_bible,
        },
        "characters": snapshot.characters.iter().map(|character| json!({
            "id": character.id,
            "name": character.name,
            "role": character.role,
            "appearance": character.appearance,
            "lockedTraits": character.locked_traits,
            "referenceAssetId": character.reference_asset_id,
            "sheetAssetId": character.sheet_asset_id,
            "providerId": character.provider_id,
            "model": character.model,
            "seed": character.seed,
        })).collect::<Vec<_>>(),
        "shots": shots,
        "timeline": {
            "id": snapshot.timeline.id,
            "aspectRatio": snapshot.timeline.aspect_ratio,
            "width": snapshot.timeline.width,
            "height": snapshot.timeline.height,
            "fps": snapshot.timeline.fps,
            "musicAssetId": snapshot.timeline.music_asset_id,
            "bpm": snapshot.timeline.bpm,
            "clips": snapshot.timeline.clips.len(),
        },
        "assets": assets,
        "jobs": jobs,
    }))
}

pub fn write(store: &ProjectStore) -> AppResult<PathBuf> {
    let dir = store.paths.exports_dir(&store.project_id)?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("provenance.json");
    let body = serde_json::to_string_pretty(&build(store)?)?;
    std::fs::write(&path, body)?;
    Ok(path)
}
