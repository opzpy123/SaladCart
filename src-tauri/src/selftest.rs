use crate::error::{AppError, AppResult};
use crate::models::ExportOptions;
use crate::paths::AppPaths;
use crate::services::{demo, media};
use crate::settings;
use crate::store::ProjectStore;

pub async fn run(paths: &AppPaths) -> AppResult<String> {
    let settings = settings::default_settings();
    let snapshot = demo::create(paths, &settings).await?;
    let mut store = ProjectStore::open(paths, &snapshot.project.id)?;
    let timeline = store.auto_assemble(false)?;
    if timeline.clips.is_empty() {
        return Err(AppError::Message("selftest timeline has no clips".into()));
    }
    let options = ExportOptions {
        width: 360,
        height: 640,
        fps: 12,
        burn_subtitles: false,
        music_asset_id: None,
        filename: format!("selftest-{}.mp4", std::process::id()),
    };
    let plan = media::build_render_plan(&store, &settings, &options)?;
    let ffmpeg = media::ensure_ffmpeg(&settings)?;
    let render = ffmpeg.render(&plan, |_| {}).await?;
    let probe = ffmpeg.probe(std::path::Path::new(&render.path));
    if probe.duration_ms.unwrap_or(0) < 1_000 {
        return Err(AppError::Message(
            "selftest export has no readable duration".into(),
        ));
    }
    Ok(format!(
        "selftest ok: project={} clips={} export={} duration_ms={} size_bytes={}",
        snapshot.project.name,
        timeline.clips.len(),
        render.path,
        render.duration_ms,
        render.size_bytes
    ))
}
