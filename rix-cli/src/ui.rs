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
    let col1 = "PACKAGE";
    let col2 = "GROUP";
    let col3 = "DESCRIPTION";
    println!("\n{:<20} {:<15} {}", col1, col2, col3);
    println!("{}", "-".repeat(70));
    for (name, group, comment) in packages {
        let desc = if comment.is_empty() { "-" } else { &comment };
        println!("{:<20} {:<15} {}", name, group, desc);
    }
    println!();
}

pub fn print_success(message: &str) {
    println!("✓ {}", message);
}

pub fn print_info(message: &str) {
    println!("ℹ {}", message);
}

pub fn print_warning(message: &str) {
    eprintln!("⚠ {}", message);
}

pub fn print_error(message: &str) {
    eprintln!("✗ {}", message);
}
