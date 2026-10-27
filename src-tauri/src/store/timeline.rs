use crate::error::{AppError, AppResult, OptionExt};
use crate::models::*;
use crate::store::mappers::{map_clip, map_subtitle, map_timeline, now_iso};
use crate::store::{dimensions_for, normalize_aspect, ProjectStore};
use rusqlite::{params, OptionalExtension};

impl ProjectStore {
    pub fn timeline(&self) -> AppResult<Timeline> {
        let mut timeline = self
            .conn
            .query_row("SELECT * FROM timeline LIMIT 1", [], map_timeline)
            .optional()?
            .or_not_found("timeline")?;
        let mut clip_stmt = self.conn.prepare(
            "SELECT * FROM timeline_clips WHERE timeline_id=?1 ORDER BY order_index ASC",
        )?;
        let clips = clip_stmt.query_map(params![timeline.id], map_clip)?;
        for clip in clips {
            timeline.clips.push(clip?);
        }
        let mut sub_stmt = self
            .conn
            .prepare("SELECT * FROM subtitles WHERE timeline_id=?1 ORDER BY start_ms ASC")?;
        let subtitles = sub_stmt.query_map(params![timeline.id], map_subtitle)?;
        for subtitle in subtitles {
            timeline.subtitles.push(subtitle?);
        }
        Ok(timeline)
    }

    pub fn update_timeline(&mut self, patch: TimelinePatch) -> AppResult<Timeline> {
        let TimelinePatch {
            name,
            aspect_ratio,
            width,
            height,
            fps,
            clips,
            subtitles,
            music_asset_id,
            clear_music,
            music_volume,
            bpm,
            beat_grid_ms,
            beat_offset_ms,
            clear_beats,
        } = patch;
        let mut timeline = self.timeline()?;
        if let Some(v) = name {
            timeline.name = v;
        }
        if let Some(v) = aspect_ratio {
            let new_aspect = normalize_aspect(&v);
            timeline.aspect_ratio = new_aspect.clone();
            if width.is_none() && height.is_none() {
                let (w, h) = dimensions_for(&new_aspect);
                timeline.width = w;
                timeline.height = h;
            }
        }
        if let Some(v) = width {
            timeline.width = v;
        }
        if let Some(v) = height {
            timeline.height = v;
        }
        if let Some(v) = fps {
            timeline.fps = v.clamp(12, 60);
        }
        if clear_music {
            timeline.music_asset_id = None;
        } else if let Some(v) = music_asset_id {
            if v.is_empty() {
                timeline.music_asset_id = None;
            } else {
                let asset = self.get_asset(&v)?;
                if asset.kind != "audio" {
                    return Err(AppError::Invalid(
                        "only audio assets can be used as a music bed".into(),
                    ));
                }
                timeline.music_asset_id = Some(asset.id);
            }
        }
        if let Some(v) = music_volume {
            timeline.music_volume = v.clamp(0.0, 2.0);
        }
        if clear_beats {
            timeline.bpm = None;
            timeline.beat_grid_ms = Vec::new();
            timeline.beat_offset_ms = 0;
        }
        if let Some(v) = bpm {
            timeline.bpm = Some(v.clamp(20.0, 300.0));
        }
        if let Some(v) = beat_grid_ms {
            timeline.beat_grid_ms = v;
        }
        if let Some(v) = beat_offset_ms {
            timeline.beat_offset_ms = v;
        }
        timeline.updated_at = now_iso();

        struct PreparedClip {
            shot_id: Option<String>,
            asset_id: String,
            in_ms: i64,
            out_ms: i64,
            duration_ms: i64,
            label: String,
            muted: bool,
            transition: String,
        }

        let replace_clips = clips.is_some();
        let mut prepared_clips = Vec::new();
        if let Some(clips) = clips {
            for input in clips {
                let mut asset_id = input.asset_id;
                let mut duration_hint = 5000_i64;
                let mut label = input.label;
                if let Some(shot_id) = &input.shot_id {
                    if let Ok(shot) = self.get_shot(shot_id) {
                        duration_hint = shot.duration_ms;
                        if label.trim().is_empty() {
                            label = shot.title;
                        }
                        if asset_id.is_none() {
                            asset_id = shot.video_asset_id.or(shot.image_asset_id);
                        }
                    }
                }
                let Some(asset_id) = asset_id else {
                    continue;
                };
                let Ok(asset) = self.get_asset(&asset_id) else {
                    continue;
                };
                let asset_duration = asset.duration_ms.unwrap_or(duration_hint);
                let in_ms = input.in_ms.max(0).min(asset_duration.saturating_sub(1));
                let out_ms = if input.out_ms > in_ms {
                    input.out_ms.min(asset_duration)
                } else {
                    asset_duration
                };
                prepared_clips.push(PreparedClip {
                    shot_id: input.shot_id,
                    asset_id,
                    in_ms,
                    out_ms,
                    duration_ms: (out_ms - in_ms).max(1),
                    label,
                    muted: input.muted,
                    transition: if input.transition.is_empty() {
                        "cut".into()
                    } else {
                        input.transition
                    },
                });
            }
        }

        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE timeline SET name=?1,aspect_ratio=?2,width=?3,height=?4,fps=?5,music_asset_id=?6,music_volume=?7,bpm=?8,beat_grid_json=?9,beat_offset_ms=?10,updated_at=?11 WHERE id=?12",
            params![
                timeline.name,
                timeline.aspect_ratio,
                timeline.width,
                timeline.height,
                timeline.fps,
                timeline.music_asset_id,
                timeline.music_volume,
                timeline.bpm,
                serde_json::to_string(&timeline.beat_grid_ms)?,
                timeline.beat_offset_ms,
                timeline.updated_at,
                timeline.id
            ],
        )?;

