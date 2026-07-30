use rix_cli::{config};
use rix_core::RixContext;
use std::process;

slint::include_modules!();

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = config::get_config_dir();
    let ctx = RixContext::new(config_dir);

    let ui = MainWindow::new()?;

    // Load packages
    if let Ok(pkgs) = ctx.list_all_packages() {
        let packages: Vec<Package> = pkgs
            .into_iter()
            .map(|(name, group, description)| Package {
                name: name.into(),
                group: group.into(),
                description: description.into(),
            })
            .collect();

        let packages_model = std::rc::Rc::new(slint::VecModel::from(packages));
        ui.set_packages(packages_model.into());
    }

    // Set up callbacks
    let ui_handle = ui.as_weak();
    ui.on_show_packages_screen(move || {
        let ui = ui_handle.upgrade().unwrap();
        ui.set_current_screen(1);
    });

    let ui_handle = ui.as_weak();
    ui.on_show_history_screen(move || {
        let ui = ui_handle.upgrade().unwrap();
        ui.set_current_screen(2);
    });

    let ui_handle = ui.as_weak();
    ui.on_show_menu_screen(move || {
        let ui = ui_handle.upgrade().unwrap();
        ui.set_current_screen(0);
    });

    let ui_handle = ui.as_weak();
    ui.on_install_package(move |package_name| {
        let ui = ui_handle.upgrade().unwrap();
        println!("Installing package: {}", package_name);
        ui.set_current_screen(1);
    });

    let ui_handle = ui.as_weak();
    ui.on_remove_package(move |package_name| {
        let ui = ui_handle.upgrade().unwrap();
        println!("Removing package: {}", package_name);
    });

    let ui_handle = ui.as_weak();
    ui.on_rollback_to_state(move |state_num| {
        let ui = ui_handle.upgrade().unwrap();
        println!("Rolling back to state: {}", state_num);
        ui.set_current_screen(2);
    });

    ui.run()?;
    Ok(())
}
