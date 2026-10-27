use saladcart_lib::providers::image::ImageRequest;
use saladcart_lib::providers::mock;
use saladcart_lib::services::media::{Ffmpeg, RenderClip, RenderPlan, VoiceClip};
use saladcart_lib::settings;

#[tokio::test]
async fn ffmpeg_renders_a_real_mp4() {
    let Some(ffmpeg) = Ffmpeg::discover(&settings::default_settings()) else {
        eprintln!("skipping: FFmpeg not available");
        return;
    };
    let temp = tempfile::tempdir().unwrap();
    let mut clips = Vec::new();
    for (index, color) in ["green", "red"].iter().enumerate() {
        let image = mock::image(&ImageRequest {
            model: "mock".into(),
            prompt: format!("{color} test frame"),
            negative_prompt: "".into(),
            aspect_ratio: "9:16".into(),
            width: 360,
            height: 640,
            count: 1,
            seed: Some(index as i64),
            references: vec![],
            title: format!("TEST {index}"),
        })
        .unwrap();
        let image_path = temp.path().join(format!("frame-{index}.png"));
        std::fs::write(&image_path, image.bytes).unwrap();
        let video_path = temp.path().join(format!("clip-{index}.mp4"));
        ffmpeg
            .write_placeholder_video(
                &std::fs::read(&image_path).unwrap(),
                &video_path,
                360,
                640,
                1_200,
            )
            .await
            .unwrap();
        clips.push(RenderClip {
            path: video_path,
            is_image: false,
            in_ms: 0,
            out_ms: 1_200,
        });
    }
    let output = temp.path().join("export.mp4");
    let plan = RenderPlan {
        clips,
        output: output.clone(),
        width: 360,
        height: 640,
        fps: 24,
        subtitles: None,
        music: None,
        music_volume: 0.8,
        voices: vec![],
        metadata: vec![("title".into(), "SaladCart test".into())],
    };
    let result = ffmpeg.render(&plan, |_| {}).await.unwrap();
    assert!(output.exists());
    assert!(std::fs::metadata(&output).unwrap().len() > 2_000);
    assert!(result.duration_ms >= 2_000);

    // ffprobe is intentionally not bundled; the bundled-FFmpeg path must still
    // recover duration and dimensions for metadata.
    let no_probe = Ffmpeg {
        program: ffmpeg.program.clone(),
        ffprobe: None,
    };
    let clip = temp.path().join("clip-0.mp4");
    let probe = no_probe.probe(&clip);
    assert_eq!(probe.width, Some(360));
    assert_eq!(probe.height, Some(640));
    assert!(probe.duration_ms.unwrap_or(0) >= 1_000);

    let voice = mock::music("voice test", 2_000, Some(4)).unwrap();
    let voice_path = temp.path().join("voice.wav");
    std::fs::write(&voice_path, voice.bytes).unwrap();
    let voice_output = temp.path().join("export-with-voice.mp4");
    let voice_plan = RenderPlan {
        clips: plan.clips.clone(),
        output: voice_output.clone(),
        width: 360,
        height: 640,
        fps: 24,
        subtitles: None,
        music: None,
        music_volume: 0.8,
        voices: vec![VoiceClip {
            path: voice_path,
            start_ms: 300,
        }],
        metadata: vec![],
    };
    ffmpeg.render(&voice_plan, |_| {}).await.unwrap();
    assert!(voice_output.exists());
    assert!(std::fs::metadata(&voice_output).unwrap().len() > 2_000);
}
