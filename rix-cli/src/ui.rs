use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_spinner(message: &'static str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message(message);
    pb
}

pub fn print_package_table(packages: Vec<(String, String, String)>) {
    if packages.is_empty() {
        println!("No declarative environment packages tracked yet.");
        return;
    }
    println!("\n{:<15} {:<15} {}", "PACKAGE", "GROUP", "DESCRIPTION");
    println!("{}", "-".repeat(60));
    for (name, group, comment) in packages {
        println!("{:<15} {:<15} {}", name, group, comment);
    }
    println!();
}
