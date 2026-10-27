pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod paths;
pub mod providers;
pub mod secrets;
pub mod services;
pub mod settings;
pub mod state;
pub mod store;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = state::AppState::initialize(app.handle())?;
            if let Err(error) = store::recover_interrupted_jobs(&state.paths) {
                eprintln!("could not recover interrupted jobs: {error}");
            }
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::list_projects,
            commands::create_project,
            commands::open_project,
            commands::update_project,
            commands::archive_project,
            commands::generate_shot_voice,
            commands::list_project_references,
            commands::import_project_style,
            commands::import_character,
            commands::update_scene,
            commands::update_shot,
            commands::set_shot_status,
            commands::delete_shot,
            commands::delete_scene,
            commands::create_character,
            commands::update_character,
            commands::delete_character,
            commands::import_asset,
            commands::import_asset_path,
            commands::list_assets,
            commands::set_asset_status,
            commands::get_context_preview,
            commands::generate_script,
            commands::generate_media,
            commands::list_jobs,
            commands::cancel_job,
            commands::retry_job,
            commands::get_timeline,
            commands::update_timeline,
            commands::auto_assemble_timeline,
            commands::export_timeline,
            commands::get_settings,
            commands::update_provider,
            commands::update_preferences,
            commands::set_provider_secret,
            commands::clear_provider_secret,
            commands::test_provider,
            commands::get_system_status,
            commands::open_storage_folder,
            commands::reveal_project_folder,
            commands::search_music,
            commands::import_music_url,
            commands::generate_music,
            commands::analyze_music_beats,
            commands::auto_beat_sync,
            commands::attach_music,
            commands::create_demo_project,
            commands::export_provenance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SaladCart");
}
