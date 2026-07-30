use rix_core::RixContext;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> std::io::Result<()> {
    let config_dir = rix_cli::config::get_config_dir();
    let ctx = RixContext::new(config_dir);
    rix_cli::tui::run_tui(&ctx)
}
