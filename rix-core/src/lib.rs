pub mod errors;
pub mod parser;
pub mod discovery;
pub mod writer;

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
        let file_path = self.home_manager_dir.join(format!("groups/upstream/{}.nix", package.group));
        
        if !file_path.exists() {
            fs::write(&file_path, "{ pkgs, ... }:\n\n[\n]\n")?;
        }
        
        let content = fs::read_to_string(&file_path)?;
        let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
        let list_node = parser::find_list_node(&root_node)
            .ok_or_else(|| RixError::ParseError("No list block [ ... ] found".to_string()))?;

        let mut packages = parser::extract_packages_from_list(&list_node);
        let formatted_pkg = format!("pkgs.{}", package.name);

        if packages.iter().any(|(name, _)| name == &formatted_pkg) {
            return Ok(());
        }

        let comment = package.description.unwrap_or_else(|| "Installed via Rix CLI".to_string());
        packages.push((formatted_pkg, comment));
        packages.sort_by(|a, b| a.0.cmp(&b.0));

        writer::write_nix_file(&file_path, packages)
    }

    pub fn lookup_packages(&self, query: &str) -> Result<Vec<FoundPackage>, RixError> {
        let upstream_dir = self.home_manager_dir.join("groups/upstream");
        discovery::find_packages_in_upstream(&upstream_dir, query)
    }

    pub fn remove_package_from_file(&self, name: &str, file_path: &PathBuf) -> Result<(), RixError> {
        let content = fs::read_to_string(file_path)?;
        let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
        let list_node = parser::find_list_node(&root_node)
            .ok_or_else(|| RixError::ParseError("No list block [ ... ] found".to_string()))?;

        let formatted_pkg = format!("pkgs.{}", name);
        let packages = parser::extract_packages_from_list(&list_node);
        let filtered_packages: Vec<(String, String)> = packages
            .into_iter()
            .filter(|(pkg_name, _)| pkg_name != &formatted_pkg)
            .collect();

        writer::write_nix_file(file_path, filtered_packages)
    }
}
