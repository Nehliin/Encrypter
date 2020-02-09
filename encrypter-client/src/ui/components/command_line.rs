use crate::ui::containers::main_container::MainState;
use crate::ui::Component;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::Widget;
use tui::Frame;

use tui::widgets::{Block, Paragraph, Text};
#[derive(PartialEq)]
enum DisplayMode {
    Input,
    Error,
    Info,
    Warn,
    Default,
}

pub struct CommandLine {
    content: String,
    display_mode: DisplayMode,
    is_active: bool,
    is_hovered: bool,
}

impl CommandLine {
    pub fn new() -> Self {
        CommandLine {
            content: String::from("Commandline"),
            display_mode: DisplayMode::Default,
            is_active: false,
            is_hovered: false,
        }
    }

    fn get_style(&self) -> (Style, Style) {
        match &self.display_mode {
            DisplayMode::Default => {
                let block_style = Style::default();
                let text_style = block_style.fg(Color::Black);
                (text_style, block_style)
            }
            DisplayMode::Input => {
                let block_style = Style::default().bg(Color::White);
                let text_style = block_style.fg(Color::Black);
                (text_style, block_style)
            }

            DisplayMode::Error => {
                let block_style = Style::default().bg(Color::Red);
                let text_style = block_style.fg(Color::Black).modifier(Modifier::BOLD);
                (text_style, block_style)
            }
            DisplayMode::Info => {
                let block_style = Style::default().bg(Color::Green);
                let text_style = block_style.fg(Color::White).modifier(Modifier::BOLD);
                (text_style, block_style)
            }
            DisplayMode::Warn => {
                let block_style = Style::default().bg(Color::Yellow);
                let text_style = block_style.fg(Color::Black).modifier(Modifier::BOLD);
                (text_style, block_style)
            }
        }
    }

    fn update_state(&mut self, content: String, mode: DisplayMode) {
        if self.display_mode != DisplayMode::Input {
            self.content = content;
            self.display_mode = mode;
        }
    }

    fn handle_command(&self, command: String) {
        info!("A command was sent! {}", command);
    }

    pub fn show_error<S: AsRef<str>>(&mut self, error: S) {
        self.update_state(format!("[ERROR]: {}", error.as_ref()), DisplayMode::Error);
    }

    pub fn show_info_message<S: AsRef<str>>(&mut self, message: S) {
        self.update_state(format!("[INFO]: {}", message.as_ref()), DisplayMode::Info);
    }

    pub fn show_warning<S: AsRef<str>>(&mut self, warning: S) {
        self.update_state(format!("[WARN]: {}", warning.as_ref()), DisplayMode::Warn);
    }
}

impl Component<MainState> for CommandLine {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, layout_chunk: Rect, state: &MainState) {
        let (text_style, block_style) = self.get_style();
        let text = Text::styled(&self.content, text_style);
        Paragraph::new([text].iter())
            .block(Block::default())
            .style(block_style)
            .render(frame, layout_chunk);
    }

    fn handle_event(&mut self, input_key: Key, state: &mut MainState) {
        self.display_mode = DisplayMode::Input;
        match input_key {
            Key::Char(':') => {
                self.content.clear();
                self.content.push(':');
            }
            Key::Char('\n') => {
                // skip first ':' sign
                let command = self.content.drain(1..).collect::<String>();
                self.content.pop();
                self.handle_command(command);
                self.display_mode = DisplayMode::Default;
            }
            Key::Char(c) => {
                self.content.push(c);
            }
            Key::Backspace => {
                self.content.pop();
            }
            _ => {}
        }
    }
    fn is_hovered(&self) -> bool {
        self.is_hovered
    }
    fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }
    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
    fn is_active(&self) -> bool {
        self.is_active
    }
}
