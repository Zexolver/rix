use super::elevate_privileges;
use crate::ui;
use rix_core::RixContext;

pub fn handle_update(ctx: &RixContext) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    if let Err(e) = std::env::set_current_dir(&ctx.config_dir) {
        ui::print_warning(&format!("Failed to switch to configuration directory: {}", e));
    }

    let spinner = ui::create_spinner("Syncing package indexes");
    if let Err(e) = ctx.update_indexes() {
        spinner.finish_and_clear();
        ui::print_error(&format!("Failed to update indexes: {}", e));
        std::process::exit(1);
    }
    spinner.finish_with_message("✓ Package indexes updated successfully".to_string());
}

pub fn handle_refresh(ctx: &RixContext) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    let spinner = ui::create_spinner("Scanning hardware configuration");

    if let Err(e) = rix_core::ops::detect_and_lock_hardware(&ctx.config_dir) {
        spinner.finish_and_clear();
        ui::print_error(&format!("Failed to generate hardware lockfile: {}", e));
        std::process::exit(1);
    }

    spinner.finish_with_message("✓ Hardware profile synchronized".to_string());
    ui::print_info("Hardware will be injected automatically on next environment update");
}

pub fn handle_upgrade(ctx: &RixContext, dry_run: bool) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    if let Err(e) = std::env::set_current_dir(&ctx.config_dir) {
        ui::print_warning(&format!("Failed to switch to configuration directory: {}", e));
    }

    let msg = if dry_run {
        "Performing dry-run upgrade preview"
    } else {
        "Applying environment upgrade"
    };

    let spinner = ui::create_spinner(msg);

    if let Err(e) = ctx.apply_upgrade(dry_run) {
        spinner.finish_and_clear();
        ui::print_error(&format!("Upgrade failed: {}", e));
        std::process::exit(1);
    }

    if dry_run {
        spinner.finish_with_message("✓ Dry-run complete - no system changes applied".to_string());
    } else {
        spinner.finish_with_message("✓ Environment upgrade complete".to_string());
    }
}
