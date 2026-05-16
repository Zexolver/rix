mod args;

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
        // Path A: Beginner 'install' syntax
        Commands::Install { name, group, description } => {
            let package = Package {
                name,
                description,
                group,
                is_local_recipe: false,
            };
            execute_add(&ctx, package);
        }

        // Path B: Intermediate 'profile add' syntax
        Commands::Profile(ProfileCommands::Add { installable, description }) => {
            let (target, group) = installable.split_once('@').unwrap_or((&installable, "default"));
            let package_name = target.split_once('#').map(|(_, pkg)| pkg).unwrap_or(target);

            let package = Package {
                name: package_name.to_string(),
                description,
                group: group.to_string(),
                is_local_recipe: target.contains(':'),
            };
            execute_add(&ctx, package);
        }

        // Placeholders for removal commands
        Commands::Remove { name } => {
            println!("Beginner removal sequence for '{}' coming up!", name);
        }
        Commands::Profile(ProfileCommands::Remove { installable }) => {
            println!("Intermediate removal sequence for '{}' coming up!", installable);
        }
    }
}

fn execute_add(ctx: &RixContext, package: Package) {
    println!("Syncing '{}' into target profile group '{}'...", package.name, package.group);
    if let Err(e) = ctx.add_package(package) {
        eprintln!("Operation failed: {:?}", e);
        std::process::exit(1);
    }
    println!("Successfully optimized environment config changes!");
}
