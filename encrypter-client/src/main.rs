use std::io::{self, Write};

use async_std::io::{stdin, BufReader};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;
use futures::FutureExt;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Paragraph, SelectableList, Text, Widget};
use tui::{Frame, Terminal};


use crate::events::{Event, Events};
use std::env::current_exe;
use std::sync::atomic::Ordering::AcqRel;

mod events;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const DEFAULT_ROUTE: Route = Route {
    id: RouteId::StartScreen,
    active_block: ActiveBlock::Id,
    hovered_block: ActiveBlock::Id,
};

pub fn get_color((is_active, is_hovered): (bool, bool)) -> Style {
    match (is_active, is_hovered) {
        (true, _) => Style::default().fg(Color::LightCyan),
        (false, true) => Style::default().fg(Color::Magenta),
        _ => Style::default().fg(Color::Gray),
    }
}

#[derive(Default, Debug)]
pub struct App {
    id: String,
    server_addr: String,
    messages: Vec<String>,
    navigation_stack: Vec<Route>,
    input_cursor_pos: u16,
    cursor_vertical_offset: u16,
    cursor_horizontal_offset: u16,
    message_draft: String,
}

impl App {
    fn new() -> Self {
        App {
            navigation_stack: vec![DEFAULT_ROUTE],
            cursor_vertical_offset: 4,
            cursor_horizontal_offset: 4,
            ..App::default()
        }
    }

    fn get_current_route(&self) -> &Route {
        match self.navigation_stack.last() {
            Some(route) => route,
            None => &DEFAULT_ROUTE,
        }
    }

    fn get_current_route_mut(&mut self) -> &mut Route {
        self.navigation_stack.last_mut().unwrap()
    }

    pub fn push_route(&mut self, route: Route) {
        self.navigation_stack.push(route);
    }

    pub fn set_current_route_state(
        &mut self,
        active_block: Option<ActiveBlock>,
        hovered_block: Option<ActiveBlock>,
    ) {
        let mut current_route = self.get_current_route_mut();
        if let Some(active_block) = active_block {
            current_route.active_block = active_block;
        }
        if let Some(hovered_block) = hovered_block {
            current_route.hovered_block = hovered_block;
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RouteId {
    StartScreen,
    Chat,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActiveBlock {
    Empty,
    Id,
    ServerAddr,
    ChatWindow,
    ChatList,
}

#[derive(Debug)]
pub struct Route {
    pub id: RouteId,
    pub active_block: ActiveBlock,
    pub hovered_block: ActiveBlock,
}

pub fn draw_routes<B>(frame: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(frame.size());

    draw_chat_list(frame, app, chunks[0]);

    match app.get_current_route().id {
        RouteId::StartScreen => {} // full screen route drawn i main
        RouteId::Chat => {
            draw_chat_window(frame, app, chunks[1]);
        }
    };
}

fn draw_chat_window<B>(frame: &mut Frame<B>, app: &App, layout_chunk: Rect)
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
        .margin(2)
        .constraints([Constraint::Percentage(90), Constraint::Min(3)].as_ref())
        .split(layout_chunk);

    let messages = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| Text::raw(format!("{}: {}", i, m)));

    List::new(messages)
        .block(
            Block::default()
                .title_style(get_color(highlight_state))
                .border_style(get_color(highlight_state))
                .borders(Borders::ALL)
                .title("Messages"),
        )
        .render(frame, chunks[0]);
    Paragraph::new([Text::raw(&app.message_draft)].iter())
        .block(
            Block::default()
                .title_style(get_color(highlight_state))
                .border_style(get_color(highlight_state))
                .borders(Borders::ALL)
                .title("Type Message"),
        )
        .render(frame, chunks[1]);
}

fn draw_chat_list<B>(frame: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::ChatList,
        current_route.hovered_block == ActiveBlock::ChatList,
    );

    SelectableList::default()
        .block(
            Block::default()
                .title("Chats:")
                .borders(Borders::ALL)
                .title_style(get_color(highlight_state))
                .border_style(get_color(highlight_state)),
        )
        .items(vec!["Kalle kule", "Bertil Hulgesson", "Hubert Snubert", "Aleks"].as_slice())
        .style(Style::default().fg(Color::White))
        .select(Some(0))
        .highlight_style(get_color(highlight_state).modifier(Modifier::BOLD))
        .render(frame, layout_chunk);
}

fn draw_start_screen<B>(frame: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let current_route = app.get_current_route();
    let highlight_id_state = (
        current_route.active_block == ActiveBlock::Id,
        current_route.hovered_block == ActiveBlock::Id,
    );
    let highlight_server_state = (
        current_route.active_block == ActiveBlock::ServerAddr,
        current_route.hovered_block == ActiveBlock::ServerAddr,
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
    Paragraph::new([Text::raw(&app.server_addr)].iter())
        .block(
            Block::default()
                .title_style(get_color(highlight_server_state))
                .border_style(get_color(highlight_server_state))
                .borders(Borders::ALL)
                .title("Server addr"),
        )
        .render(frame, chunks[1]);
}

fn main() -> Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let events = Events::new();

    // Create default app state
    let mut app = App::new();
    loop {
        let current_route = app.get_current_route();
        terminal.draw(|mut f| match current_route.id {
            RouteId::StartScreen => {
                draw_start_screen(&mut f, &app);
            }
            RouteId::Chat => {
                draw_routes(&mut f, &app);
            }
        })?;

        if current_route.id == RouteId::StartScreen {
            terminal.show_cursor().unwrap();
        } else {
            terminal.hide_cursor().unwrap();
        }

        // Put the cursor back inside the input box
        write!(
            terminal.backend_mut(),
            "{}",
            Goto(4 + app.input_cursor_pos, app.cursor_vertical_offset)
        )?;
        // stdout is buffered, flush it to see the effect immediately when hitting backspace
        io::stdout().flush().ok();

        // Handle input
        match events.next()? {
            Event::Input(input) => match input {
                Key::Ctrl('c') => {
                    break;
                }
                _ => {
                    handle_block_events(input, &mut app);
                }
            },
            _ => {}
        }
    }

    Ok(())
}

/*
Input struktur:
1. kolla universiella kommandon (görs i main nu)
2. Kör den här metoden som väljer vilken handler
3. Varje handler tar hand om sin egna navigering men logiken kan extraheras ut till generell metod
typ som handle right och handle left
*/
fn handle_block_events(input: Key, app: &mut App) {
    match app.get_current_route().active_block {
        ActiveBlock::Id => {
            id_handler(input, app);
        }
        ActiveBlock::ServerAddr => {
            server_handler(input, app);
        }
        ActiveBlock::ChatList => {
            chat_list_handler(input, app);
        }
        ActiveBlock::ChatWindow => {
            chat_window_handler(input, app);
        }
        ActiveBlock::Empty => match input {
            Key::Char('\n') => {
                app.set_current_route_state(
                    Some(app.get_current_route().hovered_block.clone()),
                    None,
                );
            }
            Key::Right => {
                handle_right_event(app);
            }
            Key::Left => {
                handle_left_event(app);
            }
            _ => {}
        },
    }
}

fn handle_right_event(app: &mut App) {
    let current_route = app.get_current_route();
    match current_route.hovered_block {
        ActiveBlock::ChatList => app.set_current_route_state(None, Some(ActiveBlock::ChatWindow)),
        _ => {}
    }
}

fn handle_left_event(app: &mut App) {
    let current_route = app.get_current_route();
    match current_route.hovered_block {
        ActiveBlock::ChatWindow => app.set_current_route_state(None, Some(ActiveBlock::ChatList)),
        _ => {}
    }
}

// ha mer generell struktur
fn chat_list_handler(input: Key, app: &mut App) {
    match input {
        Key::Right => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::ChatWindow));
        }
        _ => {}
    }
}

