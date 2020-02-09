use crate::ui::containers::chat_container::ChatState;
use crate::ui::get_color;
use crate::ui::Component;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, SelectableList, Widget};
use tui::Frame;

pub struct ChatList {
    is_active: bool,
    is_hovered: bool,
}

impl ChatList {
    pub fn new() -> Self {
        ChatList {
            is_active: false,
            is_hovered: false,
        }
    }
}

impl Component<ChatState> for ChatList {
    fn is_active(&self) -> bool {
        self.is_active
    }
    fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    fn set_hovered(&mut self, hoverd: bool) {
        self.is_hovered = hoverd;
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn handle_event(&mut self, input_key: Key, state: &mut ChatState) {
        match input_key {
            Key::Up => {
                if let Some(index) = state.current_chat_index {
                    if index > 0 {
                        state.current_chat_index = Some(index - 1);
                    }
                }
            }
            Key::Down => {
                if let Some(index) = state.current_chat_index {
                    if index < state.chats.len() - 1 {
                        state.current_chat_index = Some(index + 1);
                    }
                }
            }
            Key::Char('\n') => {
                if !state.chats.is_empty() {
                    state.current_chat_index = Some(0);
                }
            }
            _ => {}
        }
    }
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect, state: &ChatState) {
        let contacts = state
            .chats
            .iter()
            .map(|(user, _)| user)
            .collect::<Vec<&String>>();
        SelectableList::default()
            .block(
                Block::default()
                    .title("Chats:")
                    .borders(Borders::ALL)
                    .title_style(get_color((self.is_active, self.is_hovered)))
                    .border_style(get_color((self.is_active, self.is_hovered))),
            )
            .items(contacts.as_slice())
            .style(Style::default().fg(Color::White))
            .select(state.current_chat_index)
            .highlight_style(
                get_color((self.is_active, self.is_hovered)).modifier(Modifier::REVERSED),
            )
            .render(frame, layout_chunk);
    }
}
