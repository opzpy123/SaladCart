pub mod assets;
pub mod characters;
pub mod jobs;
mod mappers;
pub mod novel;
pub mod script;
pub mod timeline;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::paths::AppPaths;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use std::path::{Path, PathBuf};

pub use mappers::now_iso;

#[derive(Debug)]
pub struct ProjectStore {
    pub paths: AppPaths,
    pub project_id: String,
    pub dir: PathBuf,
    pub conn: Connection,
}

pub fn dimensions_for(aspect: &str) -> (i64, i64) {
    match aspect {
        "16:9" => (1920, 1080),
        "1:1" => (1080, 1080),
        "4:5" => (1080, 1350),
        _ => (1080, 1920),
    }
}

pub fn normalize_aspect(aspect: &str) -> String {
    match aspect {
        "16:9" | "1:1" | "4:5" | "9:16" => aspect.to_string(),
        _ => "9:16".to_string(),
    }
}

pub fn preset_style(name: Option<&str>) -> StyleBible {
    match name.unwrap_or("anime-cel") {
        "cinematic-3d" => StyleBible {
            visual_style: "cinematic 3D animation, expressive stylized characters, physically based rendering".into(),
            palette: "deep teal shadows, warm amber practical lights, muted crimson accents".into(),
            lighting: "dramatic volumetric lighting, soft rim light, shallow depth of field".into(),
            camera_language: "35mm film language, slow push-ins, low-angle hero frames, clean eyeline cuts".into(),
            negative_prompt: "flat lighting, inconsistent face, extra fingers, text, watermark, low quality".into(),
            reference_notes: "Keep facial structure and costume details identical across every shot.".into(),
        },
        "guofeng-ink" => StyleBible {
            visual_style: "Chinese ink-wash animation blended with modern cel shading, elegant negative space".into(),
            palette: "rice-paper ivory, ink black, cinnabar red, jade green".into(),
            lighting: "soft diffused daylight, misty backlight, painterly gradients".into(),
            camera_language: "horizontal scroll composition, deliberate wide shots, whip-pan only for action".into(),
            negative_prompt: "photorealism, cluttered composition, neon colors, text, watermark".into(),
            reference_notes: "Preserve ink texture and character silhouette; avoid over-detailed rendering.".into(),
        },
        "manga-noir" => StyleBible {
            visual_style: "high-contrast manga noir, heavy ink shadows, halftone texture, graphic framing".into(),
            palette: "black, bone white, one saturated spotlight color per episode".into(),
            lighting: "hard chiaroscuro, silhouettes, isolated pools of light".into(),
            camera_language: "Dutch angles, tight close-ups, dramatic negative space".into(),
            negative_prompt: "soft gradients, pastel colors, flat lighting, text, watermark".into(),
            reference_notes: "Maintain consistent hatching density and graphic shadow shapes.".into(),
        },
        _ => StyleBible {
            visual_style: "premium Chinese anime series, clean cel shading, expressive faces, detailed costumes".into(),
            palette: "warm sunlit neutrals, emerald and teal accents, vermilion highlights".into(),
            lighting: "soft cinematic light, warm-cool separation, gentle bloom".into(),
            camera_language: "storyboard-driven compositions, 24–35mm equivalents, motivated camera moves".into(),
            negative_prompt: "photorealistic, 3D render, deformed hands, inconsistent face, text, watermark, low resolution".into(),
            reference_notes: "Each character must keep identical hair, face shape, costume and color palette in every frame.".into(),
        },
    }
}

