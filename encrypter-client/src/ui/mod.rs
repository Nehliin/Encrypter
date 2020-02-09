use crate::App;

use termion::event::Key;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Frame;

pub mod components;
pub mod containers;

pub fn get_color((is_active, is_hovered): (bool, bool)) -> Style {
    match (is_active, is_hovered) {
        (true, _) => Style::default().fg(Color::LightCyan),
        (false, true) => Style::default().fg(Color::Magenta),
        _ => Style::default().fg(Color::Gray),
    }
}

pub fn draw_start_screen<B>(frame: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
            ]
            .as_ref(),
        )
        .split(frame.size());

    Paragraph::new([Text::raw(&app.id)].iter())
        .block(
            Block::default()
                .title_style(get_color((true, true)))
                .border_style(get_color((true, true)))
                .borders(Borders::ALL)
                .title("Id"),
        )
        .render(frame, chunks[0]);
}
