use std::fs;
use std::path::Path;
use std::env;
use rnix::Root;
use crate::errors::RixError;

pub fn write_content_to_file(file_path: &Path, content: &str) -> Result<(), RixError> {
    // Pass the content through the rnix AST parser to guarantee valid Nix syntax
    let parse = Root::parse(content);
    if !parse.errors().is_empty() {
        let err_msgs: Vec<String> = parse.errors().iter().map(|e| e.to_string()).collect();
        return Err(RixError::InvalidNixSyntax(format!(
            "rnix AST validation failed before write: {:?}", 
            err_msgs
        )));
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

/// Formats and serializes a collection of package tuples back into standard Nix list format
pub fn write_nix_file(file_path: &Path, packages: Vec<(String, String)>) -> Result<(), RixError> {
    let mut content = String::from("{ pkgs, ... }:\n[\n");
    for (name, description) in packages {
        content.push_str(&format!("  pkgs.{} # {}\n", name, description));
    }
    content.push_str("]\n");
    write_content_to_file(file_path, &content)
}

/// Returns the master entry point Flake file layout, injected with user environment variables
pub fn get_bootstrap_flake_template() -> String {
    let user = env::var("USER").unwrap_or_else(|_| "default".to_string());
    let home = env::var("HOME").unwrap_or_else(|_| format!("/home/{}", user));
    
    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64-linux",
        "aarch64" => "aarch64-linux",
        _ => "x86_64-linux", 
    };

    // Fixed the import path to match the CLI's default group
    format!(r#"{{
  description = "Rix automated system layout profile configuration";

  inputs = {{
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    home-manager = {{
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
  }};

  outputs = {{ self, nixpkgs, home-manager, ... }}:
    let
      system = "{}";
      pkgs = nixpkgs.legacyPackages.${{system}};
    in {{
      homeConfigurations."{}" = home-manager.lib.homeManagerConfiguration {{
        inherit pkgs;
        modules = [
          {{
            home.username = "{}";
            home.homeDirectory = "{}";
            home.stateVersion = "24.05";
            
            home.packages = import ./groups/upstream/default.nix {{ inherit pkgs; }};
          }}
        ];
      }};
    }};
}}"#, arch, user, user, home)
}

/// Returns a clean group template file layout
pub fn get_empty_group_template() -> &'static str {
    r#"{ pkgs, ... }:
[
  # Packages managed by Rix
]"#
}
