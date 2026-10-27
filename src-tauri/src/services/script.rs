use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::providers::llm;
use crate::store::preset_style;

pub async fn generate(
    profile: &ProviderProfile,
    secret: Option<&str>,
    request: &GenerateScriptRequest,
    project: &Project,
) -> AppResult<GeneratedScript> {
    if profile.id.starts_with("mock-") {
        return Ok(mock_script(request, project));
    }
    let style_notes = if request.style_notes.trim().is_empty() {
        project.style_bible.visual_style.clone()
    } else {
        format!(
            "{}\n{}",
            project.style_bible.visual_style, request.style_notes
        )
    };
    let system = "You are the showrunner and head writer of a premium vertical AI comic drama. \
You design stories for 1–3 minute episodes with a hook in the first 3 seconds, a turn around 15 seconds, \
and a cliffhanger. You write production-ready, visual, budget-aware scripts that can be turned into \
5–10 second AI video shots. You always return a single valid JSON object. Never include markdown fences or commentary.";
    let user = format!(
        r#"Create a production-ready comic-drama bible from the premise below.

PREMISE: {premise}
GENRE: {genre}
TONE: {tone}
LANGUAGE: {language}
STYLE NOTES: {style}
PROJECT FORMAT: {aspect}, target episode length {target}s.
SCOPE: {episodes} episode(s), {scenes} scene(s) per episode, {shots} shot(s) per scene.

HARD REQUIREMENTS
1. Every character in every shot must use an exact `name` from the characters array.
2. 2–5 recurring characters. Give each specific, visual, locked identity traits (face, hair, costume, silhouette, signature colors). No generic descriptions.
3. Shot 1 of episode 1 MUST open with a 3-second visual hook (no logo, no text card).
4. One camera move and one clear action per shot; 3–10 seconds each. Include a concrete end state for continuity.
5. Dialogue must be short, natural, speakable, in the requested language, and belong to the scene.
6. Keep locations and costumes reusable for series production.
7. The style bible must describe visual style, palette, lighting, camera language and a negative prompt.
8. Generate a strong logline and synopsis. Avoid purple prose.
9. Return as many scenes/shots as the scope requests. Never return empty arrays.

JSON SHAPE
{{
  "title": "string",
  "logline": "string",
  "synopsis": "string",
  "styleBible": {{"visualStyle":"","palette":"","lighting":"","cameraLanguage":"","negativePrompt":"","referenceNotes":""}},
  "characters": [{{"name":"","role":"","description":"","appearance":"","personality":"","voice":"","lockedTraits":["",""]}}],
  "episodes": [{{
    "title":"","summary":"",
    "scenes":[{{
      "title":"","slugline":"INT. LOCATION - DAY","location":"","timeOfDay":"","synopsis":"","scriptText":"",
      "shots":[{{
        "title":"","description":"","camera":"","lens":"","durationSec":5,"dialogue":"","sound":"",
        "characters":["exact name"],"prompt":"","negativePrompt":""
      }}]
    }}]
  }}]
}}"#,
        premise = request.premise.trim(),
        genre = request.genre,
        tone = request.tone,
        language = if request.language.is_empty() {
            "Simplified Chinese"
        } else {
            &request.language
        },
        style = style_notes,
        aspect = project.aspect_ratio,
        target = project.target_episode_seconds,
        episodes = request.episodes.clamp(1, 20),
        scenes = request.scenes_per_episode.clamp(1, 8),
        shots = request.shots_per_scene.clamp(1, 12)
    );
    let temperature = profile
        .options
        .get("temperature")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.8);
    let result = llm::complete(
        profile,
        secret,
        llm::LlmRequest {
            model: request
                .model
                .clone()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| profile.default_model.clone()),
            system: system.into(),
            user,
            temperature,
            max_tokens: 8_000,
            json: true,
        },
    )
    .await?;
    let mut script: GeneratedScript = llm::parse_json_loose(&result.text)?;
    if script.episodes.is_empty() {
        return Err(AppError::Provider(
            "The model returned a script without episodes. Try a stronger model or a simpler premise.".into(),
        ));
    }
    if script.characters.is_empty() {
        return Err(AppError::Provider(
            "The model returned no characters. Character consistency needs at least one character."
                .into(),
        ));
    }
    if script.style_bible.visual_style.trim().is_empty() {
        script.style_bible = project.style_bible.clone();
    }
    Ok(script)
}

