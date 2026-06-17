use git2::{Repository, IndexAddOption, Signature, Config};
use std::path::Path;
use crate::errors::RixError;

pub fn initialize_state_repo(target_dir: &Path) -> Result<(), RixError> {
    // 0. Automatically trust the directory to bypass Git's "dubious ownership" warning.
    if let Ok(mut config) = Config::open_default() {
        let path_str = target_dir.to_string_lossy();
        
        let mut already_safe = false;
        
        // ConfigEntries provides an iterator-like interface via next()
        if let Ok(mut entries) = config.entries(Some("safe.directory")) {
            while let Some(entry) = entries.next() {
                if let Ok(entry) = entry {
                    // In git2 v0.21+, value() returns a Result, so we unpack it
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
            // "^$" matches nothing, which tells libgit2 to append to the multivar 
            // without overwriting any existing safe.directory paths.
            let _ = config.set_multivar("safe.directory", "^$", path_str.as_ref());
        }
    }

    // Prevent double-initialization if the repo already exists
    if target_dir.join(".git").exists() {
        return Ok(());
    }

    // 1. Initialize the barebone repository
    let repo = Repository::init(target_dir)?;

    // 2. Stage all newly generated configuration files
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // 3. Write the tree from the index
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // 4. Resolve commit signature with a safe fallback
    let sig = repo.signature().unwrap_or_else(|_| {
        Signature::now("Rix Provisioner", "rix@local").unwrap()
    });

    // 5. Create the initial root commit
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_git_repo_initialization() {
        let dir = tempdir().expect("Failed to create temp dir");
        let repo_path = dir.path();

        fs::write(repo_path.join("flake.nix"), "{ }").unwrap();

        initialize_state_repo(repo_path).expect("Git init failed");

        assert!(repo_path.join(".git").exists(), ".git directory missing");

        let repo = git2::Repository::open(repo_path).expect("Could not open repo");
        let mut revwalk = repo.revwalk().expect("Could not create revwalk");
        revwalk.push_head().expect("Could not push HEAD");
        
        assert_eq!(revwalk.count(), 1, "Expected exactly 1 commit in history");
    }
}