impl ProjectStore {
    pub fn create(paths: &AppPaths, input: &CreateProjectInput) -> AppResult<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let dir = paths.project_dir(&id)?;
        for sub in [
            "assets/images",
            "assets/videos",
            "assets/audio",
            "thumbs",
            "exports",
        ] {
            std::fs::create_dir_all(dir.join(sub))?;
        }
        let db_path = dir.join("project.db");
        let conn = db::open(&db_path)?;
        let now = now_iso();
        let aspect = normalize_aspect(&input.aspect_ratio);
        let style = input
            .style_bible
            .clone()
            .unwrap_or_else(|| preset_style(input.style_preset.as_deref()));
        let name = if input.name.trim().is_empty() {
            "未命名漫剧".to_string()
        } else {
            input.name.trim().to_string()
        };
        let language = if input.language.is_empty() {
            "zh-CN"
        } else {
            input.language.as_str()
        };
        conn.execute(
            "INSERT INTO project (id,name,premise,logline,genre,tone,language,aspect_ratio,target_episode_seconds,budget_cap,style_json,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12)",
            params![
                id,
                name,
                input.premise.trim(),
                "",
                &input.genre,
                &input.tone,
                language,
                aspect,
                input.target_episode_seconds.clamp(15, 600),
                input.budget_cap.max(0.0),
                serde_json::to_string(&style)?,
                now
            ],
        )?;
        let (width, height) = dimensions_for(&aspect);
        conn.execute(
            "INSERT INTO timeline (id,name,aspect_ratio,width,height,fps,updated_at) VALUES (?1,'Main cut',?2,?3,?4,30,?5)",
            params![uuid::Uuid::new_v4().to_string(), aspect, width, height, now],
        )?;
        let store = Self {
            paths: paths.clone(),
            project_id: id,
            dir,
            conn,
        };
        store.write_project_json()?;
        Ok(store)
    }

    pub fn open(paths: &AppPaths, project_id: &str) -> AppResult<Self> {
        let dir = paths.project_dir(project_id)?;
        let db_path = dir.join("project.db");
        if !db_path.exists() {
            return Err(AppError::NotFound(format!("project {project_id}")));
        }
        let conn = db::open(&db_path)?;
        let exists: Option<i64> = conn
            .query_row("SELECT 1 FROM project LIMIT 1", [], |row| row.get(0))
            .optional()?;
        if exists.is_none() {
            return Err(AppError::NotFound(format!("project {project_id}")));
        }
        for sub in [
            "assets/images",
            "assets/videos",
            "assets/audio",
            "thumbs",
            "exports",
        ] {
            std::fs::create_dir_all(dir.join(sub))?;
        }
        let store = Self {
            paths: paths.clone(),
            project_id: project_id.to_string(),
            dir,
            conn,
        };
        let has_timeline: Option<i64> = store
            .conn
            .query_row("SELECT 1 FROM timeline LIMIT 1", [], |row| row.get(0))
            .optional()?;
        if has_timeline.is_none() {
            let project = store.project()?;
            let (w, h) = dimensions_for(&project.aspect_ratio);
            store.conn.execute(
                "INSERT INTO timeline (id,name,aspect_ratio,width,height,fps,updated_at) VALUES (?1,'Main cut',?2,?3,?4,30,?5)",
                params![uuid::Uuid::new_v4().to_string(), project.aspect_ratio, w, h, now_iso()],
            )?;
        }
        Ok(store)
    }

    pub fn project(&self) -> AppResult<Project> {
        self.conn
            .query_row("SELECT * FROM project LIMIT 1", [], mappers::map_project)
            .map_err(Into::into)
    }

    pub fn update_project(&mut self, input: UpdateProjectInput) -> AppResult<Project> {
        let mut project = self.project()?;
        if let Some(v) = input.name {
            if !v.trim().is_empty() {
                project.name = v.trim().to_string();
            }
        }
        if let Some(v) = input.premise {
            project.premise = v;
        }
        if let Some(v) = input.logline {
            project.logline = v;
        }
        if let Some(v) = input.genre {
            project.genre = v;
        }
        if let Some(v) = input.tone {
            project.tone = v;
        }
        if let Some(v) = input.language {
            project.language = v;
        }
        if let Some(v) = input.aspect_ratio {
            project.aspect_ratio = normalize_aspect(&v);
        }
        if let Some(v) = input.target_episode_seconds {
            project.target_episode_seconds = v.clamp(15, 600);
        }
        if let Some(v) = input.budget_cap {
            project.budget_cap = v.max(0.0);
        }
        if let Some(v) = input.style_bible {
            project.style_bible = v;
        }
        project.updated_at = now_iso();
        self.conn.execute(
            "UPDATE project SET name=?1,premise=?2,logline=?3,genre=?4,tone=?5,language=?6,aspect_ratio=?7,target_episode_seconds=?8,budget_cap=?9,style_json=?10,updated_at=?11",
            params![
                project.name,
                project.premise,
                project.logline,
                project.genre,
                project.tone,
                project.language,
                project.aspect_ratio,
                project.target_episode_seconds,
                project.budget_cap,
                serde_json::to_string(&project.style_bible)?,
                project.updated_at
            ],
        )?;
        self.write_project_json()?;
        Ok(project)
    }

    pub fn counts(&self) -> AppResult<ProjectCounts> {
        let one =
            |sql: &str| -> AppResult<i64> { Ok(self.conn.query_row(sql, [], |row| row.get(0))?) };
        Ok(ProjectCounts {
            characters: one("SELECT COUNT(*) FROM characters")?,
            episodes: one("SELECT COUNT(*) FROM episodes")?,
            scenes: one("SELECT COUNT(*) FROM scenes")?,
            shots: one("SELECT COUNT(*) FROM shots")?,
            videos: one("SELECT COUNT(*) FROM assets WHERE kind='video'")?,
            images: one("SELECT COUNT(*) FROM assets WHERE kind='image'")?,
        })
    }

    pub fn summary(&self) -> AppResult<ProjectSummary> {
        let project = self.project()?;
        let counts = self.counts()?;
        let cover: Option<(Option<String>, String)> = self
            .conn
            .query_row(
                "SELECT thumb_rel_path, rel_path FROM assets WHERE kind IN ('image','video')
                 ORDER BY CASE WHEN thumb_rel_path IS NULL THEN 1 ELSE 0 END, created_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(ProjectSummary {
            id: project.id,
            name: project.name,
            logline: project.logline,
            premise: project.premise,
            genre: project.genre,
            aspect_ratio: project.aspect_ratio,
            updated_at: project.updated_at,
            cover_path: cover.map(|(thumb, rel)| {
                self.dir
                    .join(thumb.unwrap_or(rel))
                    .to_string_lossy()
                    .to_string()
            }),
            storage_path: self.dir.to_string_lossy().to_string(),
            counts,
        })
    }

    pub fn snapshot(&self) -> AppResult<ProjectSnapshot> {
        Ok(ProjectSnapshot {
            project: self.project()?,
            characters: self.list_characters()?,
            episodes: self.list_episodes()?,
            assets: self.list_assets(None)?,
            jobs: self.list_jobs(80)?,
            timeline: self.timeline()?,
            novel: self.novel_summary()?,
            storage_path: self.dir.to_string_lossy().to_string(),
        })
    }

    pub fn write_project_json(&self) -> AppResult<()> {
        let project = self.project()?;
        let body = json!({
            "schemaVersion": db::SCHEMA_VERSION,
            "id": project.id,
            "name": project.name,
            "createdAt": project.created_at,
            "updatedAt": project.updated_at,
            "app": "SaladCart"
        });
        std::fs::write(
            self.dir.join("project.json"),
            serde_json::to_string_pretty(&body)?,
        )?;
        Ok(())
    }

    pub fn touch(&self) -> AppResult<()> {
        self.conn
            .execute("UPDATE project SET updated_at=?1", params![now_iso()])?;
        Ok(())
    }

    pub fn import_style_from(&mut self, source: &ProjectStore) -> AppResult<Project> {
        let style = source.project()?.style_bible;
        self.update_project(UpdateProjectInput {
            style_bible: Some(style),
            ..Default::default()
        })
    }

    pub fn asset_abs(&self, rel: &str) -> PathBuf {
        self.dir.join(rel)
    }
}