pub(crate) fn mock_script(request: &GenerateScriptRequest, project: &Project) -> GeneratedScript {
    let premise = request.premise.trim();
    let premise = if premise.is_empty() {
        "一个能听见旧物记忆的女孩，被迫在一座雨城里追查自己失踪的过去。"
    } else {
        premise
    };
    let title_seed: String = premise.chars().take(12).collect();
    let title = format!("{title_seed}…");
    let mut style = if project.style_bible.visual_style.trim().is_empty() {
        preset_style(Some("anime-cel"))
    } else {
        project.style_bible.clone()
    };
    if !request.style_notes.trim().is_empty() {
        style.visual_style = format!("{}. {}", style.visual_style, request.style_notes.trim());
    }
    let names = [
        (
            "沈知遥",
            "主角 / 记忆修复师",
            "24岁，克制、敏锐，习惯先观察再行动。她能触碰旧物读到残留情绪，但每次读取都会失去一段自己的记忆。",
            "冷白皮，黑色及肩短发，左侧一绺银灰挑染，琥珀色眼睛；深灰高领风衣、旧铜色耳夹，右手缠白色绷带。",
            "克制、好奇、对失去记忆有隐秘恐惧",
            "低而稳的年轻女声，句尾常轻轻收住",
            ["黑色及肩短发", "左侧银灰挑染", "琥珀色眼睛", "深灰高领风衣", "右腕白色绷带"],
        ),
        (
            "顾临川",
            "对手 / 调查记者",
            "28岁，表面散漫，实际把每一条线索都记在心里。他追踪同一桩失踪案，却从不解释自己的动机。",
            "深色微卷短发，眉骨有浅疤，灰绿色眼睛；卡其色短夹克、黑色高领衫，随身带一台老式录音机。",
            "讥诮、执着、在关键时刻可靠",
            "温和但带一点沙哑的男声",
            ["深色微卷短发", "眉骨浅疤", "灰绿色眼睛", "卡其色短夹克", "老式录音机"],
        ),
        (
            "阿箬",
            "伙伴 / 旧物店老板",
            "52岁，经营一家只在雨天开门的旧物店，知道城市里被刻意遗忘的事，但从不直接说破。",
            "银灰盘发，细框眼镜，深青色盘扣外套，手边总有一盏黄铜台灯。",
            "从容、神秘、护短",
            "较慢的中年女声，带笑意",
            ["银灰盘发", "细框眼镜", "深青色盘扣外套", "黄铜台灯", "旧铜钥匙串"],
        ),
    ];
    let characters = names
        .iter()
        .map(
            |(name, role, description, appearance, personality, voice, traits)| {
                GeneratedCharacter {
                    name: name.to_string(),
                    role: role.to_string(),
                    description: description.to_string(),
                    appearance: appearance.to_string(),
                    personality: personality.to_string(),
                    voice: voice.to_string(),
                    locked_traits: traits.iter().map(|t| t.to_string()).collect(),
                }
            },
        )
        .collect();

    let episode_count = request.episodes.clamp(1, 3) as usize;
    let scene_count = request.scenes_per_episode.clamp(1, 2) as usize;
    let shot_count = request.shots_per_scene.clamp(1, 3) as usize;
    let locations = ["旧城区天台", "雨夜车站", "阿箬的旧物店", "封闭的档案馆"];
    let times = ["NIGHT", "DAWN", "EVENING", "NIGHT"];
    let mut episodes = Vec::new();
    for episode_index in 0..episode_count {
        let mut scenes = Vec::new();
        for scene_index in 0..scene_count {
            let location = locations[(episode_index + scene_index) % locations.len()];
            let time = times[(episode_index + scene_index) % times.len()];
            let mut shots = Vec::new();
            for shot_index in 0..shot_count {
                let (shot_title, camera, lens, action, dialogue, cast, sound) =
                    match (scene_index + shot_index) % 6 {
                        0 => (
                            "钩子：雨中的回声",
                            "slow push-in",
                            "35mm",
                            format!(
                                "开场3秒钩子：一枚旧铜钥匙在积水里震动，沈知遥停住脚步。远处站牌灯灭的瞬间，顾临川的身影从玻璃反光中闪过。{}",
                                premise
                            ),
                            "沈知遥（低声）：……它在叫我的名字。".to_string(),
                            vec!["沈知遥", "顾临川"],
                            "雨声突然抽空，只剩钥匙的高频震动",
                        ),
                        1 => (
                            "旧物开口",
                            "handheld follow",
                            "28mm",
                            "沈知遥戴上绷带，指尖触到钥匙。画面边缘浮起不属于现在的记忆碎片，阿箬在门口抬手关掉一半的灯。".to_string(),
                            "阿箬：你每读一次，就少一段自己的。\n沈知遥：那我就先记住今天。".to_string(),
                            vec!["沈知遥", "阿箬"],
                            "台灯电流声、远处列车进站",
                        ),
                        2 => (
                            "对质",
                            "shot / reverse shot",
                            "50mm",
                            "顾临川把录音机放在桌上，按下播放。磁带里传来沈知遥自己的声音，说出一个她从未去过的地址。".to_string(),
                            "顾临川：你不是忘了。你是被人从记忆里剪掉了。".to_string(),
                            vec!["沈知遥", "顾临川"],
                            "磁带底噪、雨点敲铁皮",
                        ),
                        3 => (
                            "追到站台",
                            "lateral tracking",
                            "24mm",
                            "沈知遥冲进雨夜车站，灯光一盏盏熄灭。她在最后一节车厢玻璃上看见童年的自己。".to_string(),
                            "沈知遥（喘息）：别关门——".to_string(),
                            vec!["沈知遥"],
                            "列车制动、急促脚步",
                        ),
                        4 => (
                            "交换条件",
                            "over-the-shoulder",
                            "40mm",
                            "阿箬把一枚旧车票推到顾临川面前。两人都不看对方，镜头停在车票背面的日期：十年前。".to_string(),
                            "阿箬：你要查真相，她要活下去。你只能选一个。".to_string(),
                            vec!["阿箬", "顾临川"],
                            "纸张摩擦、雨声压低",
                        ),
                        _ => (
                            "未寄出的信",
                            "slow dolly out",
                            "85mm",
                            "沈知遥拆开一封写给自己的信，信纸空白，只有夹在里面的银白发丝。她抬头，窗外的雨停了。".to_string(),
                            "沈知遥：原来我早就来过这里。".to_string(),
                            vec!["沈知遥"],
                            "雨停后的滴水声、远处钟声",
                        ),
                    };
                let prompt = format!(
                    "{}. {}. {}, {}. Cinematic Chinese anime. Palette: {}. Lighting: {}. No on-screen text.",
                    action, shot_title, camera, lens, style.palette, style.lighting
                );
                shots.push(GeneratedShot {
                    title: format!("S{}-{} {}", scene_index + 1, shot_index + 1, shot_title),
                    description: action,
                    camera: camera.into(),
                    lens: lens.into(),
                    duration_sec: 4 + (shot_index as i64 % 3),
                    dialogue,
                    sound: sound.into(),
                    characters: cast.into_iter().map(String::from).collect(),
                    prompt,
                    negative_prompt: style.negative_prompt.clone(),
                });
            }
            scenes.push(GeneratedScene {
                title: format!("第{}场 · {}", scene_index + 1, location),
                slugline: format!(
                    "{} {} - {}",
                    if scene_index % 2 == 0 { "EXT." } else { "INT." },
                    location,
                    time
                ),
                location: location.into(),
                time_of_day: time.into(),
                synopsis: format!(
                    "{}。沈知遥追踪钥匙的来历，顾临川带来一段不该存在的录音，阿箬则守着更大的秘密。",
                    premise
                ),
                script_text: shots
                    .iter()
                    .map(|s| format!("{}\n{}\n", s.description, s.dialogue))
                    .collect::<Vec<_>>()
                    .join("\n"),
                shots,
            });
        }
        episodes.push(GeneratedEpisode {
            title: format!(
                "第{}集 · {}",
                episode_index + 1,
                if episode_index == 0 {
                    "雨中的钥匙"
                } else {
                    "被剪掉的人"
                }
            ),
            summary: format!(
                "第{}集：{} 沈知遥与顾临川被迫结盟，阿箬的条件揭开了十年前失踪案的第一个缺口。",
                episode_index + 1,
                premise
            ),
            scenes,
        });
    }
    GeneratedScript {
        title,
        logline: format!(
            "{} 一件旧物让她听见过去，也让她的现在一点点消失。",
            premise
        ),
        synopsis: format!(
            "在常年落雨的海临城，记忆修复师沈知遥触碰旧物时能读到残留情绪。她接手一枚没有主人的铜钥匙，却发现钥匙里存着自己的声音。调查记者顾临川带着一段十年前失踪案的录音出现，旧物店老板阿箬则要求她停下。每一次读取都让沈知遥失去一段记忆，而真相正藏在她忘掉的那部分里。"
        ),
        style_bible: style,
        characters,
        episodes,
    }
}
