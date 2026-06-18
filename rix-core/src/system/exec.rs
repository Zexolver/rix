use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use crate::errors::RixError;
use super::platform::{detect_target_platform, TargetPlatform};

/// Intercepts command output streams, parses progress, and filters out noise
fn run_quiet_command(mut cmd: Command, error_msg: &str) -> Result<(), RixError> {
    let start_time = Instant::now();

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
            // Filter out common Nix/CMake build noise
            if line.contains("%]")   
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
            {
                continue; 
            }

            // Hide the raw derivation path spam since the progress bar covers it
            if line.starts_with("  /nix/store/") && line.ends_with(".drv") {
                continue;
            }

            // 2. DYNAMIC PROGRESS BAR: Catch "these X derivations will be built:"
            if line.contains("these ") && line.contains(" derivations will be built:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(total) = parts[1].parse::<u64>() {
                        pb.set_length(total);
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                                .unwrap()
                                .progress_chars("=> ")
                        );
                        pb.set_message("Building...");
                    }
                }
                continue; // Skip printing this literal line
            }

            // 3. Increment the bar for every package being built
            if line.starts_with("building '/nix/store/") {
                pb.inc(1);
                
                // Extract a cleaner package name for the UI string
                if let Some(drv) = line.split('-').nth(1) {
                    let name = drv.trim_end_matches(".drv'...").trim_end_matches(".drv'");
                    pb.set_message(format!("Building {}...", name));
                }
                continue; // Skip printing the raw build log
            }

            // Print everything else cleanly ABOVE the progress bar so it doesn't break the UI
            pb.println(&line);
        }
    }

    let status = child.wait().map_err(|_| RixError::ParseError(error_msg.to_string()))?;
    
    // Erase the progress bar from the terminal once finished
    pb.finish_and_clear();

    if status.success() {
        // Cargo-style completion message
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
// Add this to the bottom of rix-core/src/system/exec.rs

use std::fs;
use std::os::unix::fs::symlink;

/// Bridges Nix binaries into standard system paths so `sudo` can find them
pub fn bridge_system_binaries() -> Result<(), RixError> {
    let source_bin_dir = Path::new("/nix/var/nix/profiles/default/bin");
    let target_bin_dir = Path::new("/usr/local/bin");

    if !source_bin_dir.exists() {
        return Ok(()); // Nothing to bridge
    }

    for entry in fs::read_dir(source_bin_dir).map_err(|e| RixError::ParseError(e.to_string()))? {
        let entry = entry.map_err(|e| RixError::ParseError(e.to_string()))?;
        let source_path = entry.path();
        
        if let Some(file_name) = source_path.file_name() {
            let target_path = target_bin_dir.join(file_name);

            // Clean up old symlinks if they exist
            if target_path.exists() || target_path.is_symlink() {
                let _ = fs::remove_file(&target_path); 
            }

            // Create the new symlink
            symlink(&source_path, &target_path).map_err(|e| {
                RixError::ParseError(format!("Failed to bridge binary {:?}: {}", file_name, e))
            })?;
        }
    }
    
    Ok(())
}
