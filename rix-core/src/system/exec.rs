use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use crate::errors::RixError;
use super::platform::{detect_target_platform, TargetPlatform};

/// Intercepts command output streams and filters out noisy compiler/build logs
fn run_quiet_command(mut cmd: Command, error_msg: &str) -> Result<(), RixError> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|_| RixError::ParseError(error_msg.to_string()))?;
    
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            // Filter out common Nix/CMake build noise and redundant fetch logs from the terminal
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
                continue; // Skip these lines silently
            }
            // Print everything else (warnings, errors, Home Manager activation)
            println!("{}", line);
        }
    }

    let status = child.wait().map_err(|_| RixError::ParseError(error_msg.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(RixError::ParseError(error_msg.to_string()))
    }
}

pub fn update_indexes() -> Result<(), RixError> {
    // Globally inject Nix config via environment variables to cover child processes
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
        
        // NixOS uses action verbs for dry-runs rather than flags
        let action = if dry_run { "dry-build" } else { "switch" };
        
        c.args([
            "nixos-rebuild", action,   
            "--flake", &format!("{}#system", config_str)
        ]);
        c
    } else {
        let mut c = Command::new("nix");
        // This guarantees home-manager inherits the experimental features for its internal Nix calls
        c.env("NIX_CONFIG", "experimental-features = nix-command flakes");
        c.args([
            "run", "nixpkgs#home-manager", "--",   
            "switch", "--flake", &config_str
        ]);
        
        // Home Manager's switch command accepts -n for dry runs
        if dry_run {
            c.arg("-n");
        }
        
        c
    };

    run_quiet_command(cmd, "Failed to materialize declarative generation updates")
}
