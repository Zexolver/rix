use rix_core::RixContext;
use crate::handlers;
use super::elevate_privileges;

pub fn handle_init(ctx: &RixContext) {
    if std::env::var("RIX_INIT_NONINTERACTIVE").is_ok() {
        println!("✨ Environment workspace layout is already fully initialized at: /etc/rix");
    } else {
        handlers::execute_init(ctx);
        
        // Automatically check if they selected system scope and escalate if not root
        if unsafe { libc::geteuid() != 0 } {
            if let Ok(home) = std::env::var("HOME") {
                let config_path = format!("{}/.config/rix/rix.toml", home);
                if let Ok(content) = std::fs::read_to_string(config_path) {
                    if content.contains("scope = 'system'") || content.contains("scope = \"system\"") {
                        println!("🔒 System scope selected. Elevating privileges to initialize /etc/rix workspace...");
                        unsafe {
                            std::env::set_var("RIX_INIT_NONINTERACTIVE", "system");
                        }
                        elevate_privileges();
                    }
                }
            }
        }
    }
    
    // Explicitly target `/etc/rix` if we are the post-escalation child initialization process
    let target_config_dir = if unsafe { libc::geteuid() == 0 } && std::env::var("RIX_INIT_NONINTERACTIVE").is_ok() {
        std::path::PathBuf::from("/etc/rix")
    } else {
        ctx.config_dir.clone()
    };

    // Wire up Git init!
    if let Err(e) = rix_core::system::sync::init_local_repo(&target_config_dir) {
        eprintln!("⚠ Warning: Failed to initialize Git repository: {:?}", e);
    } else {
        println!("✅ Initialized local Git repository for version control.");
        // Do a base commit so the first installs have a clean working tree
        let _ = rix_core::system::sync::auto_commit(&target_config_dir, "rix: initialized environment");
    }
}
