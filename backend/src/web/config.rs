use crate::core::model::AppConfig;
use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    PathBuf::from("config.json")
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(config_path(), json)?;
    Ok(())
}

pub fn load_or_detect_config() -> Result<AppConfig> {
    if let Ok(content) = std::fs::read_to_string(config_path()) {
        if let Ok(config) = serde_json::from_str(&content) {
            println!("Loaded config from config.json");
            return Ok(config);
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