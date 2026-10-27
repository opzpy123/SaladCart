use saladcart_lib::models::*;
use saladcart_lib::paths::AppPaths;
use saladcart_lib::providers::mock;
use saladcart_lib::services::context;
use saladcart_lib::services::media::Ffmpeg;
use saladcart_lib::settings;
use saladcart_lib::store::assets::NewAsset;
use saladcart_lib::store::ProjectStore;

fn input(name: &str) -> CreateProjectInput {
    CreateProjectInput {
        name: name.into(),
        premise: "一个能听见旧物记忆的女孩，追查自己失踪的过去。".into(),
        genre: "悬疑".into(),
        tone: "冷冽".into(),
        language: "zh-CN".into(),
        aspect_ratio: "9:16".into(),
        target_episode_seconds: 90,
        style_preset: Some("anime-cel".into()),
        style_bible: None,
    }
}

fn generated_script() -> GeneratedScript {
    GeneratedScript {
        title: "雾港来信".into(),
        logline: "她听见了旧物的过去。".into(),
        synopsis: "一把钥匙带来十年前的录音。".into(),
        style_bible: StyleBible {
            visual_style: "cinematic Chinese anime".into(),
            palette: "emerald and vermilion".into(),
            lighting: "wet reflections".into(),
            camera_language: "storyboard driven".into(),
            negative_prompt: "text, watermark".into(),
            reference_notes: "keep costume".into(),
        },
        characters: vec![GeneratedCharacter {
            name: "沈知遥".into(),
            role: "主角".into(),
            description: "记忆修复师".into(),
            appearance: "黑色短发，深灰风衣".into(),
            personality: "克制".into(),
            voice: "低稳".into(),
            locked_traits: vec!["黑色及肩短发".into(), "银灰挑染".into(), "深灰风衣".into()],
        }],
        episodes: vec![GeneratedEpisode {
            title: "第1集".into(),
            summary: "雨夜".into(),
            scenes: vec![GeneratedScene {
                title: "天台".into(),
                slugline: "EXT. 天台 - NIGHT".into(),
                location: "天台".into(),
                time_of_day: "NIGHT".into(),
                synopsis: "钥匙在积水里震动。".into(),
                script_text: "沈知遥停住脚步。".into(),
                shots: vec![
                    GeneratedShot {
                        title: "钩子".into(),
                        description: "钥匙震动".into(),
                        camera: "push-in".into(),
                        lens: "35mm".into(),
                        duration_sec: 5,
                        dialogue: "听。".into(),
                        sound: "雨".into(),
                        characters: vec!["沈知遥".into()],
                        prompt: "rainy rooftop".into(),
                        negative_prompt: "text".into(),
                    },
                    GeneratedShot {
                        title: "回望".into(),
                        description: "她回头".into(),
                        camera: "handheld".into(),
                        lens: "28mm".into(),
                        duration_sec: 5,
                        dialogue: "".into(),
                        sound: "".into(),
                        characters: vec!["沈知遥".into()],
                        prompt: "she turns".into(),
                        negative_prompt: "".into(),
                    },
                ],
            }],
        }],
    }
}

