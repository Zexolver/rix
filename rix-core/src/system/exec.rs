use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};

use crate::errors::RixError;
use super::platform::{detect_target_platform, TargetPlatform};

/// Intercepts command output streams, parses progress, and filters out noise
fn run_quiet_command(mut cmd: Command, error_msg: &str) -> Result<(), RixError> {
    let start_time = Instant::now();
    let mut total_fetches = 0;
    let mut current_fetches = 0;

    // 1. Start with a spinner during the evaluation phase
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
    );
    pb.set_message("Evaluating configuration...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|_| RixError::ParseError(error_msg.to_string()))?;
    
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let trimmed = line.trim();

            // 2a. Intercept lock file updates
            if line.contains("warning: updating lock file") {
                pb.set_message("Updating flake.lock inputs...");
                continue;
            }

            // Filter out lockfile changelog lines and common Nix noise
            if trimmed.starts_with('•') 
                || trimmed.starts_with("'github:") 
                || trimmed.starts_with("follows ")
                || trimmed.starts_with("'git+file:")
                || line.contains("%]")   
                || line.contains("Built target")    
                || line.contains("Install the project...")
                || line.contains("-- Install configuration:")
                || line.contains("separating debug info")
                || line.contains("shrinking RPATHs")
                || line.contains("stripping (with command")
                || line.contains("making symlink relative")
                || line.contains("checking for references")
                || line.contains("gzipping man pages")
                || line.contains("fetching path input")
                || line.contains("fetching github input")
                || line.contains("fetching git input")
                || line.contains("warning: Git tree")
                || line.contains("warning: 'system' has been renamed")
                || line.contains("warning: using or as an identifier is deprecated")
                || line.contains("warning: Ignoring setting")
                || trimmed.starts_with("at /nix/store/")
            {
                continue;  
            }

            // Filter out Nix's multi-line code snippets
            if trimmed.contains('|') && (trimmed.starts_with(|c: char| c.is_ascii_digit()) || trimmed.starts_with('|')) {
                continue;
            }

            // Hide the raw derivation path spam
            if line.starts_with("  /nix/store/") && line.ends_with(".drv") {
                continue;
            }

            // 2b. DYNAMIC PROGRESS BAR FOR FETCHES: Intercept path download lists
            if line.contains("paths will be fetched") {
                if let Some(count_str) = line.split_whitespace().find(|s| s.parse::<u64>().is_ok()) {
                    if let Ok(total) = count_str.parse::<u64>() {
                        total_fetches = total;
                        current_fetches = 0;
                        pb.set_length(total);
                        pb.set_position(0);
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.magenta/blue}] {pos}/{len} ({eta}) {msg}")
                                .unwrap()
                                .progress_chars("=> ")
                        );
                        pb.set_message("Fetching system paths...");
                    }
                }
                continue;
            }

            // 2c. Intercept active copying/downloading from caches
            if line.starts_with("copying path '/nix/store/") {
                current_fetches += 1;
                if total_fetches > 0 {
                    pb.set_position(current_fetches);
                }
                if let Some(path_part) = line.strip_prefix("copying path '/nix/store/") {
                    if let Some(end_idx) = path_part.find('\'') {
                        let full_name = &path_part[..end_idx];
                        let clean_name = if full_name.len() > 33 && full_name.as_bytes()[32] == b'-' {
                            &full_name[33..]
                        } else {
                            full_name
                        };
                        pb.set_message(format!("Downloading {}...", clean_name));
                    }
                }
                continue;
            }

            // Compress individual raw fetch paths into single-line status updates
            if trimmed.starts_with("/nix/store/") && !trimmed.ends_with(".drv") {
                current_fetches += 1;
                if total_fetches > 0 {
                    pb.set_position(current_fetches);
                }
                if let Some(name_part) = trimmed.strip_prefix("/nix/store/") {
                    let clean_name = if name_part.len() > 33 && name_part.as_bytes()[32] == b'-' {
                        &name_part[33..]
                    } else {
                        name_part
                    };
                    pb.set_message(format!("Fetching {}...", clean_name));
                }
                continue;
            }

            // 3a. DYNAMIC PROGRESS BAR FOR BUILDS
            if line.contains("these ") && line.contains(" derivations will be built:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(total) = parts[1].parse::<u64>() {
                        pb.set_length(total);
                        pb.set_position(0);
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                                .unwrap()
                                .progress_chars("=> ")
                        );
                        pb.set_message("Building dependencies...");
                    }
                }
                continue;
            }

            // 3b. Increment the bar for every package being built
            if line.starts_with("building '/nix/store/") {
                pb.inc(1);
                
                if let Some(drv) = line.split('-').nth(1) {
                    let name = drv.trim_end_matches(".drv'...").trim_end_matches(".drv'");
                    pb.set_message(format!("Building {}...", name));
                }
                continue;
            }

            // 4. SQUASH BUILD PHASES: Intercept builder-specific steps
            if let Some(idx) = trimmed.find('>') {
                let prefix = &trimmed[..idx];
                if !prefix.is_empty() && prefix.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    let log_msg = trimmed[idx + 1..].trim();
                    if !log_msg.is_empty() {
                        pb.set_message(format!("{}: {}", prefix, log_msg));
                    }
                    continue; 
                }
            }

            if trimmed.is_empty() {
                continue;
            }

            pb.println(&line);
        }
    }

    let status = child.wait().map_err(|_| RixError::ParseError(error_msg.to_string()))?;
    
    pb.finish_and_clear();

    if status.success() {
        println!("    \x1b[1;32mFinished\x1b[0m environment generation in {:.2}s", start_time.elapsed().as_secs_f64());
        Ok(())
    } else {
        Err(RixError::ParseError(error_msg.to_string()))
    }
}

