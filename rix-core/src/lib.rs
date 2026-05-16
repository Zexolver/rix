use std::fs;
use std::io;
use std::path::PathBuf;

/// Custom error types for Rix engine operations
#[derive(Debug)]
pub enum RixError {
    Io(io::Error),
    ParseError(String),
    PackageNotFound(String),
}

impl From<io::Error> for RixError {
    fn from(err: io::Error) -> Self {
        RixError::Io(err)
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub description: Option<String>,
    pub group: String,
    pub is_local_recipe: bool,
}

pub struct RixContext {
    pub home_manager_dir: PathBuf,
}

impl RixContext {
    pub fn new(home_manager_dir: PathBuf) -> Self {
        Self { home_manager_dir }
    }

    /// Ensures the basic directory structure exists layout-wise
    pub fn initialize_layout(&self) -> Result<(), RixError> {
        let upstream_dir = self.home_manager_dir.join("groups/upstream");
        let local_dir = self.home_manager_dir.join("groups/local");

        fs::create_dir_all(&upstream_dir)?;
        fs::create_dir_all(&local_dir)?;

        Ok(())
    }

    /// Generates a blank group configuration file with structural educational comments
    pub fn create_empty_upstream_group(&self, group_name: &str) -> Result<PathBuf, RixError> {
        let file_path = self.home_manager_dir.join(format!("groups/upstream/{}.nix", group_name));
        
        if !file_path.exists() {
            let initial_template = String::from(
                "# This Nix module defines an isolated package group profile.\n\
                 # It is structured as a function accepting 'pkgs' (the Nix package collection)\n\
                 # and outputs a flat list containing package derivations.\n\
                 { pkgs, ... }:\n\n\
                 [\n\
                   # --- Managed by Rix: Packaged Tools ---\n\
                 ]\n"
            );
            fs::write(&file_path, initial_template)?;
        }

        Ok(file_path)
    }
}
