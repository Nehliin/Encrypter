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

pub trait Container {
    fn update(&mut self);
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect);
    fn handle_event(&mut self, input_key: Key);
}

pub trait Component<S> {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect, state: &S);

    fn handle_event(&mut self, input_key: Key, state: &mut S);

    fn set_active(&mut self, active: bool);

    fn set_hovered(&mut self, hoverd: bool);

    fn is_active(&self) -> bool;
    fn is_hovered(&self) -> bool;
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
