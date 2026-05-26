use rix_core::{Package, RixContext};
use crate::handlers;
use crate::ui;

pub fn handle_install(ctx: &RixContext, name: String, group: String, description: Option<String>) {
    println!("Searching online package indices for '{}'...", name);
    match rix_core::verify::verify_online_package_architecture(&name) {
        Ok(verified_name) => {
            handlers::execute_add(ctx, Package { 
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

pub fn handle_search(_ctx: &RixContext, query: String) {
    let spinner = ui::create_spinner("Querying modern Flake registry...");
    
    match rix_core::verify::run_nix_search(&query) {
        Ok(results) => {
            spinner.finish_and_clear();
            if results.is_empty() {
                println!("No packages matched your query.");
            } else {
                println!("\n{:<40} {}", "PACKAGE ATTRIBUTE PATH", "DESCRIPTION");
                println!("{}", "-".repeat(80));
                for (path, desc) in results {
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

pub fn handle_remove(ctx: &RixContext, name: String) {
    handlers::handle_interactive_removal(ctx, &name);
}

pub fn handle_purge(ctx: &RixContext, group: String) {
    println!("Purging profile group configuration layout '{}'...", group);
    if let Err(e) = ctx.purge_group_profile(&group) {
        eprintln!("Purge sequence failed: {:?}", e);
        std::process::exit(1);
    }
    println!("Successfully purged profile configuration!");
}
