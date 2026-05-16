use std::process::Command;
use crate::errors::RixError;

/// Pulls down channel/flake index updates from upstream repositories
pub fn update_indexes() -> Result<(), RixError> {
    let status = Command::new("nix-channel").arg("--update").status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(RixError::ParseError("Failed to synchronize Nix upstream index channels".to_string())),
    }
}

/// Executes home-manager build activation cycle to materialize packages
pub fn apply_upgrade() -> Result<(), RixError> {
    let status = Command::new("home-manager").arg("switch").status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(RixError::ParseError("Failed to build or activate environment generation changes".to_string())),
    }
}
