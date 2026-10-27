use saladcart_lib::models::{ProviderCapabilities, ProviderProfile};
use saladcart_lib::providers::image::{self, ImageRequest};
use saladcart_lib::providers::video::{self, VideoRequest};
use serde_json::json;
use wiremock::matchers::{header, method, path};
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
