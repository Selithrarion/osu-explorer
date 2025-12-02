use crate::core::{
    self,
    config::save_config,
    model::{AppConfig, MapInfo},
};
use crate::{error::AppError, AppState, IndexingProgress};
use std::path::Path;
use tauri::{Emitter, Manager, State};

#[derive(serde::Serialize)]
pub struct SearchResultItem {
    score: f32,
    divergence: f32,
    map_info: MapInfo,
    cover_url: String,
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn is_db_indexed(state: State<AppState>) -> bool {
    state.db.lock().unwrap().is_some()
}

#[tauri::command]
pub async fn search(
    beatmap_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResultItem>, AppError> {
    let db_guard = state.db.lock().unwrap();
    let map_infos_guard = state.map_infos.lock().unwrap();
    let map_id_to_index_guard = state.map_id_to_index.lock().unwrap();

    let (Some(db), Some(map_infos)) = (db_guard.as_ref(), map_infos_guard.as_ref()) else {
        return Err(AppError::DatabaseNotIndexed);
    };

    let Some(&query_map_index) = map_id_to_index_guard.get(&beatmap_id) else {
        return Err(AppError::MapNotFound(beatmap_id));
    };

    let query_map_info = &map_infos[query_map_index];

    let beatmap = core::parser::parse_beatmap_from_file(&query_map_info.path)
        .map_err(|e| AppError::IoError(e.to_string()))?;

    let query_vector =
        core::features::extract_features(&beatmap).ok_or(AppError::FeatureExtractionFailed)?;

    let search_results =
        core::searcher::perform_search(db, map_infos, &query_vector, query_map_index, 10);

    let response: Vec<SearchResultItem> = search_results
        .into_iter()
        .map(|(score, map_info)| {
            let cover_url = format!(
                "https://assets.ppy.sh/beatmaps/{}/covers/list.jpg",
                map_info.beatmapset_id
            );
            SearchResultItem {
                score,
                divergence: (1.0 - score) * 100.0,
                map_info: map_info.clone(),
                cover_url,
            }
        })
        .collect();

    Ok(response)
}

#[tauri::command]
pub async fn index(
    songs_path: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<(), AppError> {
    let songs_path_buf = Path::new(&songs_path).to_path_buf();
    let mut config_guard = state.config.lock().unwrap();
    config_guard.songs_path = Some(songs_path_buf.clone());
    save_config(&config_guard).map_err(|e| AppError::ConfigError(e.to_string()))?;

    *state.indexing_progress.lock().unwrap() = IndexingProgress::default();

    let db_path_arc = state.db_path.clone();
    let progress_tracker_arc = state.indexing_progress.clone();
    let app_handle = window.app_handle().clone();
    tokio::task::spawn_blocking(move || {
        let db_path = db_path_arc.lock().unwrap();
        let _ = core::indexer::run_indexing(
            Path::new(&songs_path),
            &db_path,
            limit,
            Some(progress_tracker_arc),
        );

        if let Err(e) = app_handle.emit("index-complete", ()) {
            eprintln!("Failed to emit index-complete event: {}", e);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_indexing_status(state: State<AppState>) -> IndexingProgress {
    state.indexing_progress.lock().unwrap().clone()
}
