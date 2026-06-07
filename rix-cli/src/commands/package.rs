use rix_core::{Package, RixContext};
use rix_core::parser;
use rix_core::ops::flake;
use regex::Regex;
use crate::handlers;
use crate::ui;

/// Formats raw Nix AST strings into human-readable package names for the UI table.
/// Import and call this from `handlers.rs` (or wherever your `list` logic lives).
pub fn format_package_name(raw_name: &str) -> String {
    let ext_re = Regex::new(r"__ext_flake or \(([^.]+)\.packages").unwrap();
    if let Some(caps) = ext_re.captures(raw_name) {
        return caps.get(1).unwrap().as_str().to_string();
    }

    if raw_name.starts_with("pkgs.") {
        return raw_name.replacen("pkgs.", "", 1);
    }

    if raw_name.contains("if ") || raw_name.len() > 40 {
        let mut truncated = raw_name.chars().take(35).collect::<String>();
        truncated.push_str("...");
        return truncated;
    }

    raw_name.to_string()
}

pub fn handle_install(ctx: &RixContext, name: String, group: String, description: Option<String>) {
    // 1. 🌐 INTERCEPT: Is this an external Flake URL or URI?
    if name.starts_with("http://") || name.starts_with("https://") || name.contains(':') {
        println!("🌐 Detected external flake URI. Normalizing...");
        
        let uri = parser::normalize_flake_uri(&name);
        let alias = parser::infer_flake_alias(&uri);  
        
        println!("💉 Injecting flake input '{}' into flake.nix...", alias);
        if let Err(e) = flake::add_external_input(&ctx.config_dir, &alias, &uri, &group) {
            eprintln!("Failed to inject flake input: {:?}", e);
            std::process::exit(1);
        }
        
        // Wrap in parentheses to bypass the writer's auto-prefixing, and safely quote the architecture interpolation
        let pkg_expr = format!("__ext_flake or ({}.packages.${{pkgs.system}}.default)", alias);
        let desc = description.unwrap_or_else(|| format!("External flake: {}", uri));
        
        println!("📦 Adding package output to environment...");
        handlers::execute_add(ctx, Package {
            name: pkg_expr,
            group,
            description: Some(desc),
            is_local_recipe: false,
        });
        
        println!("Applying environmental upgrade generations...");
        if let Err(e) = ctx.apply_upgrade() {
            eprintln!("Failed to apply target updates to environment: {:?}", e);
        } else {
            println!("✅ Successfully installed remote flake '{}'", alias);
        }
        return;
    }

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
                
                // UI FIX: Only display the top 15 results
                let display_limit = 15;
                for (path, desc) in results.iter().take(display_limit) {
                    let short_path = path.splitn(3, '.').nth(2).unwrap_or(path);
                    
                    // Truncate description so it doesn't wrap wildly in standard terminals
                    let clean_desc = if desc.len() > 60 {
                        format!("{}...", &desc[..57])
                    } else {
                        desc.to_string()
                    };
                    
                    println!("{:<40} {}", short_path, clean_desc);
                }
                
                if results.len() > display_limit {
                    println!("\n... and {} more results hidden. (Showing top {})", results.len() - display_limit, display_limit);
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
