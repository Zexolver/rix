mod app;
mod ui;
mod events;
mod screens;

pub use app::TuiApp;
pub use events::EventHandler;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

pub fn run_tui(ctx: &rix_core::RixContext) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new(ctx.clone());
    let event_handler = EventHandler::new(250);

    let res = app.run(terminal, event_handler);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    res
}
