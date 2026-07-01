use crate::errors::RixError;
use std::path::Path;
use std::process::Command;

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
                "Generated file failed syntax validation: {}",
                stderr.trim()
            )))
        }
        Err(_) => Err(RixError::ParseError(
            "Could not execute syntax validator".to_string(),
        )),
    }
}
