use std::process::Command;
use std::path::Path;
use std::fs;
use crate::errors::RixError;

pub enum TargetPlatform {
    NixOS,
    LinuxX86_64,
    LinuxARM64,
    MacOS,
}

/// Dynamic live probe to identify the operating system environment and architecture profile
pub fn detect_target_platform() -> TargetPlatform {
    if cfg!(target_os = "macos") {
        return TargetPlatform::MacOS;
    }

    if Path::new("/etc/NIXOS").exists() || Path::new("/run/current-system").exists() {
        return TargetPlatform::NixOS;
    }

    if cfg!(target_arch = "aarch64") {
        TargetPlatform::LinuxARM64
    } else {
        TargetPlatform::LinuxX86_64
    }
}

/// Identifies the live hardware graphics driver stack on non-NixOS Linux machines
pub fn detect_live_gpu_driver() -> String {
    if Path::new("/proc/driver/nvidia/version").exists() {
        return "nixGLNvidia".to_string();
    }
    
    // Check up to three distinct card entries to accommodate composite SoC nodes (like Chromebooks)
    for card_idx in 0..3 {
        let symlink_path = format!("/sys/class/drm/card{}/device/driver", card_idx);
        if let Ok(target) = fs::read_link(symlink_path) {
            let driver_name = target.file_name().unwrap_or_default().to_string_lossy();
            if driver_name == "panfrost" || driver_name == "lima" {
                return "nixGLMesa".to_string();
            }
        }
    }

    if Path::new("/sys/class/drm/renderD128").exists() && cfg!(target_arch = "aarch64") {
        return "nixGLMesa".to_string();
    }

    "nixGLDefault".to_string()
}

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
