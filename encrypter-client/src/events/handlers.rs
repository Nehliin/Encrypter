use crate::{network::ServerConnection, App};

use termion::event::Key;

pub fn id_handler(input: Key, app: &mut App) {
    match input {
        Key::Char('\n') => match ServerConnection::new("127.0.0.1:1337", app.id.clone()) {
            Ok(connection) => {
                app.connection = Some(connection);
            }
            Err(err) => {
                error!("Couldn't connect to server {:#?}", err);
            }
        },
        Key::Char(c) => {
            if app.id.len() < encrypter_core::ID_MAX_SIZE {
                app.id.push(c);
            }
        }
        Key::Backspace => {
            app.id.pop();
        }
        _ => {}
    }
    app.input_cursor_pos = app.id.len() as u16;
}
