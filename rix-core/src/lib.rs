pub mod errors;
pub mod parser;
pub mod discovery;
pub mod writer;
pub mod system;
pub mod ops;

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

    pub fn initialize_layout(&self) -> Result<(), RixError> {
        fs::create_dir_all(self.home_manager_dir.join("groups/upstream"))?;
        fs::create_dir_all(self.home_manager_dir.join("groups/local"))?;
        Ok(())
    }

    pub fn add_package(&self, package: Package) -> Result<(), RixError> {
        self.initialize_layout()?;
        let upstream_dir = self.home_manager_dir.join("groups/upstream");
        ops::add_package(&upstream_dir, package)
    }

    pub fn lookup_packages(&self, query: &str) -> Result<Vec<FoundPackage>, RixError> {
        let upstream_dir = self.home_manager_dir.join("groups/upstream");
        discovery::find_packages_in_upstream(&upstream_dir, query)
    }

    pub fn list_all_packages(&self) -> Result<Vec<(String, String, String)>, RixError> {
        let upstream_dir = self.home_manager_dir.join("groups/upstream");
        let mut all_packages = Vec::new();

        if let Ok(entries) = fs::read_dir(upstream_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "nix") {
                    let group_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    let content = fs::read_to_string(&path)?;
                    if let Ok(root) = parser::parse_root_node(&content) {
                        if let Some(list) = parser::find_list_node(&root) {
                            for (full_pkg, comment) in parser::extract_packages_from_list(&list) {
                                let clean_name = full_pkg.strip_prefix("pkgs.").unwrap_or(&full_pkg).to_string();
                                all_packages.push((clean_name, group_name.clone(), comment));
                            }
                        }
                    }
                }
            }
        }
        all_packages.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(all_packages)
    }

    pub fn remove_package_from_file(&self, name: &str, file_path: &PathBuf) -> Result<(), RixError> {
        ops::remove_package_from_file(name, file_path)
    }

    pub fn purge_group_profile(&self, group: &str) -> Result<(), RixError> {
        let file_path = self.home_manager_dir.join(format!("groups/upstream/{}.nix", group));
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    pub fn update_indexes(&self) -> Result<(), RixError> {
        system::update_indexes()
    }

    pub fn apply_upgrade(&self) -> Result<(), RixError> {
        system::apply_upgrade()
    }
}
