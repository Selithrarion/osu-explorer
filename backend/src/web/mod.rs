pub mod handlers;
pub mod state;
pub mod config;
pub mod static_files;

use crate::core::searcher;
use seli_vector_db::VectorDB;
use anyhow::Result;
use axum::{routing::get, routing::post, Router};
use config::load_or_detect_config;
use state::AppState;
use std::collections::HashMap;
use std::path::Path as StdPath;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use static_files::static_handler;

pub async fn serve(db_path: &StdPath) -> Result<()> {
    let config = load_or_detect_config()?;

    let db = VectorDB::load_from_file(db_path).ok();
    let map_infos = if db.is_some() {
        searcher::load_map_infos(db_path).ok()
    } else {
        None
    };

    let mut map_id_to_index = HashMap::new();
    if let Some(infos) = &map_infos {
        for (index, info) in infos.iter().enumerate() {
            map_id_to_index.insert(info.beatmap_id, index);
        }
    }

    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        map_infos: Arc::new(Mutex::new(map_infos)),
        map_id_to_index: Arc::new(Mutex::new(map_id_to_index)),
        db_path: db_path.to_path_buf(),
        config: Arc::new(Mutex::new(config)),
        indexing_progress: Arc::new(std::sync::Mutex::new(Default::default())),
    };

    let cors = CorsLayer::new().allow_origin(Any);
    let app = Router::new()
        .route("/api/index", post(handlers::index::index_handler))
        .route("/api/index/status", get(handlers::index::get_status_handler))
        .route("/api/search/:beatmap_id", get(handlers::search::search_handler))
        .route(
            "/api/config",
            get(handlers::config::get_config_handler).post(handlers::config::set_config_handler),
        )
        .fallback(static_handler)
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server listening on http://0.0.0.0:3000");
    opener::open("http://localhost:3000")?;

    axum::serve(listener, app).await?;

    Ok(())
}