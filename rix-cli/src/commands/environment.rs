use rix_core::RixContext;
use crate::handlers;
use crate::ui;

pub fn handle_init(ctx: &RixContext) {
    handlers::execute_init(ctx);
}

pub fn handle_update(ctx: &RixContext) {
    println!("Syncing package index state references from upstream repositories...");
    if let Err(e) = ctx.update_indexes() {
        eprintln!("Update sequence failed: {:?}", e);
        std::process::exit(1);
    }
    println!("Upstream indexes updated successfully!");
}

pub fn handle_upgrade(ctx: &RixContext) {
    println!("Applying generational upgrade across declarative sets...");
    if let Err(e) = ctx.apply_upgrade() {
        eprintln!("Upgrade realization failed: {:?}", e);
        std::process::exit(1);
    }
    println!("System configuration environment generation fully built!");
}

pub fn handle_list(ctx: &RixContext) {
    match ctx.list_all_packages() {
        Ok(packages) => ui::print_package_table(packages),
        Err(e) => { 
            eprintln!("Failed to retrieve packages: {:?}", e); 
            std::process::exit(1); 
        }
    }
}
