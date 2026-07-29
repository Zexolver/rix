mod args;
mod commands;
mod config;
mod handlers;
mod ui;

use clap::Parser;
use rix_core::RixContext;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = args::Cli::parse();
    let config_dir = config::get_config_dir();
    let ctx = RixContext::new(config_dir);
    commands::handle(cli, ctx);
    Ok(())
}
