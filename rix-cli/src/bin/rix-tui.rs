use rix_cli::{config, tui};
use rix_core::RixContext;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = config::get_config_dir();
    let ctx = RixContext::new(config_dir);

    tui::run_tui(&ctx)?;

    Ok(())
}
