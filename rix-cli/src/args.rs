use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rix", version, about = "A fast, automated Nix environment optimizer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
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

    /// [Beginner] Remove a package cleanly (e.g., 'rix remove eza')
    Remove {
        /// Name of the package to remove
        name: String,
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
