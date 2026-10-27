use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::settings;
use crate::store::ProjectStore;

pub fn compile(
    store: &ProjectStore,
    settings: &Settings,
    request: &GenerationRequest,
) -> AppResult<ContextPreview> {
    let provider_id = request
        .provider_id
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| match request.kind.as_str() {
            "video" => settings.defaults.video.clone(),
            _ => settings.defaults.image.clone(),
        });
    let profile = settings::find_provider(settings, &provider_id)?;
    if profile.enabled == false {
        return Err(AppError::Invalid(format!(
            "provider {} is disabled",
            profile.label
        )));
    }
    let model = request
        .model
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| profile.default_model.clone());
    let project = store.project()?;
    let aspect_ratio = request
        .options
        .aspect_ratio
        .clone()
        .unwrap_or_else(|| project.aspect_ratio.clone());
    let mut warnings = Vec::new();
    let mut notes = Vec::new();
    let mut refs: Vec<ContextAssetRef> = Vec::new();

    let (
        title,
        target_id,
        system_prompt,
        user_prompt,
        negative_prompt,
        duration_ms,
        continuity_shot_id,
        continuity_note,
    ) = match request.target_type.as_str() {
        "shot" => {
            let shot_id = request
                .target_id
                .clone()
                .ok_or_else(|| AppError::Invalid("targetId is required for a shot".into()))?;
            let shot = store.get_shot(&shot_id)?;
            let scene = store.shot_scene(&shot_id)?;
            let episode = store.episode_for_scene(&scene.id)?;
            let characters = store.character_for_shot(&shot_id)?;
            let duration_ms = request.options.duration_ms.unwrap_or(shot.duration_ms);

            let system = "You are the supervising art director of a premium AI comic drama. \
Keep every character, costume, palette and camera direction consistent with the supplied project bible. \
Never add captions, logos or watermarks. Return only the generated media."
                    .to_string();

            let mut prompt = String::new();
            prompt.push_str("[PROJECT STYLE]\n");
            prompt.push_str(&format!(
                "Visual: {}\nPalette: {}\nLighting: {}\nCamera language: {}\n",
                empty_or(&project.style_bible.visual_style, "cinematic anime"),
                empty_or(&project.style_bible.palette, "warm cinematic"),
                empty_or(
                    &project.style_bible.lighting,
                    "motivated cinematic lighting"
                ),
                empty_or(
                    &project.style_bible.camera_language,
                    "controlled cinematic coverage"
                )
            ));
            if !project.style_bible.reference_notes.trim().is_empty() {
                prompt.push_str(&format!(
                    "Continuity notes: {}\n",
                    project.style_bible.reference_notes
                ));
            }
            prompt.push_str("\n[SCENE]\n");
            prompt.push_str(&format!(
                "{} · {} — {}. Location: {}. Time: {}.\n",
                episode.title,
                scene.title,
                scene.slugline,
                empty_or(&scene.location, "unspecified"),
                empty_or(&scene.time_of_day, "cinematic time")
            ));
            if !scene.synopsis.trim().is_empty() {
                prompt.push_str(&format!("Story: {}\n", scene.synopsis.trim()));
            }
            prompt.push_str("\n[SHOT]\n");
            prompt.push_str(&format!(
                "{} — {}\nCamera: {}. Lens: {}. Duration: {:.1}s.\n",
                shot.title,
                shot.description,
                empty_or(&shot.camera, "medium shot"),
                empty_or(&shot.lens, "35mm"),
                duration_ms as f64 / 1000.0
            ));
            if !shot.prompt.trim().is_empty() {
                prompt.push_str(&format!("Shot direction: {}\n", shot.prompt.trim()));
            }
            if !shot.dialogue.trim().is_empty() {
                prompt.push_str(&format!(
                    "Performance cue (do not render as text): {}\n",
                    shot.dialogue.trim()
                ));
            }
            if !shot.sound.trim().is_empty() {
                prompt.push_str(&format!("Sound mood: {}\n", shot.sound.trim()));
            }

            prompt.push_str("\n[CAST — identity locked]\n");
            if characters.is_empty() {
                warnings.push(
                    "This shot has no cast attached; character consistency cannot be guaranteed."
                        .into(),
                );
            }
            for character in &characters {
                prompt.push_str(&format!(
                    "- {} ({}): {}. Trait lock: {}.\n",
                    character.name,
                    empty_or(&character.role, "cast"),
                    empty_or(&character.appearance, &character.description),
                    if character.locked_traits.is_empty() {
                        "preserve reference identity".into()
                    } else {
                        character.locked_traits.join(", ")
                    }
                ));
                if character.reference_asset_id.is_none() {
                    warnings.push(format!(
                            "{} has no reference portrait yet. Lock traits and generate a sheet for stable identity.",
                            character.name
                        ));
                }
                if character.locked_traits.is_empty() {
                    warnings.push(format!("{} has no locked traits.", character.name));
                }
                if let Some(asset_id) = &character.reference_asset_id {
                    if let Ok(asset) = store.get_asset(asset_id) {
                        refs.push(ContextAssetRef {
                            id: asset.id,
                            name: format!("{} · portrait", character.name),
                            abs_path: asset.abs_path,
                            role: "character_reference".into(),
                        });
                    }
                }
                if let Some(asset_id) = &character.sheet_asset_id {
                    if let Ok(asset) = store.get_asset(asset_id) {
                        refs.push(ContextAssetRef {
                            id: asset.id,
                            name: format!("{} · character sheet", character.name),
                            abs_path: asset.abs_path,
                            role: "character_sheet".into(),
                        });
                    }
                }
            }

            let mut continuity_shot_id = None;
            let mut continuity_note = None;
            if request.options.use_continuity {
                if let Some(prev_id) = shot
                    .continuity_from_shot_id
                    .clone()
                    .or_else(|| store.previous_shot(&shot_id).ok().flatten().map(|s| s.id))
                {
                    if let Ok(prev) = store.get_shot(&prev_id) {
                        continuity_shot_id = Some(prev.id.clone());
                        let has_video = prev.video_asset_id.is_some();
                        continuity_note = Some(if has_video {
                            format!(
                                    "Continue from the last frame of “{}”. Costume, light and screen direction must match.",
                                    prev.title
                                )
                        } else if prev.image_asset_id.is_some() {
                            format!(
                                    "Continue from the still frame of “{}”. Preserve costume, light and screen direction.",
                                    prev.title
                                )
                        } else {
                            format!(
                                "Continuity is linked to “{}”, but it has no video or image yet.",
                                prev.title
                            )
                        });
                        if !has_video && prev.image_asset_id.is_none() {
                            warnings.push("Continuity source has no generated media yet.".into());
                        }
                        prompt.push_str("\n[CONTINUITY]\n");
                        prompt.push_str(&format!(
                            "Previous shot: {}. {}\n",
                            prev.title, prev.description
                        ));
                    }
                }
            }

            for id in &request.options.reference_asset_ids {
                if let Ok(asset) = store.get_asset(id) {
                    if !refs.iter().any(|r| r.id == asset.id) {
                        refs.push(ContextAssetRef {
                            id: asset.id,
                            name: asset.name,
                            abs_path: asset.abs_path,
                            role: "user_reference".into(),
                        });
                    }
                }
            }

            if !request.options.extra_prompt.trim().is_empty() {
                prompt.push_str("\n[EXTRA DIRECTION]\n");
                prompt.push_str(request.options.extra_prompt.trim());
                prompt.push('\n');
            }
            prompt.push_str("\n[OUTPUT]\n");
            prompt.push_str(&format!(
                "{} aspect ratio, {} seconds, coherent motion, stable faces, no on-screen text.",
                aspect_ratio,
                duration_ms / 1000
            ));

            let mut negative = vec![
                project.style_bible.negative_prompt.clone(),
                shot.negative_prompt.clone(),
            ];
            negative.extend(characters.iter().map(|c| c.negative_prompt.clone()));
            let negative = negative
                .into_iter()
                .filter(|s| !s.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ");

            (
                format!("{} · {}", scene.title, shot.title),
                shot_id,
                system,
                prompt,
                negative,
                duration_ms,
                continuity_shot_id,
                continuity_note,
            )
        }
        "character" => {
            let character_id = request
                .target_id
                .clone()
                .ok_or_else(|| AppError::Invalid("targetId is required for a character".into()))?;
            let character = store.get_character(&character_id)?;
            let system = "You are a character designer for a premium AI comic drama. \
Produce a clean, production-ready character reference with a consistent identity. \
No text, no logos, no watermark."
                .to_string();
            let is_sheet = request
                .options
                .variant
                .as_deref()
                .map(|v| v == "sheet")
                .unwrap_or(false);
            let mut prompt = String::new();
            prompt.push_str("[PROJECT STYLE]\n");
            prompt.push_str(&format!(
                "Visual: {}\nPalette: {}\n",
                empty_or(&project.style_bible.visual_style, "premium anime"),
                empty_or(&project.style_bible.palette, "cinematic")
            ));
            prompt.push_str("\n[CHARACTER BIBLE]\n");
            prompt.push_str(&format!(
                "Name: {}\nRole: {}\nDescription: {}\nAppearance: {}\nPersonality: {}\n",
                character.name,
                empty_or(&character.role, "lead"),
                empty_or(&character.description, "a compelling lead"),
                empty_or(
                    &character.appearance,
                    "distinctive, production-ready design"
                ),
                empty_or(&character.personality, "expressive")
            ));
            if !character.locked_traits.is_empty() {
                prompt.push_str(&format!(
                    "Hard identity locks: {}.\n",
                    character.locked_traits.join(", ")
                ));
            } else {
                warnings.push(
                    "Add locked traits so this character stays consistent across shots.".into(),
                );
            }
            prompt.push_str("\n[OUTPUT]\n");
            if is_sheet {
                prompt.push_str(
                        "Character turnaround sheet on a neutral background: front view, three-quarter view, side view, and three facial expressions. Same costume, same hairstyle, same palette, full body, readable silhouette.",
                    );
            } else {
                prompt.push_str(
                        "Cinematic character key art, waist-up hero framing, neutral studio background, clear face, consistent costume, soft rim light.",
                    );
            }
            prompt.push_str(&format!(" {} aspect ratio.", aspect_ratio));
            if let Some(asset_id) = &character.reference_asset_id {
                if let Ok(asset) = store.get_asset(asset_id) {
                    refs.push(ContextAssetRef {
                        id: asset.id,
                        name: format!("{} · existing portrait", character.name),
                        abs_path: asset.abs_path,
                        role: "existing_reference".into(),
                    });
                    notes
                        .push("The existing portrait is attached as an identity reference.".into());
                }
            }
            for id in &request.options.reference_asset_ids {
                if let Ok(asset) = store.get_asset(id) {
                    if !refs.iter().any(|r| r.id == asset.id) {
                        refs.push(ContextAssetRef {
                            id: asset.id,
                            name: asset.name,
                            abs_path: asset.abs_path,
                            role: "user_reference".into(),
                        });
                    }
                }
            }
            if !request.options.extra_prompt.trim().is_empty() {
                prompt.push_str("\n[EXTRA DIRECTION]\n");
                prompt.push_str(request.options.extra_prompt.trim());
            }
            let negative = [
                project.style_bible.negative_prompt.clone(),
                character.negative_prompt.clone(),
                "multiple characters, extra limbs, text, watermark, logo".to_string(),
            ]
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(", ");
            (
                character.name.clone(),
                character_id,
                system,
                prompt,
                negative,
                0,
                None,
                None,
            )
        }
        other => {
            return Err(AppError::Invalid(format!(
                "unsupported target type: {other}"
            )))
        }
    };

    if let Some(max_refs) = Some(profile.capabilities.max_reference_images).filter(|v| *v > 0) {
        if refs.len() > max_refs as usize {
            warnings.push(format!(
                "{} accepts at most {} reference image(s); extras will be dropped.",
                profile.label, max_refs
            ));
            refs.truncate(max_refs as usize);
        }
    }
    if request.kind == "video" {
        if !profile.capabilities.aspect_ratios.is_empty()
            && !profile.capabilities.aspect_ratios.contains(&aspect_ratio)
        {
            warnings.push(format!(
                "{} does not list {} support; the provider may reject this format.",
                profile.label, aspect_ratio
            ));
        }
        if duration_ms > 0 {
            let seconds = (duration_ms / 1000) as u32;
            let supported = &profile.capabilities.durations_sec;
            if !supported.is_empty() && !supported.contains(&seconds) {
                warnings.push(format!(
                    "{} supports {:?} second clips; {}s will be adjusted.",
                    profile.label, supported, seconds
                ));
            }
        }
        if profile.capabilities.reference_image && refs.is_empty() {
            notes.push(
                "No reference image; the video provider will rely on the text prompt only.".into(),
            );
        }
    }
    if profile.id.starts_with("mock-") {
        notes.push("Mock provider: generates local placeholder media with zero cost.".into());
    }
    let cost = crate::providers::estimate_cost(&profile.id, &request.kind, duration_ms, 1);
    let token_estimate =
        ((system_prompt.chars().count() + user_prompt.chars().count()) as i64 / 2).max(1);
    Ok(ContextPreview {
        target_type: request.target_type.clone(),
        target_id,
        title,
        provider_id: profile.id.clone(),
        provider_label: profile.label.clone(),
        model,
        system_prompt,
        user_prompt,
        negative_prompt,
        character_refs: refs,
        style_bible: project.style_bible,
        warnings,
        notes,
        continuity_from_shot_id: continuity_shot_id,
        continuity_note,
        estimated_cost: cost,
        aspect_ratio,
        duration_ms,
        token_estimate,
    })
}

fn empty_or(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}
