use std::fs;
use std::path::Path;
use crate::errors::RixError;
use crate::{parser, writer, Package};

pub fn add_package(upstream_dir: &Path, package: Package) -> Result<(), RixError> {
    let file_path = upstream_dir.join(format!("{}.nix", package.group));
    
    if !file_path.exists() {
        fs::write(&file_path, writer::get_empty_group_template())?;
    }
    
    let content = fs::read_to_string(&file_path)?;
    let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
    let list_node = parser::find_list_node(&root_node)
        .ok_or_else(|| RixError::ParseError("No list block [ ... ] found in target file".to_string()))?;

    // Extract existing packages (already clean names)
    let mut packages = parser::extract_packages_from_list(&list_node);
    
    // Direct match check on the pure package name
    if packages.iter().any(|(name, _)| name == &package.name) {
        return Ok(()); 
    }

    let description = package.description.unwrap_or_else(|| "Installed via Rix".to_string());
    packages.push((package.name, description));

    // Rewrites file cleanly via AST validation path
    writer::write_nix_file(&file_path, packages)
}

pub fn remove_package_from_file(name: &str, file_path: &Path) -> Result<(), RixError> {
    let content = fs::read_to_string(file_path)?;
    let root_node = parser::parse_root_node(&content).map_err(RixError::ParseError)?;
    let list_node = parser::find_list_node(&root_node)
        .ok_or_else(|| RixError::ParseError("No list block [ ... ] found".to_string()))?;

    let packages = parser::extract_packages_from_list(&list_node);
    
    let filtered_packages: Vec<(String, String)> = packages
        .into_iter()
        .filter(|(pkg_name, _)| pkg_name != name)
        .collect();

    writer::write_nix_file(file_path, filtered_packages)
}
