use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::providers::llm;
use crate::services::script;
use crate::store::preset_style;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const MAX_CHUNK_CHARS: usize = 6_000;
pub const MAX_CHUNKS: usize = 120;
pub const MAX_EPISODES: usize = 60;

#[derive(Debug, Clone)]
pub struct NovelChapter {
    pub index: usize,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct NovelChunk {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct AdaptationResult {
    pub script: GeneratedScript,
    pub analysis: Value,
    pub chunk_count: i64,
    pub calls: i64,
}

fn is_heading(line: &str) -> bool {
    let text = line.trim();
    if text.is_empty() || text.chars().count() > 60 {
        return false;
    }
    if text.starts_with('#') {
        return true;
    }
    if text.starts_with('第') {
        let prefix: String = text.chars().take(12).collect();
        return prefix
            .chars()
            .any(|ch| matches!(ch, '章' | '回' | '节' | '卷'));
    }
    let lower = text.to_ascii_lowercase();
    lower.starts_with("chapter ")
        || lower.starts_with("prologue")
        || lower.starts_with("epilogue")
        || lower.starts_with("序章")
        || lower.starts_with("楔子")
}

pub fn split_chapters(text: &str) -> Vec<NovelChapter> {
    let mut chapters: Vec<NovelChapter> = Vec::new();
    let mut current_title = String::new();
    let mut current_text = String::new();
    let mut preamble = String::new();

    let mut flush = |title: &str, body: &str| {
        if body.trim().chars().count() < 2 {
            return;
        }
        let index = chapters.len() + 1;
        chapters.push(NovelChapter {
            index,
            title: if title.trim().is_empty() {
                format!("第{index}节")
            } else {
                title.trim().to_string()
            },
            text: body.trim().to_string(),
        });
    };

    for line in text.replace("\r\n", "\n").lines() {
        if is_heading(line) {
            if current_title.is_empty() {
                if !preamble.trim().is_empty() {
                    flush("开篇", &preamble);
                }
            } else {
                flush(&current_title, &current_text);
            }
            current_title = line.trim().trim_start_matches('#').trim().to_string();
            current_text.clear();
        } else if current_title.is_empty() {
            preamble.push_str(line);
            preamble.push('\n');
        } else {
            current_text.push_str(line);
            current_text.push('\n');
        }
    }
    if current_title.is_empty() {
        if !preamble.trim().is_empty() {
            flush("全文", &preamble);
        }
    } else {
        flush(&current_title, &current_text);
    }
    if chapters.is_empty() && !text.trim().is_empty() {
        chapters.push(NovelChapter {
            index: 1,
            title: "全文".into(),
            text: text.trim().to_string(),
        });
    }
    chapters
}

fn split_by_chars(text: &str, max_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut count = 0;
    for ch in text.chars() {
        current.push(ch);
        count += 1;
        if count >= max_chars {
            out.push(current.trim().to_string());
            current.clear();
            count = 0;
        }
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_string());
    }
    out
}

pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(500);
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_chars = 0usize;
    for line in text.lines() {
        let line = line.trim_end();
        let line_chars = line.chars().count();
        if line_chars > max_chars {
            if !current.trim().is_empty() {
                chunks.push(current.trim().to_string());
                current.clear();
                current_chars = 0;
            }
            chunks.extend(split_by_chars(line, max_chars));
            continue;
        }
        if current_chars > 0 && current_chars + line_chars + 1 > max_chars {
            chunks.push(current.trim().to_string());
            current.clear();
            current_chars = 0;
        }
        current.push_str(line);
        current.push('\n');
        current_chars += line_chars + 1;
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks
}

pub fn plan(text: &str) -> NovelPlan {
    let chapters = split_chapters(text);
    let mut chunk_count = 0i64;
    let mut infos = Vec::new();
    for chapter in &chapters {
        let chunks = chunk_text(&chapter.text, MAX_CHUNK_CHARS).len().max(1);
        chunk_count += chunks as i64;
        infos.push(NovelChapterInfo {
            index: chapter.index as i64,
            title: chapter.title.clone(),
            char_count: chapter.text.chars().count() as i64,
        });
    }
    NovelPlan {
        char_count: text.trim().chars().count() as i64,
        chapter_count: chapters.len() as i64,
        chunk_count,
        estimated_calls: chunk_count,
        chapters: infos,
    }
}

pub fn estimate_cost(profile: &ProviderProfile, plan: &NovelPlan) -> f64 {
    let input = profile
        .options
        .get("input_usd_per_1k")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let output = profile
        .options
        .get("output_usd_per_1k")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    if input <= 0.0 && output <= 0.0 {
        return 0.0;
    }
    let input_tokens = plan.char_count as f64 / 2.0 / 1000.0;
    let output_tokens = plan.chunk_count as f64 * 1.2;
    input_tokens * input + output_tokens * output
}

fn groups<'a>(chapters: &'a [NovelChapter], max_episodes: u32) -> Vec<Vec<&'a NovelChapter>> {
    if chapters.is_empty() {
        return Vec::new();
    }
    let cap = if max_episodes == 0 {
        MAX_EPISODES
    } else {
        (max_episodes as usize).clamp(1, MAX_EPISODES)
    };
    if chapters.len() <= cap {
        return chapters.iter().map(|chapter| vec![chapter]).collect();
    }
    let mut out = Vec::new();
    let mut start = 0usize;
    for index in 0..cap {
        let remaining = chapters.len() - start;
        let take = (remaining + (cap - index) - 1) / (cap - index);
        out.push(chapters[start..start + take].iter().collect());
        start += take;
    }
    out
}

