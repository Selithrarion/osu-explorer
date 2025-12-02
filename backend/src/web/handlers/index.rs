use crate::web::config::save_config;
 use crate::core::indexer;
 use crate::core::searcher;
 use seli_vector_db::VectorDB;
use crate::web::state::{AppState, IndexingProgress};
 use axum::{extract::State, http::StatusCode, Json};
 use serde::Deserialize;
 use std::{collections::HashMap, path::Path as StdPath};
use std::sync::Arc;

 #[derive(Debug, Deserialize)]
 pub struct IndexRequest {
     pub songs_path: String,
     pub limit: Option<usize>,
 }

pub async fn get_status_handler(State(state): State<AppState>) -> Json<IndexingProgress> {
	let progress = state.indexing_progress.lock().unwrap();
	Json(progress.clone())
}

pub async fn index_handler(
    State(state): State<AppState>,
    Json(payload): Json<IndexRequest>,
) -> Result<(), (StatusCode, String)> {
    println!("Starting indexing for path: {}", payload.songs_path);

    let songs_path = StdPath::new(&payload.songs_path);
    if !songs_path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Provided songs_path does not exist".into(),
        ));
    }

    let mut config_guard = state.config.lock().await;
    config_guard.songs_path = Some(songs_path.to_path_buf());
    save_config(&config_guard).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let songs_path_owned = payload.songs_path.clone();
    let db_path_clone = state.db_path.clone();
    let limit_clone = payload.limit;
	let progress_clone = Arc::clone(&state.indexing_progress);

    tokio::task::spawn_blocking(move || {
		indexer::run_indexing(songs_path_owned.as_ref(), &db_path_clone, limit_clone, Some(progress_clone))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    println!("Indexing complete. Reloading database...");
    let new_db = VectorDB::load_from_file(&state.db_path).ok();
    let new_map_infos = searcher::load_map_infos(&state.db_path).ok();
    let mut new_map_id_to_index = HashMap::new();
    if let Some(infos) = &new_map_infos {
        for (index, info) in infos.iter().enumerate() {
            new_map_id_to_index.insert(info.beatmap_id, index);
        }
    }

    *state.db.lock().await = new_db;
    *state.map_infos.lock().await = new_map_infos;
    *state.map_id_to_index.lock().await = new_map_id_to_index;

    println!("Database reloaded successfully!");
    Ok(())
}