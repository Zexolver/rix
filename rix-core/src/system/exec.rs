use std::path::Path;
use std::process::Command;
use crate::errors::RixError;
use super::platform::{detect_target_platform, TargetPlatform};

pub fn update_indexes() -> Result<(), RixError> {
    // Globally inject Nix config via environment variables to cover child processes
    let status = Command::new("nix")
        .env("NIX_CONFIG", "experimental-features = nix-command flakes")
        .args(["flake", "update"])
        .status();
        
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(RixError::ParseError("Failed to update Flake lock references".to_string())),
    }
}

pub fn apply_upgrade(config_path: &Path) -> Result<(), RixError> {
    let platform = detect_target_platform();
    let config_str = config_path.to_string_lossy().to_string();

    let mut cmd = if platform == TargetPlatform::NixOS {
        let mut c = Command::new("sudo");
        c.env("NIX_CONFIG", "experimental-features = nix-command flakes");
        c.args([
            "nixos-rebuild", "switch", 
            "--flake", &format!("{}#system", config_str)
        ]);
        c
    } else {
        let mut c = Command::new("nix");
        // This guarantees home-manager inherits the experimental features for its internal Nix calls
        c.env("NIX_CONFIG", "experimental-features = nix-command flakes");
        c.args([
            "run", "nixpkgs#home-manager", "--", 
            "switch", "--flake", &config_str
        ]);
        c
    };

    let status = cmd.status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(RixError::ParseError("Failed to materialize declarative generation updates".to_string())),
    }
}
