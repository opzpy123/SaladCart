use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::paths::AppPaths;
use crate::providers::{self, music::sniff_audio_mime};
use crate::services::media;
use crate::settings;
use crate::store::assets::NewAsset;
use crate::store::ProjectStore;
use serde_json::Value;

const SAMPLE_RATE: u32 = 22_050;
const HOP: usize = 512;
const FRAME: usize = 1024;

pub async fn search(
    paths: &AppPaths,
    provider_id: &str,
    query: &str,
    limit: u32,
) -> AppResult<MusicSearchResult> {
    let settings = settings::load(paths);
    let profile = settings::find_provider(&settings, provider_id)?;
    if profile.id != "jamendo-music" {
        return Err(AppError::Invalid(
            "music search currently supports the Jamendo provider".into(),
        ));
    }
    let client_id = settings::resolve_secret(profile)?;
    let client = providers::client()?;
    let response = client
        .get(format!(
            "{}/tracks/",
            profile.api_base.trim_end_matches('/')
        ))
        .query(&[
            ("client_id", client_id.as_str()),
            ("format", "json"),
            ("limit", &limit.clamp(1, 30).to_string()),
            ("search", query),
            ("include", "licenses"),
            ("audioformat", "mp32"),
            ("order", "popularity_total"),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let value: Value = response.json().await?;
    let provider_label = profile.label.clone();
    let mut tracks = Vec::new();
    if let Some(results) = value.get("results").and_then(|v| v.as_array()) {
        for item in results {
            let download_allowed = item
                .get("audiodownload_allowed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let download_url = item
                .get("audiodownload")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !download_allowed || download_url.is_empty() {
                continue;
            }
            let preview_url = item
                .get("audio")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let license_url = item
                .get("license_ccurl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let license = license_label(&license_url);
            let tags = item
                .pointer("/musicinfo/tags/genres")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            tracks.push(MusicTrack {
                id: format!(
                    "jamendo-{}",
                    item.get("id").map(|v| v.to_string()).unwrap_or_default()
                ),
                title: item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string(),
                artist: item
                    .get("artist_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown artist")
                    .to_string(),
                provider_id: profile.id.clone(),
                provider_label: profile.label.clone(),
                preview_url,
                download_url,
                duration_ms: item
                    .get("duration")
                    .and_then(|v| v.as_i64())
                    .map(|s| s * 1000)
                    .unwrap_or(0),
                requires_attribution: !license.to_lowercase().contains("publicdomain"),
                license,
                license_url,
                tags,
                cover_url: item
                    .get("image")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            });
        }
    }
    Ok(MusicSearchResult {
        tracks,
        provider_label,
        query: query.to_string(),
        error: None,
    })
}

fn license_label(url: &str) -> String {
    if url.contains("publicdomain") {
        "CC0 / Public Domain".into()
    } else if url.contains("by-sa") {
        "CC BY-SA".into()
    } else if url.contains("by-nc") {
        "CC BY-NC".into()
    } else if url.contains("by-nd") {
        "CC BY-ND".into()
    } else if url.contains("/by/") {
        "CC BY".into()
    } else if url.is_empty() {
        "Unknown licence".into()
    } else {
        "Creative Commons".into()
    }
}

pub async fn import_track(
    paths: &AppPaths,
    project_id: &str,
    track: &MusicTrack,
) -> AppResult<Asset> {
    if track.download_url.trim().is_empty() {
        return Err(AppError::Invalid(
            "this track does not allow downloading for offline edits".into(),
        ));
    }
    if !track.download_url.starts_with("https://") && !track.download_url.starts_with("http://") {
        return Err(AppError::Invalid("music download URL must be HTTP(S)".into()));
    }
    let client = providers::client()?;
    let response = client.get(&track.download_url).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(providers::error_from_response(status, &body));
    }
    let bytes = response.bytes().await?;
    if bytes.len() > 80 * 1024 * 1024 {
        return Err(AppError::Invalid("music file is larger than 80 MB".into()));
    }
    let mime = sniff_audio_mime(&bytes).to_string();
    let extension = match mime.as_str() {
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        _ => "mp3",
    };
    let id = uuid::Uuid::new_v4().to_string();
    let rel = format!("assets/audio/{id}.{extension}");
    let mut store = ProjectStore::open(paths, project_id)?;
    let abs = store.asset_abs(&rel);
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, &bytes)?;
    let duration = media::Ffmpeg::discover(&settings::load(paths))
        .and_then(|ffmpeg| ffmpeg.probe(&abs).duration_ms)
        .or_else(|| if track.duration_ms > 0 { Some(track.duration_ms) } else { None });
    let asset = store.insert_asset(NewAsset {
        kind: "audio".into(),
        name: track.title.clone(),
        rel_path: rel,
        thumb_rel_path: None,
        mime,
        width: None,
        height: None,
        duration_ms: duration,
        source: "stock".into(),
        provider_id: Some(track.provider_id.clone()),
        model: Some("stock".into()),
        prompt: None,
        negative_prompt: None,
        seed: None,
        version: 1,
        parent_asset_id: None,
        license: Some(track.license.clone()),
        license_url: Some(track.license_url.clone()),
        attribution: Some(format!("{} — {}", track.title, track.artist)),
        source_url: Some(track.download_url.clone()),
        status: "needs_review".into(),
    })?;
    Ok(asset)
}

pub async fn analyze_beats(
    paths: &AppPaths,
    project_id: &str,
    settings: &Settings,
) -> AppResult<BeatAnalysis> {
    let (asset, duration) = {
        let store = ProjectStore::open(paths, project_id)?;
        let timeline = store.timeline()?;
        let asset_id = timeline
            .music_asset_id
            .ok_or_else(|| AppError::Invalid("attach a music track before analyzing beats".into()))?;
        let asset = store.get_asset(&asset_id)?;
        if asset.kind != "audio" {
            return Err(AppError::Invalid("timeline music is not an audio asset".into()));
        }
        let duration = asset.duration_ms.unwrap_or(0);
        (asset, duration)
    };
    let ffmpeg = media::ensure_ffmpeg(settings)?;
    let samples = ffmpeg
        .decode_mono_pcm(std::path::Path::new(&asset.abs_path), SAMPLE_RATE)
        .await?;
    if samples.len() < SAMPLE_RATE as usize {
        return Err(AppError::Invalid("music is too short to analyze".into()));
    }
    let duration_ms = if duration > 0 {
        duration
    } else {
        (samples.len() as i64 * 1000) / SAMPLE_RATE as i64
    };
    let analysis = analyze_pcm(&samples, SAMPLE_RATE, duration_ms);
    {
        let mut store = ProjectStore::open(paths, project_id)?;
        store.update_timeline(TimelinePatch {
            bpm: Some(analysis.bpm),
            beat_grid_ms: Some(analysis.beat_grid_ms.clone()),
            beat_offset_ms: Some(analysis.offset_ms),
            ..Default::default()
        })?;
    }
    Ok(BeatAnalysis {
        asset_id: asset.id,
        bpm: analysis.bpm,
        confidence: analysis.confidence,
        offset_ms: analysis.offset_ms,
        beat_grid_ms: analysis.beat_grid_ms,
        waveform: analysis.waveform,
        duration_ms,
    })
}

pub fn auto_beat_sync(store: &mut ProjectStore) -> AppResult<BeatSyncResult> {
    let timeline = store.timeline()?;
    if timeline.beat_grid_ms.len() < 2 {
        return Err(AppError::Invalid(
            "analyze the music track before auto-cutting to the beat".into(),
        ));
    }
    let beat_interval = timeline
        .bpm
        .filter(|bpm| *bpm > 20.0)
        .map(|bpm| 60_000.0 / bpm)
        .unwrap_or_else(|| {
            (timeline.beat_grid_ms[1] - timeline.beat_grid_ms[0]).max(1) as f64
        });
    let minimum = (beat_interval * 2.0).max(900.0) as i64;
    let mut cursor = 0_i64;
    let mut aligned = 0_usize;
    let mut offset_total = 0_i64;
    let mut inputs = Vec::new();
    for clip in &timeline.clips {
        let asset_duration = clip
            .asset_id
            .as_ref()
            .and_then(|id| store.get_asset(id).ok())
            .and_then(|asset| asset.duration_ms)
            .unwrap_or(clip.duration_ms);
        let desired_end = cursor + clip.duration_ms;
        let nearest = timeline
            .beat_grid_ms
            .iter()
            .copied()
            .filter(|beat| (*beat - desired_end).abs() <= 180)
            .min_by_key(|beat| (*beat - desired_end).abs());
        let mut end = desired_end;
        if let Some(beat) = nearest {
            let candidate_duration = beat - cursor;
            if candidate_duration >= minimum && clip.in_ms + candidate_duration <= asset_duration {
                end = beat;
                aligned += 1;
                offset_total += (beat - desired_end).abs();
            }
        }
        let in_ms = clip.in_ms.max(0);
        let out_ms = (end - cursor + in_ms).min(asset_duration).max(in_ms + 1);
        inputs.push(TimelineClipInput {
            shot_id: clip.shot_id.clone(),
            asset_id: clip.asset_id.clone(),
            in_ms,
            out_ms,
            label: clip.label.clone(),
            muted: clip.muted,
            transition: clip.transition.clone(),
        });
        cursor += out_ms - in_ms;
    }
    let updated = store.update_timeline(TimelinePatch {
        clips: Some(inputs),
        ..Default::default()
    })?;
    Ok(BeatSyncResult {
        aligned_cuts: aligned,
        total_cuts: updated.clips.len(),
        average_offset_ms: if aligned > 0 {
            (offset_total as f64 / aligned as f64).round() as i64
        } else {
            0
        },
        timeline: updated,
    })
}

pub fn attach_music(
    store: &mut ProjectStore,
    asset_id: &str,
    volume: Option<f64>,
) -> AppResult<Timeline> {
    let asset = store.get_asset(asset_id)?;
    if asset.kind != "audio" {
        return Err(AppError::Invalid("only audio assets can be used as music".into()));
    }
    store.update_timeline(TimelinePatch {
        music_asset_id: Some(asset.id),
        music_volume: Some(volume.unwrap_or(0.8).clamp(0.0, 2.0)),
        ..Default::default()
    })
}

struct PcmAnalysis {
    bpm: f64,
    confidence: f64,
    offset_ms: i64,
    beat_grid_ms: Vec<i64>,
    waveform: Vec<f32>,
}

fn analyze_pcm(samples: &[f32], sample_rate: u32, duration_ms: i64) -> PcmAnalysis {
    let frame_count = samples.len().saturating_sub(FRAME) / HOP;
    let mut energy = vec![0.0_f32; frame_count];
    for index in 0..frame_count {
        let start = index * HOP;
        let mut sum = 0.0_f32;
        for sample in &samples[start..start + FRAME] {
            sum += sample * sample;
        }
        energy[index] = (sum / FRAME as f32).sqrt();
    }
    let mut onset = vec![0.0_f32; frame_count];
    for index in 1..frame_count {
        onset[index] = (energy[index] - energy[index - 1]).max(0.0);
    }
    let smooth = onset
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let prev = if i > 0 { onset[i - 1] } else { 0.0 };
            let next = onset.get(i + 1).copied().unwrap_or(0.0);
            (prev + value * 2.0 + next) / 4.0
        })
        .collect::<Vec<_>>();
    let peak = smooth.iter().fold(0.0_f32, |acc, v| acc.max(*v));
    if peak < 1e-6 || frame_count < 64 {
        return fallback_grid(duration_ms);
    }
    let onset: Vec<f32> = smooth.iter().map(|v| v / peak).collect();
    let frame_rate = sample_rate as f64 / HOP as f64;
    let min_lag = (60.0 * frame_rate / 200.0).round().max(2.0) as usize;
    let max_lag = ((60.0 * frame_rate / 60.0).round() as usize)
        .min(onset.len().saturating_sub(2))
        .max(min_lag + 1);
    let mut best_lag = min_lag;
    let mut best_score = 0.0_f64;
    let mut best_corr = 0.0_f64;
    for lag in min_lag..max_lag {
        let mut dot = 0.0_f64;
        let mut left = 0.0_f64;
        let mut right = 0.0_f64;
        for i in 0..onset.len().saturating_sub(lag) {
            let a = onset[i] as f64;
            let b = onset[i + lag] as f64;
            dot += a * b;
            left += a * a;
            right += b * b;
        }
        if left <= 1e-9 || right <= 1e-9 {
            continue;
        }
        let corr = dot / (left.sqrt() * right.sqrt());
        let bpm = 60.0 * frame_rate / lag as f64;
        let prior = 1.0 + 0.22 * (-((bpm - 108.0) / 46.0).powi(2)).exp();
        let score = corr * prior;
        if score > best_score {
            best_score = score;
            best_lag = lag;
            best_corr = corr;
        }
    }
    if best_lag <= 1 {
        return fallback_grid(duration_ms);
    }
    let bpm = (60.0 * frame_rate / best_lag as f64).clamp(40.0, 220.0);
    let mut offset = 0_usize;
    let mut best_phase = f64::MIN;
    for phase in 0..best_lag {
        let mut sum = 0.0_f64;
        let mut count = 0_usize;
        let mut index = phase;
        while index < onset.len() {
            sum += onset[index] as f64;
            count += 1;
            index += best_lag;
        }
        if count > 0 {
            let score = sum / count as f64;
            if score > best_phase {
                best_phase = score;
                offset = phase;
            }
        }
    }
    let frame_ms = HOP as f64 * 1000.0 / sample_rate as f64;
    let mut beat_grid_ms = Vec::new();
    let mut index = offset;
    while index < onset.len() {
        let ms = (index as f64 * frame_ms).round() as i64;
        if ms > duration_ms {
            break;
        }
        beat_grid_ms.push(ms);
        index += best_lag;
    }
    if beat_grid_ms.len() < 2 {
        return fallback_grid(duration_ms);
    }
    let waveform = waveform(samples, 160);
    PcmAnalysis {
        bpm: (bpm * 10.0).round() / 10.0,
        confidence: (best_corr * 0.75 + (best_phase.max(0.0).min(1.0)) * 0.25)
            .clamp(0.0, 0.99),
        offset_ms: beat_grid_ms[0],
        beat_grid_ms,
        waveform,
    }
}

