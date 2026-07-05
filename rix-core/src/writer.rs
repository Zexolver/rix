use crate::errors::RixError;
use rnix::Root;
use std::env;
use std::fs;
use std::path::Path;

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

/// Automatically generates a .gitignore file to prevent hardware-specific state
/// from polluting the Git repository.
pub fn write_default_gitignore(config_dir: &Path) -> Result<(), RixError> {
    let gitignore_path = config_dir.join(".gitignore");
    // Ignore the hardware lockfile and any Nix build 'result' symlinks
    let content = "hardware.lock\nresult\n";

    if !gitignore_path.exists() {
        fs::write(gitignore_path, content)?;
    }
    Ok(())
}

/// Formats and serializes a collection of package tuples back into standard Nix list format.
/// Injects the nixGL wrapper if a hardware lockfile specifies one.
pub fn write_nix_file(
    file_path: &Path,
    packages: Vec<(String, String)>,
    nixgl_wrapper: Option<String>,
) -> Result<(), RixError> {
    let mut content = String::from("{ pkgs, ... }:\n[\n");

    for (name, description) in packages {
        let is_complex_expr = name.starts_with('(')
            || name.starts_with('{')
            || name.starts_with('[')
            || name.starts_with('"')
            || name.starts_with("let ")
            || name.starts_with("with ");

        let formatted_name = if is_complex_expr {
            name.to_string()
        } else {
            if let Some(ref wrapper) = nixgl_wrapper {
                format!(
                    "  (pkgs.writeShellScriptBin \"{}\" ''exec ${{pkgs.nixgl.{}}}/bin/{} ${{pkgs.{}}}/bin/{}'')",
                    name, wrapper, wrapper, name, name
                )
            } else {
                format!("  pkgs.{}", name)
            }
        };

        if description.is_empty() {
            content.push_str(&format!("  {}\n", formatted_name.trim_start()));
        } else {
            content.push_str(&format!(
                "  {} # {}\n",
                formatted_name.trim_start(),
                description
            ));
        }
    }

    content.push_str("]\n");
    write_content_to_file(file_path, &content)
}

/// Returns the master entry point Flake file layout, designed for multi-architecture support
pub fn get_bootstrap_flake_template() -> String {
    let user = env::var("USER").unwrap_or_else(|_| "default".to_string());
    let home = env::var("HOME").unwrap_or_else(|_| format!("/home/{}", user));

    // We keep the dynamic arch to set the primary default target,
    // but the pure Nix layout allows it to be easily extended later.
    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64-linux",
        "aarch64" => "aarch64-linux",
        _ => "x86_64-linux",
    };

    format!(
        r#"{{
  description = "Rix automated system layout profile configuration";

  inputs = {{
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    home-manager = {{
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
    nixgl.url = "github:nix-community/nixGL";
  }};

  outputs = {{ self, nixpkgs, home-manager, nixgl, ... }}:
    let
      system = "{}";
      pkgs = import nixpkgs {{
        inherit system;
        overlays = [ nixgl.overlay ];
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
}}"#,
        arch, user, user, home
    )
}

/// Returns a clean group template file layout
pub fn get_empty_group_template() -> &'static str {
    r#"{ pkgs, ... }:
[
  # Packages managed by Rix
]"#
}
