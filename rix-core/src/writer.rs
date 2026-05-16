use std::fs;
use std::path::Path;
use crate::errors::RixError;

pub fn write_content_to_file(file_path: &Path, content: &str) -> Result<(), RixError> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

/// Formats and serializes a collection of package tuples back into standard Nix list format
pub fn write_nix_file(file_path: &Path, packages: Vec<(String, String)>) -> Result<(), RixError> {
    let mut content = String::from("{ pkgs }: [\n");
    for (name, description) in packages {
        content.push_str(&format!("  pkgs.{} # {}\n", name, description));
    }
    content.push_str("]\n");
    write_content_to_file(file_path, &content)
}

/// Returns the master entry point Flake file layout
pub fn get_bootstrap_flake_template() -> &'static str {
    r#"{
  description = "Rix automated system layout profile configuration";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }: {
    # System structural configurations go here
  };
}"#
}

/// Returns a clean group template file layout
pub fn get_empty_group_template() -> &'static str {
    r#"{ pkgs }: [
  # Packages managed by Rix
]"#
}
