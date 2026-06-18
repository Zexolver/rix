mod sanity;
mod syntax;
mod registry;

pub use sanity::{check_system_sanity, verify_flake_resolves};
pub use syntax::verify_nix_syntax;
pub use registry::{verify_online_package_architecture, run_nix_search};
