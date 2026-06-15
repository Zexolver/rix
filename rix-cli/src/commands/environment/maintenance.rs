use rix_core::RixContext;
use std::process::Command;
use std::time::Instant;
use crate::ui;
use super::elevate_privileges;

pub fn handle_clean(deep: bool) {
    let msg = if deep {
        "Deep cleaning Nix store (removing old generations & orphans)..."
    } else {
        "Sweeping Nix store (removing orphaned derivations)..."
    };
    
    let static_msg: &'static str = Box::leak(msg.to_string().into_boxed_str());
    let spinner = ui::create_spinner(static_msg);
    let start_time = Instant::now();

    let mut cmd = Command::new("nix-collect-garbage");
    if deep {
        cmd.arg("-d");     
    }

    match cmd.output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            spinner.finish_and_clear();
            
            if output.status.success() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                let summary = stdout_str
                    .lines()
                    .filter(|l| !l.is_empty())
                    .last()
                    .unwrap_or("Garbage collection complete.");
                    
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

pub fn handle_rollback(ctx: &RixContext, version: Option<String>) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    let msg = match &version {
        Some(v) => format!("Rolling back environment state to version {}...", v),
        None => "Rolling back environment state to previous version...".to_string(),
    };
    
    let static_msg: &'static str = Box::leak(msg.into_boxed_str());
    let spinner = ui::create_spinner(static_msg);
    let start_time = Instant::now();

    let mut cmd = if ctx.is_system {
        let mut c = Command::new("nix-env");
        c.args(["--profile", "/nix/var/nix/profiles/default"]);
        if let Some(v) = version {
            c.arg("--switch-generation").arg(v);
        } else {
            c.arg("--rollback");
        }
        c
    } else {
        let mut c = Command::new("nix-env");
        if let Ok(home) = std::env::var("HOME") {
            let hm_profile = format!("{}/.local/state/nix/profiles/home-manager", home);
            c.args(["--profile", &hm_profile]);
        }
        if let Some(v) = version {
            c.arg("--switch-generation").arg(v);
        } else {
            c.arg("--rollback");
        }
        c
    };

    match cmd.output() {
        Ok(output) => {
            let duration = start_time.elapsed();
            spinner.finish_and_clear();

            if output.status.success() {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                let output_text = if !stderr_str.trim().is_empty() { stderr_str } else { stdout_str };
                
                let summary = output_text
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
