use ratatui::{
    prelude::*,
    widgets::{Block, Borders, BorderType, Paragraph},
};

pub fn title_style() -> Style {
    Style::default().fg(Color::Cyan).bold()
}

pub fn error_style() -> Style {
    Style::default().fg(Color::Red).bold()
}

pub fn success_style() -> Style {
    Style::default().fg(Color::Green).bold()
}

pub fn warning_style() -> Style {
    Style::default().fg(Color::Yellow).bold()
}

pub fn selected_style() -> Style {
    Style::default().bg(Color::Blue).fg(Color::White).bold()
}

pub fn unselected_style() -> Style {
    Style::default().fg(Color::Gray)
}

pub fn create_bordered_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
}
