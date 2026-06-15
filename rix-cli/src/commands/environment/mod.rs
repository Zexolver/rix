pub mod init;
pub mod sync;
pub mod query;
pub mod maintenance;

pub use init::handle_init;
pub use sync::{handle_update, handle_refresh, handle_upgrade};
pub use query::{handle_list, handle_history};
pub use maintenance::{handle_clean, handle_rollback};

use std::process::Command;
use std::os::unix::process::CommandExt;

/// Intercepts the current running process and morphs it into a `sudo` process image,
/// preserving exit states, signals, and active environmental pointers.
pub(crate) fn elevate_privileges() {
    let args: Vec<String> = std::env::args().collect();
    let err = Command::new("sudo")
        .arg("-E")
        .args(&args)
        .exec();
    eprintln!("❌ Failed to automatically elevate privileges via sudo: {:?}", err);
    std::process::exit(1);
}
