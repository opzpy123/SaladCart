use crate::error::{AppError, AppResult, OptionExt};
use crate::models::*;
use crate::store::mappers::{map_asset, now_iso};
use crate::store::{ProjectStore, ensure_asset_rel};
use rusqlite::{params, OptionalExtension};

pub fn asset_subdir(kind: &str) -> &'static str {
    match kind {
        "video" => "assets/videos",
        "audio" => "assets/audio",
        _ => "assets/images",
    }
}

pub struct NewAsset {
    pub kind: String,
    pub name: String,
    pub rel_path: String,
    pub thumb_rel_path: Option<String>,
    pub mime: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration_ms: Option<i64>,
    pub source: String,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub seed: Option<i64>,
    pub version: i64,
    pub parent_asset_id: Option<String>,
    pub license: Option<String>,
    pub license_url: Option<String>,
    pub attribution: Option<String>,
    pub source_url: Option<String>,
    pub status: String,
}

impl ProjectStore {
    pub fn insert_asset(&mut self, new: NewAsset) -> AppResult<Asset> {
        ensure_asset_rel(&new.rel_path)?;
        let path = self.asset_abs(&new.rel_path);
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "asset file does not exist: {}",
                path.display()
            )));
        }
        let size = std::fs::metadata(&path).ok().map(|m| m.len() as i64);
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_iso();
        self.conn.execute(
            "INSERT INTO assets
             (id,kind,name,rel_path,thumb_rel_path,mime,width,height,duration_ms,size_bytes,source,provider_id,model,prompt,negative_prompt,seed,version,parent_asset_id,license,license_url,attribution,source_url,status,created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)",
            params![
                id,
                new.kind,
                new.name,
                new.rel_path,
                new.thumb_rel_path,
                new.mime,
                new.width,
                new.height,
                new.duration_ms,
                size,
                new.source,
                new.provider_id,
                new.model,
                new.prompt,
                new.negative_prompt,
                new.seed,
                new.version,
                new.parent_asset_id,
                new.license,
                new.license_url,
                new.attribution,
                new.source_url,
                new.status,
                now
            ],
        )?;
        self.touch()?;
        self.get_asset(&id)
    }

    pub fn get_asset(&self, asset_id: &str) -> AppResult<Asset> {
        self.conn
            .query_row(
                "SELECT * FROM assets WHERE id=?1",
                params![asset_id],
                |row| map_asset(row, &self.dir),
            )
            .optional()?
            .or_not_found("asset")
    }

    pub fn list_assets(&self, kind: Option<&str>) -> AppResult<Vec<Asset>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM assets WHERE (?1 IS NULL OR kind = ?1) ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![kind], |row| map_asset(row, &self.dir))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn latest_asset_for_target(
        &self,
        kind: &str,
        target_type: &str,
        target_id: &str,
    ) -> AppResult<Option<Asset>> {
        // Provenance is stored on the asset prompt lineage; shot/character pointer
        // is the authoritative fast path.
        let asset_id: Option<String> = match target_type {
            "character" => {
                let column = if kind == "video" { return Ok(None) } else { "reference_asset_id" };
                self.conn.query_row(
                    &format!("SELECT {column} FROM characters WHERE id=?1"),
                    params![target_id],
                    |row| row.get(0),
                ).optional()?.flatten()
            }
            "shot" => {
                let column = if kind == "video" { "video_asset_id" } else { "image_asset_id" };
                self.conn.query_row(
                    &format!("SELECT {column} FROM shots WHERE id=?1"),
                    params![target_id],
                    |row| row.get(0),
                ).optional()?.flatten()
            }
            _ => None,
        };
        match asset_id {
            Some(id) => Ok(Some(self.get_asset(&id)?)),
            None => Ok(None),
        }
    }

    pub fn set_asset_status(&self, asset_id: &str, status: &str) -> AppResult<Asset> {
        self.conn.execute(
            "UPDATE assets SET status=?1 WHERE id=?2",
            params![status, asset_id],
        )?;
        self.get_asset(asset_id)
    }

    pub fn set_asset_probe(
        &self,
        asset_id: &str,
        width: Option<i64>,
        height: Option<i64>,
        duration_ms: Option<i64>,
        thumb_rel_path: Option<String>,
    ) -> AppResult<Asset> {
        self.conn.execute(
            "UPDATE assets SET width=COALESCE(?1,width),height=COALESCE(?2,height),duration_ms=COALESCE(?3,duration_ms),thumb_rel_path=COALESCE(?4,thumb_rel_path) WHERE id=?5",
            params![width, height, duration_ms, thumb_rel_path, asset_id],
        )?;
        self.get_asset(asset_id)
    }

    pub fn import_asset(&mut self, input: ImportAssetInput) -> AppResult<Asset> {
        let source = std::path::Path::new(&input.source_path);
        if !source.is_file() {
            return Err(AppError::NotFound(format!(
                "file {}",
                input.source_path
            )));
        }
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin")
            .to_lowercase();
        let kind = if !input.kind.is_empty() {
            input.kind
        } else {
            match ext.as_str() {
                "mp4" | "mov" | "webm" | "m4v" => "video",
                "mp3" | "wav" | "m4a" | "aac" | "flac" => "audio",
                _ => "image",
            }
            .to_string()
        };
        let sub = asset_subdir(&kind);
        let rel = format!("{sub}/{}.{ext}", uuid::Uuid::new_v4());
        let dest = self.asset_abs(&rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source, &dest)?;
        let (width, height) = if kind == "image" {
            image::image_dimensions(&dest)
                .map(|(w, h)| (Some(w as i64), Some(h as i64)))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };
        let mime = mime_guess::from_path(&dest)
            .first_or_octet_stream()
            .to_string();
        let name = if input.name.trim().is_empty() {
            source
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("素材")
                .to_string()
        } else {
            input.name
        };
        let asset = self.insert_asset(NewAsset {
            kind,
            name,
            rel_path: rel,
            thumb_rel_path: None,
            mime,
            width,
            height,
            duration_ms: None,
            source: "upload".into(),
            provider_id: None,
            model: None,
            prompt: None,
            negative_prompt: None,
            seed: None,
            version: 1,
            parent_asset_id: None,
            license: None,
            license_url: None,
            attribution: None,
            source_url: None,
            status: "ready".into(),
        })?;
        self.touch()?;
        Ok(asset)
    }

    pub fn asset_public_url(&self, asset: &Asset) -> String {
        asset.abs_path.clone()
    }
}
