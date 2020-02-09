use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::Frame;

pub mod chat_list;
pub mod chat_window;
pub mod command_line;

pub trait Component<S> {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect, state: &S);

    fn handle_event(&mut self, input_key: Key, state: &mut S);

    fn set_active(&mut self, active: bool);

    fn set_hovered(&mut self, hoverd: bool);

    fn is_active(&self) -> bool;
    fn is_hovered(&self) -> bool;
}
