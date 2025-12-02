 use super::model::MapInfo;
use anyhow::{Context, Result};
use seli_vector_db::{Vector, VectorDB};
use std::path::Path;

pub fn load_map_infos(db_path: &Path) -> Result<Vec<MapInfo>> {
    let paths_path = db_path.with_extension("paths.json");
    serde_json::from_str(&std::fs::read_to_string(&paths_path).context(format!(
        "Failed to load map paths from '{}'. Make sure it exists and 'index' was run.",
        paths_path.display()
    ))?)
    .context("Failed to parse map paths JSON.")
}

pub fn perform_search<'a>(
    db: &'a VectorDB,
    map_infos: &'a [MapInfo],
    query_vector: &Vector,
    query_map_index: usize,
    k: usize,
) -> Vec<(f32, &'a MapInfo)> {
    let nprobe = (db.num_clusters().unwrap_or(1) as f64).sqrt() as usize + 1;
    const SEARCH_BUFFER: usize = 1;
    let results = db.search(&query_vector, k + SEARCH_BUFFER, nprobe);

    results
        .iter()
        .filter(|r| r.id != query_map_index)
        .take(k)
        .map(|r| (r.score, &map_infos[r.id]))
        .collect()
}

/*
/// old CLI-based search function.
pub fn run_search(query_map_path: &PathBuf, db_path: &PathBuf, k: usize) -> Result<()> {
    println!("Loading database from '{}'...", db_path.display());
    let db = VectorDB::load_from_file(db_path)
        .context("Failed to load database. Did you run the 'index' command first?")?;

    let map_infos = load_map_infos(db_path)?;

    println!("Analyzing query map: '{}'", query_map_path.display());
    let beatmap = parser::parse_beatmap_from_file(query_map_path)?;
    let query_vector = features::extract_features(&beatmap)
        .context("Could not extract features from the query map.")?;

    println!("\nSearching for the top {} similar maps...", k);

    let search_results = perform_search(&db, &map_infos, &query_vector, 0, k); // Note: query_map_index is simplified here

    if search_results.is_empty() {
        println!("No similar maps found.");
    } else {
        println!("--- Search Results ---");
        search_results
            .iter()
            .enumerate()
            .for_each(|(i, (score, map_info))| {
                let map_display = format!(
                    "{} - {} [{}]",
                    map_info.artist, map_info.title, map_info.difficulty_name
                );

                let divergence = (1.0 - score) * 100.0;
                let formatted_divergence = format!("{:.2}", divergence);

                let (label, colored_divergence) = if divergence <= 0.5 {
                    ("Close", formatted_divergence.green())
                } else if divergence <= 1.5 {
                    ("Similar", formatted_divergence.yellow())
                } else if divergence <= 3.0 {
                    ("Related", formatted_divergence.bright_yellow())
                } else {
                    ("Distant", formatted_divergence.red())
                };

                println!(
                    "{}. Score: {:.4} (Divergence: {}, {}) | {} (https://osu.ppy.sh/beatmapsets/{}#osu/{})",
                    (i + 1).to_string().dimmed(),
                    score,
                    colored_divergence,
                    label,
                    map_display,
                    map_info.beatmapset_id, map_info.beatmap_id
                );
            });
    }

    Ok(())
}
*/
