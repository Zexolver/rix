use std::path::Path;
use std::process::Command;
use std::io::{Error, ErrorKind};
use crate::errors::RixError;

/// Restores the local repository initializer wrapper
pub fn init_local_repo(target_dir: &Path) -> Result<(), RixError> {
    crate::git::initialize_state_repo(target_dir)
}

/// Restores the automatic state commit wrapper
pub fn auto_commit(target_dir: &Path, message: &str) -> Result<(), RixError> {
    crate::git::commit_state(target_dir, message)
}

pub fn sync_to_remote(target_dir: &Path, remote_url: Option<&str>) -> Result<(), RixError> {
    // 1. If a remote URL is provided via the CLI, configure the 'origin' remote
    if let Some(url) = remote_url {
        // Attempt to add the remote. This will fail if 'origin' already exists, which is fine!
        let add_status = Command::new("git")
            .current_dir(target_dir)
            .args(["remote", "add", "origin", url])
            .output()
            .map_err(|_| RixError::IOError(Error::new(ErrorKind::Other, "Failed to execute git remote add")))?;

        // If 'origin' already existed, update its URL just to be safe
        if !add_status.status.success() {
            Command::new("git")
                .current_dir(target_dir)
                .args(["remote", "set-url", "origin", url])
                .output()
                .map_err(|_| RixError::IOError(Error::new(ErrorKind::Other, "Failed to execute git remote set-url")))?;
        }
    }

    // 2. Perform the push, setting the upstream to 'origin/main'
    let output = Command::new("git")
        .current_dir(target_dir)
        .args(["push", "-u", "origin", "main"])
        .output()
        .map_err(|_| RixError::IOError(Error::new(ErrorKind::Other, "Failed to execute git push command")))?;

    // 3. Catch push failures (e.g., authentication issues, missing origin)
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(RixError::IOError(Error::new(ErrorKind::Other, format!(
            "Failed to sync with the remote repository. Ensure you have permissions and the remote is reachable.\nGit output: {}",
            err_msg.trim()
        ))));
    }

    Ok(())
}
