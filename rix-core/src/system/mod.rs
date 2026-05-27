pub mod platform;
pub mod exec;

// Re-export types and functions seamlessly for external usage
pub use platform::{NixInstallType, TargetPlatform, detect_nix_installation, detect_target_platform};
pub use exec::{update_indexes, apply_upgrade};
