use std::env;
use std::fs;
use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    // 1. Check for an explicit environment override variable first
    if let Ok(custom_path) = env::var("RIX_CONFIG_DIR") {
        return PathBuf::from(custom_path);
    }

    // 2. Resolve the real user's home directory (even if running under sudo)
    let home_dir = env::var("SUDO_USER")
        .map(|u| format!("/home/{}", u))
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| "/root".to_string());

    let user_config_dir = PathBuf::from(home_dir).join(".config").join("rix");
    let toml_path = user_config_dir.join("rix.toml");

    // 3. Read the default scope from rix.toml
    let mut default_is_system = false;
    
    if let Ok(content) = fs::read_to_string(&toml_path) {
        // Basic text parsing to avoid bringing in heavy TOML dependencies for one key
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("default_scope") && trimmed.contains("\"system\"") {
                default_is_system = true;
                break;
            }
        }
    } else {
        // Fallback: If no config exists, guess based on execution privileges
        default_is_system = env::var("USER").unwrap_or_default() == "root"  
            || env::var("SUDO_USER").is_ok();
    }

    // 4. Route to the appropriate standard directory layout
    if default_is_system {
        PathBuf::from("/etc/rix")
    } else {
        user_config_dir
    }
}
