use super::elevate_privileges;
use crate::commands::package::format_package_name;
use crate::ui;
use rix_core::RixContext;
use std::process::Command;
use std::time::Instant;

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
    // Escalate if we are targeting the system scope so we can read /etc/rix/.git
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    let start_time = Instant::now();
    let spinner = ui::create_spinner("Reading declarative history...");

    // Query the local Git repository backing the configuration
    let mut cmd = Command::new("git");
    cmd.current_dir(&ctx.config_dir);
    cmd.args([
        "log",
        "--pretty=format:%h|%ad|%s",
        "--date=format:%Y-%m-%d %H:%M",
        "-n",
        "20", // Limit to recent 20 commits for speed and readability
    ]);

    match cmd.output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            spinner.finish_and_clear();

            if output.status.success() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);

                println!(
                    "⏳ Declarative Environment History [finished in {:.2}s]\n",
                    duration.as_secs_f64()
                );

                let lines: Vec<&str> = stdout_str.lines().collect();
                if lines.is_empty() {
                    println!("No environment history found. Try installing a package!");
                    return;
                }

                // Since Git log outputs newest first, we count backwards from the total lines
                let total_states = lines.len();

                for (i, line) in lines.iter().enumerate() {
                    let parts: Vec<&str> = line.splitn(3, '|').collect();
                    if parts.len() == 3 {
                        let hash = parts[0];
                        let date = parts[1];
                        let msg = parts[2];
                        let state_num = total_states - i;

                        if i == 0 {
                            println!("• \x1b[1;36mState {} (Active)\x1b[0m", state_num);
                        } else {
                            println!("• \x1b[1;36mState {}\x1b[0m", state_num);
                        }
                        println!("  \x1b[2m↳ Hash:   {}\x1b[0m", hash);
                        println!("  \x1b[2m↳ Date:   {}\x1b[0m", date);
                        println!("  \x1b[2m↳ Action: {}\x1b[0m\n", msg);
                    }
                }
            } else {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                // If it fails because the repo isn't there yet, handle gracefully
                if stderr_str.contains("not a git repository") {
                    println!("No Git history initialized yet. Try running 'rix init'.");
                } else {
                    eprintln!(
                        "❌ Failed to read git history state after {:.2}s:\n{}",
                        duration.as_secs_f64(),
                        stderr_str
                    );
                }
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("Failed to execute git log sequence: {:?}", e);
        }
    }
}
