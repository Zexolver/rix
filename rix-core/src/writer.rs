use std::fs;
use std::path::Path;
use crate::errors::RixError;

pub fn write_nix_file(path: &Path, packages: Vec<(String, String)>) -> Result<(), RixError> {
    let mut new_content = String::from("{ pkgs, ... }:\n\n[\n");
    for (pkg_name, pkg_comment) in packages {
        new_content.push_str(&format!("  {} # {}\n", pkg_name, pkg_comment));
    }
    new_content.push_str("]\n");
    fs::write(path, new_content)?;
    Ok(())
}