pub fn list_summaries(paths: &AppPaths) -> AppResult<Vec<ProjectSummary>> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(&paths.projects_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(out),
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(id) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if let Ok(store) = ProjectStore::open(paths, &id) {
            if let Ok(summary) = store.summary() {
                out.push(summary);
            }
        }
    }
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(out)
}

pub fn archive_project(paths: &AppPaths, project_id: &str) -> AppResult<()> {
    let dir = paths.project_dir(project_id)?;
    if !dir.exists() {
        return Err(AppError::NotFound(format!("project {project_id}")));
    }
    let stamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let target = paths.trash_dir.join(format!("{project_id}-{stamp}"));
    std::fs::rename(&dir, &target).map_err(|e| {
        AppError::Message(format!(
            "could not move project to trash ({}): {e}",
            target.display()
        ))
    })?;
    Ok(())
}

pub fn project_dir_for(paths: &AppPaths, project_id: &str) -> AppResult<PathBuf> {
    let dir = paths.project_dir(project_id)?;
    if !dir.exists() {
        return Err(AppError::NotFound(format!("project {project_id}")));
    }
    Ok(dir)
}

pub fn ensure_asset_rel(rel: &str) -> AppResult<()> {
    let p = Path::new(rel);
    if p.is_absolute()
        || p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(AppError::Invalid("illegal asset path".into()));
    }
    Ok(())
}

pub fn get_project_snapshot(paths: &AppPaths, project_id: &str) -> AppResult<ProjectSnapshot> {
    ProjectStore::open(paths, project_id)?.snapshot()
}

pub fn require_project(paths: &AppPaths, project_id: &str) -> AppResult<ProjectStore> {
    ProjectStore::open(paths, project_id).map_err(|e| match e {
        AppError::NotFound(_) => AppError::NotFound(format!("project {project_id}")),
        other => other,
    })
}

pub fn recover_interrupted_jobs(paths: &AppPaths) -> AppResult<usize> {
    let mut count = 0;
    for entry in std::fs::read_dir(&paths.projects_dir)?.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(id) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if let Ok(store) = ProjectStore::open(paths, &id) {
            count += store.conn.execute(
                "UPDATE jobs SET status='failed', error='App closed before this job finished', finished_at=?1
                 WHERE status IN ('queued','running')",
                params![now_iso()],
            )?;
        }
    }
    Ok(count)
}

pub fn query_asset_paths(
    store: &ProjectStore,
    ids: &[String],
) -> AppResult<Vec<(String, String, String)>> {
    let mut result = Vec::new();
    for id in ids {
        if let Ok(asset) = store.get_asset(id) {
            result.push((asset.id, asset.name, asset.abs_path));
        }
    }
    Ok(result)
}
