use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::store::ProjectStore;
use image::ImageFormat;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone)]
pub struct Ffmpeg {
    pub program: PathBuf,
    pub ffprobe: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct MediaProbe {
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration_ms: Option<i64>,
    pub has_audio: bool,
}

#[derive(Debug, Clone)]
pub struct RenderClip {
    pub path: PathBuf,
    pub is_image: bool,
    pub in_ms: i64,
    pub out_ms: i64,
    pub has_audio: bool,
    pub muted: bool,
}

#[derive(Debug, Clone)]
pub struct VoiceClip {
    pub path: PathBuf,
    pub start_ms: i64,
}

#[derive(Debug, Clone)]
pub struct RenderPlan {
    pub clips: Vec<RenderClip>,
    pub output: PathBuf,
    pub width: i64,
    pub height: i64,
    pub fps: i64,
    pub subtitles: Option<String>,
    pub music: Option<PathBuf>,
    pub music_volume: f64,
    pub voices: Vec<VoiceClip>,
    pub metadata: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct RenderResult {
    pub path: String,
    pub duration_ms: i64,
    pub width: i64,
    pub height: i64,
    pub size_bytes: i64,
}

impl Ffmpeg {
    pub fn discover(settings: &Settings) -> Option<Self> {
        let ffmpeg = settings
            .ffmpeg_path
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| {
                std::env::var("SALADCART_FFMPEG")
                    .ok()
                    .map(PathBuf::from)
                    .filter(|p| p.is_file())
            })
            .or_else(|| bundled_binary("ffmpeg"))
            .or_else(|| which("ffmpeg"))?;
        let ffprobe = {
            let sibling = ffmpeg.with_file_name(if cfg!(windows) {
                "ffprobe.exe"
            } else {
                "ffprobe"
            });
            if sibling.is_file() {
                Some(sibling)
            } else {
                bundled_binary("ffprobe").or_else(|| which("ffprobe"))
            }
        };
        Some(Self {
            program: ffmpeg,
            ffprobe,
        })
    }

    pub fn version(&self) -> Option<String> {
        let output = std::process::Command::new(&self.program)
            .arg("-version")
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        text.lines().next().map(str::to_string)
    }

    pub fn probe(&self, path: &Path) -> MediaProbe {
        let Some(ffprobe) = &self.ffprobe else {
            return self.probe_with_ffmpeg(path);
        };
        let output = std::process::Command::new(ffprobe)
            .args([
                "-v",
                "error",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(path)
            .output();
        let Ok(output) = output else {
            return MediaProbe::default();
        };
        if !output.status.success() {
            return MediaProbe::default();
        }
        let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) else {
            return MediaProbe::default();
        };
        let duration_ms = value
            .pointer("/format/duration")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<f64>().ok())
            .map(|s| (s * 1000.0) as i64)
            .or_else(|| {
                value
                    .get("streams")
                    .and_then(|v| v.as_array())
                    .and_then(|streams| {
                        streams
                            .iter()
                            .find_map(|s| s.get("duration").and_then(|d| d.as_str()))
                    })
                    .and_then(|d| d.parse::<f64>().ok())
                    .map(|s| (s * 1000.0) as i64)
            });
        let video = value
            .get("streams")
            .and_then(|v| v.as_array())
            .and_then(|streams| {
                streams
                    .iter()
                    .find(|s| s.get("codec_type").and_then(|c| c.as_str()) == Some("video"))
            });
        let has_audio = value
            .get("streams")
            .and_then(|v| v.as_array())
            .map(|streams| {
                streams
                    .iter()
                    .any(|s| s.get("codec_type").and_then(|c| c.as_str()) == Some("audio"))
            })
            .unwrap_or(false);
        MediaProbe {
            width: video.and_then(|s| s.get("width")).and_then(|v| v.as_i64()),
            height: video.and_then(|s| s.get("height")).and_then(|v| v.as_i64()),
            duration_ms,
            has_audio,
        }
    }

