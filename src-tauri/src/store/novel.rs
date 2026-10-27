use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::store::mappers::now_iso;
use crate::store::ProjectStore;
use rusqlite::{params, OptionalExtension};

fn map_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<NovelSummary> {
    let analysis_json: String = row.get("analysis_json")?;
    Ok(NovelSummary {
        id: row.get("id")?,
        title: row.get("title")?,
        source_path: row.get("source_path")?,
        char_count: row.get("char_count")?,
        chapter_count: row.get("chapter_count")?,
        chunk_count: row.get("chunk_count")?,
        status: row.get("status")?,
        analysis: serde_json::from_str(&analysis_json).unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

impl ProjectStore {
    pub fn save_novel(
        &mut self,
        input: &NovelInput,
        chapter_count: i64,
        chunk_count: i64,
    ) -> AppResult<NovelSummary> {
        let text = input.text.trim().to_string();
        let char_count = text.chars().count();
        if char_count < 50 {
            return Err(AppError::Invalid(
                "novel text is too short; paste at least 50 characters".into(),
            ));
        }
        if char_count > 2_000_000 {
            return Err(AppError::Invalid(
                "novel text is over 2,000,000 characters; split it into volumes".into(),
            ));
        }
        let title = if input.title.trim().is_empty() {
            text.lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(|line| line.chars().take(40).collect::<String>())
                .unwrap_or_else(|| "未命名小说".into())
        } else {
            input.title.trim().to_string()
        };
        let source_path = input
            .source_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let rel = "assets/novels/novel.txt";
        let abs = self.asset_abs(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&abs, text.as_bytes())?;
        let now = now_iso();
        self.conn.execute("DELETE FROM novel_source", [])?;
        self.conn.execute(
            "INSERT INTO novel_source
             (id,title,text,source_path,char_count,chapter_count,chunk_count,status,analysis_json,created_at,updated_at)
             VALUES ('current',?1,?2,?3,?4,?5,?6,'ready','{}',?7,?7)",
            params![
                title,
                text,
                source_path,
                char_count as i64,
                chapter_count,
                chunk_count,
                now
            ],
        )?;
        self.touch()?;
        self.novel_summary()?
            .ok_or_else(|| AppError::Message("novel was not saved".into()))
    }

    pub fn novel_summary(&self) -> AppResult<Option<NovelSummary>> {
        self.conn
            .query_row(
                "SELECT * FROM novel_source WHERE id='current'",
                [],
                map_summary,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn novel_source(&self) -> AppResult<Option<NovelSource>> {
        self.conn
            .query_row("SELECT * FROM novel_source WHERE id='current'", [], |row| {
                Ok(NovelSource {
                    summary: map_summary(row)?,
                    text: row.get("text")?,
                })
            })
            .optional()
            .map_err(Into::into)
    }

    pub fn set_novel_analysis(
        &mut self,
        analysis: &serde_json::Value,
        status: &str,
    ) -> AppResult<()> {
        self.conn.execute(
            "UPDATE novel_source SET status=?1,analysis_json=?2,updated_at=?3 WHERE id='current'",
            params![status, serde_json::to_string(analysis)?, now_iso()],
        )?;
        self.touch()
    }
}
