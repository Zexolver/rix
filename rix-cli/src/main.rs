mod args;
mod commands;
mod config;
mod handlers;
mod ui;

use clap::Parser; // Essential trait bound inclusion to unlock .parse()
use rix_core::RixContext;

fn main() {
    // 1. Parse standard input CLI argument states
    let cli = args::Cli::parse();

    // 2. Identify the system platform context pathing targets
    let config_dir = config::get_config_dir();
    let ctx = RixContext::new(config_dir);

    // 3. Delegate execution directly to our operational router
    commands::handle(cli, ctx);
}
