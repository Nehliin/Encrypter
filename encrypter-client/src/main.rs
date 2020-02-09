#[macro_use]
extern crate log;

use simplelog::*;

use crate::events::{Event, Events};
use crate::ui::containers::Container;
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

use crate::ui::containers::main_container::MainContainer;

mod chat;
mod events;
mod network;
mod ui;

pub struct App {
    id: String,
    input_cursor_pos: u16,
    cursor_vertical_offset: u16,
    main_container: Option<MainContainer>,
    connection: Option<network::ServerConnection>,
}

impl App {
    fn new() -> Self {
        App {
            cursor_vertical_offset: 4,
            id: String::new(),
            input_cursor_pos: 0,
            main_container: None,
            connection: None,
        }
    }
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
        if app.connection.is_some() {
            let connection = app.connection.take();
            app.main_container = Some(MainContainer::new(app.id.clone(), connection.unwrap()));
        }
        terminal
            .draw(|mut f| {
                if let Some(main_container) = app.main_container.as_mut() {
                    main_container.update();
                    let rect = f.size();
                    main_container.draw(&mut f, rect);
                } else {
                    ui::draw_start_screen(&mut f, &app);
                }
            })
            .unwrap();
        if app.main_container.is_none() {
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
                    if let Some(container) = app.main_container.as_mut() {
                        container.handle_event(input);
                    } else {
                        events::handlers::id_handler(input, &mut app);
                    }
                }
            }
        }
    }
    //app.net_thread_scope.unwrap();
    Ok(())
}
