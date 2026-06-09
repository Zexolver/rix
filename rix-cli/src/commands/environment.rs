use rix_core::RixContext;
use std::process::Command;
use std::time::Instant;
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

pub fn handle_upgrade(ctx: &RixContext, dry_run: bool) {
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

pub fn handle_clean(deep: bool) {
    let msg = if deep {
        "Deep cleaning Nix store (removing old generations & orphans)..."
    } else {
        "Sweeping Nix store (removing orphaned derivations)..."
    };
    
    // Coerce into a static string slice for the spinner
    let static_msg: &'static str = Box::leak(msg.to_string().into_boxed_str());
    let spinner = ui::create_spinner(static_msg);

    // Start the stopwatch
    let start_time = Instant::now();

    let mut cmd = Command::new("nix-collect-garbage");
    
    if deep {
        cmd.arg("-d");    
    }

    match cmd.output() {
        Ok(output) => {
            // Stop the stopwatch right after the command finishes
            let duration = start_time.elapsed();
            spinner.finish_and_clear();
            
            if output.status.success() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                
                // Extract only the final summary line from the massive nix output
                let summary = stdout_str
                    .lines()
                    .filter(|l| !l.is_empty())
                    .last()
                    .unwrap_or("Garbage collection complete.");
                    
                // Append the Cargo-style execution timer to the output
                println!("🧹 {} [finished in {:.2}s]", summary, duration.as_secs_f64());
            } else {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                eprintln!("❌ Cleanup sequence failed after {:.2}s:\n{}", duration.as_secs_f64(), stderr_str);
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("Failed to invoke Nix garbage collector: {:?}", e);
        }
    }
}

pub fn handle_history(_ctx: &RixContext) {
    let spinner = ui::create_spinner("Reading profile generation history...");

    let mut cmd = Command::new("nix");
    cmd.args(["profile", "history"]);

    match cmd.output() {
        Ok(output) => {
            spinner.finish_and_clear();

            if output.status.success() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                
                println!("⏳ Environment Generation History\n");
                
                for line in stdout_str.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Parse out generation or version headers cleanly
                    if trimmed.starts_with("Generation") || trimmed.starts_with("Version") {
                        let gen_line = trimmed.replace(":", "");
                        println!("• \x1b[1;36m{}\x1b[0m", gen_line);
                    } else {
                        // Apply indentation and a subtle grey style to diff modification text
                        println!("  \x1b[2m↳ {}\x1b[0m", trimmed);
                    }
                }
            } else {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                eprintln!("❌ Failed to read history state:\n{}", stderr_str);
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("Failed to execute Nix history sequence: {:?}", e);
        }
    }
}

pub fn handle_rollback(_ctx: &RixContext, version: Option<String>) {
    let msg = match &version {
        Some(v) => format!("Rolling back environment state to version {}...", v),
        None => "Rolling back environment state to previous version...".to_string(),
    };
    
    let static_msg: &'static str = Box::leak(msg.into_boxed_str());
    let spinner = ui::create_spinner(static_msg);
    let start_time = Instant::now();

    let mut cmd = Command::new("nix");
    cmd.arg("profile").arg("rollback");

    if let Some(v) = version {
        cmd.arg("--to").arg(v);
    }

    match cmd.output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            spinner.finish_and_clear();

            if output.status.success() {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                
                // Nix usually outputs tracking logs to stderr for rollbacks
                let summary = stderr_str
                    .lines()
                    .filter(|l| !l.is_empty())
                    .last()
                    .unwrap_or("Environment generation rollback successful.");

                println!("⏪ {} [finished in {:.2}s]", summary.trim(), duration.as_secs_f64());
            } else {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                eprintln!("❌ Rollback failed after {:.2}s:\n{}", duration.as_secs_f64(), stderr_str.trim());
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("Failed to execute Nix rollback sequence: {:?}", e);
        }
    }
}
