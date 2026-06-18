use std::process::Command;
use crate::errors::RixError;

pub fn check_system_sanity() -> Result<(), RixError> {
    let status = Command::new("which")
        .arg("nix")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    
    if !matches!(status, Ok(s) if s.success()) {
        return Err(RixError::MissingSystemDependency(
            "Critical dependency 'nix' not found on system PATH. Ensure modern Nix is installed.".to_string()
        ));
    }
    Ok(())
}

/// Performs a dry-run check to ensure a remote flake URI actually resolves
pub fn verify_flake_resolves(uri: &str) -> Result<(), RixError> {
    let output = Command::new("nix")
        .args(["flake", "metadata", uri, "--json"])
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(RixError::InvalidNixSyntax(format!(
            "Flake validation failed. The target '{}' may not be a valid Nix flake.\n{}", 
            uri, err_msg
        )));
    }
    
    Ok(())
}
