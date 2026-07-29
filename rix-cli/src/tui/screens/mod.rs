pub mod menu;
pub mod packages;
pub mod history;

pub use menu::MenuScreen;
pub use packages::PackagesScreen;
pub use history::HistoryScreen;

use crossterm::event::KeyCode;
use ratatui::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Packages,
    History,
}

pub trait ScreenRenderer {
    fn draw(&self, f: &mut Frame);
    fn handle_input(&mut self, key: KeyCode, current_screen: &mut Screen);
}