    fn probe_with_ffmpeg(&self, path: &Path) -> MediaProbe {
        let output = std::process::Command::new(&self.program)
            .args(["-hide_banner", "-i"])
            .arg(path)
            .output();
        let Ok(output) = output else {
            return MediaProbe::default();
        };
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let duration_ms = text.find("Duration: ").and_then(|start| {
            let value = text.get(start + 10..start + 21)?;
            let mut parts = value.trim().split(':');
            let hours: f64 = parts.next()?.trim().parse().ok()?;
            let minutes: f64 = parts.next()?.trim().parse().ok()?;
            let seconds: f64 = parts.next()?.trim().parse().ok()?;
            Some(((hours * 3600.0 + minutes * 60.0 + seconds) * 1000.0) as i64)
        });
        let mut width = None;
        let mut height = None;
        for token in text.split_whitespace() {
            let token = token.trim_matches(|c: char| !c.is_ascii_digit() && c != 'x');
            if let Some((w, h)) = token.split_once('x') {
                if let (Ok(w), Ok(h)) = (w.parse::<i64>(), h.parse::<i64>()) {
                    if w >= 16 && h >= 16 {
                        width = Some(w);
                        height = Some(h);
                        break;
                    }
                }
            }
        }
        MediaProbe {
            width,
            height,
            duration_ms,
            has_audio: text.contains("Audio:"),
        }
    }

    pub async fn extract_frame(&self, video: &Path, at_ms: i64, output: &Path) -> AppResult<()> {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let seconds = (at_ms.max(0) as f64 / 1000.0).to_string();
        let status = tokio::process::Command::new(&self.program)
            .args(["-y", "-ss", &seconds, "-i"])
            .arg(video)
            .args(["-frames:v", "1", "-q:v", "2"])
            .arg(output)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;
        if !status.success() || !output.exists() {
            return Err(AppError::Message("could not extract a video frame".into()));
        }
        Ok(())
    }

    pub async fn write_placeholder_video(
        &self,
        png_bytes: &[u8],
        output: &Path,
        width: i64,
        height: i64,
        duration_ms: i64,
    ) -> AppResult<()> {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let frame_path = output.with_extension("placeholder.png");
        std::fs::write(&frame_path, png_bytes)?;
        let seconds = (duration_ms.max(1000) as f64 / 1000.0).to_string();
        let scale = format!(
            "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2,format=yuv420p"
        );
        let status = tokio::process::Command::new(&self.program)
            .args(["-y", "-loop", "1", "-i"])
            .arg(&frame_path)
            .args(["-t", &seconds, "-vf", &scale, "-r", "30"])
            .args([
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-crf",
                "22",
                "-movflags",
                "+faststart",
                "-an",
            ])
            .arg(output)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await?;
        let _ = std::fs::remove_file(&frame_path);
        if !status.status.success() {
            let err = String::from_utf8_lossy(&status.stderr);
            return Err(AppError::Message(format!(
                "FFmpeg could not render the placeholder video: {}",
                err.chars().take(400).collect::<String>()
            )));
        }
        Ok(())
    }

    pub async fn decode_mono_pcm(&self, audio: &Path, sample_rate: u32) -> AppResult<Vec<f32>> {
        let output = tokio::process::Command::new(&self.program)
            .args(["-v", "error", "-i"])
            .arg(audio)
            .args([
                "-ac",
                "1",
                "-ar",
                &sample_rate.to_string(),
                "-t",
                "600",
                "-f",
                "f32le",
                "pipe:1",
            ])
            .output()
            .await?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Message(format!(
                "could not decode audio: {}",
                err.chars().take(300).collect::<String>()
            )));
        }
        let mut samples = Vec::with_capacity(output.stdout.len() / 4);
        for chunk in output.stdout.chunks_exact(4) {
            samples.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Ok(samples)
    }

