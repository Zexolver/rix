use std::io::{self, Write};
use rix_core::{Package, RixContext, FoundPackage};

pub fn execute_init(ctx: &RixContext) {
    println!("Initializing modern declarative Nix profile environment...");
    
    // Check if a layout already exists by verifying the root flake configuration path
    let flake_path = ctx.config_dir.join("flake.nix");
    if flake_path.exists() {
        println!("✨ Environment workspace layout is already fully initialized at: {}", ctx.config_dir.display());
        return;
    }

    match ctx.initialize_layout() {
        Ok(_) => {
            println!("🎉 Successfully generated file layout structural scaffolding!");
            println!("   ↳ Configuration directory: {}", ctx.config_dir.display());
            println!("   ↳ Base declarative flake: {}/flake.nix", ctx.config_dir.display());
            println!("   ↳ Default group template: {}/groups/upstream/default.nix", ctx.config_dir.display());
            println!("\nYou are ready to optimize! Try installing your first tool: 'rix install fastfetch'");
        }
        Err(e) => {
            eprintln!("Initialization failed: {:?}", e);
            std::process::exit(1);
        }
    }
}

pub fn execute_add(ctx: &RixContext, package: Package) {
    println!("Syncing '{}' into target profile group '{}'...", package.name, package.group);
    if let Err(e) = ctx.add_package(package) {
        eprintln!("Operation failed: {:?}", e);
        std::process::exit(1);
    }
    println!("Successfully optimized environment config changes!");
}

pub fn handle_interactive_removal(ctx: &RixContext, query: &str) {
    let matches = match ctx.lookup_packages(query) {
        Ok(m) => m,
        Err(e) => { eprintln!("Search failed: {:?}", e); std::process::exit(1); }
    };

    if matches.is_empty() {
        eprintln!("Error: No packages matching '{}' found in configuration profiles.", query);
        std::process::exit(1);
    }

    let selected: &FoundPackage = if matches.len() == 1 {
        &matches[0]
    } else {
        println!("\nMultiple package tracking matches detected for '{}':", query);
        for (i, pkg) in matches.iter().enumerate() {
            let filename = pkg.file_path.file_name().unwrap_or_default().to_string_lossy();
            println!("  [{}] {} (found in group file: {})", i + 1, pkg.name, filename);
        }
        print!("Select which application to strip out (1-{}): ", matches.len());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice: usize = input.trim().parse().unwrap_or(0);

        if choice < 1 || choice > matches.len() {
            eprintln!("Invalid selection. Operation canceled.");
            std::process::exit(1);
        }
        &matches[choice - 1]
    };

    println!("Removing '{}' from configuration file...", selected.name);
    if let Err(e) = ctx.remove_package_from_file(&selected.name, &selected.file_path) {
        eprintln!("Removal failed: {:?}", e);
        std::process::exit(1);
    }
    println!("Successfully stripped tool tracking from profile configuration!");

    // Apply configuration updates to the active live generation environment automatically
    println!("Synchronizing environmental generational upgrades...");
    if let Err(e) = ctx.apply_upgrade(false) {
        eprintln!("Failed to realize system environment state changes: {:?}", e);
        std::process::exit(1);
    }
    println!("Environment configuration synchronized successfully!");
}
