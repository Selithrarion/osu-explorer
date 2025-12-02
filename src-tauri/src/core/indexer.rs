use super::model::IndexingProgress;
use super::{features, model::MapInfo, parser};
use anyhow::Result;
use rayon::prelude::*;
use seli_vector_db::{Vector, VectorDB};
use std::fs;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use walkdir::WalkDir;

pub fn run_indexing(
    songs_path: &Path,
    db_path: &Path,
    limit: Option<usize>,
    progress_tracker: Option<Arc<Mutex<IndexingProgress>>>,
) -> Result<()> {
    println!(
        "Phase 1: Indexing beatmaps from '{}'...",
        songs_path.display()
    );

    let osu_files_iter = WalkDir::new(songs_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "osu"));

    let osu_files: Vec<_> = if let Some(l) = limit {
        osu_files_iter.take(l).collect()
    } else {
        osu_files_iter.collect()
    };

    if let Some(tracker) = &progress_tracker {
        let mut progress = tracker.lock().unwrap();
        progress.total = osu_files.len() as u64;
        progress.message = "Parsing beatmaps...".to_string();
    }

    let maps_processed = AtomicUsize::new(0);
    let indexed_data: Vec<(Vector, MapInfo)> = osu_files
        .par_iter()
        .filter_map(|entry| {
            if let Some(tracker) = &progress_tracker {
                let count = maps_processed.fetch_add(1, Ordering::Relaxed);
                if count % 100 == 0 {
                    tracker.lock().unwrap().progress = count as u64;
                }
            }
            let beatmap = parser::parse_beatmap_from_file(entry.path()).ok()?;
            let feature_vector = features::extract_features(&beatmap)?;
            let map_info = MapInfo {
                path: entry.path().to_path_buf(),
                beatmap_id: beatmap.beatmap_id,
                beatmapset_id: beatmap.beatmapset_id,
                title: beatmap.title,
                artist: beatmap.artist,
                difficulty_name: beatmap.difficulty_name,
            };
            Some((feature_vector, map_info))
        })
        .collect();

    let (vectors, map_infos): (Vec<Vector>, Vec<MapInfo>) = indexed_data.into_iter().unzip();
    println!("Total maps to be indexed: {}", map_infos.len());

    if !map_infos.is_empty() {
        if let Some(tracker) = &progress_tracker {
            let mut progress = tracker.lock().unwrap();
            progress.progress = progress.total;
            progress.message = "Building search index...".to_string();
        }

        println!("Phase 2: Building search index...");
        let mut db = VectorDB::new();
        for v in vectors {
            db.add(v);
        }

        let num_clusters = (db.len() as f64).sqrt() as usize;
        db.build_index(num_clusters, 20)?;
        println!("Index built with {} clusters.", num_clusters);

        println!("Saving database to '{}'...", db_path.display());
        db.save_to_file(db_path)?;
        let paths_path = db_path.with_extension("paths.json");
        let infos_json = serde_json::to_string_pretty(&map_infos)?;
        fs::write(paths_path, infos_json)?;

        println!("Database and paths saved successfully!");
    } else {
        println!("No maps were found to index.");
    }

    Ok(())
}
