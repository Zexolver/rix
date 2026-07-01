pub mod init;
pub mod maintenance;
pub mod query;
pub mod sync;

pub use init::handle_init;
pub use maintenance::{handle_clean, handle_rollback};
pub use query::{handle_history, handle_list};
pub use sync::{handle_refresh, handle_update, handle_upgrade};

use std::os::unix::process::CommandExt;
use std::process::Command;

/// Intercepts the current running process and morphs it into a `sudo` process image,
/// preserving exit states, signals, and active environmental pointers.
pub(crate) fn elevate_privileges() {
    let args: Vec<String> = std::env::args().collect();
    let err = Command::new("sudo").arg("-H").args(&args).exec();
    eprintln!(
        "❌ Failed to automatically elevate privileges via sudo: {:?}",
        err
    );
    std::process::exit(1);
}
