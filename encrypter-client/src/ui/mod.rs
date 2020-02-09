use crate::{ActiveBlock, App, RouteId};

use termion::event::Key;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Paragraph, SelectableList, Text, Widget};
use tui::Frame;

pub mod command_line;

pub fn get_color((is_active, is_hovered): (bool, bool)) -> Style {
    match (is_active, is_hovered) {
        (true, _) => Style::default().fg(Color::LightCyan),
        (false, true) => Style::default().fg(Color::Magenta),
        _ => Style::default().fg(Color::Gray),
    }
}

pub trait StatefulWidget {
    fn draw<B>(&self, frame: &mut Frame<B>, layout_chunk: Rect)
    where
        B: Backend;
    fn handle_event(&mut self, input_key: Key);
}

pub fn draw_main_screen<B>(frame: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(95), Constraint::Max(2)].as_ref())
        .split(frame.size());
    draw_chat_content(frame, app, chunks[0]);
    app.command_line.draw(frame, chunks[1]);
}

pub fn draw_chat_content<B>(frame: &mut Frame<B>, app: &mut App, layout_chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(layout_chunk);

    draw_chat_list(frame, app, chunks[0]);

    match app.get_current_route().id {
        RouteId::StartScreen => {} // full screen route drawn i main
        RouteId::Chat => {
            draw_chat_window(frame, app, chunks[1]);
        }
    };
}

pub fn draw_chat_window<B>(frame: &mut Frame<B>, app: &mut App, layout_chunk: Rect)
where
    B: Backend,
{
    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::ChatWindow,
        current_route.hovered_block == ActiveBlock::ChatWindow,
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Max(3)].as_ref())
        .split(layout_chunk);

    let textbox_message = if app.current_chat_index.is_some() {
        "Type message..."
    } else {
        "You need to select someone from the chat list before writing a message!"
    };

    if let Some(chat) = app.get_current_chat() {
        List::new(chat.messages.iter().map(Text::raw))
            .block(
                Block::default()
                    .title_style(get_color(highlight_state))
                    .border_style(get_color(highlight_state))
                    .borders(Borders::ALL)
                    .title("Messages"),
            )
            .render(frame, chunks[0]);
    }
    Paragraph::new([Text::raw(&app.message_draft)].iter())
        .block(
            Block::default()
                .title_style(get_color(highlight_state))
                .border_style(get_color(highlight_state))
                .borders(Borders::ALL)
                .title(textbox_message),
        )
        .render(frame, chunks[1]);
}

pub fn draw_chat_list<B>(frame: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::ChatList,
        current_route.hovered_block == ActiveBlock::ChatList,
    );
    let contacts = app
        .chats
        .iter()
        .map(|(user, _)| user)
        .collect::<Vec<&String>>();
    SelectableList::default()
        .block(
            Block::default()
                .title("Chats:")
                .borders(Borders::ALL)
                .title_style(get_color(highlight_state))
                .border_style(get_color(highlight_state)),
        )
        .items(contacts.as_slice())
        .style(Style::default().fg(Color::White))
        .select(app.current_chat_index)
        .highlight_style(get_color(highlight_state).modifier(Modifier::REVERSED))
        .render(frame, layout_chunk);
}

pub fn draw_start_screen<B>(frame: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let current_route = app.get_current_route();
    let highlight_id_state = (
        current_route.active_block == ActiveBlock::Id,
        current_route.hovered_block == ActiveBlock::Id,
    );

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
                .title_style(get_color(highlight_id_state))
                .border_style(get_color(highlight_id_state))
                .borders(Borders::ALL)
                .title("Id"),
        )
        .render(frame, chunks[0]);
}
