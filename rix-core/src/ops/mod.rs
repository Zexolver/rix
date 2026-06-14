pub mod list;
pub mod flake;
pub mod refresh;

// Re-export so the rest of the codebase doesn't need to change its imports
pub use list::{add_package, remove_package_from_file};
pub use flake::link_group_to_flake;
pub use refresh::detect_and_lock_hardware;
