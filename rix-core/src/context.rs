use std::fs;
use std::path::PathBuf;
use crate::{verify, system, ops, writer, discovery, hardware};
use crate::errors::RixError;
use crate::discovery::FoundPackage;

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub description: Option<String>,
    pub group: String,
    pub is_local_recipe: bool,
}

pub struct RixContext {
    pub config_dir: PathBuf,
    pub is_system: bool,
}

impl RixContext {
    pub fn new(config_dir: PathBuf) -> Self {
        // Automatically classify as system scope if targeting /etc/rix   
        // or if the process was explicitly escalated via root/sudo privileges
        let is_system = config_dir.starts_with("/etc/rix")   
            || std::env::var("USER").unwrap_or_default() == "root"
            || std::env::var("SUDO_USER").is_ok();

        // Security patch: `sudo` strips environment paths on standard Linux distributions.
        // We inject the default Nix profile bin back into the Rust process environment PATH
        // so all subsequent Command::new("nix") calls execute successfully when run under sudo.
        if is_system {
            if let Ok(current_path) = std::env::var("PATH") {
                let nix_path = "/nix/var/nix/profiles/default/bin";
                if !current_path.contains(nix_path) {
                    // SAFETY: Modifying the environment is unsafe in multithreaded contexts.
                    // This is safe here because Context initialization happens synchronously   
                    // at application startup before any threads are spawned.
                    unsafe {
                        std::env::set_var("PATH", format!("{}:{}", current_path, nix_path));
                    }
                }
            }
        }

        Self { config_dir, is_system }
    }

    pub fn verify_system(&self) -> Result<(), RixError> {
        verify::check_system_sanity()
    }

    pub fn initialize_layout(&self) -> Result<(), RixError> {
        self.verify_system()?;
         
        let upstream_dir = self.config_dir.join("groups/upstream");
        let local_dir = self.config_dir.join("groups/local");
         
        fs::create_dir_all(&upstream_dir)?;
        fs::create_dir_all(&local_dir)?;

        let flake_path = self.config_dir.join("flake.nix");
        if !flake_path.exists() {
            writer::write_content_to_file(&flake_path, &writer::get_bootstrap_flake_template())?;
        }

        let default_upstream = upstream_dir.join("default.nix");
        if !default_upstream.exists() {
            writer::write_content_to_file(&default_upstream, writer::get_empty_group_template())?;
        }

        // Initialize Git and snapshot the layout immediately after file creation
        crate::git::initialize_state_repo(&self.config_dir)?;

        Ok(())
    }

    pub fn add_package(&self, package: Package) -> Result<(), RixError> {
        self.initialize_layout()?;
         
        let group_name = package.group.clone();
        let target_file = self.config_dir.join(format!("groups/upstream/{}.nix", group_name));
         
        // Fetch the hardware wrapper if a lockfile exists
        let wrapper = hardware::get_nixgl_wrapper(&self.config_dir);

        // 1. Add the package to the group module (passing the wrapper down)
        ops::add_package(&self.config_dir.join("groups/upstream"), package, wrapper)?;
         
        // 2. Ensure this group is dynamically imported into the master flake.nix
        ops::link_group_to_flake(&self.config_dir, &group_name)?;
         
        // 3. Verify final syntax
        verify::verify_nix_syntax(&target_file)
    }

    pub fn lookup_packages(&self, query: &str) -> Result<Vec<FoundPackage>, RixError> {
        discovery::find_packages_in_upstream(&self.config_dir.join("groups/upstream"), query)
    }

    pub fn list_all_packages(&self) -> Result<Vec<(String, String, String)>, RixError> {
        discovery::list_all_packages(&self.config_dir.join("groups/upstream"))
    }

    pub fn remove_package_from_file(&self, name: &str, file_path: &PathBuf) -> Result<(), RixError> {
        // Fetch the hardware wrapper if a lockfile exists
        let wrapper = hardware::get_nixgl_wrapper(&self.config_dir);

        // Pass the wrapper down to match the function signature,  
        // though our new bulletproof ops::list.rs safely ignores it during removal.
        ops::remove_package_from_file(name, file_path, wrapper)?;
         
        verify::verify_nix_syntax(file_path)
    }

    pub fn purge_group_profile(&self, group: &str) -> Result<(), RixError> {
        let file_path = self.config_dir.join(format!("groups/upstream/{}.nix", group));
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    pub fn update_indexes(&self) -> Result<(), RixError> {
        self.verify_system()?;
        system::update_indexes()
    }

    pub fn apply_upgrade(&self, dry_run: bool) -> Result<(), RixError> {
        self.verify_system()?;
        system::apply_upgrade(&self.config_dir, self.is_system, dry_run)
    }
}
