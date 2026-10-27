use crate::error::{AppError, AppResult};
use crate::models::Settings;
use crate::paths::AppPaths;
use crate::settings;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

pub struct JobRuntime {
    pub semaphore: Arc<Semaphore>,
    handles: Mutex<HashMap<String, JoinHandle<()>>>,
}

impl JobRuntime {
    pub fn new(concurrency: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(concurrency.clamp(1, 4) as usize)),
            handles: Mutex::new(HashMap::new()),
        }
    }

    pub fn track(&self, id: String, handle: JoinHandle<()>) {
        if let Ok(mut handles) = self.handles.lock() {
            handles.insert(id, handle);
        }
    }

    pub fn finish(&self, id: &str) {
        if let Ok(mut handles) = self.handles.lock() {
            handles.remove(id);
        }
    }

    pub fn cancel(&self, id: &str) -> bool {
        if let Ok(mut handles) = self.handles.lock() {
            if let Some(handle) = handles.remove(id) {
                handle.abort();
                return true;
            }
        }
        false
    }
}

pub struct AppState {
    pub paths: AppPaths,
    pub settings: std::sync::RwLock<Settings>,
    pub jobs: Arc<JobRuntime>,
}

impl AppState {
    pub fn initialize(app: &tauri::AppHandle) -> AppResult<Self> {
        let root = if let Ok(custom) = std::env::var("SALADCART_DATA_DIR") {
            std::path::PathBuf::from(custom)
        } else {
            app.path()
                .app_data_dir()
                .map_err(|e| AppError::Message(format!("could not resolve app data dir: {e}")))?
        };
        let paths = AppPaths::new(root)?;
        let settings = settings::load(&paths);
        let concurrency = settings.concurrency;
        Ok(Self {
            paths,
            settings: std::sync::RwLock::new(settings),
            jobs: Arc::new(JobRuntime::new(concurrency)),
        })
    }

    pub fn settings(&self) -> Settings {
        self.settings
            .read()
            .map(|s| s.clone())
            .unwrap_or_else(|_| settings::default_settings())
    }

    pub fn reload_settings(&self) -> Settings {
        let next = settings::load(&self.paths);
        if let Ok(mut current) = self.settings.write() {
            *current = next.clone();
        }
        next
    }
}
