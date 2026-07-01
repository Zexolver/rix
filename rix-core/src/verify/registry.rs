use crate::errors::RixError;
use std::path::Path;
use std::process::Command;

pub fn verify_online_package_architecture(package_name: &str) -> Result<String, RixError> {
    let matches = run_nix_search(package_name)?;

    if matches.is_empty() {
        return Err(RixError::ParseError(format!(
            "Package '{}' is not available in the nixpkgs flake for your architecture.",
            package_name
        )));
    }

    // Since `run_nix_search` now sorts the best match to index 0, we can confidently grab it!
    for (attr_path, _) in &matches {
        let parts: Vec<&str> = attr_path.splitn(3, '.').collect();
        if parts.len() == 3 {
            let extracted_name = parts[2];
            let final_segment = extracted_name.split('.').last().unwrap_or(extracted_name);

            if final_segment == package_name
                || extracted_name.ends_with(&format!(".{}", package_name))
            {
                return Ok(extracted_name.to_string());
            }
        }
    }

    // Fallback to field .0 of the first (highest-ranked) tuple match
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
            "--extra-experimental-features",
            "nix-command flakes",
            "search",
            "nixpkgs",
            query,
            "--json",
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
            let description = val["description"]
                .as_str()
                .unwrap_or("No description available.")
                .to_string();
            results.push((key.clone(), description));
        }
    }

    // THE FIX: Sort results to prioritize exact matches and shorter attribute paths
    let query_lower = query.to_lowercase();
    results.sort_by(|a, b| {
        let a_short = a.0.split('.').last().unwrap_or(&a.0).to_lowercase();
        let b_short = b.0.split('.').last().unwrap_or(&b.0).to_lowercase();

        let a_exact = a_short == query_lower;
        let b_exact = b_short == query_lower;

        if a_exact && !b_exact {
            std::cmp::Ordering::Less
        } else if b_exact && !a_exact {
            std::cmp::Ordering::Greater
        } else {
            // If both are exact or neither are exact, the shorter path wins (banishing deep nesting)
            a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0))
        }
    });

    Ok(results)
}

pub fn search_local_db(
    _config_dir: &Path,
    query: &str,
) -> Result<Vec<(String, String, String)>, RixError> {
    // XDG Standard: Point directly to the system cache directory
    let db_path = Path::new("/var/cache/rix/pkgs.db");

    let conn =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| {
                RixError::ParseError(format!(
                    "Failed to open local database at {:?}. Did you run 'rix update'?: {}",
                    db_path, e
                ))
            })?;

    // Try to pull description if it exists; SQLite coalesces missing fields gracefully
    let sql =
        "SELECT name, version, COALESCE(description, '') FROM packages WHERE name LIKE ? LIMIT 50";
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| RixError::ParseError(format!("Failed to prepare query: {}", e)))?;

    let query_param = format!("%{}%", query);
    let pkg_iter = stmt
        .query_map([&query_param], |row| {
            let name: String = row.get(0)?;
            let version: String = row.get(1).unwrap_or_else(|_| "unknown".to_string());
            let desc: String = row.get(2).unwrap_or_else(|_| "".to_string());
            Ok((name, version, desc))
        })
        .map_err(|e| RixError::ParseError(format!("Query failed: {}", e)))?;

    let mut results = Vec::new();
    for pkg in pkg_iter {
        if let Ok(p) = pkg {
            results.push(p);
        }
    }

    // Apply the exact same smart sorting logic to your SQLite results
    let query_lower = query.to_lowercase();
    results.sort_by(|a, b| {
        let a_exact = a.0.to_lowercase() == query_lower;
        let b_exact = b.0.to_lowercase() == query_lower;

        if a_exact && !b_exact {
            std::cmp::Ordering::Less
        } else if b_exact && !a_exact {
            std::cmp::Ordering::Greater
        } else {
            a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0))
        }
    });

    Ok(results)
}