fn merge_character(
    merged: &mut BTreeMap<String, GeneratedCharacter>,
    character: GeneratedCharacter,
) {
    let name = character.name.trim();
    if name.is_empty() {
        return;
    }
    if let Some(existing) = merged.get_mut(name) {
        if existing.appearance.trim().is_empty() {
            existing.appearance = character.appearance;
        }
        if existing.personality.trim().is_empty() {
            existing.personality = character.personality;
        }
        for trait_name in character.locked_traits {
            if !trait_name.trim().is_empty()
                && !existing.locked_traits.contains(&trait_name)
                && existing.locked_traits.len() < 8
            {
                existing.locked_traits.push(trait_name);
            }
        }
    } else {
        merged.insert(name.to_string(), character);
    }
}

fn take_chars(text: &str, limit: usize) -> String {
    let value: String = text.trim().chars().take(limit).collect();
    if text.trim().chars().count() > limit {
        format!("{value}…")
    } else {
        value
    }
}

fn mock_adaptation(
    project: &Project,
    source: &NovelSource,
    request: &NovelAdaptRequest,
    chapters: &[NovelChapter],
    chunk_count: usize,
    progress: &(dyn Fn(i64, &str) -> AppResult<()> + Send + Sync),
) -> AppResult<AdaptationResult> {
    let mut merged = BTreeMap::new();
    let mut analyses = Vec::new();
    let mut chapter_episodes = Vec::new();
    for (index, chapter) in chapters.iter().enumerate() {
        progress(
            10 + (70 * index as i64 / chapters.len().max(1) as i64),
            &format!("adapting chapter {} / {}", index + 1, chapters.len()),
        )?;
        let premise = take_chars(&chapter.text, 500);
        let generated = script::mock_script(
            &GenerateScriptRequest {
                premise,
                genre: project.genre.clone(),
                tone: project.tone.clone(),
                language: project.language.clone(),
                episodes: 1,
                scenes_per_episode: 1,
                shots_per_scene: 3,
                style_notes: request.style_notes.clone(),
                provider_id: None,
                model: None,
                replace_existing: false,
                seed: None,
            },
            project,
        );
        for character in generated.characters {
            merge_character(&mut merged, character);
        }
        let mut episode =
            generated
                .episodes
                .into_iter()
                .next()
                .unwrap_or_else(|| GeneratedEpisode {
                    title: chapter.title.clone(),
                    summary: take_chars(&chapter.text, 160),
                    scenes: Vec::new(),
                });
        episode.title = chapter.title.clone();
        episode.summary = take_chars(&chapter.text, 180);
        analyses.push(json!({
            "index": chapter.index,
            "title": chapter.title,
            "summary": take_chars(&chapter.text, 220),
        }));
        chapter_episodes.push(episode);
    }

    let grouped = groups(chapters, request.max_episodes);
    let mut episodes = Vec::new();
    for (index, group) in grouped.iter().enumerate() {
        let mut scenes = Vec::new();
        let mut titles = Vec::new();
        let mut summaries = Vec::new();
        for chapter in group {
            titles.push(chapter.title.clone());
            summaries.push(take_chars(&chapter.text, 160));
            if let Some(episode) = chapter_episodes.get(chapter.index - 1) {
                scenes.extend(episode.scenes.clone());
            }
        }
        episodes.push(GeneratedEpisode {
            title: format!("第{}集 · {}", index + 1, titles.join(" / ")),
            summary: summaries.join(" "),
            scenes,
        });
    }
    let character_count = merged.len();
    let characters: Vec<GeneratedCharacter> = merged.into_values().collect();
    let style = if project.style_bible.visual_style.trim().is_empty() {
        preset_style(Some("anime-cel"))
    } else {
        project.style_bible.clone()
    };
    let synopsis = chapters
        .iter()
        .take(3)
        .map(|chapter| take_chars(&chapter.text, 160))
        .collect::<Vec<_>>()
        .join(" ");
    Ok(AdaptationResult {
        script: GeneratedScript {
            title: source.summary.title.clone(),
            logline: take_chars(&chapters[0].text, 120),
            synopsis,
            style_bible: style,
            characters,
            episodes,
        },
        analysis: json!({
            "mode": "mock",
            "chapters": analyses,
            "characters": character_count,
            "chunkCount": chunk_count,
        }),
        chunk_count: chunk_count as i64,
        calls: chapters.len() as i64,
    })
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct AnalyzedCharacter {
    #[serde(default)]
    name: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    appearance: String,
    #[serde(default)]
    personality: String,
    #[serde(default)]
    locked_traits: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct AnalyzedBeat {
    #[serde(default)]
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    time_of_day: String,
    #[serde(default)]
    characters: Vec<String>,
    #[serde(default)]
    dialogue: String,
    #[serde(default)]
    visual: String,
    #[serde(default)]
    camera: String,
    #[serde(default)]
    duration_sec: i64,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct ChunkAnalysis {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    characters: Vec<AnalyzedCharacter>,
    #[serde(default)]
    beats: Vec<AnalyzedBeat>,
}

async fn analyze_chunk(
    profile: &ProviderProfile,
    secret: Option<&str>,
    source: &NovelSource,
    chapter: &NovelChapter,
    chunk: &str,
    index: usize,
    total: usize,
    previous: &str,
) -> AppResult<ChunkAnalysis> {
    let system = "You are a story analyst for a premium vertical comic drama. Read the novel excerpt and extract only production-ready story material. Return one JSON object, no markdown.";
    let user = format!(
        r#"Analyze novel "{title}", chapter "{chapter}", chunk {index}/{total}.
Previous chapter context: {previous}

TEXT:
{chunk}

Extract:
- summary: 1-2 sentences of what changes in this chunk.
- characters: recurring named characters with visual identity and 2-5 lockedTraits (face, hair, costume, silhouette, colors).
- beats: 3-8 visual beats. Each beat must have one action, one location/time, participating character names, short speakable dialogue (or empty), a visual prompt, camera move and 3-10 second duration.

JSON SHAPE:
{{"summary":"","characters":[{{"name":"","role":"","appearance":"","personality":"","lockedTraits":[""]}}],"beats":[{{"title":"","summary":"","location":"","timeOfDay":"","characters":[""],"dialogue":"","visual":"","camera":"","durationSec":5}}]}}"#,
        title = source.summary.title,
        chapter = chapter.title,
        index = index + 1,
        total = total,
        previous = if previous.trim().is_empty() {
            "none"
        } else {
            previous
        },
        chunk = chunk
    );
    let temperature = profile
        .options
        .get("temperature")
        .and_then(Value::as_f64)
        .unwrap_or(0.6);
    let result = llm::complete(
        profile,
        secret,
        llm::LlmRequest {
            model: profile.default_model.clone(),
            system: system.into(),
            user,
            temperature,
            max_tokens: 2_000,
            json: true,
        },
    )
    .await?;
    llm::parse_json_loose(&result.text)
}

async fn llm_adaptation(
    profile: &ProviderProfile,
    secret: Option<&str>,
    project: &Project,
    source: &NovelSource,
    request: &NovelAdaptRequest,
    chapters: &[NovelChapter],
    progress: &(dyn Fn(i64, &str) -> AppResult<()> + Send + Sync),
) -> AppResult<AdaptationResult> {
    let mut per_chapter: Vec<Vec<ChunkAnalysis>> = Vec::new();
    let mut merged = BTreeMap::new();
    let mut calls = 0i64;
    let total_chunks: usize = chapters
        .iter()
        .map(|chapter| chunk_text(&chapter.text, MAX_CHUNK_CHARS).len().max(1))
        .sum();
    if total_chunks > MAX_CHUNKS {
        return Err(AppError::Invalid(format!(
            "novel needs {total_chunks} LLM calls (limit {MAX_CHUNKS}); split it into volumes or raise the limit"
        )));
    }
    let mut done = 0usize;
    let mut previous_summary = String::new();
    for chapter in chapters {
        let chunks = chunk_text(&chapter.text, MAX_CHUNK_CHARS);
        let chunks = if chunks.is_empty() {
            vec![chapter.text.clone()]
        } else {
            chunks
        };
        let mut analyses = Vec::new();
        for chunk in &chunks {
            progress(
                10 + (70 * done as i64 / total_chunks.max(1) as i64),
                &format!(
                    "analyzing chapter {} · chunk {}/{}",
                    chapter.index,
                    done + 1,
                    total_chunks
                ),
            )?;
            let analysis = analyze_chunk(
                profile,
                secret,
                source,
                chapter,
                chunk,
                done,
                total_chunks,
                &previous_summary,
            )
            .await?;
            calls += 1;
            done += 1;
            if !analysis.summary.trim().is_empty() {
                previous_summary = take_chars(&analysis.summary, 240);
            }
            analyses.push(analysis);
        }
        per_chapter.push(analyses);
    }

    for analyses in &per_chapter {
        for analysis in analyses {
            for character in &analysis.characters {
                merge_character(
                    &mut merged,
                    GeneratedCharacter {
                        name: character.name.clone(),
                        role: character.role.clone(),
                        description: character.role.clone(),
                        appearance: character.appearance.clone(),
                        personality: character.personality.clone(),
                        voice: String::new(),
                        locked_traits: character.locked_traits.clone(),
                    },
                );
            }
        }
    }
    if merged.is_empty() {
        return Err(AppError::Provider(
            "the model returned no recurring characters; retry with a stronger model".into(),
        ));
    }
    let character_names: Vec<String> = merged.keys().cloned().collect();
    let style = if project.style_bible.visual_style.trim().is_empty() {
        preset_style(Some("anime-cel"))
    } else {
        project.style_bible.clone()
    };
    let grouped = groups(chapters, request.max_episodes);
    let mut episodes = Vec::new();
    let mut chapter_analysis = Vec::new();
    for (group_index, group) in grouped.iter().enumerate() {
        let mut scenes = Vec::new();
        let mut summaries = Vec::new();
        let mut titles = Vec::new();
        for chapter in group {
            titles.push(chapter.title.clone());
            let analyses = &per_chapter[chapter.index - 1];
            for analysis in analyses {
                summaries.push(analysis.summary.clone());
                for (beat_index, beat) in analysis.beats.iter().enumerate().take(10) {
                    let location = if beat.location.trim().is_empty() {
                        "未命名场景"
                    } else {
                        beat.location.trim()
                    };
                    let time = if beat.time_of_day.trim().is_empty() {
                        "NIGHT"
                    } else {
                        beat.time_of_day.trim()
                    };
                    let cast: Vec<String> = beat
                        .characters
                        .iter()
                        .filter(|name| merged.contains_key(name.as_str()))
                        .cloned()
                        .collect();
                    let camera = if beat.camera.trim().is_empty() {
                        "slow push-in"
                    } else {
                        beat.camera.trim()
                    };
                    let duration = beat.duration_sec.clamp(3, 10);
                    let prompt = format!(
                        "{}. {}. {}, 35mm. Cinematic Chinese anime. Palette: {}. Lighting: {}. No on-screen text.",
                        beat.visual.trim().if_empty(&beat.summary),
                        camera,
                        beat.title,
                        style.palette,
                        style.lighting
                    );
                    let shot = GeneratedShot {
                        title: format!(
                            "S{}-{} {}",
                            scenes.len() + 1,
                            beat_index + 1,
                            if beat.title.trim().is_empty() {
                                "节拍"
                            } else {
                                beat.title.trim()
                            }
                        ),
                        description: beat.summary.clone(),
                        camera: camera.into(),
                        lens: "35mm".into(),
                        duration_sec: duration,
                        dialogue: beat.dialogue.clone(),
                        sound: String::new(),
                        characters: cast,
                        prompt,
                        negative_prompt: style.negative_prompt.clone(),
                    };
                    let script_text = format!("{}\n{}\n", shot.description, shot.dialogue);
                    scenes.push(GeneratedScene {
                        title: format!("第{}场 · {}", scenes.len() + 1, location),
                        slugline: format!(
                            "{} {} - {}",
                            if beat.time_of_day.eq_ignore_ascii_case("DAY") {
                                "EXT."
                            } else {
                                "INT."
                            },
                            location,
                            time
                        ),
                        location: location.into(),
                        time_of_day: time.into(),
                        synopsis: beat.summary.clone(),
                        script_text,
                        shots: vec![shot],
                    });
                }
            }
            chapter_analysis.push(json!({
                "index": chapter.index,
                "title": chapter.title,
                "summary": analyses.iter().map(|a| a.summary.clone()).collect::<Vec<_>>().join(" "),
            }));
        }
        if scenes.is_empty() {
            continue;
        }
        episodes.push(GeneratedEpisode {
            title: format!("第{}集 · {}", group_index + 1, titles.join(" / ")),
            summary: take_chars(&summaries.join(" "), 300),
            scenes,
        });
    }
    if episodes.is_empty() {
        return Err(AppError::Provider(
            "the model returned no usable beats; try a stronger model or a smaller chunk".into(),
        ));
    }
    let characters: Vec<GeneratedCharacter> = merged.into_values().collect();
    let first_summary = episodes
        .first()
        .map(|e| e.summary.clone())
        .unwrap_or_default();
    let last_summary = episodes
        .last()
        .map(|e| e.summary.clone())
        .unwrap_or_default();
    Ok(AdaptationResult {
        script: GeneratedScript {
            title: source.summary.title.clone(),
            logline: take_chars(&first_summary, 120),
            synopsis: format!("{first_summary} {last_summary}"),
            style_bible: style,
            characters,
            episodes,
        },
        analysis: json!({
            "mode": "llm",
            "chapters": chapter_analysis,
            "characters": character_names,
            "chunkCount": total_chunks,
        }),
        chunk_count: total_chunks as i64,
        calls,
    })
}

pub async fn adapt(
    profile: &ProviderProfile,
    secret: Option<&str>,
    project: &Project,
    source: &NovelSource,
    request: &NovelAdaptRequest,
    progress: &(dyn Fn(i64, &str) -> AppResult<()> + Send + Sync),
) -> AppResult<AdaptationResult> {
    let chapters = split_chapters(&source.text);
    if chapters.is_empty() {
        return Err(AppError::Invalid("novel has no readable text".into()));
    }
    let chunk_count: usize = chapters
        .iter()
        .map(|chapter| chunk_text(&chapter.text, MAX_CHUNK_CHARS).len().max(1))
        .sum();
    if profile.id.starts_with("mock-") {
        return mock_adaptation(project, source, request, &chapters, chunk_count, progress);
    }
    llm_adaptation(
        profile, secret, project, source, request, &chapters, progress,
    )
    .await
}

trait EmptyFallback {
    fn if_empty(&self, fallback: &str) -> String;
}

impl EmptyFallback for str {
    fn if_empty(&self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.trim().to_string()
        } else {
            self.trim().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_chinese_and_english_chapters() {
        let cn = "序章\n雨落在旧站台。\n第一章 起点\n她捡到一把钥匙。\n第二章 转折\n门后传来自己的声音。";
        let chapters = split_chapters(cn);
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[1].title, "第一章 起点");
        assert!(chapters[1].text.contains("钥匙"));

        let en = "Chapter 1\nA letter arrives.\nChapter 2\nThe letter is blank.";
        let chapters = split_chapters(en);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter 1");
    }

    #[test]
    fn chunking_respects_the_character_limit() {
        let line = "雨".repeat(1_200);
        let chunks = chunk_text(&line, 500);
        assert!(chunks.len() >= 3);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 500));
    }

    #[test]
    fn plan_reports_chapters_and_chunks() {
        let text = "第一章 起点\n她捡到钥匙。\n第二章 转折\n门后有人。";
        let plan = plan(text);
        assert_eq!(plan.chapter_count, 2);
        assert_eq!(plan.chunk_count, 2);
        assert_eq!(plan.chapters.len(), 2);
        assert!(plan.char_count > 10);
    }
}