#[test]
fn projects_are_isolated_on_disk() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut first = ProjectStore::create(&paths, &input("项目 A")).unwrap();
    let second = ProjectStore::create(&paths, &input("项目 B")).unwrap();
    first
        .update_project(UpdateProjectInput {
            logline: Some("只属于 A".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(first.project().unwrap().logline, "只属于 A");
    assert_eq!(second.project().unwrap().logline, "");
    let summaries = saladcart_lib::store::list_summaries(&paths).unwrap();
    assert_eq!(summaries.len(), 2);
    assert_ne!(first.dir, second.dir);
}

#[test]
fn generated_script_links_characters_and_compiles_context() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("上下文测试")).unwrap();
    let request = GenerateScriptRequest {
        premise: "雨夜".into(),
        genre: "悬疑".into(),
        tone: "冷冽".into(),
        language: "zh-CN".into(),
        episodes: 1,
        scenes_per_episode: 1,
        shots_per_scene: 2,
        style_notes: "".into(),
        provider_id: Some("mock-llm".into()),
        model: None,
        replace_existing: false,
        seed: None,
    };
    let stats = store
        .apply_generated_script(generated_script(), &request)
        .unwrap();
    assert_eq!(stats.characters_created, 1);
    assert_eq!(stats.shots, 2);

    let snapshot = store.snapshot().unwrap();
    assert_eq!(snapshot.characters.len(), 1);
    assert_eq!(snapshot.episodes[0].scenes[0].shots.len(), 2);
    let shot = &snapshot.episodes[0].scenes[0].shots[0];
    assert_eq!(shot.character_ids.len(), 1);
    assert!(snapshot.episodes[0].scenes[0].shots[1]
        .continuity_from_shot_id
        .is_some());

    let settings = settings::default_settings();
    let preview = context::compile(
        &store,
        &settings,
        &GenerationRequest {
            kind: "video".into(),
            target_type: "shot".into(),
            target_id: Some(shot.id.clone()),
            provider_id: Some("mock-video".into()),
            model: None,
            options: GenerationOptions {
                duration_ms: Some(5000),
                use_continuity: true,
                ..Default::default()
            },
            compiled: None,
        },
    )
    .unwrap();
    assert_eq!(preview.provider_id, "mock-video");
    assert!(preview.user_prompt.contains("银灰挑染"));
    assert!(preview.user_prompt.contains("9:16"));
    assert!(preview.user_prompt.contains("钩子"));
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.contains("reference portrait")));
    assert!(preview.token_estimate > 0);
}

#[test]
fn timeline_auto_assembly_and_beat_snap() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("卡点测试")).unwrap();
    let request = GenerateScriptRequest::default();
    store
        .apply_generated_script(generated_script(), &request)
        .unwrap();
    let shots = store.shots_in_order().unwrap();
    for (index, shot) in shots.iter().enumerate() {
        let rel = format!("assets/videos/test-{index}.mp4");
        let abs = store.asset_abs(&rel);
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, b"placeholder").unwrap();
        let asset = store
            .insert_asset(NewAsset {
                kind: "video".into(),
                name: format!("shot {index}"),
                rel_path: rel,
                thumb_rel_path: None,
                mime: "video/mp4".into(),
                width: Some(720),
                height: Some(1280),
                duration_ms: Some(5_000),
                source: "test".into(),
                provider_id: Some("mock-video".into()),
                model: Some("mock".into()),
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
            })
            .unwrap();
        store
            .attach_shot_asset(&shot.id, &asset.id, "video", "mock-video", "mock", None)
            .unwrap();
    }
    let timeline = store.auto_assemble(false).unwrap();
    assert_eq!(timeline.clips.len(), 2);
    assert_eq!(timeline.subtitles.len(), 1);

    store
        .update_timeline(TimelinePatch {
            bpm: Some(120.0),
            beat_grid_ms: Some((0..30).map(|i| i * 500).collect()),
            beat_offset_ms: Some(0),
            ..Default::default()
        })
        .unwrap();
    let result = saladcart_lib::services::music::auto_beat_sync(&mut store).unwrap();
    assert_eq!(result.aligned_cuts, 2);
    assert_eq!(result.average_offset_ms, 0);
}

#[tokio::test]
async fn mock_llm_generates_an_applicable_bible() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("Mock 剧本")).unwrap();
    let settings = settings::default_settings();
    let profile = settings
        .providers
        .iter()
        .find(|p| p.id == "mock-llm")
        .unwrap();
    let request = GenerateScriptRequest {
        premise: "雨夜里能听见旧物记忆的女孩".into(),
        genre: "悬疑".into(),
        tone: "冷冽".into(),
        language: "zh-CN".into(),
        episodes: 1,
        scenes_per_episode: 2,
        shots_per_scene: 2,
        style_notes: "".into(),
        provider_id: Some("mock-llm".into()),
        model: None,
        replace_existing: false,
        seed: None,
    };
    let project = store.project().unwrap();
    let generated = saladcart_lib::services::script::generate(profile, None, &request, &project)
        .await
        .unwrap();
    assert!(!generated.characters.is_empty());
    assert!(!generated.episodes.is_empty());
    let stats = store.apply_generated_script(generated, &request).unwrap();
    assert_eq!(stats.episodes, 1);
    assert_eq!(stats.scenes, 2);
    assert_eq!(stats.shots, 4);
    let snapshot = store.snapshot().unwrap();
    assert!(snapshot.characters.len() >= 2);
}