    pub async fn render(
        &self,
        plan: &RenderPlan,
        progress: impl Fn(i64) + Send + Sync,
    ) -> AppResult<RenderResult> {
        if plan.clips.is_empty() {
            return Err(AppError::Invalid(
                "the timeline has no clips to export".into(),
            ));
        }
        if let Some(parent) = plan.output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let total_ms: i64 = plan.clips.iter().map(|c| (c.out_ms - c.in_ms).max(1)).sum();
        let mut args: Vec<String> = vec!["-y".into()];
        for clip in &plan.clips {
            if clip.is_image {
                args.push("-loop".into());
                args.push("1".into());
                args.push("-t".into());
                args.push(format!("{:.3}", (clip.out_ms - clip.in_ms) as f64 / 1000.0));
            }
            args.push("-i".into());
            args.push(clip.path.to_string_lossy().to_string());
        }
        let music_index = plan.clips.len();
        if let Some(music) = &plan.music {
            args.push("-stream_loop".into());
            args.push("-1".into());
            args.push("-i".into());
            args.push(music.to_string_lossy().to_string());
        } else {
            args.push("-f".into());
            args.push("lavfi".into());
            args.push("-i".into());
            args.push("anullsrc=r=48000:cl=stereo".into());
        }
        for voice in &plan.voices {
            args.push("-i".into());
            args.push(voice.path.to_string_lossy().to_string());
        }

        let mut filters = Vec::new();
        for (index, clip) in plan.clips.iter().enumerate() {
            let start = clip.in_ms.max(0) as f64 / 1000.0;
            let duration = (clip.out_ms - clip.in_ms).max(1) as f64 / 1000.0;
            if clip.is_image {
                filters.push(format!(
                    "[{index}:v]trim=start=0:end={duration:.3},setpts=PTS-STARTPTS,scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps={}[v{index}]",
                    plan.width, plan.height, plan.width, plan.height, plan.fps
                ));
            } else {
                filters.push(format!(
                    "[{index}:v]trim=start={start:.3}:end={:.3},setpts=PTS-STARTPTS,scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps={}[v{index}]",
                    start + duration,
                    plan.width,
                    plan.height,
                    plan.width,
                    plan.height,
                    plan.fps
                ));
            }
        }
        let concat_inputs = (0..plan.clips.len())
            .map(|i| format!("[v{i}]"))
            .collect::<String>();
        filters.push(format!(
            "{concat_inputs}concat=n={}:v=1:a=0[vcat]",
            plan.clips.len()
        ));

        let mut audio_inputs = String::new();
        for (index, clip) in plan.clips.iter().enumerate() {
            let start = clip.in_ms.max(0) as f64 / 1000.0;
            let duration = (clip.out_ms - clip.in_ms).max(1) as f64 / 1000.0;
            if clip.has_audio {
                let volume = if clip.muted { 0.0 } else { 1.0 };
                filters.push(format!(
                    "[{index}:a]atrim=start={start:.3}:end={:.3},asetpts=PTS-STARTPTS,aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,apad,atrim=0:{duration:.3},volume={volume:.2}[a{index}]",
                    start + duration
                ));
            } else {
                filters.push(format!(
                    "anullsrc=r=48000:cl=stereo,atrim=0:{duration:.3},asetpts=PTS-STARTPTS[a{index}]"
                ));
            }
            audio_inputs.push_str(&format!("[a{index}]"));
        }
        filters.push(format!(
            "{audio_inputs}concat=n={}:v=0:a=1[acat]",
            plan.clips.len()
        ));

        let font = if cfg!(target_os = "macos") {
            "PingFang SC"
        } else if cfg!(windows) {
            "Microsoft YaHei"
        } else {
            "DejaVu Sans"
        };
        // libass lays SRT out on a 288-line play resolution and scales
        // FontSize/MarginV by video_height / 288. Compute the style in those
        // units so 4.2% / 7.5% of the frame stays consistent at every size.
        let ass_play_res_y: f64 = 288.0;
        let font_size = ((ass_play_res_y * 0.042).round() as i64).max(10);
        let margin_v = ((ass_play_res_y * 0.075).round() as i64).max(10);
        if let Some(srt) = &plan.subtitles {
            let escaped = escape_filter_path(Path::new(srt));
            filters.push(format!(
                "[vcat]subtitles=filename='{}':force_style='FontName={},FontSize={},PrimaryColour=&H00FFFFFF,OutlineColour=&H00000000,BorderStyle=1,Outline=2,Shadow=0,Alignment=2,MarginV={}'[vout]",
                escaped, font, font_size, margin_v
            ));
        } else {
            filters.push("[vcat]null[vout]".into());
        }
        let mix_count = 2 + plan.voices.len();
        // amix normalises by input count on every FFmpeg we ship, but the
        // `normalize` option only exists from 4.4 (Windows bundles 4.1).
        // Pre-scale by N so the default averaging preserves intended levels.
        filters.push(format!("[acat]volume={mix_count}[acatmix]"));
        filters.push(format!(
            "[{music_index}:a]volume={:.2}[music]",
            (plan.music_volume.clamp(0.0, 2.0) * mix_count as f64).min(6.0)
        ));
        let mut mix_labels = vec!["[acatmix]".to_string(), "[music]".to_string()];
        for (index, voice) in plan.voices.iter().enumerate() {
            let input = music_index + 1 + index;
            let delay = voice.start_ms.max(0);
            filters.push(format!(
                "[{input}:a]adelay={delay}|{delay},volume={}[voice{index}]",
                mix_count
            ));
            mix_labels.push(format!("[voice{index}]"));
        }
        if mix_labels.len() == 1 {
            filters.push("[music]anull[aout]".into());
        } else {
            filters.push(format!(
                "{}amix=inputs={}:duration=longest:dropout_transition=0[aout]",
                mix_labels.concat(),
                mix_labels.len()
            ));
        }
        args.push("-filter_complex".into());
        args.push(filters.join(";"));
        args.push("-map".into());
        args.push("[vout]".into());
        args.push("-map".into());
        args.push("[aout]".into());
        args.extend(
            [
                "-c:v",
                "libx264",
                "-preset",
                "medium",
                "-crf",
                "20",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
                "-shortest",
                "-movflags",
                "+faststart",
                "-progress",
                "pipe:1",
                "-nostats",
                "-threads",
                "4",
                "-preset",
                "veryfast",
            ]
            .into_iter()
            .map(str::to_string),
        );
        for (key, value) in &plan.metadata {
            args.push("-metadata".into());
            args.push(format!("{key}={value}"));
        }
        // Infinite silent/music inputs plus filter_complex can keep FFmpeg alive
        // longer than the visual timeline; cap the muxer to the intended length.
        args.push("-t".into());
        args.push(format!("{:.3}", total_ms as f64 / 1000.0));
        args.push(plan.output.to_string_lossy().to_string());

        progress(4);
        let mut child = tokio::process::Command::new(&self.program)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::Message("ffmpeg pipe unavailable".into()))?;
        let stderr = child.stderr.take();
        let stderr_task = tokio::spawn(async move {
            let mut buffer = String::new();
            if let Some(stderr) = stderr {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    buffer.push_str(&line);
                    buffer.push('\n');
                    if buffer.len() > 12_000 {
                        buffer.drain(0..4_000);
                    }
                }
            }
            buffer
        });
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Some(value) = line.strip_prefix("out_time_ms=") {
                if let Ok(ms) = value.parse::<i64>() {
                    let pct = ((ms as f64 / total_ms.max(1) as f64) * 90.0) as i64 + 5;
                    progress(pct.clamp(5, 95));
                }
            } else if line.starts_with("progress=end") {
                progress(96);
            }
        }
        let status = child.wait().await?;
        let stderr = stderr_task.await.unwrap_or_default();
        if !status.success() {
            return Err(AppError::Message(format!(
                "FFmpeg export failed: {}",
                stderr
                    .chars()
                    .rev()
                    .take(900)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
            )));
        }
        let probe = self.probe(&plan.output);
        let size = std::fs::metadata(&plan.output)
            .map(|m| m.len() as i64)
            .unwrap_or(0);
        progress(100);
        Ok(RenderResult {
            path: plan.output.to_string_lossy().to_string(),
            duration_ms: probe.duration_ms.unwrap_or(total_ms),
            width: probe.width.unwrap_or(plan.width),
            height: probe.height.unwrap_or(plan.height),
            size_bytes: size,
        })
    }
}

