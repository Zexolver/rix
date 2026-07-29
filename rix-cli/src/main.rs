mod args;
mod commands;
mod config;
mod handlers;
mod ui;
mod tui;

use clap::Parser;
use rix_core::RixContext;
use std::env;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let config_dir = config::get_config_dir();
    let ctx = RixContext::new(config_dir);

    if args.len() == 1 || (args.len() == 2 && args[1] == "tui") {
        tui::run_tui(&ctx)?;
    } else {
        let cli = args::Cli::parse();
        commands::handle(cli, ctx);
    }

    Ok(())
}
