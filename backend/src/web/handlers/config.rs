use crate::web::config::save_config;
use crate::core::model::AppConfig;
use crate::web::state::AppState;
use axum::{extract::State, http::StatusCode, Json};

pub async fn get_config_handler(State(state): State<AppState>) -> Json<AppConfig> {
    let config = state.config.lock().await;
    Json(config.clone())
}

pub async fn set_config_handler(
    State(state): State<AppState>,
    Json(payload): Json<AppConfig>,
) -> Result<(), (StatusCode, String)> {
    let mut config = state.config.lock().await;
    *config = payload;
    save_config(&config).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}