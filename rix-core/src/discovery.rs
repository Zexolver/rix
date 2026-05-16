use std::fs;
use std::path::{Path, PathBuf};
use crate::parser;
use crate::errors::RixError;

pub struct FoundPackage {
    pub name: String,
    pub file_path: PathBuf,
}

/// Dynamic lookups. Returns a list of packages matching full string or prefix.
pub fn find_packages_in_upstream(upstream_dir: &Path, query: &str) -> Result<Vec<FoundPackage>, RixError> {
    let mut matches = Vec::new();
    let query_lower = query.to_lowercase();

    if let Ok(entries) = fs::read_dir(upstream_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "nix") {
                let content = fs::read_to_string(&path)?;
                if let Ok(root) = parser::parse_root_node(&content) {
                    if let Some(list) = parser::find_list_node(&root) {
                        for (full_pkg_name, _) in parser::extract_packages_from_list(&list) {
                            let clean_name = full_pkg_name.strip_prefix("pkgs.").unwrap_or(&full_pkg_name);
                            if clean_name.to_lowercase().starts_with(&query_lower) {
                                matches.push(FoundPackage {
                                    name: clean_name.to_string(),
                                    file_path: path.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(matches)
}

/// Pulls out absolutely every package tracking element across all group files
pub fn list_all_packages(upstream_dir: &Path) -> Result<Vec<(String, String, String)>, RixError> {
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
