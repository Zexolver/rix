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
