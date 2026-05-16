pub mod errors;
pub mod parser;

use std::fs;
use std::path::PathBuf;
pub use errors::RixError;

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
        let upstream_dir = self.home_manager_dir.join("groups/upstream");
        let local_dir = self.home_manager_dir.join("groups/local");
        fs::create_dir_all(upstream_dir)?;
        fs::create_dir_all(local_dir)?;
        Ok(())
    }

    pub fn create_empty_upstream_group(&self, group_name: &str) -> Result<PathBuf, RixError> {
        let file_path = self.home_manager_dir.join(format!("groups/upstream/{}.nix", group_name));
        if !file_path.exists() {
            let initial_template = String::from(
                "{ pkgs, ... }:\n\n[\n  # --- Managed by Rix: Packaged Tools ---\n]\n"
            );
            fs::write(&file_path, initial_template)?;
        }
        Ok(file_path)
    }

    pub fn add_package(&self, package: Package) -> Result<(), RixError> {
        self.initialize_layout()?;
        let file_path = self.create_empty_upstream_group(&package.group)?;
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

        let mut new_content = String::from("{ pkgs, ... }:\n\n[\n");
        for (pkg_name, pkg_comment) in packages {
            new_content.push_str(&format!("  {} # {}\n", pkg_name, pkg_comment));
        }
        new_content.push_str("]\n");

        fs::write(&file_path, new_content)?;
        Ok(())
    }
}
