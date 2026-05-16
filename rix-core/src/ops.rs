use std::fs;
use std::path::Path;
use crate::errors::RixError;
use crate::{parser, writer, Package};

pub fn add_package(upstream_dir: &Path, package: Package) -> Result<(), RixError> {
    let file_path = upstream_dir.join(format!("{}.nix", package.group));
    
    if !file_path.exists() {
        fs::write(&file_path, "{ pkgs, ... }:\n\n[\n]\n")?;
    }
    
    let content = fs::read_to_string(&file_path)?;
    let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
    let list_node = parser::find_list_node(&root_node)
        .ok_or_else(|| RixError::ParseError("No list block [ ... ] found".to_string()))?;

    let mut packages = parser::extract_packages_from_list(&list_node);
    let formatted_pkg = format!("pkgs.{}", package.name);

    if let Some(pos) = packages.iter().position(|(name, _)| name == &formatted_pkg) {
        if let Some(new_desc) = package.description {
            packages[pos].1 = new_desc;
            return writer::write_nix_file(&file_path, packages);
        }
        return Ok(());
    }

    let comment = package.description.unwrap_or_else(|| "Installed via Rix CLI".to_string());
    packages.push((formatted_pkg, comment));
    packages.sort_by(|a, b| a.0.cmp(&b.0));

    writer::write_nix_file(&file_path, packages)
}

pub fn remove_package_from_file(name: &str, file_path: &Path) -> Result<(), RixError> {
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
