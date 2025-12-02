use super::model::AppConfig;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn data_dir() -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .context("Failed to get parent directory of executable")?;
    let data_path = exe_dir.join("osu-explorer-data");
    if !data_path.exists() {
        fs::create_dir_all(&data_path)?;
    }
    Ok(data_path)
}

pub fn config_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("config.json"))
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_or_detect_config() -> Result<AppConfig> {
    if let Ok(path) = config_path() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&content) {
                println!("Loaded config from config.json");
                return Ok(config);
            }
        }
    }

    println!("No config.json found, running auto-detection...");
    let mut potential_paths = Vec::new();

    if let Some(proj_dirs) = ProjectDirs::from("", "", "osu!") {
        potential_paths.push(proj_dirs.data_local_dir().join("Songs"));
    }

    for drive in &["C:", "D:", "E:", "F:"] {
        potential_paths.push(PathBuf::from(format!("{}\\Games\\osu!\\Songs", drive)));
        potential_paths.push(PathBuf::from(format!("{}\\osu!\\Songs", drive)));
    }

    for path in potential_paths {
        if path.exists() && path.is_dir() {
            println!("Found potential songs folder at: {}", path.display());
            let config = AppConfig {
                songs_path: Some(path),
            };
            save_config(&config)?;
            return Ok(config);
        }
    }

    println!("Could not auto-detect songs folder.");
    Ok(AppConfig { songs_path: None })
}
