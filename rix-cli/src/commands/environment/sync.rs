use rix_core::RixContext;
use super::elevate_privileges;

pub fn handle_update(ctx: &RixContext) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    // Change working directory to the configuration directory so Nix executes against the correct flake context
    if let Err(e) = std::env::set_current_dir(&ctx.config_dir) {
        eprintln!("⚠ Warning: Failed to switch to configuration directory: {:?}", e);
    }

    println!("Syncing package index state references from upstream repositories... 🤔");
    if let Err(e) = ctx.update_indexes() {
        eprintln!("Update sequence failed: {:?}", e);
        std::process::exit(1);
    }
    println!("Upstream indexes updated successfully!");
}

pub fn handle_refresh(ctx: &RixContext) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    println!("Scanning system PCI interfaces for graphics hardware...\n");
    
    if let Err(e) = rix_core::ops::detect_and_lock_hardware(&ctx.config_dir) {
        eprintln!("Error: Failed to generate hardware lockfile: {}", e);
        std::process::exit(1);
    }

    println!("\nHardware profile synchronized successfully.");
    println!("Note: This hardware state will automatically be injected the next time you modify your environment (e.g., via 'rix install').");
}

pub fn handle_upgrade(ctx: &RixContext, dry_run: bool) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    // Change working directory to the configuration directory so Nix builds against the correct flake context
    if let Err(e) = std::env::set_current_dir(&ctx.config_dir) {
        eprintln!("⚠ Warning: Failed to switch to configuration directory: {:?}", e);
    }

    if dry_run {
        println!("🔍 Executing dry-run upgrade preview...");
    } else {
        println!("Applying generational upgrade across declarative sets...");
    }

    if let Err(e) = ctx.apply_upgrade(dry_run) {
        eprintln!("Upgrade realization failed: {:?}", e);
        std::process::exit(1);
    }
    
    if dry_run {
        println!("Dry-run complete. No system changes were applied.");
    } else {
        println!("System configuration environment generation fully built!");
    }
}
