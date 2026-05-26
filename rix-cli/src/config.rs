use std::env;
use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    // Check for an explicit environment override variable first, otherwise default to user-space XDG configuration path
    if let Ok(custom_path) = env::var("RIX_CONFIG_DIR") {
        PathBuf::from(custom_path)
    } else {
        let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"));
        home_dir.join(".config/rix")
    }
}
