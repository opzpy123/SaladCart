use crate::error::{AppError, AppResult, OptionExt};
use crate::models::*;
use crate::store::mappers::{map_scene, map_shot, now_iso};
use crate::store::ProjectStore;
use rusqlite::{params, OptionalExtension};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ScriptApplyStats {
    pub characters_created: usize,
    pub episodes: usize,
    pub scenes: usize,
    pub shots: usize,
}

impl ProjectStore {
    pub fn list_episodes(&self) -> AppResult<Vec<Episode>> {
        let mut episode_stmt = self
            .conn
            .prepare("SELECT * FROM episodes ORDER BY order_index ASC")?;
        let mut episodes: Vec<Episode> = {
            let rows = episode_stmt.query_map([], |row| {
                Ok(Episode {
                    id: row.get("id")?,
                    title: row.get("title")?,
                    summary: row.get("summary")?,
                    order_index: row.get("order_index")?,
                    scenes: Vec::new(),
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                })
            })?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            out
        };

        let mut scene_stmt = self
            .conn
            .prepare("SELECT * FROM scenes ORDER BY order_index ASC")?;
        let mut scenes: Vec<Scene> = {
            let rows = scene_stmt.query_map([], map_scene)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            out
        };

        let mut shot_stmt = self
            .conn
            .prepare("SELECT * FROM shots ORDER BY order_index ASC")?;
        let mut shots: Vec<Shot> = {
            let rows = shot_stmt.query_map([], map_shot)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            out
        };

        let mut links: HashMap<String, Vec<String>> = HashMap::new();
        let mut link_stmt = self
            .conn
            .prepare("SELECT shot_id, character_id FROM shot_characters")?;
        let rows = link_stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (shot_id, character_id) = row?;
            links.entry(shot_id).or_default().push(character_id);
        }
        for shot in &mut shots {
            if let Some(ids) = links.remove(&shot.id) {
                shot.character_ids = ids;
            }
        }

