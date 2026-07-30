use rix_cli::tui::app::TuiApp;
use rix_core::RixContext;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = rix_cli::config::get_config_dir();
    let ctx = RixContext::new(config_dir);

    let mut app = TuiApp::new(ctx)?;
    app.run()?;

    Ok(())
}
