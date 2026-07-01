use crate::errors::RixError;
use std::fs;
use std::path::PathBuf;

/// Scans the system for PCI GPU vendor IDs and writes the correct NixGL wrapper to an untracked local file.
pub fn detect_and_lock_hardware(config_dir: &PathBuf) -> Result<(), RixError> {
    println!("Detecting system GPU hardware...");

    // Default to the open-source Mesa wrapper if we can't find anything
    let mut wrapper = "nixGLIntel";

    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            // Only look at primary card directories
            if file_name.starts_with("card") && !file_name.contains('-') {
                let vendor_path = path.join("device/vendor");
                if let Ok(vendor_id) = fs::read_to_string(vendor_path) {
                    let vid = vendor_id.trim();
                    if vid == "0x10de" {
                        println!("Found Nvidia GPU (Vendor: {})", vid);
                        wrapper = "nixGLNvidia";
                        break; // Nvidia takes priority on hybrid laptops
                    } else if vid == "0x1002" {
                        println!("Found AMD GPU (Vendor: {})", vid);
                        wrapper = "nixGLIntel"; // ✅ Fixed: Maps AMD to Intel/Mesa wrapper
                    } else if vid == "0x8086" {
                        println!("Found Intel GPU (Vendor: {})", vid);
                        wrapper = "nixGLIntel";
                    }
                }
            }
        }
    }

    // Write the detected wrapper string as a valid Nix string expression
    let state_file_path = config_dir.join("hardware-state.nix");
    fs::write(&state_file_path, format!("\"{}\"\n", wrapper)).map_err(RixError::IOError)?;

    // Enforce Git isolation: Ensure hardware-state.nix is ignored
    let gitignore_path = config_dir.join(".gitignore");
    let gitignore_entry = "hardware-state.nix\n";
    if !gitignore_path.exists() {
        let _ = fs::write(&gitignore_path, gitignore_entry);
    } else if let Ok(content) = fs::read_to_string(&gitignore_path) {
        if !content.contains("hardware-state.nix") {
            let _ = fs::write(&gitignore_path, format!("{}{}", content, gitignore_entry));
        }
    }

    println!(
        "Successfully locked hardware state to local machine: {}",
        wrapper
    );
    Ok(())
}
