use rix_core::RixContext;
use crate::handlers;
use crate::ui;
use crate::commands::package::format_package_name;

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
        Ok(packages) => {
            // Map over the packages tuple vector and clean up the names via regex 
            // before handing them off to the UI table renderer
            let polished_packages = packages
                .into_iter()
                .map(|(name, group, desc)| (format_package_name(&name), group, desc))
                .collect();

            ui::print_package_table(polished_packages);
        }
        Err(e) => {  
            eprintln!("Failed to retrieve packages: {:?}", e);  
            std::process::exit(1);  
        }
    }
}
