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

/// Dynamically injects a new group import into the master flake.nix modules array
pub fn link_group_to_flake(config_dir: &Path, group: &str) -> Result<(), RixError> {
    let flake_path = config_dir.join("flake.nix");
    if !flake_path.exists() {
        return Ok(()); // Handled by initialization, but safe fallback
    }

    let content = fs::read_to_string(&flake_path)?;
    let import_statement = format!("import ./groups/upstream/{}.nix", group);

    // If the group is already linked, do nothing
    if content.contains(&import_statement) {
        return Ok(());
    }

    // Prepare the inline Nix module block
    let module_inject = format!("          {{ home.packages = {} {{ inherit pkgs; }}; }}", import_statement);

    let mut new_content = String::new();
    let mut injected = false;

    for line in content.lines() {
        new_content.push_str(line);
        new_content.push('\n');

        // Look for the exact opening of the modules array
        if !injected && line.contains("modules = [") {
            new_content.push_str(&module_inject);
            new_content.push('\n');
            injected = true;
        }
    }

    if !injected {
        return Err(RixError::ParseError(
            "Could not find 'modules = [' array in flake.nix to auto-link group".into()
        ));
    }

    // The safety net: rnix will parse this reconstructed Flake before saving it!
    writer::write_content_to_file(&flake_path, &new_content)
}
