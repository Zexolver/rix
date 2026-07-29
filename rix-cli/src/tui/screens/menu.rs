use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph},
};
use crate::tui::app::TuiApp;
use super::{Screen, ScreenRenderer};

#[derive(Clone)]
pub struct MenuScreen {
    selected: usize,
    items: Vec<MenuItem>,
}

#[derive(Clone)]
struct MenuItem {
    label: &'static str,
    description: &'static str,
    screen: Screen,
}

impl MenuScreen {
    pub fn new() -> Self {
        let items = vec![
            MenuItem {
                label: "📦 Packages",
                description: "View and manage installed packages",
                screen: Screen::Packages,
            },
            MenuItem {
                label: "📜 History",
                description: "View environment change history",
                screen: Screen::History,
            },
            MenuItem {
                label: "🔄 Update Indexes",
                description: "Sync package indices from upstream",
                screen: Screen::Menu,
            },
            MenuItem {
                label: "🛠️  Settings",
                description: "Configure Rix preferences",
                screen: Screen::Menu,
            },
            MenuItem {
                label: "❓ Help",
                description: "Show keyboard shortcuts and help",
                screen: Screen::Menu,
            },
        ];

        Self {
            selected: 0,
            items,
        }
    }
}

impl ScreenRenderer for MenuScreen {
    fn draw(&self, f: &mut Frame) {
        let size = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(size);

        // Header
        let header = Paragraph::new("🚀 Rix Package Manager")
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Menu Items
        let menu_items: Vec<Line> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == self.selected;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White).bold()
                } else {
                    Style::default().fg(Color::Gray)
                };

                let prefix = if is_selected { "▶ " } else { "  " };
                Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(item.label, style),
                    Span::raw("\n"),
                    Span::styled(
                        format!("     {}", item.description),
                        if is_selected {
                            Style::default().fg(Color::White).italic()
                        } else {
                            Style::default().fg(Color::DarkGray).italic()
                        },
                    ),
                ])
            })
            .collect();

        let menu = Paragraph::new(menu_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Main Menu")
                    .padding(Padding::new(2, 2, 1, 1))
            );

        f.render_widget(menu, chunks[1]);

        // Footer
        let footer = Paragraph::new("↑/↓: Navigate | Enter: Select | q: Quit | ?: Help")
            .style(Style::default().fg(Color::DarkGray).italic())
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }

    fn handle_input(&mut self, key: KeyCode, current_screen: &mut Screen) {
        match key {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = self.items.len() - 1;
                }
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1) % self.items.len();
            }
            KeyCode::Enter => {
                if self.selected < self.items.len() {
                    *current_screen = self.items[self.selected].screen;
                }
            }
            _ => {}
        }
    }
}
