mod commands;
mod core;
mod error;

use crate::core::model::{AppConfig, IndexingProgress, MapInfo};
use seli_vector_db::VectorDB;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager};
#[derive(Default)]
pub struct AppState {
    pub db: Arc<Mutex<Option<VectorDB>>>,
    pub map_infos: Arc<Mutex<Option<Vec<MapInfo>>>>,
    pub map_id_to_index: Arc<Mutex<HashMap<i32, usize>>>,
    pub db_path: Arc<Mutex<PathBuf>>,
    pub config: Arc<Mutex<AppConfig>>,
    pub indexing_progress: Arc<Mutex<IndexingProgress>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = core::config::load_or_detect_config().expect("Failed to load or detect config");

    let data_dir = core::config::data_dir().expect("Failed to create data directory");
    let db_path = data_dir.join("osu_maps.db");

    let db = VectorDB::load_from_file(&db_path).ok();
    let map_infos = if db.is_some() {
        core::searcher::load_map_infos(&db_path).ok()
    } else {
        None
    };

    let mut map_id_to_index = HashMap::new();
    if let Some(infos) = &map_infos {
        for (index, info) in infos.iter().enumerate() {
            map_id_to_index.insert(info.beatmap_id, index);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Arc::new(Mutex::new(db)),
            map_infos: Arc::new(Mutex::new(map_infos)),
            map_id_to_index: Arc::new(Mutex::new(map_id_to_index)),
            db_path: Arc::new(Mutex::new(db_path)),
            config: Arc::new(Mutex::new(config)),
            indexing_progress: Arc::new(Mutex::new(IndexingProgress::default())),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::search,
            commands::index,
            commands::get_indexing_status,
            commands::is_db_indexed
        ])
        .setup(|app| -> Result<(), Box<dyn std::error::Error>> {
            let app_handle = app.handle().clone();
            app.listen("index-complete", move |_event| {
                let success = reload_app_state(&app_handle);
                if let Err(e) = app_handle.emit("state-reloaded", success) {
                    eprintln!("Failed to emit state-reloaded event: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn reload_app_state(app_handle: &tauri::AppHandle) -> bool {
    println!("Received index-complete event. Reloading state...");
    let state: tauri::State<AppState> = app_handle.state();
    let db_path = state.db_path.lock().unwrap().clone();

    let new_db = VectorDB::load_from_file(&db_path).ok();
    let new_map_infos = if new_db.is_some() {
        core::searcher::load_map_infos(&db_path).ok()
    } else {
        None
    };
    let mut new_map_id_to_index = HashMap::new();
    if let Some(infos) = &new_map_infos {
        for (index, info) in infos.iter().enumerate() {
            new_map_id_to_index.insert(info.beatmap_id, index);
        }
    }

    *state.db.lock().unwrap() = new_db;
    *state.map_infos.lock().unwrap() = new_map_infos;
    *state.map_id_to_index.lock().unwrap() = new_map_id_to_index;
    println!("State reloaded successfully!");

    let is_indexed = state.db.lock().unwrap().is_some();
    is_indexed
}
