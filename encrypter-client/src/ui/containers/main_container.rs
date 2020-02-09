use crate::network::ServerConnection;
use crate::ui::components::command_line::CommandLine;
use crate::ui::containers::chat_container::ChatContainer;
use crate::ui::Component;
use crate::ui::Container;
use termion::event::Key;
use tui::backend::Backend;

use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::Frame;
#[derive(Default)]
pub struct MainState;

// TODO: Add frame to the containers and add second lifetime
pub struct MainContainer {
    command_line: Box<CommandLine>,
    chat_container: Box<ChatContainer>,
    state: MainState,
}

impl MainContainer {
    pub fn new(id: String, connection: ServerConnection) -> Self {
        MainContainer {
            chat_container: Box::new(ChatContainer::new(id, connection)),
            command_line: Box::new(CommandLine::new()),
            state: MainState::default(),
        }
    }
}
impl Container for MainContainer {
    fn handle_event(&mut self, input_key: Key) {
        match input_key {
            Key::Esc => {
                self.command_line.set_active(true);
                //TODO: Make other container disabled here
            }
            _ => {
                if !self.command_line.is_active() {
                    self.chat_container.handle_event(input_key);
                } else {
                    self.command_line.handle_event(input_key, &mut self.state);
                }
            }
        }
    }
    fn update(&mut self) {
        self.chat_container.update();
    }
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, _: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(95), Constraint::Max(2)].as_ref())
            .split(frame.size());
        self.chat_container.draw(frame, chunks[0]);
        self.command_line.draw(frame, chunks[1], &self.state);
    }
}
