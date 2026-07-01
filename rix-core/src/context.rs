use crate::discovery::FoundPackage;
use crate::errors::RixError;
use crate::{discovery, git, hardware, ops, system, verify, writer};
use std::fs;
use std::path::PathBuf;

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
        let is_system = config_dir.starts_with("/etc/rix")
            || std::env::var("USER").unwrap_or_default() == "root"
            || std::env::var("SUDO_USER").is_ok();

        if is_system {
            if let Ok(current_path) = std::env::var("PATH") {
                let nix_path = "/nix/var/nix/profiles/default/bin";
                if !current_path.contains(nix_path) {
                    unsafe {
                        std::env::set_var("PATH", format!("{}:{}", current_path, nix_path));
                    }
                }
            }
        }

        Self {
            config_dir,
            is_system,
        }
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
            // Auto-generate the .gitignore alongside the flake initialization
            writer::write_default_gitignore(&self.config_dir)?;
        }

        let default_upstream = upstream_dir.join("default.nix");
        if !default_upstream.exists() {
            writer::write_content_to_file(&default_upstream, writer::get_empty_group_template())?;
        }

        git::initialize_state_repo(&self.config_dir)?;

        Ok(())
    }

    pub fn add_package(&self, package: Package) -> Result<(), RixError> {
        self.initialize_layout()?;

        // 1. PRE-FLIGHT CHECK: Sniff for external URIs and validate them
        if package.name.contains("://")
            || package.name.starts_with("github:")
            || package.name.starts_with("gitlab:")
        {
            verify::verify_flake_resolves(&package.name)?;
        }

        let group_name = package.group.clone();
        let target_file = self
            .config_dir
            .join(format!("groups/upstream/{}.nix", group_name));

        let wrapper = hardware::get_nixgl_wrapper(&self.config_dir);

        ops::add_package(&self.config_dir.join("groups/upstream"), package, wrapper)?;
        ops::link_group_to_flake(&self.config_dir, &group_name)?;

        verify::verify_nix_syntax(&target_file)
    }

    pub fn lookup_packages(&self, query: &str) -> Result<Vec<FoundPackage>, RixError> {
        discovery::find_packages_in_upstream(&self.config_dir.join("groups/upstream"), query)
    }

    pub fn list_all_packages(&self) -> Result<Vec<(String, String, String)>, RixError> {
        discovery::list_all_packages(&self.config_dir.join("groups/upstream"))
    }

    pub fn remove_package_from_file(
        &self,
        name: &str,
        file_path: &PathBuf,
    ) -> Result<(), RixError> {
        let wrapper = hardware::get_nixgl_wrapper(&self.config_dir);
        ops::remove_package_from_file(name, file_path, wrapper)?;
        verify::verify_nix_syntax(file_path)
    }

    pub fn purge_group_profile(&self, group: &str) -> Result<(), RixError> {
        let file_path = self
            .config_dir
            .join(format!("groups/upstream/{}.nix", group));
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    pub fn update_indexes(&self) -> Result<(), RixError> {
        self.verify_system()?;
        // PASS THE CONFIG DIRECTORY AND ENVIRONMENT SCOPE DOWN TO THE EXECUTOR
        system::update_indexes(&self.config_dir, self.is_system)
    }

    pub fn apply_upgrade(&self, dry_run: bool) -> Result<(), RixError> {
        self.verify_system()?;

        // 2. TRANSACTIONAL BUILD: Attempt the upgrade
        match system::apply_upgrade(&self.config_dir, self.is_system, dry_run) {
            Ok(_) => {
                // SUCCESS: Lock in the new state automatically
                if !dry_run {
                    let _ = git::commit_state(
                        &self.config_dir,
                        "chore: automated Rix environment update",
                    );

                    // 🌟 NEW: Bridge binaries to /usr/local/bin if system-wide
                    if self.is_system {
                        let _ = system::bridge_system_binaries();
                    }
                }
                Ok(())
            }
            Err(e) => {
                // FAILURE: Rollback to the previous known-good state automatically
                if !dry_run {
                    let _ = git::rollback_to_head(&self.config_dir);
                }
                Err(e)
            }
        }
    }
}
