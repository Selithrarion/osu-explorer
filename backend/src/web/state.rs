use crate::core::model::{AppConfig, MapInfo};
use seli_vector_db::VectorDB;
use serde::{Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
pub struct SearchResultItem {
    pub score: f32,
    pub divergence: f32,
    pub map_info: MapInfo,
    pub cover_url: String,
}

#[derive(Serialize, Clone, Debug, Default)]
 pub struct IndexingProgress {
 	pub progress: u64,
 	pub total: u64,
 	pub message: String,
 }

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Option<VectorDB>>>,
    pub map_infos: Arc<Mutex<Option<Vec<MapInfo>>>>,
    pub map_id_to_index: Arc<Mutex<HashMap<i32, usize>>>,
    pub db_path: PathBuf,
    pub config: Arc<Mutex<AppConfig>>,
    pub indexing_progress: Arc<std::sync::Mutex<IndexingProgress>>,
}