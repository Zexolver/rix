pub mod args;
pub mod commands;
pub mod config;
pub mod handlers;
pub mod ui;
pub mod tui;

// Re-export for GUI use
pub use config::get_config_dir;
