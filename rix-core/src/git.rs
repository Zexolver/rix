use git2::{Repository, IndexAddOption, Signature};
use std::path::Path;
use crate::errors::RixError;

pub fn initialize_state_repo(target_dir: &Path) -> Result<(), RixError> {
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
        // Create a temporary directory that automatically deletes itself after the test
        let dir = tempdir().expect("Failed to create temp dir");
        let repo_path = dir.path();

        // Put a dummy file in it to simulate Rix generating templates
        fs::write(repo_path.join("flake.nix"), "{ }").unwrap();

        // Run our new Git initialization logic!
        initialize_state_repo(repo_path).expect("Git init failed");

        // 1. Assert the .git folder was created
        assert!(repo_path.join(".git").exists(), ".git directory missing");

        // 2. Open the repo and assert the commit was actually made
        let repo = git2::Repository::open(repo_path).expect("Could not open repo");
        let mut revwalk = repo.revwalk().expect("Could not create revwalk");
        revwalk.push_head().expect("Could not push HEAD");
        
        // Assert there is exactly 1 commit in the history
        assert_eq!(revwalk.count(), 1, "Expected exactly 1 commit in history");
    }
}
