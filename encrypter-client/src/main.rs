#[macro_use]
extern crate log;

use simplelog::*;

use crate::events::{Event, Events};
use chat::Chat;
use encrypter_core::Protocol;
use encrypter_core::Result;
use std::fs::File;
use std::io::Write;
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

mod chat;
mod events;
mod network;
mod ui;

const DEFAULT_ROUTE: Route = Route {
    id: RouteId::StartScreen,
    active_block: ActiveBlock::Id,
    hovered_block: ActiveBlock::Id,
};

pub struct App {
    id: String,
    server_addr: String,
    current_chat_index: Option<usize>,
    chats: Vec<(String, Chat)>,
    navigation_stack: Vec<Route>,
    input_cursor_pos: u16,
    cursor_vertical_offset: u16,
    message_draft: String,
    connection: Option<network::ServerConnection>,
}

impl App {
    fn new() -> Self {
        App {
            navigation_stack: vec![DEFAULT_ROUTE],
            cursor_vertical_offset: 4,
            id: String::new(),
            connection: None,
            current_chat_index: None,
            message_draft: String::new(),
            chats: Vec::new(),
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

    pub(crate) fn get_current_chat(&mut self) -> Option<&mut Chat> {
        if let Some(index) = self.current_chat_index {
            Some(&mut self.chats[index].1)
        } else {
            None
        }
    }

    pub(crate) fn get_chat_for(&mut self, contact: &str) -> Option<&mut Chat> {
        self.chats
            .iter_mut()
            .find(|(user, _chat)| user == contact)
            .map(|(_, chat)| chat)
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

fn main() -> Result<()> {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("client_logs.log").expect("Can't create log file"),
    );

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
            if let Some(protocol_message) = connection.step()? {
                match protocol_message {
                    Protocol::Message(encrypted_incoming) => {
                        let (from, _to) = encrypted_incoming.get_info();
                        if let Some(chat) = app.get_chat_for(from) {
                            let incoming = encrypted_incoming.decrypt_message(&chat.shared_key);
                            if let Some(chat) = app.get_chat_for(&incoming.from) {
                                chat.messages.push(format!(
                                    "{}: {}",
                                    incoming.from,
                                    String::from_utf8_lossy(&incoming.content)
                                ));
                            }
                        } else {
                            error!("Missing decryption key from {}", from);
                            todo!("Handle this error in a better way");
                        }
                    }
                    Protocol::PeerList(peers) => {
                        info!("Received peerlist of length {}", peers.len());
                        app.chats = peers
                            .into_iter()
                            .filter(|(peer_id, _)| peer_id != &app.id)
                            .map(|(peer_id, public_key_buffer)| {
                                (peer_id, Chat::new(public_key_buffer))
                            })
                            .collect::<Vec<(String, Chat)>>();
                    }
                    Protocol::Disconnect(id) => {
                        info!("Received disconnect for: {}", id);
                        if let Some(index) =
                            app.chats.iter().position(|(peer_id, _)| peer_id == &id)
                        {
                            if let Some(current_index) = app.current_chat_index {
                                if index == current_index {
                                    app.current_chat_index = None;
                                }
                            }
                            info!("Removed a chat!");
                            app.chats.remove(index);
                        }
                    }
                    _ => {}
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
