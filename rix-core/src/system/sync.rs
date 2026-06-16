use std::path::Path;
use std::io::{Error, ErrorKind};
use std::process::{Command, Stdio};
use git2::{Repository, Signature, IndexAddOption};
use crate::errors::RixError;

/// Initializes a barebones local Git repository in the config directory
pub fn init_local_repo(config_dir: &Path) -> Result<(), RixError> {
    match Repository::init(config_dir) {
        Ok(_) => Ok(()),
        Err(e) => Err(RixError::IOError(Error::new(
            ErrorKind::Other,
            format!("Failed to initialize Git repository: {}", e)
        ))),
    }
}

/// Automatically stages all changed files and commits them with the given message
pub fn auto_commit(config_dir: &Path, message: &str) -> Result<(), RixError> {
    let repo = Repository::open(config_dir)
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to open Git repository: {}", e))))?;

    let mut index = repo.index()
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to open Git index: {}", e))))?;

    // Stage all changes (equivalent to `git add .`)
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to stage files: {}", e))))?;
    index.write()
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to write Git index: {}", e))))?;

    let oid = index.write_tree()
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to write Git tree: {}", e))))?;
    let tree = repo.find_tree(oid)
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to find Git tree: {}", e))))?;

    // We use a generic signature for the auto-commits
    let sig = Signature::now("Rix Optimizer", "rix@localhost")
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to create commit signature: {}", e))))?;

    // Check if there is a parent commit (HEAD)
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit().map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to peel HEAD: {}", e))))?),
        Err(_) => None, // This is the very first commit
    };

    let mut parents = Vec::new();
    if let Some(ref parent) = parent_commit {
        parents.push(parent);
    }

    // Create the commit
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &parents,
    ).map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to create commit: {}", e))))?;

    Ok(())
}

/// Syncs the local declarative state to an upstream Git remote
pub fn sync_to_remote(config_dir: &Path, remote_url: Option<&str>) -> Result<(), RixError> {
    // 1. Ensure the default branch is strictly named 'main'
    Command::new("git")
        .current_dir(config_dir)
        .args(["branch", "-M", "main"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to set branch to main: {}", e))))?;

    // 2. If a remote URL is provided, link it (add or update)
    if let Some(url) = remote_url {
        let add_status = Command::new("git")
            .current_dir(config_dir)
            .args(["remote", "add", "origin", url])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap_or_default();

        // If 'origin' already exists, the add command fails. We catch that and update the URL instead.
        if !add_status.success() {
            Command::new("git")
                .current_dir(config_dir)
                .args(["remote", "set-url", "origin", url])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to update remote URL: {}", e))))?;
        }
    }

    // 3. Construct the push command structure
    let mut push_cmd = Command::new("git");
    push_cmd.current_dir(config_dir).args(["push", "-u", "origin", "main"]);

    // 🌟 AUTOMAGIC PRIVILEGE BRIDGE: Detect if root context is executing via sudo
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let user_home = format!("/home/{}", sudo_user);
        let ed25519_key = format!("{}/.ssh/id_ed25519", user_home);
        let rsa_key = format!("{}/.ssh/id_rsa", user_home);

        // Map candidate private key locations
        let key_path = if Path::new(&ed25519_key).exists() {
            Some(ed25519_key)
        } else if Path::new(&rsa_key).exists() {
            Some(rsa_key)
        } else {
            None
        };

        // Inject environment bypass for the spawned Git process if a key match hits
        if let Some(key) = key_path {
            push_cmd.env("GIT_SSH_COMMAND", format!("ssh -i {} -o IdentitiesOnly=yes", key));
        }
    }

    // 4. Push to the remote (inherits native console feedback)
    let status = push_cmd
        .status()
        .map_err(|e| RixError::IOError(Error::new(ErrorKind::Other, format!("Failed to execute git push: {}", e))))?;

    if !status.success() {
        return Err(RixError::IOError(Error::new(
            ErrorKind::Other,
            "Failed to sync with the remote repository. Ensure you have permissions and the remote is reachable."
        )));
    }

    Ok(())
}
