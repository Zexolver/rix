use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum NixInstallType {
    MultiUser,
    SingleUser,
    NotInstalled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetPlatform {
    NixOS,
    MultiUserLinux,
    SingleUserLinux,
    MacOS,
}

pub fn detect_nix_installation() -> NixInstallType {
    // 1. Check for the multi-user daemon socket.
    let daemon_socket = Path::new("/nix/var/nix/daemon-socket/socket");
    if daemon_socket.exists() {
        return NixInstallType::MultiUser;
    }

    // 2. Fallback check: Look at who owns the /nix/store
    if let Ok(metadata) = fs::metadata("/nix/store") {
        if metadata.uid() == 0 {
            // Root owns the store, but daemon socket is missing. Still multi-user.
            return NixInstallType::MultiUser;
        } else {
            // The current user owns the store. Single-user installation.
            return NixInstallType::SingleUser;
        }
    }

    NixInstallType::NotInstalled
}

pub fn detect_target_platform() -> TargetPlatform {
    if cfg!(target_os = "macos") {
        return TargetPlatform::MacOS;
    }

    if Path::new("/etc/NIXOS").exists() || Path::new("/run/current-system").exists() {
        return TargetPlatform::NixOS;
    }

    let install_type = detect_nix_installation();
    if install_type == NixInstallType::MultiUser {
        return TargetPlatform::MultiUserLinux;
    }

    TargetPlatform::SingleUserLinux
}
