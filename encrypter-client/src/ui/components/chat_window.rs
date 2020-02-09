use crate::ui::components::Component;
use crate::ui::containers::chat_container::ChatState;
use crate::ui::get_color;
use encrypter_core::{EncryptedMessage, Message, Protocol};
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, Borders, List, Paragraph, Text, Widget};
use tui::Frame;

pub struct ChatWindow {
    is_active: bool,
    is_hovered: bool,
}

impl ChatWindow {
    pub fn new() -> Self {
        ChatWindow {
            is_active: false,
            is_hovered: false,
        }
    }
}

impl Component<ChatState> for ChatWindow {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect, state: &ChatState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(90), Constraint::Max(3)].as_ref())
            .split(layout_chunk);

        let textbox_message = if state.current_chat_index.is_some() {
            "Type message..."
        } else {
            "You need to select someone from the chat list before writing a message!"
        };

        if let Some(chat) = state.get_current_chat() {
            List::new(chat.messages.iter().map(Text::raw))
                .block(
                    Block::default()
                        .title_style(get_color((self.is_active, self.is_hovered)))
                        .border_style(get_color((self.is_active, self.is_hovered)))
                        .borders(Borders::ALL)
                        .title("Messages"),
                )
                .render(frame, chunks[0]);
        }
        Paragraph::new([Text::raw(&state.message_draft)].iter())
            .block(
                Block::default()
                    .title_style(get_color((self.is_active, self.is_hovered)))
                    .border_style(get_color((self.is_active, self.is_hovered)))
                    .borders(Borders::ALL)
                    .title(textbox_message),
            )
            .render(frame, chunks[1]);
    }

    fn handle_event(&mut self, input_key: Key, state: &mut ChatState) {
        if state.current_chat_index.is_some() {
            match input_key {
                Key::Char('\n') => {
                    let message = state.message_draft.drain(..).collect::<String>();
                    let message = Message {
                        from: state.id.clone(),
                        to: state.chats[state.current_chat_index.unwrap()].0.clone(), // Safe because of previous if let
                        content: message.as_bytes().to_vec(),
                    };
                    if let Some(current_chat) = state.get_current_chat_mut() {
                        current_chat
                            .messages
                            .push(format!("Me: {}", String::from_utf8_lossy(&message.content)));
                        let encrypted_message =
                            EncryptedMessage::create(message, &current_chat.shared_key);
                        state
                            .connection
                            .as_ref()
                            .unwrap()
                            .send(Protocol::Message(encrypted_message))
                            .expect("Failed to send message");
                    }
                }
                Key::Char(c) => {
                    if state.message_draft.len() < encrypter_core::MESSAGE_MAX_SIZE {
                        state.message_draft.push(c);
                    }
                }
                Key::Backspace => {
                    state.message_draft.pop();
                }
                _ => {}
            }
        }
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn set_hovered(&mut self, hoverd: bool) {
        self.is_hovered = hoverd;
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
    fn is_hovered(&self) -> bool {
        self.is_hovered
    }
}