        if replace_clips {
            tx.execute(
                "DELETE FROM timeline_clips WHERE timeline_id=?1",
                params![timeline.id],
            )?;
            for (index, clip) in prepared_clips.into_iter().enumerate() {
                tx.execute(
                    "INSERT INTO timeline_clips
                     (id,timeline_id,shot_id,asset_id,order_index,in_ms,out_ms,duration_ms,label,muted,transition)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        timeline.id,
                        clip.shot_id,
                        clip.asset_id,
                        index as i64,
                        clip.in_ms,
                        clip.out_ms,
                        clip.duration_ms,
                        clip.label,
                        if clip.muted { 1 } else { 0 },
                        clip.transition
                    ],
                )?;
            }
        }

        if let Some(subtitles) = subtitles {
            tx.execute(
                "DELETE FROM subtitles WHERE timeline_id=?1",
                params![timeline.id],
            )?;
            for input in subtitles {
                if input.end_ms <= input.start_ms || input.text.trim().is_empty() {
                    continue;
                }
                tx.execute(
                    "INSERT INTO subtitles (id,timeline_id,start_ms,end_ms,text) VALUES (?1,?2,?3,?4,?5)",
                    params![
                        uuid::Uuid::new_v4().to_string(),
                        timeline.id,
                        input.start_ms.max(0),
                        input.end_ms,
                        input.text.trim()
                    ],
                )?;
            }
        }

        tx.commit()?;
        self.touch()?;
        self.timeline()
    }

    pub fn auto_assemble(&mut self) -> AppResult<Timeline> {
        let project = self.project()?;
        let timeline = self.timeline()?;
        let (width, height) = dimensions_for(&project.aspect_ratio);

        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.title, s.dialogue, s.duration_ms, s.video_asset_id, sc.title as scene_title, a.duration_ms as asset_duration
             FROM shots s
             JOIN scenes sc ON sc.id=s.scene_id
             JOIN episodes e ON e.id=sc.episode_id
             JOIN assets a ON a.id=s.video_asset_id
             WHERE s.video_asset_id IS NOT NULL AND s.video_asset_id != ''
             ORDER BY e.order_index ASC, sc.order_index ASC, s.order_index ASC",
        )?;
        struct RowData {
            shot_id: String,
            label: String,
            dialogue: String,
            duration_ms: i64,
            asset_id: String,
        }
        let rows = stmt.query_map([], |row| {
            let asset_duration: Option<i64> = row.get("asset_duration")?;
            let shot_duration: i64 = row.get("duration_ms")?;
            let scene_title: String = row.get("scene_title")?;
            let shot_title: String = row.get("title")?;
            Ok(RowData {
                shot_id: row.get("id")?,
                label: format!("{scene_title} · {shot_title}"),
                dialogue: row.get("dialogue")?,
                duration_ms: asset_duration.unwrap_or(shot_duration).max(1000),
                asset_id: row.get("video_asset_id")?,
            })
        })?;
        let mut clips = Vec::new();
        let mut subtitles = Vec::new();
        let mut cursor = 0_i64;
        for row in rows {
            let row = row?;
            clips.push(TimelineClipInput {
                shot_id: Some(row.shot_id),
                asset_id: Some(row.asset_id),
                in_ms: 0,
                out_ms: row.duration_ms,
                label: row.label,
                muted: false,
                transition: "cut".into(),
            });
            if !row.dialogue.trim().is_empty() {
                subtitles.push(SubtitleInput {
                    start_ms: cursor + 120,
                    end_ms: cursor + row.duration_ms - 120,
                    text: row.dialogue.trim().to_string(),
                });
            }
            cursor += row.duration_ms;
        }
        drop(stmt);
        self.update_timeline(TimelinePatch {
            name: Some(if timeline.name.is_empty() {
                "Auto cut".into()
            } else {
                timeline.name
            }),
            aspect_ratio: Some(project.aspect_ratio),
            width: Some(width),
            height: Some(height),
            fps: Some(timeline.fps.max(24)),
            clips: Some(clips),
            subtitles: Some(subtitles),
            music_asset_id: None,
            clear_music: false,
            music_volume: None,
            bpm: None,
            beat_grid_ms: None,
            beat_offset_ms: None,
            clear_beats: false,
        })
    }

    pub fn total_timeline_ms(&self) -> AppResult<i64> {
        Ok(self.timeline()?.clips.iter().map(|c| c.duration_ms).sum())
    }

    pub fn srt_content(&self) -> AppResult<String> {
        let timeline = self.timeline()?;
        let mut out = String::new();
        for (i, subtitle) in timeline.subtitles.iter().enumerate() {
            out.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                srt_time(subtitle.start_ms),
                srt_time(subtitle.end_ms),
                subtitle.text
            ));
        }
        Ok(out)
    }

    pub fn write_srt(&self) -> AppResult<std::path::PathBuf> {
        let dir = self.paths.exports_dir(&self.project_id)?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("subtitles.srt");
        std::fs::write(&path, self.srt_content()?)?;
        Ok(path)
    }
}

fn srt_time(ms: i64) -> String {
    let ms = ms.max(0);
    let hours = ms / 3_600_000;
    let minutes = (ms % 3_600_000) / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}

pub fn validate_clip_order(clips: &[TimelineClip]) -> AppResult<()> {
    let mut last = -1;
    for clip in clips {
        if clip.order_index < last {
            return Err(AppError::Invalid("timeline clips are not ordered".into()));
        }
        last = clip.order_index;
    }
    Ok(())
}
