use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::Frame;

pub mod chat_container;
pub mod main_container;

pub trait Container {
    fn update(&mut self);
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect);
    fn handle_event(&mut self, input_key: Key);
}
