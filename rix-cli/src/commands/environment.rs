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
