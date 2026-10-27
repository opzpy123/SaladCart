use crate::error::{AppError, AppResult, OptionExt};
use crate::models::*;
use crate::store::mappers::{map_character, now_iso};
use crate::store::{ensure_asset_rel, ProjectStore};
use rusqlite::{params, OptionalExtension};

impl ProjectStore {
    pub fn list_characters(&self) -> AppResult<Vec<Character>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM characters ORDER BY created_at ASC")?;
        let rows = stmt.query_map([], map_character)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn get_character(&self, character_id: &str) -> AppResult<Character> {
        self.conn
            .query_row(
                "SELECT * FROM characters WHERE id=?1",
                params![character_id],
                map_character,
            )
            .optional()?
            .or_not_found("character")
    }

    pub fn create_character(&mut self, input: CreateCharacterInput) -> AppResult<Character> {
        if input.name.trim().is_empty() {
            return Err(AppError::Invalid("character name is required".into()));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_iso();
        self.conn.execute(
            "INSERT INTO characters
             (id,name,role,description,appearance,personality,voice,negative_prompt,locked_traits,provider_id,model,status,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,'',?8,'','','draft',?9,?9)",
            params![
                id,
                input.name.trim(),
                input.role,
                input.description,
                input.appearance,
                input.personality,
                input.voice,
                serde_json::to_string(&input.locked_traits)?,
                now
            ],
        )?;
        self.touch()?;
        self.get_character(&id)
    }

    pub fn update_character(
        &mut self,
        character_id: &str,
        input: UpdateCharacterInput,
    ) -> AppResult<Character> {
        let mut character = self.get_character(character_id)?;
        if let Some(v) = input.name {
            if !v.trim().is_empty() {
                character.name = v.trim().to_string();
            }
        }
        if let Some(v) = input.role {
            character.role = v;
        }
        if let Some(v) = input.description {
            character.description = v;
        }
        if let Some(v) = input.appearance {
            character.appearance = v;
        }
        if let Some(v) = input.personality {
            character.personality = v;
        }
        if let Some(v) = input.voice {
            character.voice = v;
        }
        if let Some(v) = input.negative_prompt {
            character.negative_prompt = v;
        }
        if let Some(v) = input.locked_traits {
            character.locked_traits = v;
        }
        if let Some(v) = input.reference_asset_id {
            self.get_asset(&v)?;
            character.reference_asset_id = Some(v);
        }
        if input.clear_reference_asset {
            character.reference_asset_id = None;
        }
        if let Some(v) = input.sheet_asset_id {
            self.get_asset(&v)?;
            character.sheet_asset_id = Some(v);
        }
        if input.clear_sheet_asset {
            character.sheet_asset_id = None;
        }
        if let Some(v) = input.seed {
            character.seed = Some(v);
        }
        if let Some(v) = input.provider_id {
            character.provider_id = v;
        }
        if let Some(v) = input.model {
            character.model = v;
        }
        if let Some(v) = input.status {
            character.status = v;
        }
        character.updated_at = now_iso();
        self.conn.execute(
            "UPDATE characters SET name=?1,role=?2,description=?3,appearance=?4,personality=?5,voice=?6,negative_prompt=?7,locked_traits=?8,reference_asset_id=?9,sheet_asset_id=?10,seed=?11,provider_id=?12,model=?13,status=?14,updated_at=?15 WHERE id=?16",
            params![
                character.name,
                character.role,
                character.description,
                character.appearance,
                character.personality,
                character.voice,
                character.negative_prompt,
                serde_json::to_string(&character.locked_traits)?,
                character.reference_asset_id,
                character.sheet_asset_id,
                character.seed,
                character.provider_id,
                character.model,
                character.status,
                character.updated_at,
                character.id
            ],
        )?;
        self.touch()?;
        self.get_character(character_id)
    }

    pub fn delete_character(&mut self, character_id: &str) -> AppResult<()> {
        self.get_character(character_id)?;
        self.conn
            .execute("DELETE FROM characters WHERE id=?1", params![character_id])?;
        self.touch()?;
        Ok(())
    }

    pub fn import_character_from(
        &mut self,
        source: &ProjectStore,
        character_id: &str,
    ) -> AppResult<Character> {
        let source_character = source.get_character(character_id)?;
        let mut name = format!("{} · 导入", source_character.name);
        let existing_names: Vec<String> = self
            .list_characters()?
            .into_iter()
            .map(|c| c.name)
            .collect();
        let mut suffix = 2;
        while existing_names.contains(&name) {
            name = format!("{} · 导入{}", source_character.name, suffix);
            suffix += 1;
        }
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_iso();
        self.conn.execute(
            "INSERT INTO characters
             (id,name,role,description,appearance,personality,voice,negative_prompt,locked_traits,seed,provider_id,model,status,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'','','draft',?11,?11)",
            params![
                id,
                name,
                source_character.role,
                source_character.description,
                source_character.appearance,
                source_character.personality,
                source_character.voice,
                source_character.negative_prompt,
                serde_json::to_string(&source_character.locked_traits)?,
                source_character.seed,
                now
            ],
        )?;

        let mut reference_asset_id = None;
        let mut sheet_asset_id = None;
        if let Some(asset_id) = &source_character.reference_asset_id {
            if let Ok(asset) = self.copy_asset_from_project(source, asset_id) {
                reference_asset_id = Some(asset.id);
            }
        }
        if let Some(asset_id) = &source_character.sheet_asset_id {
            if let Ok(asset) = self.copy_asset_from_project(source, asset_id) {
                sheet_asset_id = Some(asset.id);
            }
        }
        self.conn.execute(
            "UPDATE characters SET reference_asset_id=?1,sheet_asset_id=?2 WHERE id=?3",
            params![reference_asset_id, sheet_asset_id, id],
        )?;
        self.touch()?;
        self.get_character(&id)
    }

    pub fn attach_character_reference(
        &mut self,
        character_id: &str,
        asset_id: &str,
        as_sheet: bool,
    ) -> AppResult<Character> {
        self.get_asset(asset_id)?;
        let column = if as_sheet {
            "sheet_asset_id"
        } else {
            "reference_asset_id"
        };
        self.conn.execute(
            &format!("UPDATE characters SET {column}=?1,status='ready',updated_at=?2 WHERE id=?3"),
            params![asset_id, now_iso(), character_id],
        )?;
        self.touch()?;
        self.get_character(character_id)
    }

    pub fn character_for_shot(&self, shot_id: &str) -> AppResult<Vec<Character>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.* FROM characters c
             JOIN shot_characters sc ON sc.character_id=c.id
             WHERE sc.shot_id=?1 ORDER BY c.created_at ASC",
        )?;
        let rows = stmt.query_map(params![shot_id], map_character)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn copy_asset_from_project(
        &mut self,
        source: &ProjectStore,
        asset_id: &str,
    ) -> AppResult<Asset> {
        let asset = source.get_asset(asset_id)?;
        ensure_asset_rel(&asset.rel_path)?;
        let src = source.asset_abs(&asset.rel_path);
        if !src.exists() {
            return Err(AppError::NotFound(format!("asset file {}", src.display())));
        }
        let ext = src
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin")
            .to_string();
        let sub = crate::store::assets::asset_subdir(&asset.kind);
        let rel = format!("{sub}/{}.{ext}", uuid::Uuid::new_v4());
        let dest = self.asset_abs(&rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&src, &dest)?;
        let new_asset = crate::store::assets::NewAsset {
            kind: asset.kind,
            name: format!("{} (导入)", asset.name),
            rel_path: rel,
            thumb_rel_path: None,
            mime: asset.mime,
            width: asset.width,
            height: asset.height,
            duration_ms: asset.duration_ms,
            source: "import".into(),
            provider_id: asset.provider_id,
            model: asset.model,
            prompt: asset.prompt,
            negative_prompt: asset.negative_prompt,
            seed: asset.seed,
            version: 1,
            parent_asset_id: None,
            license: asset.license,
            license_url: asset.license_url,
            attribution: asset.attribution,
            source_url: asset.source_url,
            status: "ready".into(),
        };
        self.insert_asset(new_asset)
    }
}
