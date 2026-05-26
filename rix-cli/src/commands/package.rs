use rix_core::{Package, RixContext};
use crate::handlers;
use crate::ui;

pub fn handle_install(ctx: &RixContext, name: String, group: String, description: Option<String>) {
    // Coerce the runtime String into a &'static str so the spinner can safely hold it
    let message = Box::leak(format!("Querying upstream package indices for '{}'...", name).into_boxed_str());
    let spinner = ui::create_spinner(message);
    
    match rix_core::verify::verify_online_package_architecture(&name) {
        Ok(verified_name) => {
            // Drop the spinner completely before modifying state files or prompting for sudo
            spinner.finish_and_clear();
            
            handlers::execute_add(ctx, Package { 
                name: verified_name, 
                description, 
                group, 
                is_local_recipe: false 
            });
                                
            println!("Applying environmental upgrade generations...");
            if let Err(e) = ctx.apply_upgrade() {
                eprintln!("Failed to apply target updates to environment: {:?}", e);
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
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