pub fn build_render_plan(
    store: &ProjectStore,
    settings: &Settings,
    options: &ExportOptions,
) -> AppResult<RenderPlan> {
    let timeline = store.timeline()?;
    let probe_tool = Ffmpeg::discover(settings);
    let mut clips = Vec::new();
    for clip in &timeline.clips {
        let Some(asset_id) = &clip.asset_id else {
            continue;
        };
        let asset = store.get_asset(asset_id)?;
        let path = PathBuf::from(&asset.abs_path);
        if !path.exists() {
            continue;
        }
        let probe = if asset.kind == "image" {
            MediaProbe::default()
        } else {
            probe_tool
                .as_ref()
                .map(|ffmpeg| ffmpeg.probe(&path))
                .unwrap_or_default()
        };
        clips.push(RenderClip {
            path,
            is_image: asset.kind == "image",
            in_ms: clip.in_ms,
            out_ms: clip.out_ms,
            has_audio: probe.has_audio,
            muted: clip.muted,
        });
    }
    if clips.is_empty() {
        return Err(AppError::Invalid(
            "no rendered clips. Generate or import videos, then auto-assemble the timeline.".into(),
        ));
    }
    let file = crate::providers::sanitize_filename(&options.filename, "saladcart-export");
    let file = if file.to_lowercase().ends_with(".mp4") {
        file
    } else {
        format!("{file}.mp4")
    };
    let export_dir = settings
        .export_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(crate::paths::expand_home)
        .unwrap_or_else(crate::paths::default_export_dir);
    std::fs::create_dir_all(&export_dir).map_err(|error| {
        AppError::Message(format!(
            "无法创建导出目录 {}：{error}",
            export_dir.display()
        ))
    })?;
    let output = export_dir.join(file);
    let subtitles = if options.burn_subtitles && !timeline.subtitles.is_empty() {
        let path = store.write_srt()?;
        Some(path.to_string_lossy().to_string())
    } else {
        None
    };
    let music = if let Some(id) = &options.music_asset_id {
        let asset = store.get_asset(id)?;
        if asset.kind == "audio" {
            let path = PathBuf::from(&asset.abs_path);
            if path.exists() {
                Some(path)
            } else {
                None
            }
        } else {
            None
        }
    } else if let Some(id) = &timeline.music_asset_id {
        let asset = store.get_asset(id)?;
        if asset.kind == "audio" && Path::new(&asset.abs_path).exists() {
            Some(PathBuf::from(asset.abs_path))
        } else {
            None
        }
    } else {
        None
    };
    let _ = settings;
    let project = store.project()?;
    let mut voices = Vec::new();
    let mut cursor = 0_i64;
    for clip in &timeline.clips {
        // Keep the voice cursor aligned with clips the renderer actually emits,
        // otherwise a missing source file shifts every later line.
        let Some(asset_id) = &clip.asset_id else {
            continue;
        };
        let Ok(asset) = store.get_asset(asset_id) else {
            continue;
        };
        if !Path::new(&asset.abs_path).exists() {
            continue;
        }
        if let Some(shot_id) = &clip.shot_id {
            if let Ok(shot) = store.get_shot(shot_id) {
                if let Some(voice_asset_id) = &shot.voice_asset_id {
                    if let Ok(asset) = store.get_asset(voice_asset_id) {
                        let path = PathBuf::from(&asset.abs_path);
                        if path.exists() {
                            voices.push(VoiceClip {
                                path,
                                start_ms: cursor,
                            });
                        }
                    }
                }
            }
        }
        cursor += clip.duration_ms;
    }
    Ok(RenderPlan {
        clips,
        output,
        width: options.width.max(240),
        height: options.height.max(240),
        fps: options.fps.clamp(12, 60),
        subtitles,
        music,
        music_volume: timeline.music_volume.clamp(0.0, 2.0),
        voices,
        metadata: vec![
            ("title".into(), project.name),
            (
                "comment".into(),
                "AI-generated content produced with SaladCart. Provenance manifest: provenance.json".into(),
            ),
        ],
    })
}

