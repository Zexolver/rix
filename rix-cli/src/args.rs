use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rix", version, about = "A fast, automated Nix environment optimizer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Install a package into a specific upstream profile group
    Install {
        /// Name of the package to install (e.g., ripgrep, eza)
        name: String,

        /// The category profile group file to place the package in
        #[arg(short, long, default_value = "default")]
        group: String,

        /// An optional descriptive comment explaining what this package does
        #[arg(short, long)]
        description: Option<String>,
    },
}