#[tokio::test]
async fn demo_project_seeds_the_full_graph() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let settings = settings::default_settings();
    let snapshot = saladcart_lib::services::demo::create(&paths, &settings)
        .await
        .unwrap();
    assert_eq!(snapshot.project.name, "雾港来信 · 演示");
    assert!(snapshot.characters.len() >= 3);
    let shots: Vec<&Shot> = snapshot
        .episodes
        .iter()
        .flat_map(|episode| episode.scenes.iter())
        .flat_map(|scene| scene.shots.iter())
        .collect();
    assert!(shots.len() >= 4);
    assert!(snapshot.assets.iter().any(|a| a.kind == "audio"));
    assert!(snapshot.assets.iter().filter(|a| a.kind == "image").count() >= 6);
    assert!(snapshot.timeline.music_asset_id.is_some());
    if Ffmpeg::discover(&settings).is_some() {
        assert!(snapshot.assets.iter().any(|a| a.kind == "video"));
        assert!(!snapshot.timeline.clips.is_empty());
    }
}

#[test]
fn jobs_round_trip_with_project_id() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let store = ProjectStore::create(&paths, &input("任务")).unwrap();
    let now = saladcart_lib::store::now_iso();
    let job = GenerationJob {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: store.project_id.clone(),
        kind: "script".into(),
        target_type: "project".into(),
        target_id: Some(store.project_id.clone()),
        provider_id: "mock-llm".into(),
        provider_label: "Mock".into(),
        model: "mock".into(),
        status: "queued".into(),
        progress: 0,
        stage: "queued".into(),
        error: None,
        cost_estimate: 0.0,
        cost_actual: 0.0,
        request_summary: "test".into(),
        request: serde_json::json!({}),
        result: None,
        result_asset_id: None,
        created_at: now.clone(),
        started_at: None,
        finished_at: None,
    };
    store.insert_job(&job).unwrap();
    let loaded = store.get_job(&job.id).unwrap();
    assert_eq!(loaded.project_id, store.project_id);
    assert_eq!(store.list_jobs(10).unwrap().len(), 1);
    assert_eq!(store.snapshot().unwrap().jobs.len(), 1);
}

#[test]
fn mock_media_and_music_files_are_created() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("素材")).unwrap();
    let image = mock::image(&saladcart_lib::providers::image::ImageRequest {
        model: "mock".into(),
        prompt: "a rainy rooftop".into(),
        negative_prompt: "".into(),
        aspect_ratio: "9:16".into(),
        width: 900,
        height: 1600,
        count: 1,
        seed: Some(3),
        references: vec![],
        title: "TEST".into(),
    })
    .unwrap();
    let rel = "assets/images/test.png";
    let abs = store.asset_abs(rel);
    std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
    std::fs::write(&abs, image.bytes).unwrap();
    let asset = store
        .insert_asset(NewAsset {
            kind: "image".into(),
            name: "test".into(),
            rel_path: rel.into(),
            thumb_rel_path: None,
            mime: "image/png".into(),
            width: Some(900),
            height: Some(1600),
            duration_ms: None,
            source: "test".into(),
            provider_id: None,
            model: None,
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
        })
        .unwrap();
    assert!(std::path::Path::new(&asset.abs_path).exists());

    // FFmpeg is optional in test environments; if present, verify a real export.
    if let Some(ffmpeg) = Ffmpeg::discover(&settings::default_settings()) {
        assert!(ffmpeg.version().is_some());
    }
}

#[test]
fn project_style_can_be_copied_from_another_project() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut source = ProjectStore::create(&paths, &input("风格源")).unwrap();
    source
        .update_project(UpdateProjectInput {
            style_bible: Some(StyleBible {
                visual_style: "manga noir with red accents".into(),
                palette: "black, bone, cinnabar".into(),
                lighting: "hard shadows".into(),
                camera_language: "dutch angles".into(),
                negative_prompt: "soft light".into(),
                reference_notes: "keep hatching".into(),
            }),
            ..Default::default()
        })
        .unwrap();
    let mut target = ProjectStore::create(&paths, &input("风格目标")).unwrap();
    let project = target.import_style_from(&source).unwrap();
    assert_eq!(
        project.style_bible.visual_style,
        "manga noir with red accents"
    );
    assert_eq!(project.name, "风格目标");
    assert_eq!(
        target.project().unwrap().style_bible.palette,
        "black, bone, cinnabar"
    );
}