pub fn image_thumbnail(src: &Path, dest: &Path, max_edge: u32) -> AppResult<()> {
    let image = image::open(src)
        .map_err(|e| AppError::Message(format!("could not open image for thumbnail: {e}")))?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let thumb = image.thumbnail(max_edge, max_edge);
    thumb
        .save_with_format(dest, ImageFormat::Jpeg)
        .map_err(|e| AppError::Message(format!("could not save thumbnail: {e}")))?;
    Ok(())
}

pub fn detect_video_extension(path: &Path) -> &'static str {
    use std::io::Read;
    let mut header = [0u8; 16];
    if let Ok(mut file) = std::fs::File::open(path) {
        if file.read_exact(&mut header).is_ok() {
            if &header[4..8] == b"ftyp" {
                return "mp4";
            }
            if header[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
                return "webm";
            }
            if &header[0..4] == b"RIFF" && &header[8..12] == b"AVI " {
                return "avi";
            }
        }
    }
    "mp4"
}

pub fn status(settings: &Settings) -> (bool, Option<String>, Option<String>) {
    match Ffmpeg::discover(settings) {
        Some(ffmpeg) => (
            true,
            Some(ffmpeg.program.to_string_lossy().to_string()),
            ffmpeg.version(),
        ),
        None => (false, None, None),
    }
}

fn bundled_binary(name: &str) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    let candidates = [dir.join(&file), dir.parent()?.join("Resources").join(&file)];
    candidates.into_iter().find(|path| path.is_file())
}

fn which(binary: &str) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    };
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    for candidate in [
        format!("/opt/homebrew/bin/{name}"),
        format!("/usr/local/bin/{name}"),
        format!("/usr/bin/{name}"),
        format!(r"C:\ffmpeg\bin\{name}"),
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn escape_filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace(':', "\\:")
        .replace('\'', "\\'")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub fn ensure_ffmpeg(settings: &Settings) -> AppResult<Ffmpeg> {
    Ffmpeg::discover(settings).ok_or(AppError::FfmpegUnavailable)
}
