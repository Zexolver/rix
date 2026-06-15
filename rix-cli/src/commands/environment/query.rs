use rix_core::RixContext;
use std::process::Command;
use std::time::Instant;
use crate::ui;
use crate::commands::package::format_package_name;
use super::elevate_privileges;

pub fn handle_list(ctx: &RixContext) {
    match ctx.list_all_packages() {
        Ok(packages) => {
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

pub fn handle_history(ctx: &RixContext) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    let start_time = Instant::now();
    let spinner = ui::create_spinner("Reading profile generation history...");

    let mut cmd = if ctx.is_system {
        let mut c = Command::new("nix-env");
        c.args(["--profile", "/nix/var/nix/profiles/default", "--list-generations"]);
        c
    } else {
        let mut c = Command::new("home-manager");
        c.arg("generations");
        c
    };

    match cmd.output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            spinner.finish_and_clear();

            if output.status.success() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                
                println!("⏳ Environment Generation History [finished in {:.2}s]\n", duration.as_secs_f64());
                
                for line in stdout_str.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    if trimmed.starts_with("Generation") || trimmed.starts_with("Version") {
                        let gen_line = trimmed.replace(":", "");
                        println!("• \x1b[1;36m{}\x1b[0m", gen_line);
                    } else {
                        println!("  \x1b[2m↳ {}\x1b[0m", trimmed);
                    }
                }
            } else {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                eprintln!("❌ Failed to read history state after {:.2}s:\n{}", duration.as_secs_f64(), stderr_str);
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("Failed to execute Nix history sequence: {:?}", e);
        }
    }
}