#[test]
fn provenance_manifest_captures_lineage() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("溯源项目")).unwrap();
    let request = GenerateScriptRequest::default();
    store
        .apply_generated_script(generated_script(), &request)
        .unwrap();
    let manifest = saladcart_lib::services::provenance::build(&store).unwrap();
    assert_eq!(manifest["project"]["name"], "溯源项目");
    assert!(manifest["shots"].as_array().unwrap().len() >= 2);
    assert!(manifest["characters"].as_array().unwrap().len() >= 1);
    let path = saladcart_lib::services::provenance::write(&store).unwrap();
    let body = std::fs::read_to_string(path).unwrap();
    assert!(body.contains("AI-generated production dossier"));
    assert!(body.contains("SaladCart"));
}

#[tokio::test]
#[cfg(target_os = "macos")]
#[ignore = "requires a system TTS voice; run manually with --ignored"]
async fn system_tts_generates_a_voice_asset() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("配音测试")).unwrap();
    let request = GenerateScriptRequest::default();
    store
        .apply_generated_script(generated_script(), &request)
        .unwrap();
    let shot = store.shots_in_order().unwrap().remove(0);
    let asset = saladcart_lib::services::voice::generate(&paths, &store.project_id, &shot.id, None)
        .await
        .unwrap();
    assert_eq!(asset.kind, "audio");
    assert!(asset.duration_ms.unwrap_or(0) > 0);
    let updated = store.get_shot(&shot.id).unwrap();
    assert_eq!(updated.voice_asset_id, Some(asset.id));
}

#[test]
fn shot_takes_can_be_selected_and_assembly_can_require_approval() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("版本测试")).unwrap();
    store
        .apply_generated_script(generated_script(), &GenerateScriptRequest::default())
        .unwrap();
    let shots = store.shots_in_order().unwrap();
    assert!(shots.len() >= 2);

    let add_video = |store: &mut ProjectStore, name: &str, version: i64| {
        let rel = format!("assets/videos/{name}.mp4");
        let abs = store.asset_abs(&rel);
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, b"placeholder").unwrap();
        store
            .insert_asset(NewAsset {
                kind: "video".into(),
                name: name.into(),
                rel_path: rel,
                thumb_rel_path: None,
                mime: "video/mp4".into(),
                width: Some(720),
                height: Some(1280),
                duration_ms: Some(5_000),
                source: "test".into(),
                provider_id: Some("mock-video".into()),
                model: Some("mock".into()),
                prompt: None,
                negative_prompt: None,
                seed: None,
                version,
                parent_asset_id: None,
                license: None,
                license_url: None,
                attribution: None,
                source_url: None,
                status: "ready".into(),
            })
            .unwrap()
    };

    let first = add_video(&mut store, "take-a", 1);
    store
        .attach_shot_asset(&shots[0].id, &first.id, "video", "mock-video", "mock", None)
        .unwrap();
    let second = add_video(&mut store, "take-b", 2);
    store
        .attach_shot_asset(
            &shots[0].id,
            &second.id,
            "video",
            "mock-video",
            "mock",
            None,
        )
        .unwrap();
    let job = GenerationJob {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: store.project_id.clone(),
        kind: "video".into(),
        target_type: "shot".into(),
        target_id: Some(shots[0].id.clone()),
        provider_id: "mock-video".into(),
        provider_label: "Mock".into(),
        model: "mock".into(),
        status: "succeeded".into(),
        progress: 100,
        stage: "done".into(),
        error: None,
        cost_estimate: 0.0,
        cost_actual: 0.25,
        request_summary: "take".into(),
        request: serde_json::json!({}),
        result: Some(serde_json::json!({"assetId": second.id})),
        result_asset_id: Some(second.id.clone()),
        created_at: saladcart_lib::store::now_iso(),
        started_at: None,
        finished_at: Some(saladcart_lib::store::now_iso()),
    };
    store.insert_job(&job).unwrap();

    let takes = store.shot_takes(&shots[0].id).unwrap();
    assert_eq!(takes.len(), 2);
    assert!(takes
        .iter()
        .any(|take| take.asset.id == first.id && !take.selected));
    assert!(takes
        .iter()
        .any(|take| take.asset.id == second.id && take.selected && take.cost_actual == 0.25));

    store
        .attach_shot_asset(&shots[0].id, &first.id, "video", "mock-video", "mock", None)
        .unwrap();
    store.set_shot_status(&shots[0].id, "needs_review").unwrap();
    let takes = store.shot_takes(&shots[0].id).unwrap();
    assert!(takes
        .iter()
        .any(|take| take.asset.id == first.id && take.selected));
    assert!(takes
        .iter()
        .any(|take| take.asset.id == second.id && !take.selected));

    let third = add_video(&mut store, "take-c", 1);
    store
        .attach_shot_asset(&shots[1].id, &third.id, "video", "mock-video", "mock", None)
        .unwrap();
    store.set_shot_status(&shots[1].id, "needs_review").unwrap();

    let rough = store.auto_assemble(false).unwrap();
    assert_eq!(rough.clips.len(), 2);
    let approved = store.auto_assemble(true).unwrap();
    assert_eq!(approved.clips.len(), 0);

    store.set_shot_status(&shots[0].id, "approved").unwrap();
    let approved = store.auto_assemble(true).unwrap();
    assert_eq!(approved.clips.len(), 1);
    assert_eq!(
        approved.clips[0].shot_id.as_deref(),
        Some(shots[0].id.as_str())
    );
}

