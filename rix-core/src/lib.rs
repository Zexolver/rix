pub mod compiler;
pub mod errors;
pub mod parser;
pub mod discovery;
pub mod writer;
pub mod system;
pub mod ops;
pub mod verify;

use std::fs;
use std::path::PathBuf;
pub use errors::RixError;
pub use discovery::FoundPackage;

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

    pub fn verify_system(&self) -> Result<(), RixError> {
        verify::check_system_sanity()
    }

    pub fn initialize_layout(&self) -> Result<(), RixError> {
        self.verify_system()?;
        
        let upstream_dir = self.home_manager_dir.join("groups/upstream");
        let local_dir = self.home_manager_dir.join("groups/local");
        
        fs::create_dir_all(&upstream_dir)?;
        fs::create_dir_all(&local_dir)?;

        // Deploy the master tracking flake if it doesn't already exist
        let flake_path = self.home_manager_dir.join("flake.nix");
        if !flake_path.exists() {
            writer::write_content_to_file(&flake_path, writer::get_bootstrap_flake_template())?;
        }

        // Deploy a default upstream profile tracking group file if missing
        let default_upstream = upstream_dir.join("default.nix");
        if !default_upstream.exists() {
            writer::write_content_to_file(&default_upstream, writer::get_empty_group_template())?;
        }

        Ok(())
    }

    pub fn add_package(&self, package: Package) -> Result<(), RixError> {
        self.initialize_layout()?;
        let target_file = self.home_manager_dir.join(format!("groups/upstream/{}.nix", package.group));
        ops::add_package(&self.home_manager_dir.join("groups/upstream"), package)?;
        
        // Safety lock: Verify what we wrote didn't break Nix structural constraints
        verify::verify_nix_syntax(&target_file)
    }

    pub fn lookup_packages(&self, query: &str) -> Result<Vec<FoundPackage>, RixError> {
        discovery::find_packages_in_upstream(&self.home_manager_dir.join("groups/upstream"), query)
    }

    pub fn list_all_packages(&self) -> Result<Vec<(String, String, String)>, RixError> {
        discovery::list_all_packages(&self.home_manager_dir.join("groups/upstream"))
    }

    pub fn remove_package_from_file(&self, name: &str, file_path: &PathBuf) -> Result<(), RixError> {
        ops::remove_package_from_file(name, file_path)?;
        // Safety lock
        verify::verify_nix_syntax(file_path)
    }

    pub fn purge_group_profile(&self, group: &str) -> Result<(), RixError> {
        let file_path = self.home_manager_dir.join(format!("groups/upstream/{}.nix", group));
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    pub fn update_indexes(&self) -> Result<(), RixError> {
        self.verify_system()?;
        system::update_indexes()
    }

    pub fn apply_upgrade(&self) -> Result<(), RixError> {
        self.verify_system()?;
        system::apply_upgrade()
    }
}
