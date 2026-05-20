use std::env;
use std::path::PathBuf;
use rix_core::system::{detect_target_platform, TargetPlatform};

pub fn get_config_dir() -> PathBuf {
    match detect_target_platform() {
        TargetPlatform::NixOS | TargetPlatform::MultiUserLinux => {
            PathBuf::from("/etc/rix") // System-wide storage target location
        }
        _ => {
            let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"));
            home_dir.join(".config/rix") // Isolated user-space folder path location
        }
    }
}
