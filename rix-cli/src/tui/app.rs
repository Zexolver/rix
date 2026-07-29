use ratatui::prelude::*;
use std::io;
use rix_core::RixContext;

use super::events::EventHandler;
use super::ui;
use super::screens::{Screen, MenuScreen, PackagesScreen, HistoryScreen, ScreenRenderer};

#[derive(Clone)]
pub struct TuiApp {
    pub ctx: RixContext,
    pub current_screen: Screen,
    pub menu: MenuScreen,
    pub packages: PackagesScreen,
    pub history: HistoryScreen,
    pub should_quit: bool,
    pub show_help: bool,
}

impl TuiApp {
    pub fn new(ctx: RixContext) -> Self {
        Self {
            ctx,
            current_screen: Screen::Menu,
            menu: MenuScreen::new(),
            packages: PackagesScreen::new(),
            history: HistoryScreen::new(),
            should_quit: false,
            show_help: false,
        }
    }

    pub fn run(
        &mut self,
        mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
        event_handler: EventHandler,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if self.should_quit {
                break;
            }

            if let Some(event) = event_handler.next()? {
                self.handle_event(event);
            }
        }

        Ok(())
    }

    fn draw(&self, f: &mut Frame) {
        match self.current_screen {
            Screen::Menu => self.menu.draw(f),
            Screen::Packages => self.packages.draw(f),
            Screen::History => self.history.draw(f),
        }
    }

    fn handle_event(&mut self, event: crossterm::event::Event) {
        use crossterm::event::{Event, KeyCode, KeyEvent};

        match event {
            Event::Key(KeyEvent { code, .. }) => {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Char('?') => self.show_help = !self.show_help,
                    _ => {
                        match self.current_screen {
                            Screen::Menu => self.menu.handle_input(code, &mut self.current_screen),
                            Screen::Packages => self.packages.handle_input(code, &mut self.current_screen),
                            Screen::History => self.history.handle_input(code, &mut self.current_screen),
                        }
                    }
                }
            }
            Event::Resize(_, _) => {
                // Terminal was resized, redraw happens automatically
            }
            _ => {}
        }
    }
}
