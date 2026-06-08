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
        Commands::Upgrade => environment::handle_upgrade(&ctx),
        Commands::List => environment::handle_list(&ctx),
        Commands::Clean { deep } => environment::handle_clean(deep),
        
        // Package lifecycle commands
        Commands::Install { name, group, description } => package::handle_install(&ctx, name, group, description),
        Commands::Search { query } => package::handle_search(&ctx, query),
        Commands::Remove { name } => package::handle_remove(&ctx, name),
        Commands::Purge { group } => package::handle_purge(&ctx, group),

        // Native profile manipulation
        Commands::Profile(ProfileCommands::Add { installable, description }) => profile::handle_add(&ctx, installable, description),
        Commands::Profile(ProfileCommands::Remove { installable }) => profile::handle_remove(&ctx, installable),
    }
}
