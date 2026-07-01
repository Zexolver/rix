use std::fs;
use std::path::PathBuf;

/// Reads the dynamically generated hardware state file and returns the NixGL wrapper string.
pub fn get_nixgl_wrapper(config_dir: &PathBuf) -> Option<String> {
    let state_file_path = config_dir.join("hardware-state.nix");

    if let Ok(content) = fs::read_to_string(&state_file_path) {
        // The file contains a Nix string like `"nixGLDefault"\n`
        // We trim the newline and strip the quotes to get the raw wrapper name
        let wrapper = content.trim().trim_matches('"');
        if !wrapper.is_empty() {
            return Some(wrapper.to_string());
        }
    }

    // Return None if the file doesn't exist or is empty
    None
}
