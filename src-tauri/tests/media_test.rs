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
            has_audio: false,
            muted: false,
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

#[tokio::test]
async fn ffmpeg_mutes_source_audio_in_the_mix() {
    let Some(ffmpeg) = Ffmpeg::discover(&settings::default_settings()) else {
        eprintln!("skipping: FFmpeg not available");
        return;
    };
    let temp = tempfile::tempdir().unwrap();
    let clip_path = temp.path().join("with-audio.mp4");
    let status = std::process::Command::new(&ffmpeg.program)
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=320x240:d=1.2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1.2",
            "-shortest",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
        ])
        .arg(&clip_path)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(ffmpeg.probe(&clip_path).has_audio);

    let output = temp.path().join("muted-export.mp4");
    let plan = RenderPlan {
        clips: vec![RenderClip {
            path: clip_path,
            is_image: false,
            in_ms: 0,
            out_ms: 1_200,
            has_audio: true,
            muted: true,
        }],
        output: output.clone(),
        width: 320,
        height: 240,
        fps: 24,
        subtitles: None,
        music: None,
        music_volume: 0.8,
        voices: vec![],
        metadata: vec![],
    };
    ffmpeg.render(&plan, |_| {}).await.unwrap();
    assert!(output.exists());
    let probe = ffmpeg.probe(&output);
    assert!(probe.has_audio);
    assert!(probe.duration_ms.unwrap_or(0) >= 1_000);
}
