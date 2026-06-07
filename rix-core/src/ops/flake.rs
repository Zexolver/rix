use std::fs;
use std::path::Path;
use crate::errors::RixError;
use crate::writer;

pub fn link_group_to_flake(config_dir: &Path, group: &str) -> Result<(), RixError> {
    let flake_path = config_dir.join("flake.nix");
    if !flake_path.exists() {
        return Ok(());  
    }

    let content = fs::read_to_string(&flake_path)?;
    let import_statement = format!("import ./groups/upstream/{}.nix", group);

    if content.contains(&import_statement) {
        return Ok(());
    }

    let module_inject = format!("        {{ home.packages = {} {{ inherit pkgs; }}; }}", import_statement);

    let mut new_content = String::new();
    let mut injected = false;

    for line in content.lines() {
        new_content.push_str(line);
        new_content.push('\n');

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

    writer::write_content_to_file(&flake_path, &new_content)
}

pub fn add_external_input(config_dir: &Path, alias: &str, uri: &str, group: &str) -> Result<(), RixError> {
    let flake_path = config_dir.join("flake.nix");
    if !flake_path.exists() {
        return Err(RixError::IOError(std::io::Error::new(
            std::io::ErrorKind::NotFound, 
            "flake.nix not found in config directory"
        )));
    }

    let mut content = fs::read_to_string(&flake_path)?;

    // 1. Inject the Input Attribute
    let input_str = format!("    {}.url = \"{}\";\n  ", alias, uri);
    if !content.contains(&format!("{}.url", alias)) {
        if let Some(inputs_end_idx) = content.find("  };\n\n  outputs = {") {
            content.insert_str(inputs_end_idx, &input_str);
        } else {
            return Err(RixError::ParseError("Could not locate inputs block closure in flake.nix".into()));
        }
    }

    // 2. Inject the Output Parameter
    let output_param = format!("{}, ", alias);
    if !content.contains(&output_param) {
        if let Some(outputs_idx) = content.find("... }:") {
            content.insert_str(outputs_idx, &output_param);
        } else {
            return Err(RixError::ParseError("Could not locate outputs parameter list in flake.nix".into()));
        }
    }

    // 3. Pass the new alias down to the group imports in flake.nix
    // e.g., changes { inherit pkgs; } into { inherit pkgs neovim; }
    let target_inherit = "{ inherit pkgs";
    if !content.contains(&format!("{} {}", target_inherit, alias)) {
        content = content.replace(target_inherit, &format!("{} {}", target_inherit, alias));
    }

    writer::write_content_to_file(&flake_path, &content)?;

    // 4. Update the group's .nix file header to accept the new variable
    // e.g., changes { pkgs, ... }: into { pkgs, neovim, ... }:
    let group_path = config_dir.join(format!("groups/upstream/{}.nix", group));
    if group_path.exists() {
        let mut group_content = fs::read_to_string(&group_path)?;
        let target_header = "{ pkgs";
        // Prevent duplicate injections
        if group_content.contains(target_header) && !group_content.contains(&format!("{},", alias)) && !group_content.contains(&format!("{} ", alias)) {
            group_content = group_content.replace(target_header, &format!("{{ pkgs, {}", alias));
            writer::write_content_to_file(&group_path, &group_content)?;
        }
    }

    Ok(())
}
