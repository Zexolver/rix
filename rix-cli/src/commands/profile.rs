use rix_core::{Package, RixContext};
use crate::handlers;

pub fn handle_add(ctx: &RixContext, installable: String, description: Option<String>) {
    let (target, group) = installable.split_once('@').unwrap_or((&installable, "default"));
    let package_name = target.split_once('#').map(|(_, pkg)| pkg).unwrap_or(target);
    handlers::execute_add(ctx, Package { 
        name: package_name.to_string(), 
        description, 
        group: group.to_string(), 
        is_local_recipe: target.contains(':') 
    });
}

pub fn handle_remove(ctx: &RixContext, installable: String) {
    let (target, _) = installable.split_once('@').unwrap_or((&installable, "default"));
    let package_name = target.split_once('#').map(|(_, pkg)| pkg).unwrap_or(target);
    handlers::handle_interactive_removal(ctx, package_name);
}
