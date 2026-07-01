use super::elevate_privileges;
use crate::ui;
use rix_core::RixContext;
use std::process::Command;
use std::time::Instant;

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

                println!(
                    "🧹 {} [finished in {:.2}s]",
                    summary,
                    duration.as_secs_f64()
                );
            } else {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "❌ Cleanup sequence failed after {:.2}s:\n{}",
                    duration.as_secs_f64(),
                    stderr_str
                );
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("Failed to invoke Nix garbage collector: {:?}", e);
        }
    }
}

pub fn handle_rollback(ctx: &RixContext, target_state: Option<String>) {
    if ctx.is_system && unsafe { libc::geteuid() != 0 } {
        elevate_privileges();
    }

    let msg = match &target_state {
        Some(v) => format!("Rolling back environment state to '{}'...", v),
        None => "Rolling back environment state to previous generation...".to_string(),
    };

    let static_msg: &'static str = Box::leak(msg.into_boxed_str());
    let spinner = ui::create_spinner(static_msg);
    let start_time = Instant::now();

    // 1. Resolve target Git commit (defaults to previous state: HEAD~1)
    let target = target_state.clone().unwrap_or_else(|| "HEAD~1".to_string());

    // 2. Use `git restore` to revert the working directory and staging area to the target commit
    // without detaching HEAD. This is the pure GitOps way!
    let mut cmd_restore = Command::new("git");
    cmd_restore.current_dir(&ctx.config_dir).args([
        "restore",
        "--source",
        &target,
        "--worktree",
        "--staged",
        ".",
    ]);

    let restore_output = match cmd_restore.output() {
        Ok(out) => out,
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("❌ Failed to invoke git: {:?}", e);
            std::process::exit(1);
        }
    };

    if !restore_output.status.success() {
        spinner.finish_and_clear();
        eprintln!(
            "❌ Failed to restore git state from {}:\n{}",
            target,
            String::from_utf8_lossy(&restore_output.stderr)
        );
        std::process::exit(1);
    }

    // 3. Auto-commit the rollback so our Git history moves forward cleanly
    let commit_msg = match &target_state {
        Some(v) => format!("rix: rolled back declarative state to {}", v),
        None => "rix: rolled back declarative state to previous generation".to_string(),
    };
    let mut commit_cmd = Command::new("git");
    commit_cmd
        .current_dir(&ctx.config_dir)
        .args(["commit", "-m", &commit_msg]);
    let _ = commit_cmd.output(); // Silently commit

    // 4. Re-apply the environment using modern flake evaluation!
    // Because the older derivation is already in the Nix store, this will be lightning fast.
    // It entirely bypasses legacy `nix-env` and keeps config files perfectly in sync with the system.
    if let Err(e) = ctx.apply_upgrade(false) {
        spinner.finish_and_clear();
        eprintln!(
            "❌ Failed to re-evaluate and apply rolled-back environment: {:?}",
            e
        );
        std::process::exit(1);
    }

    spinner.finish_and_clear();
    println!(
        "⏪ Environment successfully rolled back and applied [finished in {:.2}s]",
        start_time.elapsed().as_secs_f64()
    );
}
