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

/// Formats and serializes a collection of package tuples back into standard Nix list format.
/// Injects the nixGL wrapper if a hardware lockfile specifies one.
pub fn write_nix_file(file_path: &Path, packages: Vec<(String, String)>, nixgl_wrapper: Option<String>) -> Result<(), RixError> {
    let mut content = String::from("{ pkgs, ... }:\n[\n");
    
    for (name, description) in packages {
        // DEFENSIVE FIX: Smart prefixing.
        let is_complex_expr = name.starts_with('(') || 
                              name.starts_with('{') || 
                              name.starts_with('[') || 
                              name.starts_with('"') || 
                              name.starts_with("let ") || 
                              name.starts_with("with ");

        let formatted_name = if is_complex_expr {
            name.to_string()
        } else {
            // If a wrapper exists, wrap the binary. Otherwise, just prefix with pkgs.
            if let Some(ref wrapper) = nixgl_wrapper {
                format!("  (pkgs.writeShellScriptBin \"{}\" ''exec ${{pkgs.nixgl.{}}}/bin/{} ${{pkgs.{}}}/bin/{}'')", name, wrapper, wrapper, name, name)
            } else {
                format!("  pkgs.{}", name)
            }
        };

        // Avoid trailing '#' syntax errors if there is no comment attached
        if description.is_empty() {
            content.push_str(&format!("  {}\n", formatted_name.trim_start()));
        } else {
            content.push_str(&format!("  {} # {}\n", formatted_name.trim_start(), description));
        }
    }
    
    content.push_str("]\n");
    
    // The final safety net: rnix will parse this reconstructed string before writing to disk
    write_content_to_file(file_path, &content)
}

/// Returns the master entry point Flake file layout, injected with user environment variables and nixGL
pub fn get_bootstrap_flake_template() -> String {
    let user = env::var("USER").unwrap_or_else(|_| "default".to_string());
    let home = env::var("HOME").unwrap_or_else(|_| format!("/home/{}", user));
    
    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64-linux",
        "aarch64" => "aarch64-linux",
        _ => "x86_64-linux",  
    };

    format!(r#"{{
  description = "Rix automated system layout profile configuration";

  inputs = {{
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    home-manager = {{
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
    nixgl.url = "github:nix-community/nixGL"; # <-- ADDED NIXGL INPUT
  }};

  outputs = {{ self, nixpkgs, home-manager, nixgl, ... }}:
    let
      system = "{}";
      pkgs = import nixpkgs {{
        inherit system;
        overlays = [ nixgl.overlay ]; # <-- APPLIED NIXGL OVERLAY
      }};
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
