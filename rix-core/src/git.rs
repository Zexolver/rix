use crate::errors::RixError;
use git2::{Config, IndexAddOption, Repository, ResetType, Signature};
use std::env;
use std::path::Path;

pub fn initialize_state_repo(target_dir: &Path) -> Result<(), RixError> {
    let path_str = target_dir.to_string_lossy();

    let apply_safe_dir = |mut config: Config| {
        let mut already_safe = false;
        if let Ok(mut entries) = config.entries(Some("safe.directory")) {
            while let Some(entry) = entries.next() {
                if let Ok(entry) = entry {
                    if let Ok(val) = entry.value() {
                        if val == path_str.as_ref() {
                            already_safe = true;
                            break;
                        }
                    }
                }
            }
        }
        if !already_safe {
            let _ = config.set_multivar("safe.directory", "^$", path_str.as_ref());
        }
    };

    if let Ok(config) = Config::open_default() {
        apply_safe_dir(config);
    }

    if let Ok(sudo_user) = env::var("SUDO_USER") {
        let user_config_path = format!("/home/{}/.gitconfig", sudo_user);
        if let Ok(config) = Config::open(Path::new(&user_config_path)) {
            apply_safe_dir(config);
        }
    }

    // Helper to enforce the Rix identity on the local repository
    let apply_local_identity = |repo: &Repository| {
        if let Ok(mut config) = repo.config() {
            let _ = config.set_str("user.name", "Rix System Manager");
            let _ = config.set_str("user.email", "rix@localhost");
        }
    };

    if target_dir.join(".git").exists() {
        // Ensure if it's a legacy repo on 'master', we gracefully migrate it to 'main'
        if let Ok(repo) = Repository::open(target_dir) {
            apply_local_identity(&repo); // Ensure existing repos get the identity too
            if let Ok(mut master_branch) = repo.find_branch("master", git2::BranchType::Local) {
                if repo.find_branch("main", git2::BranchType::Local).is_err() {
                    let _ = master_branch.rename("main", false);
                }
            }
        }
        return Ok(());
    }

    let repo = Repository::init(target_dir)?;
    apply_local_identity(&repo);

    // Force the default branch to be 'main' instead of 'master'
    repo.set_head("refs/heads/main")?;

    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let sig = repo
        .signature()
        .unwrap_or_else(|_| Signature::now("Rix System Manager", "rix@localhost").unwrap());

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "chore: initialize Rix environment state",
        &tree,
        &[],
    )?;

    Ok(())
}

/// Commits the current working directory to lock in a known-good state
pub fn commit_state(target_dir: &Path, message: &str) -> Result<(), RixError> {
    let repo = Repository::open(target_dir)?;
    let mut index = repo.index()?;

    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let head = repo.head()?.peel_to_commit()?;
    let sig = repo
        .signature()
        .unwrap_or_else(|_| Signature::now("Rix System Manager", "rix@localhost").unwrap());

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&head])?;
    Ok(())
}

/// Hard resets the configuration directory, wiping away broken modifications
pub fn rollback_to_head(target_dir: &Path) -> Result<(), RixError> {
    let repo = Repository::open(target_dir)?;

    // Peel down to the commit, then reference it as a generic Object for the reset
    let head_commit = repo.head()?.peel_to_commit()?;

    // Hard reset wipes working directory and index back to the last commit
    repo.reset(head_commit.as_object(), ResetType::Hard, None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_git_repo_initialization() {
        let dir = tempdir().expect("Failed to create temp dir");
        let repo_path = dir.path();

        fs::write(repo_path.join("flake.nix"), "{ }").unwrap();

        initialize_state_repo(repo_path).expect("Git init failed");

        assert!(repo_path.join(".git").exists(), ".git directory missing");

        let repo = git2::Repository::open(repo_path).expect("Could not open repo");

        // Verify identity was set
        let config = repo.config().expect("Could not get repo config");
        assert_eq!(
            config.get_string("user.name").unwrap(),
            "Rix System Manager"
        );
        assert_eq!(config.get_string("user.email").unwrap(), "rix@localhost");

        let mut revwalk = repo.revwalk().expect("Could not create revwalk");
        revwalk.push_head().expect("Could not push HEAD");

        assert_eq!(revwalk.count(), 1, "Expected exactly 1 commit in history");
        assert!(
            repo.find_branch("main", git2::BranchType::Local).is_ok(),
            "Branch should be 'main'"
        );
    }
}
