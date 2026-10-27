use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::paths::AppPaths;
#[cfg(target_os = "macos")]
use crate::settings;
use crate::store::assets::NewAsset;
use crate::store::ProjectStore;
#[cfg(target_os = "windows")]
use base64::Engine;
#[cfg(any(target_os = "windows", test))]
use std::path::Path;
use tokio::process::Command;

pub async fn generate(
    paths: &AppPaths,
    project_id: &str,
    shot_id: &str,
    requested_voice: Option<String>,
) -> AppResult<Asset> {
    let (text, project_dir) = {
        let store = ProjectStore::open(paths, project_id)?;
        let shot = store.get_shot(shot_id)?;
        let text = dialogue_for_tts(&shot.dialogue);
        if text.is_empty() {
            return Err(AppError::Invalid(
                "this shot has no dialogue to voice".into(),
            ));
        }
        (text, store.dir.clone())
    };
    let id = uuid::Uuid::new_v4().to_string();
    let rel = format!("assets/audio/{id}.wav");
    let output = project_dir.join(&rel);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp_dir = project_dir.join("exports");
    std::fs::create_dir_all(&temp_dir)?;

    #[cfg(target_os = "macos")]
    {
        let ffmpeg = crate::services::media::ensure_ffmpeg(&settings::load(paths))?;
        let temp_aiff = temp_dir.join(format!("tts-{id}.aiff"));
        let mut command = Command::new("/usr/bin/say");
        if let Some(voice) = requested_voice
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            command.args(["-v", voice]);
        }
        let status = command
            .args(["-o"])
            .arg(&temp_aiff)
            .arg(&text)
            .status()
            .await?;
        if !status.success() || !temp_aiff.exists() {
            let _ = std::fs::remove_file(&temp_aiff);
            return Err(AppError::Message(
                "macOS speech synthesis failed. Check that a system voice is installed.".into(),
            ));
        }
        let status = Command::new(&ffmpeg.program)
            .args(["-y", "-i"])
            .arg(&temp_aiff)
            .args(["-ar", "24000", "-ac", "1", "-c:a", "pcm_s16le"])
            .arg(&output)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await?;
        let _ = std::fs::remove_file(&temp_aiff);
        if !status.success() {
            return Err(AppError::Message(
                "could not convert synthesized speech to WAV".into(),
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let script_path = temp_dir.join(format!("tts-{id}.ps1"));
        let escaped_output = output.to_string_lossy().replace('\'', "''");
        let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
        let select_voice = requested_voice
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("$synth.SelectVoice('{}')\n", value.replace('\'', "''")))
            .unwrap_or_else(|| {
                // SAPI defaults to en-US; Chinese dialogue needs a zh voice.
                "$zh=$synth.GetInstalledVoices() | Where-Object {{ $_.VoiceInfo.Culture.Name -like 'zh*' }} | Select-Object -First 1\nif ($zh) {{ $synth.SelectVoice($zh.VoiceInfo.Name) }}\n".into()
            });
        // Windows PowerShell 5.1 reads BOM-less .ps1 as ANSI, mangling CJK paths.
        let script = format!(
            "\u{feff}Add-Type -AssemblyName System.Speech\n$text=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{encoded}'))\n$synth=New-Object System.Speech.Synthesis.SpeechSynthesizer\n{select_voice}$synth.SetOutputToWaveFile('{escaped_output}')\n$synth.Speak($text)\n$synth.Dispose()\n"
        );
        std::fs::write(&script_path, script)?;
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&script_path)
            .status()
            .await?;
        let _ = std::fs::remove_file(&script_path);
        if !status.success() || !output.exists() {
            return Err(AppError::Message(
                "Windows speech synthesis failed. Check that a system voice is installed.".into(),
            ));
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        return Err(AppError::Message(
            "system speech synthesis is currently available on macOS and Windows".into(),
        ));
    }

    #[cfg(target_os = "macos")]
    let duration = crate::services::media::ensure_ffmpeg(&settings::load(paths))?
        .probe(&output)
        .duration_ms
        .unwrap_or(0);
    #[cfg(target_os = "windows")]
    let duration = wav_duration_ms(&output).unwrap_or(0);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let duration = 0;
    let mut store = ProjectStore::open(paths, project_id)?;
    let asset = store.insert_asset(NewAsset {
        kind: "audio".into(),
        name: format!("配音 · {}", truncate(&text, 24)),
        rel_path: rel,
        thumb_rel_path: None,
        mime: "audio/wav".into(),
        width: None,
        height: None,
        duration_ms: Some(duration),
        source: "tts".into(),
        provider_id: Some("system-tts".into()),
        model: Some(if cfg!(target_os = "windows") {
            "windows-sapi".to_string()
        } else {
            "macos-say".to_string()
        }),
        prompt: Some(text),
        negative_prompt: None,
        seed: None,
        version: 1,
        parent_asset_id: None,
        license: Some("Operating-system speech synthesis".into()),
        license_url: None,
        attribution: None,
        source_url: None,
        status: "ready".into(),
    })?;
    store.attach_shot_voice(shot_id, &asset.id)?;
    Ok(asset)
}

fn dialogue_for_tts(dialogue: &str) -> String {
    let mut out = String::new();
    for line in dialogue.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let content = if let Some((speaker, rest)) = line.split_once('：') {
            if speaker.chars().count() <= 12 {
                rest.trim()
            } else {
                line
            }
        } else if let Some((speaker, rest)) = line.split_once(':') {
            if speaker.chars().count() <= 24 && !speaker.contains(' ') {
                rest.trim()
            } else {
                line
            }
        } else {
            line
        };
        if content.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(content);
    }
    out.chars().take(1200).collect()
}

fn truncate(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

pub fn voice_supported() -> bool {
    cfg!(any(target_os = "macos", target_os = "windows"))
}

#[cfg(any(target_os = "windows", test))]
fn wav_duration_ms(path: &Path) -> Option<i64> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let mut byte_rate = 0u32;
    let mut pos = 12usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().ok()?) as usize;
        let data_start = pos + 8;
        if id == b"fmt " && data_start + 12 <= bytes.len() {
            byte_rate = u32::from_le_bytes(bytes[data_start + 8..data_start + 12].try_into().ok()?);
        } else if id == b"data" {
            let data = size.min(bytes.len().saturating_sub(data_start));
            if byte_rate == 0 {
                return None;
            }
            return Some((data as i64) * 1000 / byte_rate as i64);
        }
        pos = data_start.saturating_add(size + (size & 1));
    }
    None
}

pub fn ensure_voice_supported() -> AppResult<()> {
    if voice_supported() {
        Ok(())
    } else {
        Err(AppError::Message(
            "system speech synthesis is currently available on macOS and Windows".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{dialogue_for_tts, wav_duration_ms};

    #[test]
    fn strips_speaker_labels_for_voice_track() {
        let text = dialogue_for_tts("沈知遥：别关门——\n顾临川：已经太晚了。");
        assert!(!text.contains('：'));
        assert!(text.contains("别关门"));
        assert!(text.contains("已经太晚了"));
    }

    #[test]
    fn reads_duration_from_a_pcm_wav() {
        let path = std::env::temp_dir().join(format!(
            "saladcart-voice-{}-{}.wav",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36u32 + 16000).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&16000u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&16000u32.to_le_bytes());
        bytes.extend_from_slice(&vec![0u8; 16000]);
        std::fs::write(&path, bytes).unwrap();
        assert_eq!(wav_duration_ms(&path), Some(1000));
        let _ = std::fs::remove_file(&path);
    }
}
