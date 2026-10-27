use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::paths::AppPaths;
use crate::services::media;
use crate::settings;
use crate::store::assets::NewAsset;
use crate::store::ProjectStore;
#[cfg(target_os = "windows")]
use base64::Engine;
use tokio::process::Command;

pub async fn generate(
    paths: &AppPaths,
    project_id: &str,
    shot_id: &str,
    requested_voice: Option<String>,
) -> AppResult<Asset> {
    let (text, project_dir, settings) = {
        let store = ProjectStore::open(paths, project_id)?;
        let shot = store.get_shot(shot_id)?;
        let text = dialogue_for_tts(&shot.dialogue);
        if text.is_empty() {
            return Err(AppError::Invalid(
                "this shot has no dialogue to voice".into(),
            ));
        }
        (text, store.dir.clone(), settings::load(paths))
    };
    let ffmpeg = media::ensure_ffmpeg(&settings)?;
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
            .unwrap_or_default();
        let script = format!(
            "Add-Type -AssemblyName System.Speech\n$text=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{encoded}'))\n$synth=New-Object System.Speech.Synthesis.SpeechSynthesizer\n{select_voice}$synth.SetOutputToWaveFile('{escaped_output}')\n$synth.Speak($text)\n$synth.Dispose()\n"
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

    let probe = ffmpeg.probe(&output);
    let duration = probe.duration_ms.unwrap_or(0);
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
    use super::dialogue_for_tts;

    #[test]
    fn strips_speaker_labels_for_voice_track() {
        let text = dialogue_for_tts("沈知遥：别关门——\n顾临川：已经太晚了。");
        assert!(!text.contains('：'));
        assert!(text.contains("别关门"));
        assert!(text.contains("已经太晚了"));
    }
}
