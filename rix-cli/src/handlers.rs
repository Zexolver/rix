use crate::ui;
use rix_core::{FoundPackage, Package, RixContext};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn execute_init(ctx: &RixContext) {
    println!("\nInitializing modern declarative Nix profile environment...\n");

    println!("Rix can operate in two primary scopes:");
    println!("  [1] User   (Local home directory, no root required)");
    println!("  [2] System (Global /etc/rix, requires sudo)");
    print!("\nSelect default operation scope for future commands (1-2) [1]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    let default_system = choice == "2";
    let scope_str = if default_system { "system" } else { "user" };

    let home_dir = std::env::var("SUDO_USER")
        .map(|u| format!("/home/{}", u))
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "/root".to_string());

    let user_config_dir = PathBuf::from(home_dir).join(".config").join("rix");

    if !user_config_dir.exists() {
        if let Err(e) = fs::create_dir_all(&user_config_dir) {
            ui::print_warning(&format!("Failed to create config directory: {}", e));
        }
    }

    let toml_path = user_config_dir.join("rix.toml");
    let toml_content = format!("[core]\ndefault_scope = \"{}\"\n", scope_str);

    match fs::write(&toml_path, toml_content) {
        Ok(_) => ui::print_success(&format!(
            "Default scope set to '{}' in {}",
            scope_str,
            toml_path.display()
        )),
        Err(e) => ui::print_warning(&format!("Failed to save default scope configuration: {}", e)),
    }

    println!();

    let flake_path = ctx.config_dir.join("flake.nix");
    if flake_path.exists() {
        ui::print_info(&format!(
            "Environment workspace layout is already fully initialized at: {}",
            ctx.config_dir.display()
        ));
        return;
    }

    match ctx.initialize_layout() {
        Ok(_) => {
            println!("\n🎉 Successfully generated file layout structural scaffolding!");
            println!("   • Configuration directory: {}", ctx.config_dir.display());
            println!(
                "   • Base declarative flake: {}/flake.nix",
                ctx.config_dir.display()
            );
            println!(
                "   • Default group template: {}/groups/upstream/default.nix",
                ctx.config_dir.display()
            );
            println!("\nYou are ready to optimize! Try installing your first tool:");
            println!("  rix install fastfetch");
        }
        Err(e) => {
            ui::print_error(&format!("Initialization failed: {}", e));
            std::process::exit(1);
        }
    }
}

pub fn execute_add(ctx: &RixContext, package: Package) {
    let msg = Box::leak(
        format!(
            "Syncing '{}' into group '{}'",
            package.name, package.group
        )
        .into_boxed_str(),
    );
    let spinner = ui::create_spinner(msg);

    if let Err(e) = ctx.add_package(package) {
        spinner.finish_and_clear();
        ui::print_error(&format!("Failed to add package: {}", e));
        std::process::exit(1);
    }
    spinner.finish_with_message("✓ Successfully added package to environment".to_string());
}

pub fn handle_interactive_removal(ctx: &RixContext, query: &str) {
    let matches = match ctx.lookup_packages(query) {
        Ok(m) => m,
        Err(e) => {
            ui::print_error(&format!("Search failed: {}", e));
            std::process::exit(1);
        }
    };

    if matches.is_empty() {
        ui::print_error(&format!(
            "No packages matching '{}' found in configuration profiles",
            query
        ));
        std::process::exit(1);
    }

    let selected: &FoundPackage = if matches.len() == 1 {
        &matches[0]
    } else {
        println!("\nMultiple packages matching '{}' found:", query);
        for (i, pkg) in matches.iter().enumerate() {
            let filename = pkg
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            println!(
                "  [{}] {} (in: {})",
                i + 1,
                pkg.name,
                filename
            );
        }
        print!("\nSelect package to remove (1-{}): ", matches.len());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice: usize = input.trim().parse().unwrap_or(0);

        if choice < 1 || choice > matches.len() {
            ui::print_error("Invalid selection. Operation canceled");
            std::process::exit(1);
        }
        &matches[choice - 1]
    };

    let msg = Box::leak(
        format!("Removing '{}' from configuration", selected.name).into_boxed_str(),
    );
    let spinner = ui::create_spinner(msg);

    if let Err(e) = ctx.remove_package_from_file(&selected.name, &selected.file_path) {
        spinner.finish_and_clear();
        ui::print_error(&format!("Failed to remove package: {}", e));
        std::process::exit(1);
    }

    spinner.finish_with_message("✓ Package removed from configuration".to_string());

    let spinner = ui::create_spinner("Synchronizing environment changes");
    if let Err(e) = ctx.apply_upgrade(false) {
        spinner.finish_and_clear();
        ui::print_error(&format!("Failed to apply changes: {}", e));
        std::process::exit(1);
    }
    spinner.finish_with_message("✓ Environment configuration synchronized".to_string());
}
