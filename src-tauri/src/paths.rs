use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub projects_dir: PathBuf,
    pub trash_dir: PathBuf,
    pub settings_file: PathBuf,
}

impl AppPaths {
    pub fn new(root: PathBuf) -> AppResult<Self> {
        let projects_dir = root.join("projects");
        let trash_dir = root.join("trash");
        std::fs::create_dir_all(&projects_dir)?;
        std::fs::create_dir_all(&trash_dir)?;
        Ok(Self {
            settings_file: root.join("settings.json"),
            root,
            projects_dir,
            trash_dir,
        })
    }

    pub fn validate_project_id(id: &str) -> AppResult<&str> {
        if id.is_empty()
            || id.len() > 64
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AppError::Invalid("illegal project id".into()));
        }
        Ok(id)
    }

    pub fn project_dir(&self, project_id: &str) -> AppResult<PathBuf> {
        Self::validate_project_id(project_id)?;
        Ok(self.projects_dir.join(project_id))
    }

    pub fn db_path(&self, project_id: &str) -> AppResult<PathBuf> {
        Ok(self.project_dir(project_id)?.join("project.db"))
    }

    pub fn assets_dir(&self, project_id: &str) -> AppResult<PathBuf> {
        Ok(self.project_dir(project_id)?.join("assets"))
    }

    pub fn thumbs_dir(&self, project_id: &str) -> AppResult<PathBuf> {
        Ok(self.project_dir(project_id)?.join("thumbs"))
    }

    pub fn exports_dir(&self, project_id: &str) -> AppResult<PathBuf> {
        Ok(self.project_dir(project_id)?.join("exports"))
    }

    pub fn absolute_asset_path(&self, project_id: &str, rel: &str) -> AppResult<PathBuf> {
        let base = self.project_dir(project_id)?;
        let safe = Path::new(rel);
        if safe.is_absolute()
            || safe
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(AppError::Invalid("illegal asset path".into()));
        }
        Ok(base.join(safe))
    }
}

pub fn default_export_dir() -> PathBuf {
    dirs::desktop_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn expand_home(input: &str) -> PathBuf {
    let path = PathBuf::from(input.trim());
    if let Ok(rest) = path.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    path
}
