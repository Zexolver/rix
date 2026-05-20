use std::process::Command;
use crate::errors::RixError;

pub fn verify_online_package_architecture(package_name: &str) -> Result<String, RixError> {
    let matches = run_nix_search(package_name)?;
    
    if matches.is_empty() {
        return Err(RixError::ParseError(format!(
            "Package '{}' is not available in the nixpkgs flake for your architecture.", 
            package_name
        )));
    }

    // Destructure the tuple to pull only the attribute path string out
    for (attr_path, _) in &matches {
        let parts: Vec<&str> = attr_path.splitn(3, '.').collect();
        if parts.len() == 3 {
            let extracted_name = parts[2];
            if extracted_name == package_name || extracted_name.ends_with(&format!(".{}", package_name)) {
                return Ok(extracted_name.to_string());
            }
        }
    }

    // Access field `.0` of the first tuple match as a fallback
    let parts: Vec<&str> = matches[0].0.splitn(3, '.').collect();
    if parts.len() == 3 {
        Ok(parts[2].to_string())
    } else {
        Ok(package_name.to_string())
    }
}

pub fn run_nix_search(query: &str) -> Result<Vec<(String, String)>, RixError> {
    let output = Command::new("nix")
        .args([
            "--extra-experimental-features", "nix-command flakes",
            "search", 
            "nixpkgs", 
            query,
            "--json"
        ])
        .output()
        .map_err(|e| RixError::ParseError(format!("Failed to invoke modern Nix search: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RixError::ParseError(format!(
            "Nix search evaluation failed. (If this is a cold run, cache initialization is required): {}", 
            stderr.trim()
        )));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let packages: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| RixError::ParseError(format!("Failed parsing Nix search payload: {}", e)))?;

    let mut results = Vec::new();
    if let Some(obj) = packages.as_object() {
        for (key, val) in obj {
            let description = val["description"].as_str().unwrap_or("No description available.").to_string();
            results.push((key.clone(), description));
        }
    }
    Ok(results)
}
