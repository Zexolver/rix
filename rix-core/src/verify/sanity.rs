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