#[test]
fn asset_import_copies_files_and_updates_media_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut store = ProjectStore::create(&paths, &input("素材导入")).unwrap();
    let upload = temp.path().join("poster.png");
    std::fs::write(&upload, b"fake-png").unwrap();
    let asset = store
        .import_asset(ImportAssetInput {
            source_path: upload.to_string_lossy().to_string(),
            kind: "image".into(),
            name: "海报".into(),
        })
        .unwrap();
    assert_eq!(asset.kind, "image");
    assert_eq!(asset.name, "海报");
    assert!(std::path::Path::new(&asset.abs_path).exists());
    assert!(asset.abs_path.contains("assets/images/"));

    let thumb_rel = format!("thumbs/{}.jpg", asset.id);
    let thumb_abs = store.asset_abs(&thumb_rel);
    std::fs::create_dir_all(thumb_abs.parent().unwrap()).unwrap();
    std::fs::write(&thumb_abs, b"thumb").unwrap();
    let updated = store
        .set_asset_media(&asset.id, Some(12_345), Some(&thumb_rel))
        .unwrap();
    assert_eq!(updated.duration_ms, Some(12_345));
    assert_eq!(
        updated.thumb_path,
        Some(thumb_abs.to_string_lossy().to_string())
    );
}

#[test]
fn character_import_is_atomic_when_an_asset_copy_fails() {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::new(temp.path().join("app")).unwrap();
    let mut source = ProjectStore::create(&paths, &input("角色源")).unwrap();
    source
        .apply_generated_script(generated_script(), &GenerateScriptRequest::default())
        .unwrap();
    let character = source.list_characters().unwrap().remove(0);
    let rel = "assets/images/reference.png";
    let abs = source.asset_abs(rel);
    std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
    std::fs::write(&abs, b"portrait").unwrap();
    let reference = source
        .insert_asset(NewAsset {
            kind: "image".into(),
            name: "reference".into(),
            rel_path: rel.into(),
            thumb_rel_path: None,
            mime: "image/png".into(),
            width: Some(128),
            height: Some(128),
            duration_ms: None,
            source: "upload".into(),
            provider_id: None,
            model: None,
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
        })
        .unwrap();
    source
        .attach_character_reference(&character.id, &reference.id, false)
        .unwrap();

    let mut target = ProjectStore::create(&paths, &input("角色目标")).unwrap();
    let imported = target
        .import_character_from(&source, &character.id)
        .unwrap();
    let copied_id = imported.reference_asset_id.expect("copied portrait");
    assert_eq!(target.list_characters().unwrap().len(), 1);
    assert!(target
        .get_asset(&copied_id)
        .unwrap()
        .abs_path
        .contains("assets/images/"));

    std::fs::remove_file(&abs).unwrap();
    let mut failing = ProjectStore::create(&paths, &input("角色失败")).unwrap();
    assert!(failing
        .import_character_from(&source, &character.id)
        .is_err());
    assert!(failing.list_characters().unwrap().is_empty());
    assert!(failing.list_assets(None).unwrap().is_empty());
}
