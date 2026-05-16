mod args;

use std::env;
use std::path::PathBuf;
use clap::Parser;
use rix_core::{Package, RixContext};

fn main() {
    let cli = args::Cli::parse();

    // Dynamically find user's home configuration directory (fallback to ~/.config)
    let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"));
    let home_manager_dir = home_dir.join(".config/home-manager");

    // Initialize the core library engine context
    let ctx = RixContext::new(home_manager_dir);

    match cli.command {
        args::Commands::Install { name, group, description } => {
            let package = Package {
                name: name.clone(),
                description,
                group: group.clone(),
                is_local_recipe: false,
            };

            println!("Installing '{}' into group '{}'...", name, group);

            if let Err(e) = ctx.add_package(package) {
                eprintln!("Error executing install: {:?}", e);
                std::process::exit(1);
            }

            println!("Successfully added '{}' and optimized profile structure!", name);
        }
    }
}
