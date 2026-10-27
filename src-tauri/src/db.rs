use rusqlite::Connection;
use std::path::Path;

pub const SCHEMA_VERSION: &str = "7";

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            rusqlite::Error::InvalidParameterName(format!("cannot create database dir: {e}"))
        })?;
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS project (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            premise TEXT NOT NULL DEFAULT '',
            logline TEXT NOT NULL DEFAULT '',
            genre TEXT NOT NULL DEFAULT '',
            tone TEXT NOT NULL DEFAULT '',
            language TEXT NOT NULL DEFAULT 'zh-CN',
            aspect_ratio TEXT NOT NULL DEFAULT '9:16',
            target_episode_seconds INTEGER NOT NULL DEFAULT 60,
            budget_cap REAL NOT NULL DEFAULT 0,
            style_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            appearance TEXT NOT NULL DEFAULT '',
            personality TEXT NOT NULL DEFAULT '',
            voice TEXT NOT NULL DEFAULT '',
            negative_prompt TEXT NOT NULL DEFAULT '',
            locked_traits TEXT NOT NULL DEFAULT '[]',
            reference_asset_id TEXT,
            sheet_asset_id TEXT,
            seed INTEGER,
            provider_id TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS episodes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            order_index INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS scenes (
            id TEXT PRIMARY KEY,
            episode_id TEXT NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            slugline TEXT NOT NULL DEFAULT '',
            location TEXT NOT NULL DEFAULT '',
            time_of_day TEXT NOT NULL DEFAULT '',
            synopsis TEXT NOT NULL DEFAULT '',
            script_text TEXT NOT NULL DEFAULT '',
            order_index INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS shots (
            id TEXT PRIMARY KEY,
            scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
            order_index INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            prompt TEXT NOT NULL DEFAULT '',
            negative_prompt TEXT NOT NULL DEFAULT '',
            camera TEXT NOT NULL DEFAULT '',
            lens TEXT NOT NULL DEFAULT '',
            duration_ms INTEGER NOT NULL DEFAULT 5000,
            dialogue TEXT NOT NULL DEFAULT '',
            sound TEXT NOT NULL DEFAULT '',
            continuity_from_shot_id TEXT,
            start_frame_asset_id TEXT,
            image_asset_id TEXT,
            video_asset_id TEXT,
            voice_asset_id TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            provider_id TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            seed INTEGER,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS shot_characters (
            shot_id TEXT NOT NULL REFERENCES shots(id) ON DELETE CASCADE,
            character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
            PRIMARY KEY (shot_id, character_id)
        );

        CREATE TABLE IF NOT EXISTS assets (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            rel_path TEXT NOT NULL,
            thumb_rel_path TEXT,
            mime TEXT NOT NULL DEFAULT '',
            width INTEGER,
            height INTEGER,
            duration_ms INTEGER,
            size_bytes INTEGER,
            source TEXT NOT NULL DEFAULT 'upload',
            provider_id TEXT,
            model TEXT,
            prompt TEXT,
            negative_prompt TEXT,
            seed INTEGER,
            version INTEGER NOT NULL DEFAULT 1,
            parent_asset_id TEXT,
            license TEXT,
            license_url TEXT,
            attribution TEXT,
            source_url TEXT,
            status TEXT NOT NULL DEFAULT 'ready',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL DEFAULT '',
            kind TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id TEXT,
            provider_id TEXT NOT NULL,
            provider_label TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL,
            progress INTEGER NOT NULL DEFAULT 0,
            stage TEXT NOT NULL DEFAULT '',
            error TEXT,
            cost_estimate REAL NOT NULL DEFAULT 0,
            cost_actual REAL NOT NULL DEFAULT 0,
            request_summary TEXT NOT NULL DEFAULT '',
            request_json TEXT NOT NULL DEFAULT '{}',
            result_json TEXT,
            result_asset_id TEXT,
            created_at TEXT NOT NULL,
            started_at TEXT,
            finished_at TEXT
        );

        CREATE TABLE IF NOT EXISTS timeline (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL DEFAULT 'Main cut',
            aspect_ratio TEXT NOT NULL DEFAULT '9:16',
            width INTEGER NOT NULL DEFAULT 1080,
            height INTEGER NOT NULL DEFAULT 1920,
            fps INTEGER NOT NULL DEFAULT 30,
            music_asset_id TEXT,
            music_volume REAL NOT NULL DEFAULT 0.8,
            bpm REAL,
            beat_grid_json TEXT NOT NULL DEFAULT '[]',
            beat_offset_ms INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS timeline_clips (
            id TEXT PRIMARY KEY,
            timeline_id TEXT NOT NULL REFERENCES timeline(id) ON DELETE CASCADE,
            shot_id TEXT,
            asset_id TEXT,
            order_index INTEGER NOT NULL,
            in_ms INTEGER NOT NULL DEFAULT 0,
            out_ms INTEGER NOT NULL DEFAULT 0,
            duration_ms INTEGER NOT NULL DEFAULT 0,
            label TEXT NOT NULL DEFAULT '',
            muted INTEGER NOT NULL DEFAULT 0,
            transition TEXT NOT NULL DEFAULT 'cut'
        );

        CREATE TABLE IF NOT EXISTS subtitles (
            id TEXT PRIMARY KEY,
            timeline_id TEXT NOT NULL REFERENCES timeline(id) ON DELETE CASCADE,
            start_ms INTEGER NOT NULL,
            end_ms INTEGER NOT NULL,
            text TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS shot_takes (
            shot_id TEXT NOT NULL REFERENCES shots(id) ON DELETE CASCADE,
            asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
            created_at TEXT NOT NULL,
            PRIMARY KEY (shot_id, asset_id)
        );

        CREATE INDEX IF NOT EXISTS idx_scenes_episode ON scenes(episode_id, order_index);
        CREATE INDEX IF NOT EXISTS idx_shots_scene ON shots(scene_id, order_index);
        CREATE INDEX IF NOT EXISTS idx_assets_kind ON assets(kind, created_at);
        CREATE INDEX IF NOT EXISTS idx_jobs_created ON jobs(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_clips_timeline ON timeline_clips(timeline_id, order_index);
        CREATE INDEX IF NOT EXISTS idx_shot_takes_shot ON shot_takes(shot_id, created_at DESC);
        "#,
    )?;
    ensure_column(conn, "timeline", "music_asset_id", "music_asset_id TEXT")?;
    ensure_column(
        conn,
        "timeline",
        "music_volume",
        "music_volume REAL NOT NULL DEFAULT 0.8",
    )?;
    ensure_column(conn, "timeline", "bpm", "bpm REAL")?;
    ensure_column(
        conn,
        "timeline",
        "beat_grid_json",
        "beat_grid_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        conn,
        "timeline",
        "beat_offset_ms",
        "beat_offset_ms INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(conn, "assets", "license", "license TEXT")?;
    ensure_column(conn, "assets", "license_url", "license_url TEXT")?;
    ensure_column(conn, "assets", "attribution", "attribution TEXT")?;
    ensure_column(conn, "assets", "source_url", "source_url TEXT")?;
    ensure_column(
        conn,
        "jobs",
        "project_id",
        "project_id TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(conn, "shots", "voice_asset_id", "voice_asset_id TEXT")?;
    ensure_column(
        conn,
        "project",
        "budget_cap",
        "budget_cap REAL NOT NULL DEFAULT 0",
    )?;
    conn.execute(
        "UPDATE jobs SET project_id=(SELECT id FROM project LIMIT 1) WHERE project_id=''",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO shot_takes(shot_id, asset_id, created_at)
         SELECT id, video_asset_id, updated_at FROM shots
         WHERE video_asset_id IS NOT NULL AND video_asset_id != ''",
        [],
    )?;
    conn.execute(
        "INSERT INTO schema_meta(key, value) VALUES('schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [SCHEMA_VERSION],
    )?;
    Ok(())
}

fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(Result::ok)
        .any(|name| name == column);
    drop(stmt);
    if !exists {
        conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {definition}"), [])?;
    }
    Ok(())
}
