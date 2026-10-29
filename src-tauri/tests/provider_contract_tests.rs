use saladcart_lib::models::{
    NovelAdaptRequest, NovelSource, NovelSummary, Project, ProviderCapabilities, ProviderProfile,
    StyleBible,
};
use saladcart_lib::providers::image::{self, ImageRequest};
use saladcart_lib::providers::video::{self, VideoRequest};
use saladcart_lib::services::novel;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn profile(id: &str, kind: &str, api_base: String, model: &str) -> ProviderProfile {
    ProviderProfile {
        id: id.into(),
        kind: kind.into(),
        label: id.into(),
        vendor: id.into(),
        enabled: true,
        builtin: true,
        api_base,
        default_model: model.into(),
        models: vec![model.into()],
        api_key_set: true,
        env_var: String::new(),
        options: json!({}),
        capabilities: ProviderCapabilities::default(),
    }
}

#[tokio::test]
async fn openai_images_contract_parses_b64_output() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/images/generations"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"b64_json": "iVBORw0KGgo="}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = profile("openai-image", "image", server.uri(), "gpt-image-1");
    let result = image::generate(
        &provider,
        Some("test-key"),
        &ImageRequest {
            model: "gpt-image-1".into(),
            prompt: "rainy rooftop".into(),
            negative_prompt: "text".into(),
            aspect_ratio: "9:16".into(),
            width: 1024,
            height: 1536,
            count: 1,
            seed: Some(7),
            references: vec![],
            title: "contract".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(result.extension, "png");
    assert_eq!(result.bytes, b"\x89PNG\r\n\x1a\n");
}

#[tokio::test]
async fn kling_video_contract_submits_and_polls() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/videos/text2video"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0,
            "data": {"task_id": "task-1", "task_status": "submitted"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/videos/text2video/task-1"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "task_id": "task-1",
                "task_status": "succeed",
                "task_result": {"videos": [{"url": "https://cdn.example/v.mp4"}]}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = profile("kling-video", "video", server.uri(), "kling-v1");
    let request = VideoRequest {
        model: "kling-v1".into(),
        prompt: "rain".into(),
        negative_prompt: String::new(),
        aspect_ratio: "9:16".into(),
        duration_ms: 5_000,
        seed: None,
        resolution: None,
        start_frame: None,
        references: vec![],
        title: "contract".into(),
    };
    let task = video::submit(&provider, Some("test-key"), &request)
        .await
        .unwrap();
    assert_eq!(task.id, "task-1");
    assert_eq!(task.status, "pending");

    let done = video::poll(&provider, Some("test-key"), &task)
        .await
        .unwrap();
    assert_eq!(done.status, "succeeded");
    assert_eq!(
        done.output_url.as_deref(),
        Some("https://cdn.example/v.mp4")
    );
}

#[tokio::test]
async fn ark_image_contract_posts_seedream_and_downloads_url() {
    let server = MockServer::start().await;
    let image_url = format!("{}/x.png", server.uri());
    Mock::given(method("POST"))
        .and(path("/images/generations"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"url": image_url}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/x.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"\x89PNG\r\n\x1a\n"))
        .expect(1)
        .mount(&server)
        .await;

    let provider = profile("ark-image", "image", server.uri(), "doubao-seedream-5-0-flash-260915");
    let result = image::generate(
        &provider,
        Some("test-key"),
        &ImageRequest {
            model: "doubao-seedream-5-0-flash-260915".into(),
            prompt: "rainy rooftop".into(),
            negative_prompt: "text".into(),
            aspect_ratio: "9:16".into(),
            width: 1080,
            height: 1920,
            count: 1,
            seed: Some(7),
            references: vec![],
            title: "contract".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(result.extension, "png");
    assert_eq!(result.bytes, b"\x89PNG\r\n\x1a\n");
}

#[tokio::test]
async fn ark_video_contract_submits_and_polls() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/contents/generations/tasks"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "cgt-1",
            "status": "queued"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/contents/generations/tasks/cgt-1"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "cgt-1",
            "status": "succeeded",
            "content": {"video_url": "https://cdn.example/ark.mp4"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = profile("ark-video", "video", server.uri(), "doubao-seedance-2-0-mini-260615");
    let request = VideoRequest {
        model: "doubao-seedance-2-0-mini-260615".into(),
        prompt: "rain".into(),
        negative_prompt: String::new(),
        aspect_ratio: "9:16".into(),
        duration_ms: 5_000,
        seed: None,
        resolution: None,
        start_frame: None,
        references: vec![],
        title: "contract".into(),
    };
    let task = video::submit(&provider, Some("test-key"), &request)
        .await
        .unwrap();
    assert_eq!(task.id, "cgt-1");
    assert_eq!(task.status, "pending");

    let done = video::poll(&provider, Some("test-key"), &task)
        .await
        .unwrap();
    assert_eq!(done.status, "succeeded");
    assert_eq!(
        done.output_url.as_deref(),
        Some("https://cdn.example/ark.mp4")
    );
}

#[tokio::test]
async fn volcengine_music_contract_signs_submits_and_polls() {
    let server = MockServer::start().await;
    let audio_url = format!("{}/music.mp3", server.uri());
    Mock::given(method("POST"))
        .and(path("/"))
        .and(query_param("Action", "GenBGM"))
        .and(query_param("Version", "2024-08-12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "TaskId": "bgm-1",
            "Status": "Processing"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(query_param("Action", "QueryTask"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "TaskId": "bgm-1",
            "Status": "Success",
            "ResultUrl": audio_url
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/music.mp3"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "audio/mpeg")
                .set_body_bytes(b"ID3"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let provider = profile("volc-music", "music", server.uri(), "v5.0");
    let result = saladcart_lib::providers::music::generate(
        &provider,
        Some("ak:sk"),
        &saladcart_lib::providers::music::MusicRequest {
            model: "v5.0".into(),
            prompt: "warm night market, guzheng and synth".into(),
            duration_ms: 30_000,
            instrumental: true,
            seed: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(result.extension, "mp3");
    assert_eq!(result.bytes, b"ID3");
}

#[tokio::test]
async fn novel_llm_contract_turns_chunk_analyses_into_episodes() {
    let server = MockServer::start().await;
    let analysis = json!({
        "summary": "沈知遥在雨夜发现钥匙会回应她的记忆。",
        "characters": [{
            "name": "沈知遥",
            "role": "主角",
            "appearance": "黑色短发，深灰风衣",
            "personality": "克制",
            "lockedTraits": ["黑色短发", "深灰风衣"]
        }],
        "beats": [
            {
                "title": "钥匙震动",
                "summary": "她在积水里捡起钥匙，听见自己的声音。",
                "location": "旧站台",
                "timeOfDay": "NIGHT",
                "characters": ["沈知遥"],
                "dialogue": "沈知遥：它在叫我的名字。",
                "visual": "雨夜特写，铜钥匙在积水中震动",
                "camera": "slow push-in",
                "durationSec": 5
            },
            {
                "title": "旧店灯光",
                "summary": "阿箬关掉一半的灯，警告她不要再读。",
                "location": "旧物店",
                "timeOfDay": "NIGHT",
                "characters": ["沈知遥"],
                "dialogue": "阿箬：你每读一次，就少一段自己的。",
                "visual": "黄铜台灯下两人对峙",
                "camera": "over-the-shoulder",
                "durationSec": 6
            }
        ]
    });
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": analysis.to_string()}}]
        })))
        .expect(2)
        .mount(&server)
        .await;

    let provider = profile("openai-llm", "llm", server.uri(), "gpt-4.1-mini");
    let project = Project {
        id: "project".into(),
        name: "雨夜钥匙".into(),
        premise: "旧物记忆".into(),
        logline: String::new(),
        genre: "悬疑".into(),
        tone: "冷冽".into(),
        language: "zh-CN".into(),
        aspect_ratio: "9:16".into(),
        target_episode_seconds: 90,
        budget_cap: 0.0,
        style_bible: StyleBible {
            visual_style: "cinematic anime".into(),
            palette: "emerald".into(),
            lighting: "rain".into(),
            camera_language: "storyboard".into(),
            negative_prompt: "text".into(),
            reference_notes: String::new(),
        },
        created_at: "2026-10-28T00:00:00Z".into(),
        updated_at: "2026-10-28T00:00:00Z".into(),
    };
    let source = NovelSource {
        summary: NovelSummary {
            id: "current".into(),
            title: "雨夜钥匙".into(),
            source_path: None,
            char_count: 140,
            chapter_count: 2,
            chunk_count: 2,
            status: "ready".into(),
            analysis: json!({}),
            created_at: project.created_at.clone(),
            updated_at: project.updated_at.clone(),
        },
        text: "第一章 雨夜\n沈知遥在旧站台捡到铜钥匙。\n第二章 旧店\n阿箬警告她不要再读。".into(),
    };
    let result = novel::adapt(
        &provider,
        Some("test-key"),
        &project,
        &source,
        &NovelAdaptRequest::default(),
        &|_, _| Ok(()),
    )
    .await
    .unwrap();

    assert_eq!(result.analysis["mode"], "llm");
    assert_eq!(result.script.episodes.len(), 2);
    assert_eq!(result.script.characters.len(), 1);
    assert!(result
        .script
        .episodes
        .iter()
        .all(|episode| episode.scenes.len() == 2));
    assert_eq!(result.chunk_count, 2);
    assert_eq!(result.calls, 2);
}
