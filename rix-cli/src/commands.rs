use rix_core::{Package, RixContext};
use crate::args::{Cli, Commands, ProfileCommands};
use crate::handlers;
use crate::ui;

pub fn handle(cli: Cli, ctx: RixContext) {
    match cli.command {
        Commands::Install { name, group, description } => {
            println!("Searching online package indices for '{}'...", name);
            match rix_core::verify::verify_online_package_architecture(&name) {
                Ok(verified_name) => {
                    handlers::execute_add(&ctx, Package { 
                        name: verified_name, 
                        description, 
                        group, 
                        is_local_recipe: false 
                    });
                                        
                    if let Err(e) = ctx.apply_upgrade() {
                        eprintln!("Failed to apply target updates to environment: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("{:?}", e); 
                    std::process::exit(1);
                }
            }
        }
        Commands::Search { query } => {
            // Initialize the spinner loader right here
            let spinner = ui::create_spinner("Querying modern Flake registry...");
            
            match rix_core::verify::run_nix_search(&query) {
                Ok(results) => {
                    // Wipe the spinner cleanly off the terminal before layout printing
                    spinner.finish_and_clear();

                    if results.is_empty() {
                        println!("No packages matched your query.");
                    } else {
                        println!("\n{:<40} {}", "PACKAGE ATTRIBUTE PATH", "DESCRIPTION");
                        println!("{}", "-".repeat(80));
                        for (path, desc) in results {
                            // Safely isolate package sub-attribute keys (e.g., legacyPackages.aarch64-linux.fastfetch -> fastfetch)
                            let short_path = path.splitn(3, '.').nth(2).unwrap_or(&path);
                            println!("{:<40} {}", short_path, desc);
                        }
                        println!();
                    }
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    eprintln!("Search sequence broken: {:?}", e);
                    std::process::exit(1);
                }
            }
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
