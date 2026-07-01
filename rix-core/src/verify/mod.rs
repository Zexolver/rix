mod registry;
mod sanity;
mod syntax;

pub use registry::{run_nix_search, search_local_db, verify_online_package_architecture};
pub use sanity::{check_system_sanity, verify_flake_resolves};
pub use syntax::verify_nix_syntax;
