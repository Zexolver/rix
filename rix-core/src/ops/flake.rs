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

    let module_inject = format!("         {{ home.packages = {} {{ inherit pkgs; }}; }}", import_statement);

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