fn chat_window_handler(input: Key, app: &mut App) {
    //app.cursor_vertical_offset = 25;
    //app.cursor_horizontal_offset = 70;
    match input {
        Key::Left => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::ChatList));
        }
        Key::Char('\n') => {
            app.messages.push(app.message_draft.drain(..).collect());
            // send message
        }
        Key::Char(c) => {
            app.message_draft.push(c);
        }
        Key::Backspace => {
            app.message_draft.pop();
        }
        _ => {}
    }
}

fn id_handler(input: Key, app: &mut App) {
    match input {
        Key::Char('\n') => {
            app.messages.push(app.id.clone());
            app.set_current_route_state(
                Some(ActiveBlock::ServerAddr),
                Some(ActiveBlock::ServerAddr),
            );
            app.cursor_vertical_offset = 7;
        }
        Key::Char(c) => {
            app.id.push(c);
        }
        Key::Backspace => {
            app.id.pop();
        }
        _ => {}
    }
    app.input_cursor_pos = app.id.len() as u16;
}

fn server_handler(input: Key, app: &mut App) {
    match input {
        Key::Char('\n') => {
            app.messages.push(app.server_addr.clone());
            app.push_route(Route {
                id: RouteId::Chat,
                hovered_block: ActiveBlock::ChatList,
                active_block: ActiveBlock::ChatList,
            });
            //app.cursor_horizontal_offset = 6;
            //app.cursor_vertical_offset = 50;
        }
        Key::Char(c) => {
            app.server_addr.push(c);
        }
        Key::Backspace => {
            app.server_addr.pop();
        }
        _ => {}
    }
    app.input_cursor_pos = app.server_addr.len() as u16;
}

async fn start_async() -> Result<()> {
    /*let server_addr = matches.value_of("encrypter-server-connection").unwrap_or("127.0.0.1:1337");
    let id = matches.value_of("id").unwrap_or("anon");
    let stream = TcpStream::connect(server_addr).await?;
    let (reader, mut writer) = (&stream, &stream);
    let hello_msg = IncomingPeer { id: id.into() };
    writer.write_all(&bincode::serialize(&hello_msg).unwrap()).await?;
    println!("reader {:?}", reader);
    // skicka runt channels istället för stream delarna
    task::spawn(handle_incomming_messages(sender));

    let stdin = BufReader::new(stdin());
    let mut lines_from_stdin = futures::StreamExt::fuse(stdin.lines());*/
    Ok(())
}
