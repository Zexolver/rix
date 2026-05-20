use std::process::Command;
use std::path::Path;
use std::fs;
use std::os::linux::fs::MetadataExt; // Crucial for UID parsing
use crate::errors::RixError;

#[derive(Debug, Clone, PartialEq)]
pub enum TargetPlatform {
    NixOS,
    MultiUserLinux,  // Your Velvet OS Chromebook environment
    SingleUserLinux, // Pure home directory user environment
    MacOS,
}

pub fn detect_target_platform() -> TargetPlatform {
    if cfg!(target_os = "macos") {
        return TargetPlatform::MacOS;
    }

    if Path::new("/etc/NIXOS").exists() || Path::new("/run/current-system").exists() {
        return TargetPlatform::NixOS;
    }

    // Sniff the installation model via the global store permissions
    if let Ok(metadata) = fs::metadata("/nix/store") {
        if metadata.st_uid() == 0 {
            return TargetPlatform::MultiUserLinux;
        }
    }

    TargetPlatform::SingleUserLinux
}

pub fn update_indexes() -> Result<(), RixError> {
    // Modern flake-native channel validation update
    let status = Command::new("nix").args(["flake", "update"]).status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(RixError::ParseError("Failed to update Flake lock references".to_string())),
    }
}

/// Applies changes across all systems using the proper backend command wrappers
pub fn apply_upgrade(config_path: &Path) -> Result<(), RixError> {
    let platform = detect_target_platform();
    let config_str = config_path.to_string_lossy().to_string();

    let mut cmd = if platform == TargetPlatform::NixOS {
        let mut c = Command::new("sudo");
        c.args(["nixos-rebuild", "switch", "--flake", &format!("{}#system", config_str)]);
        c
    } else if platform == TargetPlatform::MultiUserLinux {
        // Force the execution global profile configuration system-wide on Velvet OS
        let mut c = Command::new("sudo");
        c.args([
            "nix", "profile", "install", 
            "--profile", "/nix/var/nix/profiles/default", 
            &format!("{}#default", config_str)
        ]);
        c
    } else {
        // Pure single-user mode (No sudo required)
        let mut c = Command::new("nix");
        c.args(["profile", "install", &format!("{}#default", config_str)]);
        c
    };

    let status = cmd.status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(RixError::ParseError("Failed to materialize declarative generation updates".to_string())),
    }
}
