use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph},
};
use crate::tui::app::TuiApp;
use super::{Screen, ScreenRenderer};

#[derive(Clone)]
pub struct HistoryScreen {
    selected: usize,
    entries: Vec<HistoryEntry>,
}

#[derive(Clone)]
struct HistoryEntry {
    state_num: usize,
    hash: String,
    date: String,
    message: String,
    is_active: bool,
}

impl HistoryScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            entries: Vec::new(),
        }
    }

    fn load_history(&mut self, _app: &TuiApp) {
        // In a real implementation, this would load from git history
        // For now, show placeholder entries
        self.entries = vec![
            HistoryEntry {
                state_num: 5,
                hash: "a1b2c3d".to_string(),
                date: "2026-07-29 15:30".to_string(),
                message: "rix: installed ripgrep".to_string(),
                is_active: true,
            },
            HistoryEntry {
                state_num: 4,
                hash: "e4f5g6h".to_string(),
                date: "2026-07-29 14:15".to_string(),
                message: "rix: installed eza".to_string(),
                is_active: false,
            },
            HistoryEntry {
                state_num: 3,
                hash: "i7j8k9l".to_string(),
                date: "2026-07-29 13:00".to_string(),
                message: "rix: initialized environment".to_string(),
                is_active: false,
            },
        ];
    }
}

impl ScreenRenderer for HistoryScreen {
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
        let header = Paragraph::new("📜 Environment History")
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // History List
        if self.entries.is_empty() {
            let empty = Paragraph::new("No history available. Make changes to your environment to see history.")
                .block(Block::default().borders(Borders::ALL).title("History"))
                .alignment(Alignment::Center);
            f.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<Line> = self
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let is_selected = i == self.selected;
                    let state_marker = if entry.is_active { "●" } else { "○" };

                    let style = if is_selected {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };

                    Line::from(vec![
                        Span::styled(
                            format!("{} State {}: ", state_marker, entry.state_num),
                            style.bold(),
                        ),
                        Span::styled(entry.message.clone(), style),
                        Span::raw("\n"),
                        Span::styled(
                            format!(
                                "     {} ({})",
                                entry.hash, entry.date
                            ),
                            if is_selected {
                                Style::default().fg(Color::Gray).italic()
                            } else {
                                Style::default().fg(Color::DarkGray).italic()
                            },
                        ),
                    ])
                })
                .collect();

            let history = Paragraph::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("History")
                        .padding(Padding::new(2, 2, 1, 1))
                );

            f.render_widget(history, chunks[1]);
        }

        // Footer
        let footer = Paragraph::new("↑/↓: Navigate | r: Rollback to state | q: Back")
            .style(Style::default().fg(Color::DarkGray).italic())
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }

    fn handle_input(&mut self, key: KeyCode, current_screen: &mut Screen) {
        match key {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else if !self.entries.is_empty() {
                    self.selected = self.entries.len() - 1;
                }
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1) % self.entries.len().max(1);
            }
            KeyCode::Char('r') => {
                if self.selected < self.entries.len() {
                    // Rollback logic here
                }
            }
            KeyCode::Backspace | KeyCode::Char('q') => {
                *current_screen = Screen::Menu;
            }
            _ => {}
        }
    }
}
