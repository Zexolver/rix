use std::env;
use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    // 1. Check for an explicit environment override variable first
    if let Ok(custom_path) = env::var("RIX_CONFIG_DIR") {
        return PathBuf::from(custom_path);
    }

    // 2. Determine if executing with system-level privileges
    let is_root = env::var("USER").unwrap_or_default() == "root" 
        || env::var("SUDO_USER").is_ok();

    // 3. Route to the appropriate standard directory layout
    if is_root {
        PathBuf::from("/etc/rix")
    } else {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home_dir).join(".config/rix")
    }
}
