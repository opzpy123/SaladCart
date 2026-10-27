use saladcart_lib::paths::AppPaths;
use saladcart_lib::services::demo;
use saladcart_lib::settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::data_dir().map(|dir| dir.join("com.saladcart.studio")))
        .ok_or("could not resolve app data directory")?;
    let paths = AppPaths::new(root)?;
    let settings = settings::load(&paths);
    let snapshot = demo::create(&paths, &settings).await?;
    println!(
        "seeded {} at {}\n  characters={} episodes={} assets={} clips={}",
        snapshot.project.name,
        snapshot.storage_path,
        snapshot.characters.len(),
        snapshot.episodes.len(),
        snapshot.assets.len(),
        snapshot.timeline.clips.len()
    );
    Ok(())
}
