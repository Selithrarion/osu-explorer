use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Beatmap {
    pub title: String,
    pub artist: String,
    pub difficulty_name: String,
    pub beatmap_id: i32,
    pub beatmapset_id: i32,
    pub hit_objects: Vec<HitObject>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitObjectType {
    Circle,
    Slider,
    Spinner,
    HoldNote,
}

#[derive(Debug, Clone)]
pub struct HitObject {
    pub x: f32,
    pub y: f32,
    pub start_time: f32,
    pub obj_type: HitObjectType,
    pub curve_points: Option<Vec<(f32, f32)>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapInfo {
    pub path: PathBuf,
    pub beatmap_id: i32,
    pub beatmapset_id: i32,
    pub title: String,
    pub artist: String,
    pub difficulty_name: String,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct IndexingProgress {
    pub progress: u64,
    pub total: u64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppConfig {
    pub songs_path: Option<PathBuf>,
}
