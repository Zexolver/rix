use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Table, Row, Paragraph},
};
use crate::tui::app::TuiApp;
use super::{Screen, ScreenRenderer};

#[derive(Clone)]
pub struct PackagesScreen {
    selected: usize,
    packages: Vec<PackageItem>,
    mode: PackageMode,
    search_input: String,
}

#[derive(Clone)]
enum PackageMode {
    View,
    Search,
    Install,
    Remove,
}

#[derive(Clone)]
struct PackageItem {
    name: String,
    group: String,
    description: String,
}

impl PackagesScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            packages: Vec::new(),
            mode: PackageMode::View,
            search_input: String::new(),
        }
    }

    fn load_packages(&mut self, app: &TuiApp) {
        if let Ok(pkgs) = app.ctx.list_all_packages() {
            self.packages = pkgs
                .into_iter()
                .map(|(name, group, description)| PackageItem {
                    name,
                    group,
                    description,
                })
                .collect();
        }
    }
}

impl ScreenRenderer for PackagesScreen {
    fn draw(&self, f: &mut Frame) {
        let size = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(4),
            ])
            .split(size);

        // Header
        let header = Paragraph::new("📦 Package Manager")
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Package List
        let rows: Vec<Row> = self
            .packages
            .iter()
            .enumerate()
            .map(|(i, pkg)| {
                let is_selected = i == self.selected;
                let style = if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    pkg.name.clone(),
                    pkg.group.clone(),
                    pkg.description.clone(),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(20),
                Constraint::Percentage(50),
            ],
        )
        .header(
            Row::new(vec!["Name", "Group", "Description"])
                .style(Style::default().bold().fg(Color::Yellow)),
        )
        .block(Block::default().borders(Borders::ALL).title("Installed Packages"));

        f.render_widget(table, chunks[1]);

        // Footer
        let instructions = if self.packages.is_empty() {
            "No packages installed. Press 'i' to install".to_string()
        } else {
            format!(
                "↑/↓: Navigate | i: Install | r: Remove | d: Details | q: Back | {} of {}",
                self.selected + 1,
                self.packages.len()
            )
        };

        let footer = Paragraph::new(instructions)
            .style(Style::default().fg(Color::DarkGray).italic())
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }

    fn handle_input(&mut self, key: KeyCode, current_screen: &mut Screen) {
        match self.mode {
            PackageMode::View => {
                match key {
                    KeyCode::Up => {
                        if self.selected > 0 {
                            self.selected -= 1;
                        } else if !self.packages.is_empty() {
                            self.selected = self.packages.len() - 1;
                        }
                    }
                    KeyCode::Down => {
                        self.selected = (self.selected + 1) % self.packages.len().max(1);
                    }
                    KeyCode::Char('i') => {
                        self.mode = PackageMode::Install;
                    }
                    KeyCode::Char('r') => {
                        if self.selected < self.packages.len() {
                            self.mode = PackageMode::Remove;
                        }
                    }
                    KeyCode::Backspace | KeyCode::Char('q') => {
                        *current_screen = Screen::Menu;
                    }
                    _ => {}
                }
            }
            PackageMode::Install => {
                match key {
                    KeyCode::Char(c) => {
                        self.search_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.search_input.pop();
                    }
                    KeyCode::Enter => {
                        if !self.search_input.is_empty() {
                            // Here you would install the package
                            self.search_input.clear();
                            self.mode = PackageMode::View;
                        }
                    }
                    KeyCode::Esc => {
                        self.search_input.clear();
                        self.mode = PackageMode::View;
                    }
                    _ => {}
                }
            }
            PackageMode::Remove => {
                match key {
                    KeyCode::Char('y') => {
                        // Remove package logic here
                        self.mode = PackageMode::View;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        self.mode = PackageMode::View;
                    }
                    _ => {}
                }
            }
            PackageMode::Search => {
                // Search mode logic
            }
        }
    }
}
