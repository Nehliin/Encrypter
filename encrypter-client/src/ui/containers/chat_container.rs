use crate::chat::Chat;
use crate::network::ServerConnection;
use crate::ui::components::chat_list::ChatList;
use crate::ui::components::chat_window::ChatWindow;
use crate::ui::components::Component;
use crate::ui::containers::Container;
use encrypter_core::Protocol;
use encrypter_core::Result;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::Frame;

pub struct ChatState {
    pub current_chat_index: Option<usize>,
    pub(crate) chats: Vec<(String, Chat)>,
    pub message_draft: String,
    pub connection: Option<crate::network::ServerConnection>,
    pub id: String,
}

impl ChatState {
    pub fn new(id: String, connection: ServerConnection) -> Self {
        ChatState {
            id,
            chats: Vec::new(),
            message_draft: String::new(),
            connection: Some(connection),
            current_chat_index: None,
        }
    }
    pub(crate) fn get_current_chat_mut(&mut self) -> Option<&mut Chat> {
        if let Some(index) = self.current_chat_index {
            self.chats.get_mut(index).map(|(_, chat)| chat)
        } else {
            None
        }
    }
    pub(crate) fn get_current_chat(&self) -> Option<&Chat> {
        if let Some(index) = self.current_chat_index {
            self.chats.get(index).map(|(_, chat)| chat)
        } else {
            None
        }
    }
    pub(crate) fn get_chat_for(&mut self, contact: &str) -> Option<&mut Chat> {
        self.chats
            .iter_mut()
            .find(|(user, _)| user == contact)
            .map(|(_, chat)| chat)
    }
    fn network_update(&mut self) -> Result<()> {
        if let Some(ref mut connection) = self.connection {
            if let Some(protocol_message) = connection.step()? {
                match protocol_message {
                    Protocol::Message(encrypted_incoming) => {
                        let (from, _to) = encrypted_incoming.get_info();
                        if let Some(chat) = self.get_chat_for(from) {
                            let incoming = encrypted_incoming.decrypt_message(&chat.shared_key);
                            if let Some(chat) = self.get_chat_for(&incoming.from) {
                                chat.messages.push(format!(
                                    "{}: {}",
                                    incoming.from,
                                    String::from_utf8_lossy(&incoming.content)
                                ));
                            }
                        } else {
                            error!(
                                "Missing decryption key and/or peer in chatlist, peer: {}",
                                from
                            );
                            //   self
                            //         .command_line
                            //           .show_error(format!("Received message from unknown peer {}", from));
                            todo!("Handle this error in a better way");
                        }
                    }
                    Protocol::PeerList(peers) => {
                        info!("Received peerlist of length {}", peers.len());
                        //   self.command_line.show_info_message("Received peerlist");
                        self.chats = peers
                            .into_iter()
                            .filter(|(peer_id, _)| peer_id != &self.id)
                            .map(|(peer_id, public_key_buffer)| {
                                (peer_id, Chat::new(public_key_buffer))
                            })
                            .collect::<Vec<(String, Chat)>>();
                    }
                    Protocol::Disconnect(id) => {
                        let log = format!("Received disconnect for: {}", id);
                        info!("{}", log);
                        //   self.command_line.show_info_message(log);
                        if let Some(index) =
                            self.chats.iter().position(|(peer_id, _)| peer_id == &id)
                        {
                            if let Some(current_index) = self.current_chat_index {
                                if index == current_index {
                                    self.current_chat_index = None;
                                }
                            }
                            info!("Removed chat for {}", id);
                            self.chats.remove(index);
                        }
                    }
                    Protocol::NewConnection(id, public_key) => {
                        info!("Received connection to new peer: {}", id);
                        //self
                        //     .command_line
                        //       .show_info_message(format!("New connection to: {}", id));
                        if let Some((_, chat)) =
                            self.chats.iter_mut().find(|(old_id, _)| old_id == &id)
                        {
                            warn!("Peer already in chat list, updating public_key");
                            chat.change_key(public_key);
                        } else {
                            info!("Adding peer {} to chat list", id);
                            self.chats.push((id, Chat::new(public_key)));
                        }
                    }
                    Protocol::ConnectionLost => {
                        //               self.command_line.show_error("Lost server connection!")
                    }
                    unknown_message => {
                        //             self
                        //               .command_line
                        //             .show_warning("Received a message client can't handle");
                        warn!(
                            "Received a message client can't handle: {:?}",
                            unknown_message
                        )
                    }
                }
            }
        }
        Ok(())
    }
}

// TODO: Add frame to the containers and add second lifetime
pub struct ChatContainer {
    chat_window: Box<ChatWindow>,
    chat_list: Box<ChatList>,
    state: ChatState,
}

impl ChatContainer {
    pub fn new(id: String, connection: ServerConnection) -> Self {
        let mut chat_list = Box::new(ChatList::new());
        chat_list.set_hovered(true);
        ChatContainer {
            chat_window: Box::new(ChatWindow::new()),
            chat_list: Box::new(ChatList::new()),
            state: ChatState::new(id, connection),
        }
    }
}
impl Container for ChatContainer {
    fn handle_event(&mut self, input_key: Key) {
        match input_key {
            Key::Left => {
                self.chat_window.set_active(false);
                self.chat_window.set_hovered(false);
                self.chat_list.set_hovered(true);
            }
            Key::Right => {
                self.chat_window.set_hovered(true);
                self.chat_list.set_active(false);
                self.chat_list.set_hovered(false);
            }
            // fult som fan
            _ => {
                if self.chat_list.is_active() {
                    self.chat_list.handle_event(input_key, &mut self.state);
                } else if self.chat_window.is_active() {
                    self.chat_window.handle_event(input_key, &mut self.state)
                } else if input_key == Key::Char('\n') {
                    if self.chat_list.is_hovered() && !self.chat_list.is_active() {
                        self.chat_list.set_active(true);
                    } else if self.chat_window.is_hovered() && !self.chat_window.is_active() {
                        self.chat_window.set_active(true);
                    }
                    self.handle_event(input_key);
                }
            }
        }
    }

    fn update(&mut self) {
        self.state.network_update().unwrap();
    }

    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(layout_chunk);

        self.chat_list.draw(frame, chunks[0], &self.state);
        self.chat_window.draw(frame, chunks[1], &self.state);
    }
}
