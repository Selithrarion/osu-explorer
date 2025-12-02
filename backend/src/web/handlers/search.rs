use crate::core::{features, parser, searcher};
use crate::web::state::{AppState, SearchResultItem};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

pub async fn search_handler(
    State(state): State<AppState>,
    Path(beatmap_id): Path<i32>,
) -> Result<Json<Vec<SearchResultItem>>, (StatusCode, String)> {
    let db_guard = state.db.lock().await;
    let map_infos_guard = state.map_infos.lock().await;
    let map_id_to_index_guard = state.map_id_to_index.lock().await;

	let (Some(db), Some(map_infos)) = (db_guard.as_ref(), map_infos_guard.as_ref()) else {
		return Err((
			StatusCode::PRECONDITION_FAILED,
			"Database is not indexed yet. Please go to Settings to start indexing.".into(),
		));
    };

    let query_map_index = map_id_to_index_guard
        .get(&beatmap_id)
        .copied()
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Map with beatmap_id {} not found", beatmap_id),
            )
        })?;
    let query_map_info = &map_infos[query_map_index];

    let beatmap = parser::parse_beatmap_from_file(&query_map_info.path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let query_vector = features::extract_features(&beatmap).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not extract features".into(),
        )
    })?;

    let k = 10;

    let search_results =
        searcher::perform_search(db, map_infos, &query_vector, query_map_index, k);

    let response: Vec<SearchResultItem> = search_results
        .into_iter()
        .map(|(score, map_info)| {
            let cover_url = format!(
                "https://assets.ppy.sh/beatmaps/{}/covers/cover.jpg",
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

    Ok(Json(response))
}