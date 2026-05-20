use std::process::Command;
use std::path::Path;
use crate::errors::RixError;

pub fn check_system_sanity() -> Result<(), RixError> {
    let binaries = ["nix-env", "nix-instantiate"];
    for bin in &binaries {
        let status = Command::new("which")
            .arg(bin)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        
        if !matches!(status, Ok(s) if s.success()) {
            return Err(RixError::MissingSystemDependency(format!(
                "Critical dependency '{}' not found on system PATH.", bin
            )));
        }
    }
    Ok(())
}

pub fn verify_nix_syntax(file_path: &Path) -> Result<(), RixError> {
    if !file_path.exists() {
        return Ok(());
    }

    let output = Command::new("nix-instantiate")
        .arg("--parse")
        .arg(file_path)
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            Err(RixError::InvalidNixSyntax(format!(
                "Generated file failed syntax validation: {}", stderr.trim()
            )))
        }
        Err(_) => Err(RixError::ParseError("Could not execute syntax validator".to_string())),
    }
}

pub fn verify_online_package_architecture(package_name: &str) -> Result<String, RixError> {
    let client = reqwest::blocking::Client::new();
    let url = "https://search.nixos.org/backend/nixpkgs/_search";

    let query = serde_json::json!({
        "query": {
            "bool": {
                "must": [{ "match": { "package_attr_name": package_name } }],
                "filter": [{ "term": { "package_systems": "aarch64-linux" } }]
            }
        },
        "size": 1
    });

    let res = client.post(url).json(&query).send()
        .map_err(|e| RixError::ParseError(format!("Network lookup error: {}", e)))?;

    let body: serde_json::Value = res.json()
        .map_err(|e| RixError::ParseError(format!("Malformed API tracking packet: {}", e)))?;

    if let Some(hit) = body["hits"]["hits"].get(0) {
        let attr_name = hit["_source"]["package_attr_name"].as_str()
            .ok_or_else(|| RixError::ParseError("Missing package attribute identification tracking".to_string()))?;
        Ok(attr_name.to_string())
    } else {
        Err(RixError::ParseError(format!(
            "Package '{}' was not found or is incompatible with your CPU architecture (aarch64-linux).", 
            package_name
        )))
    }
}