fn fallback_grid(duration_ms: i64) -> PcmAnalysis {
    let bpm = 100.0;
    let interval = (60_000.0 / bpm) as i64;
    let mut beats = Vec::new();
    let mut time = 0;
    while time <= duration_ms {
        beats.push(time);
        time += interval;
    }
    PcmAnalysis {
        bpm,
        confidence: 0.0,
        offset_ms: 0,
        beat_grid_ms: beats,
        waveform: vec![0.3; 160],
    }
}

fn waveform(samples: &[f32], buckets: usize) -> Vec<f32> {
    let bucket = (samples.len() / buckets).max(1);
    let mut out = Vec::with_capacity(buckets);
    for index in 0..buckets {
        let start = index * bucket;
        let end = ((index + 1) * bucket).min(samples.len());
        if start >= end {
            out.push(0.0);
            continue;
        }
        let mut sum = 0.0_f32;
        for sample in &samples[start..end] {
            sum += sample * sample;
        }
        out.push((sum / (end - start) as f32).sqrt());
    }
    let peak = out.iter().fold(0.0_f32, |acc, v| acc.max(*v)).max(1e-6);
    out.iter().map(|v| (v / peak).clamp(0.0, 1.0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_synthetic_120_bpm_click_track() {
        let rate = 22_050_u32;
        let seconds = 12_u32;
        let bpm = 120.0_f64;
        let interval = 60.0 / bpm;
        let mut samples = vec![0.0_f32; (rate * seconds) as usize];
        let mut noise_seed = 123_456_u64;
        for beat in 0..(seconds as f64 / interval) as usize {
            let start = (beat as f64 * interval * rate as f64) as usize;
            for offset in 0..1_200 {
                if start + offset >= samples.len() {
                    break;
                }
                noise_seed = noise_seed
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let noise = ((noise_seed >> 33) as f32 / u32::MAX as f32) * 2.0 - 1.0;
                samples[start + offset] += noise * (-(offset as f32) / 220.0).exp() * 0.8;
            }
        }
        let analysis = analyze_pcm(&samples, rate, seconds as i64 * 1000);
        assert!(
            (analysis.bpm - 120.0).abs() < 10.0,
            "detected {} bpm",
            analysis.bpm
        );
        assert!(analysis.beat_grid_ms.len() >= 18);
        assert_eq!(analysis.waveform.len(), 160);
    }

    #[test]
    fn mock_music_is_a_valid_wav() {
        let track = crate::providers::mock::music("rainy jazz", 8_000, Some(3)).unwrap();
        assert_eq!(&track.bytes[0..4], b"RIFF");
        assert_eq!(&track.bytes[8..12], b"WAVE");
        assert_eq!(track.bpm, 100.0);
    }

    #[test]
    fn license_labels_are_conservative() {
        assert_eq!(license_label("http://creativecommons.org/licenses/by/4.0/"), "CC BY");
        assert!(license_label("http://creativecommons.org/licenses/by/4.0/").contains("BY"));
        assert_eq!(
            license_label("http://creativecommons.org/publicdomain/zero/1.0/"),
            "CC0 / Public Domain"
        );
    }
}
