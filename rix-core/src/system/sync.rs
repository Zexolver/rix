use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;
use crate::errors::RixError;

pub fn init_local_repo(target_dir: &Path) -> Result<(), RixError> {
    crate::git::initialize_state_repo(target_dir)
}

pub fn auto_commit(target_dir: &Path, message: &str) -> Result<(), RixError> {
    crate::git::commit_state(target_dir, message)
}

pub fn sync_to_remote(target_dir: &Path, remote_url: Option<&str>) -> Result<(), RixError> {
    // 1. Configure the remote if a URL is provided
    if let Some(url) = remote_url {
        let add_status = Command::new("git")
            .current_dir(target_dir)
            .args(["remote", "add", "origin", url])
            .output()
            .map_err(|_| RixError::IOError(Error::new(ErrorKind::Other, "Failed to execute git remote add")))?;

        if !add_status.status.success() {
            Command::new("git")
                .current_dir(target_dir)
                .args(["remote", "set-url", "origin", url])
                .output()
                .map_err(|_| RixError::IOError(Error::new(ErrorKind::Other, "Failed to execute git remote set-url")))?;
        }
    }

    // 2. Prepare the push command
    let mut push_cmd = Command::new("git");
    push_cmd.current_dir(target_dir).args(["push", "-u", "origin", "main"]);

    // --- OPTION D: Automate SSH Authentication ---
    // If running under sudo, dynamically inject the original user's SSH credentials
    if let Ok(sudo_user) = env::var("SUDO_USER") {
        let ed25519 = format!("/home/{}/.ssh/id_ed25519", sudo_user);
        let rsa = format!("/home/{}/.ssh/id_rsa", sudo_user);

        // Check standard paths for the user's private key
        let ssh_key = if Path::new(&ed25519).exists() {
            Some(ed25519)
        } else if Path::new(&rsa).exists() {
            Some(rsa)
        } else {
            None
        };

        if let Some(key_path) = ssh_key {
            // Force this specific Git subprocess to use the user's key and accept new hosts
            push_cmd.env(
                "GIT_SSH_COMMAND",
                format!("ssh -i {} -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new", key_path)
            );
        }
    }

    // 3. Execute the push
    let output = push_cmd
        .output()
        .map_err(|_| RixError::IOError(Error::new(ErrorKind::Other, "Failed to execute git push command")))?;

    // 4. Handle errors
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(RixError::IOError(Error::new(ErrorKind::Other, format!(
            "Failed to sync with the remote repository. Ensure you have permissions and the remote is reachable.\nGit output: {}",
            err_msg.trim()
        ))));
    }

    Ok(())
}
