use saladcart_lib::models::*;
use saladcart_lib::paths::AppPaths;
use saladcart_lib::secrets;
use saladcart_lib::services::script;
use saladcart_lib::settings;
use saladcart_lib::store::ProjectStore;

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
