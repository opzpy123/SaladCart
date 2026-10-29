use saladcart_lib::models::*;
use saladcart_lib::paths::AppPaths;
use saladcart_lib::secrets;
use saladcart_lib::services::{novel, script};
use saladcart_lib::settings;
use saladcart_lib::store::ProjectStore;
use serde_json::json;

// Run manually with:
//   cargo test --manifest-path src-tauri/Cargo.toml deepseek_generates_and_applies_a_real_script -- --ignored --nocapture
#[tokio::test]
#[ignore = "requires a DeepSeek key in the OS keychain and consumes API credits"]
async fn deepseek_generates_and_applies_a_real_script() {
    let profile = settings::default_settings()
        .providers
        .into_iter()
        .find(|provider| provider.id == "deepseek-llm")
        .expect("deepseek-llm is a built-in provider");
    let secret = secrets::get_secret("deepseek-llm")
        .or_else(|_| {
            std::env::var("DEEPSEEK_API_KEY")
                .map_err(|_| saladcart_lib::error::AppError::MissingCredential(
                    "DeepSeek key is not in the keychain and DEEPSEEK_API_KEY is not set".into(),
                ))
        })
        .expect("a DeepSeek API key is required for this live test");

    let dir = tempfile::tempdir().expect("temp dir");
    let paths = AppPaths::new(dir.path().to_path_buf()).expect("app paths");
    let mut store = ProjectStore::create(
        &paths,
        &CreateProjectInput {
            name: "DeepSeek 实时验证".into(),
            premise: "一个总在凌晨四点开门的便利店，店员发现每位顾客都会遗忘一件最重要的东西。".into(),
            genre: "都市奇幻".into(),
            tone: "克制、温柔".into(),
            language: "zh-CN".into(),
            aspect_ratio: "9:16".into(),
            target_episode_seconds: 60,
            budget_cap: 0.0,
            style_preset: Some("anime-cel".into()),
            style_bible: None,
        },
    )
    .expect("create project");
    let project = store.project().expect("project");

    let request = GenerateScriptRequest {
        premise: project.premise.clone(),
        genre: project.genre.clone(),
        tone: project.tone.clone(),
        language: project.language.clone(),
        episodes: 1,
        scenes_per_episode: 1,
        shots_per_scene: 2,
        style_notes: String::new(),
        provider_id: Some("deepseek-llm".into()),
        model: Some("deepseek-chat".into()),
        replace_existing: false,
        seed: None,
    };
    let generated = script::generate(&profile, Some(&secret), &request, &project)
        .await
        .expect("DeepSeek script generation");
    assert!(!generated.characters.is_empty());
    assert!(!generated.episodes.is_empty());
    assert!(!generated.episodes[0].scenes.is_empty());
    assert!(!generated.episodes[0].scenes[0].shots.is_empty());

    let stats = store
        .apply_generated_script(generated.clone(), &request)
        .expect("apply generated script");
    assert!(stats.characters_created > 0);
    assert!(stats.episodes > 0);
    assert!(stats.shots > 0);
    println!(
        "DeepSeek OK: title={:?}, characters={:?}, episodes={}, scenes={}, shots={}",
        generated.title,
        generated
            .characters
            .iter()
            .map(|character| character.name.as_str())
            .collect::<Vec<_>>(),
        stats.episodes,
        stats.scenes,
        stats.shots
    );
}

#[tokio::test]
#[ignore = "requires a DeepSeek key in the OS keychain and consumes API credits"]
async fn deepseek_adapts_a_short_novel() {
    let profile = settings::default_settings()
        .providers
        .into_iter()
        .find(|provider| provider.id == "deepseek-llm")
        .expect("deepseek-llm is a built-in provider");
    let secret = secrets::get_secret("deepseek-llm")
        .or_else(|_| {
            std::env::var("DEEPSEEK_API_KEY").map_err(|_| {
                saladcart_lib::error::AppError::MissingCredential(
                    "DeepSeek key is not in the keychain and DEEPSEEK_API_KEY is not set".into(),
                )
            })
        })
        .expect("a DeepSeek API key is required for this live test");

    let dir = tempfile::tempdir().expect("temp dir");
    let paths = AppPaths::new(dir.path().to_path_buf()).expect("app paths");
    let mut store = ProjectStore::create(
        &paths,
        &CreateProjectInput {
            name: "DeepSeek 小说改编验证".into(),
            premise: "深夜便利店的店员发现一本记录明天的失物招领簿。".into(),
            genre: "都市奇幻".into(),
            tone: "克制、温柔".into(),
            language: "zh-CN".into(),
            aspect_ratio: "9:16".into(),
            target_episode_seconds: 60,
            budget_cap: 0.0,
            style_preset: Some("anime-cel".into()),
            style_bible: None,
        },
    )
    .expect("create project");
    let project = store.project().expect("project");
    let text = "第一章 深夜便利店\n凌晨四点，林晚在便利店整理货架。一个穿雨衣的老人放下一枚硬币，说买走明天的记忆。林晚追出门，街道空无一人，硬币上刻着明天的日期。\n第二章 失物招领\n第二天，林晚在收银台下发现一本失物招领簿，第一页写着她的名字。她翻开，里面记录着她从未经历过的告别。";
    let source = NovelSource {
        summary: NovelSummary {
            id: "novel-live".into(),
            title: "失物招领簿".into(),
            source_path: None,
            char_count: text.chars().count() as i64,
            chapter_count: 2,
            chunk_count: 2,
            status: "ready".into(),
            analysis: json!({}),
            created_at: "2026-10-29T00:00:00Z".into(),
            updated_at: "2026-10-29T00:00:00Z".into(),
        },
        text: text.into(),
    };
    let request = NovelAdaptRequest {
        provider_id: Some("deepseek-llm".into()),
        model: Some("deepseek-chat".into()),
        max_episodes: 1,
        style_notes: String::new(),
        replace_existing: false,
    };
    let result = novel::adapt(
        &profile,
        Some(&secret),
        &project,
        &source,
        &request,
        &|_, _| Ok(()),
    )
    .await
    .expect("DeepSeek novel adaptation");
    assert!(!result.script.characters.is_empty());
    assert!(!result.script.episodes.is_empty());
    assert!(result.calls >= 1);
    println!(
        "DeepSeek novel OK: title={:?}, characters={:?}, episodes={}, chunks={}, calls={}",
        result.script.title,
        result
            .script
            .characters
            .iter()
            .map(|character| character.name.as_str())
            .collect::<Vec<_>>(),
        result.script.episodes.len(),
        result.chunk_count,
        result.calls
    );
}
