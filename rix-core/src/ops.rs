use std::fs;
use std::path::Path;
use rnix::{Root, SyntaxKind};
use crate::errors::RixError;
use crate::{parser, writer, Package};

pub fn add_package(upstream_dir: &Path, package: Package) -> Result<(), RixError> {
    let file_path = upstream_dir.join(format!("{}.nix", package.group));
    
    if !file_path.exists() {
        fs::write(&file_path, "{ pkgs }: [\n]")?;
    }
    
    let content = fs::read_to_string(&file_path)?;
    let parsed = Root::parse(&content);
    if !parsed.errors().is_empty() {
        return Err(RixError::ParseError(format!("Syntax errors found: {:?}", parsed.errors())));
    }

    let root_node = parsed.syntax();
    
    let mut list_node_opt = None;
    for node in root_node.descendants() {
        if node.kind() == SyntaxKind::NODE_LIST {
            list_node_opt = Some(node);
            break;
        }
    }

    let list_node = list_node_opt
        .ok_or_else(|| RixError::ParseError("No valid list array token found inside target file block".to_string()))?;

    let formatted_pkg = format!("pkgs.{}", package.name);
    if list_node.text().to_string().contains(&formatted_pkg) {
        return Ok(()); 
    }

    let mut current_text = content.clone();
    let insertion_point = list_node.text_range().end();
    let offset: usize = insertion_point.into();

    let injection_str = format!("  {} # {}\n", formatted_pkg, package.description.unwrap_or_else(|| "Installed via Rix".to_string()));
    current_text.insert_str(offset - 2, &injection_str); 

    fs::write(&file_path, current_text)?;
    Ok(())
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
