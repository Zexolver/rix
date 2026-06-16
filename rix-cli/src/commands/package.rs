use rix_core::{Package, RixContext};
use rix_core::parser;
use rix_core::ops::flake;
use regex::Regex;
use std::sync::OnceLock;
use crate::handlers;
use crate::ui;
use crate::commands::environment::elevate_privileges;

/// Formats raw Nix AST strings into human-readable package names for the UI table.
pub fn format_package_name(raw_name: &str) -> String {
    // Strip surrounding parentheses if they exist to make matching cleaner
    let clean_raw = raw_name.trim_matches(|c| c == '(' || c == ')');

    // 1. External flakes: __ext_flake or (alias.packages.${pkgs.system}.default)
    static EXT_RE: OnceLock<Regex> = OnceLock::new();
    let ext_re = EXT_RE.get_or_init(|| Regex::new(r"__ext_flake or \(([^.]+)\.packages").unwrap());
    if let Some(caps) = ext_re.captures(raw_name) {
        return caps.get(1).unwrap().as_str().to_string();
    }

    // 2. Conditional expressions: if pkgs.stdenv.isLinux then pkgs.foo else pkgs.bar (or null)
    static COND_RE: OnceLock<Regex> = OnceLock::new();
    let cond_re = COND_RE.get_or_init(|| {
        Regex::new(r"if\s+pkgs\.stdenv\.isLinux\s+then\s+pkgs\.([^\s]+)\s+else\s+(pkgs\.([^\s]+)|null)").unwrap()
    });
    if let Some(caps) = cond_re.captures(clean_raw) {
        let linux_pkg = caps.get(1).unwrap().as_str();
        let else_match = caps.get(2).unwrap().as_str();
        
        if else_match == "null" {
            return format!("{} (Linux only)", linux_pkg);
        } else if let Some(other_pkg) = caps.get(3) {
            return format!("{} (Linux) / {} (macOS)", linux_pkg, other_pkg.as_str());
        }
    }

    // 3. Standard pkgs. removal
    if clean_raw.starts_with("pkgs.") {
        return clean_raw.replacen("pkgs.", "", 1);
    }

    // 4. Fallback truncation
    if clean_raw.len() > 40 {
        let mut truncated = clean_raw.chars().take(35).collect::<String>();
        truncated.push_str("...");
        return truncated;
    }

    clean_raw.to_string()
}

pub fn handle_install(ctx: &RixContext, packages: Vec<String>, group: String, description: Option<String>) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    let mut needs_upgrade = false;

    for name in &packages {
        // 1. 🌐 INTERCEPT: Is this an external Flake URL or URI?
        if name.starts_with("http://") || name.starts_with("https://") || name.contains(':') {
            println!("🌐 Detected external flake URI for '{}'. Normalizing...", name);
            
            let uri = parser::normalize_flake_uri(&name);
            let alias = parser::infer_flake_alias(&uri);   
            
            println!("💉 Injecting flake input '{}' into flake.nix...", alias);
            if let Err(e) = flake::add_external_input(&ctx.config_dir, &alias, &uri, &group) {
                eprintln!("Failed to inject flake input: {:?}", e);
                std::process::exit(1);
            }
            
            // Wrap in parentheses to bypass the writer's auto-prefixing, and safely quote the architecture interpolation
            let pkg_expr = format!("__ext_flake or ({}.packages.${{pkgs.system}}.default)", alias);
            let desc = description.clone().unwrap_or_else(|| format!("External flake: {}", uri));
            
            println!("📦 Adding package output to environment...");
            handlers::execute_add(ctx, Package {
                name: pkg_expr,
                group: group.clone(),
                description: Some(desc),
                is_local_recipe: false,
            });
            
            needs_upgrade = true;
            continue;
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
                    description: description.clone(),   
                    group: group.clone(),   
                    is_local_recipe: false   
                });
                
                needs_upgrade = true;
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!("{:?}", e);   
                std::process::exit(1);
            }
        }
    }

    if needs_upgrade {
        println!("Successfully optimized environment config changes!");
        println!("Applying environmental upgrade generations...");
        if let Err(e) = ctx.apply_upgrade(false) {
            eprintln!("Failed to apply target updates to environment: {:?}", e);
        } else {
            println!("✅ Successfully updated environment generation!");
            
            // Auto-commit the successfully installed packages
            let commit_msg = format!("rix: installed {}", packages.join(", "));
            if let Err(e) = rix_core::system::sync::auto_commit(&ctx.config_dir, &commit_msg) {
                eprintln!("⚠ Warning: Failed to auto-commit changes: {:?}", e);
            }
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
                    
                    // UTF-8 SAFE TRUNCATION: Avoid byte-slicing panics if descriptions contain multi-byte characters
                    let clean_desc = if desc.len() > 60 {
                        let mut truncated = desc.chars().take(57).collect::<String>();
                        truncated.push_str("...");
                        truncated
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

pub fn handle_remove(ctx: &RixContext, packages: Vec<String>) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    for name in &packages {
        handlers::handle_interactive_removal(ctx, name);
    }
    
    // Auto-commit the successfully removed packages
    let commit_msg = format!("rix: removed {}", packages.join(", "));
    if let Err(e) = rix_core::system::sync::auto_commit(&ctx.config_dir, &commit_msg) {
        eprintln!("⚠ Warning: Failed to auto-commit changes: {:?}", e);
    }
}

pub fn handle_purge(ctx: &RixContext, group: String) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    println!("Purging profile group configuration layout '{}'...", group);
    if let Err(e) = ctx.purge_group_profile(&group) {
        eprintln!("Purge sequence failed: {:?}", e);
        std::process::exit(1);
    }
    println!("Successfully purged profile configuration!");
}
