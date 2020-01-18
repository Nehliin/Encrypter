use crate::events::{Event, Events};
use encrypter_core::Result;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

mod events;
mod network;
mod ui;

const DEFAULT_ROUTE: Route = Route {
    id: RouteId::StartScreen,
    active_block: ActiveBlock::Id,
    hovered_block: ActiveBlock::Id,
};

#[derive(Debug)]
pub struct App {
    id: String,
    server_addr: String,
    current_chat_index: usize,
    chats: Vec<(String, Vec<String>)>,
    navigation_stack: Vec<Route>,
    input_cursor_pos: u16,
    cursor_vertical_offset: u16,
    cursor_horizontal_offset: u16,
    message_draft: String,
    connection: Option<network::ServerConnection>,
}

impl App {
    fn new() -> Self {
        let chats: Vec<(String, Vec<String>)> = vec![
            ("Kalle".into(), Vec::new()),
            ("Bertil Hulgesson".into(), Vec::new()),
            ("Hubert Snubert".into(), Vec::new()),
            ("Aleks".into(), Vec::new()),
        ]
        .into_iter()
        .collect();
        App {
            navigation_stack: vec![DEFAULT_ROUTE],
            cursor_vertical_offset: 4,
            cursor_horizontal_offset: 4,
            id: String::new(),
            connection: None,
            current_chat_index: 0,
            message_draft: String::new(),
            chats,
            input_cursor_pos: 0,
            server_addr: String::from("127.0.0.1:1337"),
        }
    }

    fn get_current_route(&self) -> &Route {
        match self.navigation_stack.last() {
            Some(route) => route,
            None => &DEFAULT_ROUTE,
        }
    }

    pub fn get_current_chat(&mut self) -> &mut Vec<String> {
        &mut self.chats[self.current_chat_index].1
    }

    pub fn get_chat_for(&mut self, contact: &str) -> Option<&mut Vec<String>> {
        for (user, chat) in self.chats.iter_mut() {
            if contact == user {
                return Some(chat);
            }
        }
        None
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
    ChatWindow,
    ChatList,
}

#[derive(Debug)]
pub struct Route {
    pub id: RouteId,
    pub active_block: ActiveBlock,
    pub hovered_block: ActiveBlock,
}
use std::io::Write;
fn main() -> Result<()> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let events = Events::new();

    let mut app = App::new();
    loop {
        if let Some(ref mut connection) = app.connection {
            if let Some(incoming) = connection.step()? {
                if let Some(messages) = app.get_chat_for(&incoming.from) {
                    messages.push(format!("{}: {}", incoming.from, incoming.message));
                }
            }
        }
        terminal
            .draw(|mut f| match app.get_current_route().id {
                RouteId::StartScreen => {
                    ui::draw_start_screen(&mut f, &app);
                }
                RouteId::Chat => {
                    ui::draw_routes(&mut f, &mut app);
                }
            })
            .unwrap();

        if app.get_current_route().id == RouteId::StartScreen {
            terminal.show_cursor().unwrap();
        } else {
            terminal.hide_cursor().unwrap();
        }
        // Put the cursor back inside the input box
        write!(
            terminal.backend_mut(),
            "{}",
            Goto(4 + app.input_cursor_pos, app.cursor_vertical_offset)
        )
        .unwrap();
        // stdout is buffered, flush it to see the effect immediately when hitting backspace
        std::io::stdout().flush().ok();

        // Handle input
        if let Event::Input(input) = events.next().unwrap() {
            match input {
                Key::Ctrl('c') => {
                    break;
                }
                _ => {
                    events::handlers::handle_block_events(input, &mut app);
                }
            }
        }
    }
    //app.net_thread_scope.unwrap();
    Ok(())
}
