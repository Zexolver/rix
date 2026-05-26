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

    /// [Beginner] Install an upstream package directly (e.g., 'rix install eza')
    Install {
        /// Name of the package to install
        name: String,
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

    /// [Beginner] Remove a package cleanly (e.g., 'rix remove eza')
    Remove {
        /// Name of the package to remove
        name: String,
    },

    /// [Beginner] Purge an entire package profile group configuration file
    Purge {
        /// Name of the group profile file to wipe out
        group: String,
    },

    /// [Beginner] Update local channel indices tracking records
    Update,

    /// [Beginner] Upgrade environment generations to match declaration arrays
    Upgrade,

    /// List all currently managed packages across environment profile groups
    List,

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
