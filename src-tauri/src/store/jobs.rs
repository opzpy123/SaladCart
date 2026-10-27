use crate::error::{AppError, AppResult, OptionExt};
use crate::models::*;
use crate::store::mappers::{map_asset, map_job, now_iso};
use crate::store::ProjectStore;
use rusqlite::{params, OptionalExtension};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct FinishJob {
    pub status: String,
    pub error: Option<String>,
    pub result: Option<Value>,
    pub result_asset_id: Option<String>,
    pub cost_actual: f64,
}

impl ProjectStore {
    pub fn insert_job(&self, job: &GenerationJob) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO jobs
             (id,project_id,kind,target_type,target_id,provider_id,provider_label,model,status,progress,stage,error,cost_estimate,cost_actual,request_summary,request_json,result_json,result_asset_id,created_at,started_at,finished_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)",
            params![
                job.id,
                job.project_id,
                job.kind,
                job.target_type,
                job.target_id,
                job.provider_id,
                job.provider_label,
                job.model,
                job.status,
                job.progress,
                job.stage,
                job.error,
                job.cost_estimate,
                job.cost_actual,
                job.request_summary,
                serde_json::to_string(&job.request)?,
                job.result.as_ref().map(|v| serde_json::to_string(v)).transpose()?,
                job.result_asset_id,
                job.created_at,
                job.started_at,
                job.finished_at
            ],
        )?;
        Ok(())
    }

    pub fn get_job(&self, job_id: &str) -> AppResult<GenerationJob> {
        self.conn
            .query_row("SELECT * FROM jobs WHERE id=?1", params![job_id], map_job)
            .optional()?
            .or_not_found("job")
    }

    pub fn list_jobs(&self, limit: i64) -> AppResult<Vec<GenerationJob>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM jobs ORDER BY created_at DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], map_job)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn start_job(&self, job_id: &str) -> AppResult<GenerationJob> {
        self.conn.execute(
            "UPDATE jobs SET status='running',progress=2,stage='starting',started_at=?1,error=NULL WHERE id=?2",
            params![now_iso(), job_id],
        )?;
        self.get_job(job_id)
    }

    pub fn update_job_progress(
        &self,
        job_id: &str,
        progress: i64,
        stage: &str,
    ) -> AppResult<GenerationJob> {
        self.conn.execute(
            "UPDATE jobs SET status='running',progress=?1,stage=?2 WHERE id=?3",
            params![progress.clamp(0, 99), stage, job_id],
        )?;
        self.get_job(job_id)
    }

    pub fn finish_job(&self, job_id: &str, finish: FinishJob) -> AppResult<GenerationJob> {
        if finish.status != "succeeded" && finish.status != "failed" && finish.status != "canceled"
        {
            return Err(AppError::Invalid("invalid terminal job status".into()));
        }
        self.conn.execute(
            "UPDATE jobs SET status=?1,progress=CASE WHEN ?1='succeeded' THEN 100 ELSE progress END,stage=CASE WHEN ?1='succeeded' THEN 'done' ELSE stage END,error=?2,result_json=?3,result_asset_id=?4,cost_actual=?5,finished_at=?6 WHERE id=?7",
            params![
                finish.status,
                finish.error,
                finish.result.as_ref().map(|v| serde_json::to_string(v)).transpose()?,
                finish.result_asset_id,
                finish.cost_actual,
                now_iso(),
                job_id
            ],
        )?;
        self.get_job(job_id)
    }

    pub fn cancel_job_record(&self, job_id: &str) -> AppResult<GenerationJob> {
        let job = self.get_job(job_id)?;
        if job.status == "queued" || job.status == "running" {
            self.conn.execute(
                "UPDATE jobs SET status='canceled',stage='canceled',finished_at=?1 WHERE id=?2",
                params![now_iso(), job_id],
            )?;
        }
        self.get_job(job_id)
    }

    pub fn jobs_for_target(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> AppResult<Vec<GenerationJob>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM jobs WHERE target_type=?1 AND target_id=?2 ORDER BY created_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map(params![target_type, target_id], map_job)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn shot_takes(&self, shot_id: &str) -> AppResult<Vec<ShotTake>> {
        let shot = self.get_shot(shot_id)?;
        let mut stmt = self.conn.prepare(
            "SELECT a.*,
                    (SELECT j.id FROM jobs j WHERE j.result_asset_id=a.id
                     ORDER BY j.created_at DESC LIMIT 1) AS job_id,
                    COALESCE((SELECT j.cost_actual FROM jobs j WHERE j.result_asset_id=a.id
                              ORDER BY j.created_at DESC LIMIT 1), 0) AS cost_actual
             FROM shot_takes t JOIN assets a ON a.id=t.asset_id
             WHERE t.shot_id=?1 AND a.kind='video'
             ORDER BY t.created_at DESC LIMIT 50",
        )?;
        let rows = stmt.query_map(params![shot_id], |row| {
            Ok(ShotTake {
                asset: map_asset(row, &self.dir)?,
                selected: false,
                job_id: row.get("job_id")?,
                cost_actual: row.get::<_, Option<f64>>("cost_actual")?.unwrap_or(0.0),
            })
        })?;
        let mut takes: Vec<ShotTake> = Vec::new();
        for row in rows {
            let take = row?;
            if !takes.iter().any(|item| item.asset.id == take.asset.id) {
                takes.push(take);
            }
        }
        let selected_id = shot.video_asset_id.clone();
        if let Some(id) = &selected_id {
            if !takes.iter().any(|take| &take.asset.id == id) {
                if let Ok(asset) = self.get_asset(id) {
                    takes.insert(
                        0,
                        ShotTake {
                            asset,
                            selected: true,
                            job_id: None,
                            cost_actual: 0.0,
                        },
                    );
                }
            }
        }
        for take in &mut takes {
            take.selected = selected_id.as_deref() == Some(take.asset.id.as_str());
        }
        Ok(takes)
    }
}