pub fn update_indexes() -> Result<(), RixError> {
    let mut cmd = Command::new("nix");
    cmd.env("NIX_CONFIG", "experimental-features = nix-command flakes")
       .args(["flake", "update"]);
        
    run_quiet_command(cmd, "Failed to update Flake lock references")
}

pub fn apply_upgrade(config_path: &Path, is_system: bool, dry_run: bool) -> Result<(), RixError> {
    let platform = detect_target_platform();
    let config_str = config_path.to_string_lossy().to_string();

    let cmd = if is_system && platform == TargetPlatform::NixOS {
        let mut c = Command::new("sudo");
        c.env("NIX_CONFIG", "experimental-features = nix-command flakes");
        let action = if dry_run { "dry-build" } else { "switch" };
        c.args([
            "nixos-rebuild", action,    
            "--flake", &format!("{}#system", config_str)
        ]);
        c
    } else {
        let mut c = Command::new("nix");
        c.env("NIX_CONFIG", "experimental-features = nix-command flakes");
        c.args([
            "run", "nixpkgs#home-manager", "--",    
            "switch", "--flake", &config_str
        ]);
        if dry_run {
            c.arg("-n");
        }
        c
    };

    run_quiet_command(cmd, "Failed to materialize declarative generation updates")
}

pub fn bridge_system_binaries() -> Result<(), RixError> {
    let source_bin_dir = Path::new("/nix/var/nix/profiles/default/bin");
    let target_bin_dir = Path::new("/usr/local/bin");

    if !source_bin_dir.exists() {
        return Ok(()); 
    }

    for entry in fs::read_dir(source_bin_dir).map_err(|e| RixError::ParseError(e.to_string()))? {
        let entry = entry.map_err(|e| RixError::ParseError(e.to_string()))?;
        let source_path = entry.path();
        
        if let Some(file_name) = source_path.file_name() {
            let target_path = target_bin_dir.join(file_name);

            if target_path.exists() || target_path.is_symlink() {
                let _ = fs::remove_file(&target_path);  
            }

            symlink(&source_path, &target_path).map_err(|e| {
                RixError::ParseError(format!("Failed to bridge binary {:?}: {}", file_name, e))
            })?;
        }
    }
    
    Ok(())
}
