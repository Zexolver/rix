pub mod exec;
pub mod platform;
pub mod sync; // <-- Add this line

// Re-export types and functions seamlessly for external usage
pub use exec::{apply_upgrade, bridge_system_binaries, update_indexes};
pub use platform::{
    NixInstallType, TargetPlatform, detect_nix_installation, detect_target_platform,
};
pub use sync::{auto_commit, init_local_repo}; // <-- Add this line
