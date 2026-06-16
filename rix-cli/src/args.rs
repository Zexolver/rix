use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rix", version, about = "A fast, automated Nix environment optimizer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// [Beginner] Initialize a new declarative Nix environment profile layout scaffolding
    Init,

    /// [Beginner] Install upstream packages directly (e.g., 'rix install eza ripgrep')
    Install {
        /// Names of the packages to install
        #[arg(required = true)]
        packages: Vec<String>,
        /// Profile group to place the package in
        #[arg(short, long, default_value = "default")]
        group: String,
        /// Custom description comment
        #[arg(short, long)]
        description: Option<String>,
    },

    /// [Beginner] Search online package indices directly via the modern flake registry
    Search {
        /// The package name or keyword to search for
        query: String,
    },

    /// [Beginner] Remove packages cleanly (e.g., 'rix remove eza ripgrep')
    Remove {
        /// Names of the packages to remove
        #[arg(required = true)]
        packages: Vec<String>,
    },

    /// [Beginner] Purge an entire package profile group configuration file
    Purge {
        /// Name of the group profile file to wipe out
        group: String,
    },

    /// [Beginner] Update local channel indices tracking records
    Update,

    /// [System] Detect hardware and update the lockfile for NixGL graphics acceleration
    Refresh,

    /// [Beginner] Upgrade environment generations to match declaration arrays
    Upgrade {
        /// Preview changes without actually compiling or modifying the system
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },

    /// List all currently managed packages across environment profile groups
    List,

    /// [Maintenance] Sweeps the Nix store to reclaim disk space
    Clean {
        /// Aggressively removes old system/profile generations before cleaning
        #[arg(short, long)]
        deep: bool,
    },

    /// [Maintenance] View the chronological history of environment profile generations
    #[command(alias = "gen")]
    History,

    /// [Maintenance] Revert the active profile environment to a previous generation state
    Rollback {
        /// Optional specific generation target number (defaults to previous)
        version: Option<String>,
    },

    /// [Maintenance] Sync local declarative state to an upstream Git repository
    Sync {
        /// Optional Git repository URL to set as the upstream remote
        remote_url: Option<String>,
    },

    /// [Intermediate] Native Nix-style profile manipulation interface
    #[command(subcommand)]
    Profile(ProfileCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum ProfileCommands {
    /// Add an installable target (e.g., 'rix profile add nixpkgs#eza@system')
    Add {
        /// Installable target target string
        installable: String,
        /// Custom description comment
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Remove an installable target
    Remove {
        /// Installable target to remove
        installable: String,
    },
}