        let mut shots_by_scene: HashMap<String, Vec<Shot>> = HashMap::new();
        for shot in shots {
            shots_by_scene
                .entry(shot.scene_id.clone())
                .or_default()
                .push(shot);
        }
        for scene in &mut scenes {
            if let Some(mut list) = shots_by_scene.remove(&scene.id) {
                list.sort_by_key(|s| s.order_index);
                scene.shots = list;
            }
        }
        let mut scenes_by_episode: HashMap<String, Vec<Scene>> = HashMap::new();
        for scene in scenes {
            scenes_by_episode
                .entry(scene.episode_id.clone())
                .or_default()
                .push(scene);
        }
        for episode in &mut episodes {
            if let Some(mut list) = scenes_by_episode.remove(&episode.id) {
                list.sort_by_key(|s| s.order_index);
                episode.scenes = list;
            }
        }
        Ok(episodes)
    }

    pub fn get_scene(&self, scene_id: &str) -> AppResult<Scene> {
        let mut scene = self
            .conn
            .query_row(
                "SELECT * FROM scenes WHERE id=?1",
                params![scene_id],
                map_scene,
            )
            .optional()?
            .or_not_found("scene")?;
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM shots WHERE scene_id=?1 ORDER BY order_index ASC")?;
        let rows = stmt.query_map(params![scene_id], map_shot)?;
        for row in rows {
            scene.shots.push(row?);
        }
        let links = self.shot_character_links()?;
        for shot in &mut scene.shots {
            if let Some(ids) = links.get(&shot.id) {
                shot.character_ids = ids.clone();
            }
        }
        Ok(scene)
    }

    pub fn get_shot(&self, shot_id: &str) -> AppResult<Shot> {
        let mut shot = self
            .conn
            .query_row(
                "SELECT * FROM shots WHERE id=?1",
                params![shot_id],
                map_shot,
            )
            .optional()?
            .or_not_found("shot")?;
        let mut stmt = self
            .conn
            .prepare("SELECT character_id FROM shot_characters WHERE shot_id=?1")?;
        let rows = stmt.query_map(params![shot_id], |row| row.get::<_, String>(0))?;
        for row in rows {
            shot.character_ids.push(row?);
        }
        Ok(shot)
    }

    fn shot_character_links(&self) -> AppResult<HashMap<String, Vec<String>>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let mut stmt = self
            .conn
            .prepare("SELECT shot_id, character_id FROM shot_characters")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (shot_id, character_id) = row?;
            map.entry(shot_id).or_default().push(character_id);
        }
        Ok(map)
    }

    pub fn shot_scene(&self, shot_id: &str) -> AppResult<Scene> {
        self.conn
            .query_row(
                "SELECT sc.* FROM scenes sc JOIN shots s ON s.scene_id=sc.id WHERE s.id=?1",
                params![shot_id],
                map_scene,
            )
            .optional()?
            .or_not_found("scene for shot")
    }

    pub fn episode_for_scene(&self, scene_id: &str) -> AppResult<Episode> {
        self.conn
            .query_row(
                "SELECT e.* FROM episodes e JOIN scenes sc ON sc.episode_id=e.id WHERE sc.id=?1",
                params![scene_id],
                |row| {
                    Ok(Episode {
                        id: row.get("id")?,
                        title: row.get("title")?,
                        summary: row.get("summary")?,
                        order_index: row.get("order_index")?,
                        scenes: Vec::new(),
                        created_at: row.get("created_at")?,
                        updated_at: row.get("updated_at")?,
                    })
                },
            )
            .optional()?
            .or_not_found("episode for scene")
    }

    pub fn shots_in_order(&self) -> AppResult<Vec<Shot>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.* FROM shots s
             JOIN scenes sc ON sc.id=s.scene_id
             JOIN episodes e ON e.id=sc.episode_id
             ORDER BY e.order_index ASC, sc.order_index ASC, s.order_index ASC",
        )?;
        let rows = stmt.query_map([], map_shot)?;
        let links = self.shot_character_links()?;
        let mut out = Vec::new();
        for row in rows {
            let mut shot = row?;
            if let Some(ids) = links.get(&shot.id) {
                shot.character_ids = ids.clone();
            }
            out.push(shot);
        }
        Ok(out)
    }

    pub fn previous_shot(&self, shot_id: &str) -> AppResult<Option<Shot>> {
        let shots = self.shots_in_order()?;
        let idx = shots.iter().position(|s| s.id == shot_id);
        Ok(idx.and_then(|i| {
            if i == 0 {
                None
            } else {
                Some(shots[i - 1].clone())
            }
        }))
    }

    pub fn apply_generated_script(
        &mut self,
        generated: GeneratedScript,
        request: &GenerateScriptRequest,
    ) -> AppResult<ScriptApplyStats> {
        if !request.replace_existing {
            let existing: i64 =
                self.conn
                    .query_row("SELECT COUNT(*) FROM episodes", [], |row| row.get(0))?;
            if existing > 0 {
                return Err(AppError::Invalid(
                    "project already has episodes; enable replaceExisting to regenerate".into(),
                ));
            }
        }
        let project = self.project()?;
        let tx = self.conn.transaction()?;
        let now = now_iso();
        let mut stats = ScriptApplyStats::default();

        let mut character_map: HashMap<String, String> = HashMap::new();
        {
            let mut stmt = tx.prepare("SELECT id,name FROM characters")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (id, name) = row?;
                character_map.insert(name.trim().to_lowercase(), id);
            }
        }
        for character in &generated.characters {
            let key = character.name.trim().to_lowercase();
            if key.is_empty() || character_map.contains_key(&key) {
                continue;
            }
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO characters
                 (id,name,role,description,appearance,personality,voice,negative_prompt,locked_traits,provider_id,model,status,created_at,updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,'',?8,'','','draft',?9,?9)",
                params![
                    id,
                    character.name.trim(),
                    character.role,
                    character.description,
                    character.appearance,
                    character.personality,
                    character.voice,
                    serde_json::to_string(&character.locked_traits)?,
                    now
                ],
            )?;
            character_map.insert(key, id);
            stats.characters_created += 1;
        }

        // Unknown cast names are promoted to lightweight characters so that
        // generated shots never lose their subject.
        for episode in &generated.episodes {
            for scene in &episode.scenes {
                for shot in &scene.shots {
                    for name in &shot.characters {
                        let key = name.trim().to_lowercase();
                        if key.is_empty() || character_map.contains_key(&key) {
                            continue;
                        }
                        let id = uuid::Uuid::new_v4().to_string();
                        tx.execute(
                            "INSERT INTO characters (id,name,role,provider_id,model,status,created_at,updated_at)
                             VALUES (?1,?2,'supporting','','','draft',?3,?3)",
                            params![id, name.trim(), now],
                        )?;
                        character_map.insert(key, id);
                        stats.characters_created += 1;
                    }
                }
            }
        }

        tx.execute("DELETE FROM timeline_clips", [])?;
        tx.execute("DELETE FROM subtitles", [])?;
        tx.execute("DELETE FROM episodes", [])?;

        let mut previous_shot: Option<String> = None;
        for (ep_idx, episode) in generated.episodes.iter().enumerate() {
            let episode_id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO episodes (id,title,summary,order_index,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
                params![episode_id, episode.title, episode.summary, ep_idx as i64, now],
            )?;
            stats.episodes += 1;
            for (scene_idx, scene) in episode.scenes.iter().enumerate() {
                let scene_id = uuid::Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO scenes (id,episode_id,title,slugline,location,time_of_day,synopsis,script_text,order_index,status,created_at,updated_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'draft',?10,?10)",
                    params![
                        scene_id,
                        episode_id,
                        scene.title,
                        scene.slugline,
                        scene.location,
                        scene.time_of_day,
                        scene.synopsis,
                        if scene.script_text.trim().is_empty() { &scene.synopsis } else { &scene.script_text },
                        scene_idx as i64,
                        now
                    ],
                )?;
                stats.scenes += 1;
                for (shot_idx, shot) in scene.shots.iter().enumerate() {
                    let shot_id = uuid::Uuid::new_v4().to_string();
                    let duration_ms = (shot.duration_sec.clamp(1, 30)) * 1000;
                    let prompt = if shot.prompt.trim().is_empty() {
                        format!(
                            "{}. {}. Camera: {}. {}",
                            shot.description, scene.synopsis, shot.camera, shot.lens
                        )
                    } else {
                        shot.prompt.clone()
                    };
                    let negative = if shot.negative_prompt.trim().is_empty() {
                        generated.style_bible.negative_prompt.clone()
                    } else {
                        shot.negative_prompt.clone()
                    };
                    tx.execute(
                        "INSERT INTO shots
                         (id,scene_id,order_index,title,description,prompt,negative_prompt,camera,lens,duration_ms,dialogue,sound,continuity_from_shot_id,status,provider_id,model,version,created_at,updated_at)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,'draft','','',1,?14,?14)",
                        params![
                            shot_id,
                            scene_id,
                            shot_idx as i64,
                            shot.title,
                            shot.description,
                            prompt,
                            negative,
                            shot.camera,
                            shot.lens,
                            duration_ms,
                            shot.dialogue,
                            shot.sound,
                            previous_shot,
                            now
                        ],
                    )?;
                    for name in &shot.characters {
                        if let Some(character_id) = character_map.get(&name.trim().to_lowercase()) {
                            tx.execute(
                                "INSERT OR IGNORE INTO shot_characters (shot_id,character_id) VALUES (?1,?2)",
                                params![shot_id, character_id],
                            )?;
                        }
                    }
                    previous_shot = Some(shot_id);
                    stats.shots += 1;
                }
            }
        }

        let style = if generated.style_bible.visual_style.trim().is_empty() {
            project.style_bible.clone()
        } else {
            generated.style_bible.clone()
        };
        tx.execute(
            "UPDATE project SET
                name=CASE WHEN name='' OR name='未命名漫剧' THEN ?1 ELSE name END,
                logline=CASE WHEN ?2='' THEN logline ELSE ?2 END,
                style_json=?3,
                aspect_ratio=?4,
                updated_at=?5",
            params![
                generated.title,
                generated.logline,
                serde_json::to_string(&style)?,
                project.aspect_ratio,
                now
            ],
        )?;
        tx.commit()?;
        self.write_project_json()?;
        Ok(stats)
    }

    pub fn update_scene(&mut self, scene_id: &str, input: UpdateSceneInput) -> AppResult<Scene> {
        let mut scene = self.get_scene(scene_id)?;
        if let Some(v) = input.title {
            scene.title = v;
        }
        if let Some(v) = input.slugline {
            scene.slugline = v;
        }
        if let Some(v) = input.location {
            scene.location = v;
        }
        if let Some(v) = input.time_of_day {
            scene.time_of_day = v;
        }
        if let Some(v) = input.synopsis {
            scene.synopsis = v;
        }
        if let Some(v) = input.script_text {
            scene.script_text = v;
        }
        if let Some(v) = input.status {
            scene.status = v;
        }
        scene.updated_at = now_iso();
        self.conn.execute(
            "UPDATE scenes SET title=?1,slugline=?2,location=?3,time_of_day=?4,synopsis=?5,script_text=?6,status=?7,updated_at=?8 WHERE id=?9",
            params![
                scene.title,
                scene.slugline,
                scene.location,
                scene.time_of_day,
                scene.synopsis,
                scene.script_text,
                scene.status,
                scene.updated_at,
                scene.id
            ],
        )?;
        self.touch()?;
        self.get_scene(scene_id)
    }

    pub fn update_shot(&mut self, shot_id: &str, input: UpdateShotInput) -> AppResult<Shot> {
        let mut shot = self.get_shot(shot_id)?;
        if let Some(v) = input.title {
            shot.title = v;
        }
        if let Some(v) = input.description {
            shot.description = v;
        }
        if let Some(v) = input.prompt {
            shot.prompt = v;
        }
        if let Some(v) = input.negative_prompt {
            shot.negative_prompt = v;
        }
        if let Some(v) = input.camera {
            shot.camera = v;
        }
        if let Some(v) = input.lens {
            shot.lens = v;
        }
        if let Some(v) = input.duration_ms {
            shot.duration_ms = v.clamp(1000, 30_000);
        }
        if let Some(v) = input.dialogue {
            shot.dialogue = v;
        }
        if let Some(v) = input.sound {
            shot.sound = v;
        }
        if let Some(v) = input.continuity_from_shot_id {
            shot.continuity_from_shot_id = if v.is_empty() { None } else { Some(v) };
        }
        if input.clear_continuity {
            shot.continuity_from_shot_id = None;
        }
        if let Some(v) = input.start_frame_asset_id {
            shot.start_frame_asset_id = if v.is_empty() { None } else { Some(v) };
        }
        if let Some(v) = input.status {
            shot.status = v;
        }
        if let Some(v) = input.provider_id {
            shot.provider_id = v;
        }
        if let Some(v) = input.model {
            shot.model = v;
        }
        if let Some(v) = input.seed {
            shot.seed = Some(v);
        }
        shot.updated_at = now_iso();
        self.conn.execute(
            "UPDATE shots SET title=?1,description=?2,prompt=?3,negative_prompt=?4,camera=?5,lens=?6,duration_ms=?7,dialogue=?8,sound=?9,continuity_from_shot_id=?10,start_frame_asset_id=?11,status=?12,provider_id=?13,model=?14,seed=?15,updated_at=?16 WHERE id=?17",
            params![
                shot.title,
                shot.description,
                shot.prompt,
                shot.negative_prompt,
                shot.camera,
                shot.lens,
                shot.duration_ms,
                shot.dialogue,
                shot.sound,
                shot.continuity_from_shot_id,
                shot.start_frame_asset_id,
                shot.status,
                shot.provider_id,
                shot.model,
                shot.seed,
                shot.updated_at,
                shot.id
            ],
        )?;
        if let Some(ids) = input.character_ids {
            self.conn.execute(
                "DELETE FROM shot_characters WHERE shot_id=?1",
                params![shot_id],
            )?;
            for character_id in ids {
                self.get_character(&character_id)?;
                self.conn.execute(
                    "INSERT OR IGNORE INTO shot_characters (shot_id,character_id) VALUES (?1,?2)",
                    params![shot_id, character_id],
                )?;
            }
        }
        self.touch()?;
        self.get_shot(shot_id)
    }

    pub fn attach_shot_voice(&mut self, shot_id: &str, asset_id: &str) -> AppResult<Shot> {
        let asset = self.get_asset(asset_id)?;
        if asset.kind != "audio" {
            return Err(AppError::Invalid(
                "dialogue voice must be an audio asset".into(),
            ));
        }
        self.get_shot(shot_id)?;
        self.conn.execute(
            "UPDATE shots SET voice_asset_id=?1,updated_at=?2 WHERE id=?3",
            params![asset.id, now_iso(), shot_id],
        )?;
        self.touch()?;
        self.get_shot(shot_id)
    }

    pub fn set_shot_status(&self, shot_id: &str, status: &str) -> AppResult<Shot> {
        self.get_shot(shot_id)?;
        self.conn.execute(
            "UPDATE shots SET status=?1,updated_at=?2 WHERE id=?3",
            params![status, now_iso(), shot_id],
        )?;
        self.touch()?;
        self.get_shot(shot_id)
    }

    pub fn attach_shot_asset(
        &mut self,
        shot_id: &str,
        asset_id: &str,
        kind: &str,
        provider_id: &str,
        model: &str,
        seed: Option<i64>,
    ) -> AppResult<Shot> {
        let asset = self.get_asset(asset_id)?;
        self.get_shot(shot_id)?;
        let now = now_iso();
        if kind == "video" {
            self.conn.execute(
                "UPDATE shots SET video_asset_id=?1,status='ready',provider_id=?2,model=?3,seed=COALESCE(?4,seed),version=version+1,updated_at=?5 WHERE id=?6",
                params![asset.id, provider_id, model, seed, now, shot_id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE shots SET image_asset_id=?1,status='ready',provider_id=?2,model=?3,seed=COALESCE(?4,seed),updated_at=?5 WHERE id=?6",
                params![asset.id, provider_id, model, seed, now, shot_id],
            )?;
        }
        self.touch()?;
        self.get_shot(shot_id)
    }

    pub fn delete_shot(&mut self, shot_id: &str) -> AppResult<()> {
        self.get_shot(shot_id)?;
        self.conn.execute(
            "DELETE FROM timeline_clips WHERE shot_id=?1",
            params![shot_id],
        )?;
        self.conn
            .execute("DELETE FROM shots WHERE id=?1", params![shot_id])?;
        self.touch()?;
        Ok(())
    }

    pub fn delete_scene(&mut self, scene_id: &str) -> AppResult<()> {
        self.get_scene(scene_id)?;
        self.conn.execute(
            "DELETE FROM timeline_clips WHERE shot_id IN (SELECT id FROM shots WHERE scene_id=?1)",
            params![scene_id],
        )?;
        self.conn
            .execute("DELETE FROM scenes WHERE id=?1", params![scene_id])?;
        self.touch()?;
        Ok(())
    }
}
