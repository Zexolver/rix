pub mod environment;
pub mod package;
pub mod profile;

use crate::args::{Cli, Commands, ProfileCommands};
use rix_core::RixContext;

pub fn handle(cli: Cli, ctx: RixContext) {
    match cli.command {
        // Environment lifecycle commands
        Commands::Init => environment::handle_init(&ctx),
        Commands::Update => environment::handle_update(&ctx),
        Commands::Refresh => environment::handle_refresh(&ctx),
        Commands::Upgrade { dry_run } => environment::handle_upgrade(&ctx, dry_run),
        Commands::List => environment::handle_list(&ctx),
        Commands::Clean { deep } => environment::handle_clean(deep),
        Commands::History => environment::handle_history(&ctx),
        Commands::Rollback { version } => environment::handle_rollback(&ctx, version),
        
        // Remote Sync
        Commands::Sync { remote_url } => {
            println!("🔄 Syncing declarative state to remote...");
            if let Err(e) = rix_core::system::sync::sync_to_remote(&ctx.config_dir, remote_url.as_deref()) {
                eprintln!("❌ Sync failed: {}", e);
            } else {
                println!("✅ Successfully synced environment state to remote!");
            }
        }
        
        // Package lifecycle commands
        Commands::Install { packages, group, description } => package::handle_install(&ctx, packages, group, description),
        Commands::Search { query } => package::handle_search(&ctx, query),
        Commands::Remove { packages } => package::handle_remove(&ctx, packages),
        Commands::Purge { group } => package::handle_purge(&ctx, group),

        // Native profile manipulation
        Commands::Profile(ProfileCommands::Add { installable, description }) => profile::handle_add(&ctx, installable, description),
        Commands::Profile(ProfileCommands::Remove { installable }) => profile::handle_remove(&ctx, installable),
    }
}
