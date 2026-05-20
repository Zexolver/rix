use std::env;
use std::path::PathBuf;
use clap::Parser;
use rix_core::{Package, RixContext, system::detect_target_platform, system::TargetPlatform};
use args::{Commands, ProfileCommands};

fn main() {
    let cli = args::Cli::parse();
    
    // Determine the base configuration path dynamically
    let config_dir = match detect_target_platform() {
        TargetPlatform::NixOS | TargetPlatform::MultiUserLinux => {
            PathBuf::from("/etc/rix") // System-wide storage target location
        }
        _ => {
            let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"));
            home_dir.join(".config/rix") // Isolated user-space folder path location
        }
    };

    let ctx = RixContext::new(config_dir);

    match cli.command {
        Commands::Install { name, group, description } => {
            // 1. Run our online search query verification step before updating configurations
            println!("Searching online package indices for '{}'...", name);
            match rix_core::verify::verify_online_package_architecture(&name) {
                Ok(verified_name) => {
                    handlers::execute_add(&ctx, Package { 
                        name: verified_name, 
                        description, 
                        group, 
                        is_local_recipe: false 
                    });
                    
                    // 2. Run the platform upgrade routine to immediately update the system profile
                    if let Err(e) = ctx.apply_upgrade() {
                        eprintln!("Failed to apply target updates to environment: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        // Keep your existing commands down here, updating ctx access as needed...
        _ => {}
    }
}
