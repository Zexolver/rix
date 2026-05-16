use std::process::Command;
use std::path::Path;
use crate::errors::RixError;

/// Validates that required external binary dependencies exist on the user's PATH
pub fn check_system_sanity() -> Result<(), RixError> {
    let binaries = ["nix-env", "nix-instantiate"];
    for bin in &binaries {
        let status = Command::new("which")
            .arg(bin)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        
        match status {
            Ok(s) if s.success() => {}
            _ => return Err(RixError::MissingSystemDependency(format!(
                "Critical dependency '{}' not found on system PATH. Please ensure Nix is installed properly.", bin
            ))),
        }
    }
    Ok(())
}

/// Uses 'nix-instantiate --parse' to guarantee our modified file is 100% syntactically valid
pub fn verify_nix_syntax(file_path: &Path) -> Result<(), RixError> {
    if !file_path.exists() {
        return Ok(());
    }

    let output = Command::new("nix-instantiate")
        .arg("--parse")
        .arg(file_path)
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            Err(RixError::InvalidNixSyntax(format!(
                "Generated file failed syntax validation: {}", stderr.trim()
            )))
        }
        Err(_) => Err(RixError::ParseError("Could not execute syntax validator 'nix-instantiate'".to_string())),
    }
}
