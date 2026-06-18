pub mod platform;
pub mod exec;
pub mod sync; // <-- Add this line

// Re-export types and functions seamlessly for external usage
pub use platform::{NixInstallType, TargetPlatform, detect_nix_installation, detect_target_platform};
pub use exec::{update_indexes, apply_upgrade, bridge_system_binaries};
pub use sync::{init_local_repo, auto_commit}; // <-- Add this line
