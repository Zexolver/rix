use clap::Parser;
use rix_cli::{args, commands, config};
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

    let cli = args::Cli::parse();
    commands::handle(cli, ctx);

    Ok(())
}
