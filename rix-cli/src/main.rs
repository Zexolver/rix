mod args;
mod ui;
mod handlers;

use std::env;
use std::path::PathBuf;
use clap::Parser;
use rix_core::{Package, RixContext};
use args::{Commands, ProfileCommands};

fn main() {
    let cli = args::Cli::parse();
    let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"));
    let ctx = RixContext::new(home_dir.join(".config/home-manager"));

    match cli.command {
        Commands::Install { name, group, description } => {
            handlers::execute_add(&ctx, Package { name, description, group, is_local_recipe: false });
        }
        Commands::Profile(ProfileCommands::Add { installable, description }) => {
            let (target, group) = installable.split_once('@').unwrap_or((&installable, "default"));
            let package_name = target.split_once('#').map(|(_, pkg)| pkg).unwrap_or(target);
            handlers::execute_add(&ctx, Package { name: package_name.to_string(), description, group: group.to_string(), is_local_recipe: target.contains(':') });
        }
        Commands::Remove { name } => {
            handlers::handle_interactive_removal(&ctx, &name);
        }
        Commands::Profile(ProfileCommands::Remove { installable }) => {
            let (target, _) = installable.split_once('@').unwrap_or((&installable, "default"));
            let package_name = target.split_once('#').map(|(_, pkg)| pkg).unwrap_or(target);
            handlers::handle_interactive_removal(&ctx, package_name);
        }
        Commands::Purge { group } => {
            println!("Purging profile group configuration layout '{}'...", group);
            if let Err(e) = ctx.purge_group_profile(&group) {
                eprintln!("Purge sequence failed: {:?}", e);
                std::process::exit(1);
            }
            println!("Successfully purged profile configuration!");
        }
        Commands::Update => {
            println!("Syncing package index state references from upstream repositories...");
            if let Err(e) = ctx.update_indexes() {
                eprintln!("Update sequence failed: {:?}", e);
                std::process::exit(1);
            }
            println!("Upstream indexes updated successfully!");
        }
        Commands::Upgrade => {
            println!("Applying generational upgrade across declarative sets...");
            if let Err(e) = ctx.apply_upgrade() {
                eprintln!("Upgrade realization failed: {:?}", e);
                std::process::exit(1);
            }
            println!("System configuration environment generation fully built!");
        }
        Commands::List => {
            match ctx.list_all_packages() {
                Ok(packages) => ui::print_package_table(packages),
                Err(e) => { eprintln!("Failed to retrieve packages: {:?}", e); std::process::exit(1); }
            }
        }
    }
}
