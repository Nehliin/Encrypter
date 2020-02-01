use crate::{network::ServerConnection, ActiveBlock, App, Route, RouteId};

use encrypter_core::{EncryptedMessage, Message, Protocol};

use termion::event::Key;

/*
Input struktur:
1. kolla universiella kommandon (görs i main nu)
2. Kör den här metoden som väljer vilken handler
3. Varje handler tar hand om sin egna navigering men logiken kan extraheras ut till generell metod
typ som handle right och handle left
*/
pub fn handle_block_events(input: Key, app: &mut App) {
    match app.get_current_route().active_block {
        ActiveBlock::Id => {
            id_handler(input, app);
        }
        /* ActiveBlock::ServerAddr => {
            server_handler(input, app);
        }*/
        ActiveBlock::ChatList => {
            chat_list_handler(input, app);
        }
        ActiveBlock::ChatWindow => {
            chat_window_handler(input, app);
        }
        ActiveBlock::Empty => match input {
            Key::Char('\n') => {
                app.set_current_route_state(
                    Some(app.get_current_route().hovered_block.clone()),
                    None,
                );
            }
            Key::Right => {
                handle_right_event(app);
            }
            Key::Left => {
                handle_left_event(app);
            }
            _ => {}
        },
    }
}

pub fn handle_right_event(app: &mut App) {
    let current_route = app.get_current_route();
    if let ActiveBlock::ChatList = current_route.hovered_block {
        app.set_current_route_state(None, Some(ActiveBlock::ChatWindow));
    }
}

pub fn handle_left_event(app: &mut App) {
    let current_route = app.get_current_route();
    if let ActiveBlock::ChatWindow = current_route.hovered_block {
        app.set_current_route_state(None, Some(ActiveBlock::ChatList));
    }
}

// ha mer generell struktur
pub fn chat_list_handler(input: Key, app: &mut App) {
    match input {
        Key::Right => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::ChatWindow));
        }
        Key::Up => {
            if let Some(index) = app.current_chat_index {
                if index > 0 {
                    app.current_chat_index = Some(index - 1);
                }
            }
        }
        Key::Down => {
            if let Some(index) = app.current_chat_index {
                if index < app.chats.len() - 1 {
                    app.current_chat_index = Some(index + 1);
                }
            }
        }
        Key::Char('\n') => {
            if !app.chats.is_empty() {
                app.current_chat_index = Some(0);
            }
        }
        _ => {}
    }
}

pub fn chat_window_handler(input: Key, app: &mut App) {
    if input == Key::Left {
        app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::ChatList));
    } else if app.current_chat_index.is_some() {
        match input {
            Key::Char('\n') => {
                let message = app.message_draft.drain(..).collect::<String>();
                let message = Message {
                    from: app.id.clone(),
                    to: app.chats[app.current_chat_index.unwrap()].0.clone(), // Safe because of previous if let
                    content: message.as_bytes().to_vec(),
                };
                if let Some(current_chat) = app.get_current_chat() {
                    current_chat
                        .messages
                        .push(format!("Me: {}", String::from_utf8_lossy(&message.content)));
                    let encrypted_message =
                        EncryptedMessage::create(message, &current_chat.shared_key);
                    app.connection
                        .as_ref()
                        .unwrap()
                        .send(Protocol::Message(encrypted_message))
                        .expect("Failed to send message");
                }
            }
            Key::Char(c) => {
                if app.message_draft.len() < encrypter_core::MESSAGE_MAX_SIZE {
                    app.message_draft.push(c);
                }
            }
            Key::Backspace => {
                app.message_draft.pop();
            }
            _ => {}
        }
    }
}

pub fn id_handler(input: Key, app: &mut App) {
    match input {
        Key::Char('\n') => {
            match ServerConnection::new(&app.server_addr, app.id.clone()) {
                Ok(connection) => {
                    app.connection = Some(connection);
                }
                Err(err) => {
                    error!("Couldn't connect to server {:#?}", err);
                }
            }
            app.push_route(Route {
                id: RouteId::Chat,
                hovered_block: ActiveBlock::ChatList,
                active_block: ActiveBlock::ChatList,
            });
        }
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
