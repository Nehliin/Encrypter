use crate::{connect_to_server, ActiveBlock, App, Route, RouteId};
use async_std::task;
use encrypter_core::Protocol;
use futures::{channel::mpsc, SinkExt};
use std::sync::{Arc, Mutex};
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
        ActiveBlock::ServerAddr => {
            server_handler(input, app);
        }
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
    if let Key::Right = input {
        app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::ChatWindow));
    }
}

pub fn chat_window_handler(input: Key, app: &mut App) {
    //app.cursor_vertical_offset = 25;
    //app.cursor_horizontal_offset = 70;
    match input {
        Key::Left => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::ChatList));
        }
        Key::Char('\n') => {
            app.messages.push(app.message_draft.drain(..).collect());
            // incredibly stupid, io should be async as well...
            task::block_on(async {
                app.outgoing_traffic_sender
                    .as_ref()
                    .unwrap() // kör map här istället
                    .send(Protocol {
                        from: app.id.clone(),
                        to: "kalle".into(),
                        message: app.messages.last().unwrap().to_owned(),
                    })
                    .await
                    .unwrap();
            });
            // send message
        }
        Key::Char(c) => {
            app.message_draft.push(c);
        }
        Key::Backspace => {
            app.message_draft.pop();
        }
        _ => {}
    }
}

pub fn id_handler(input: Key, app: &mut App) {
    match input {
        Key::Char('\n') => {
            app.messages.push(app.id.clone());
            app.set_current_route_state(
                Some(ActiveBlock::ServerAddr),
                Some(ActiveBlock::ServerAddr),
            );
            app.cursor_vertical_offset = 7;
        }
        Key::Char(c) => {
            app.id.push(c);
        }
        Key::Backspace => {
            app.id.pop();
        }
        _ => {}
    }
    app.input_cursor_pos = app.id.len() as u16;
}

pub fn server_handler(input: Key, app: &mut App) {
    match input {
        Key::Char('\n') => {
            let (incoming_traffic_sender, incoming_traffic_receiver) = mpsc::unbounded();
            let (outgoing_traffic_sender, outgoing_traffic_receiver) = mpsc::unbounded();
            app.outgoing_traffic_sender = Some(outgoing_traffic_sender);
            app.incoming_traffic_receiver = Some(Arc::new(Mutex::new(incoming_traffic_receiver)));
            task::block_on(async {
                if let Err(err) = connect_to_server(
                    &*app.server_addr.clone(),
                    incoming_traffic_sender,
                    outgoing_traffic_receiver,
                )
                .await
                {
                    eprintln!("Couldn't connect to server {:#?}", err);
                }
            });
            app.push_route(Route {
                id: RouteId::Chat,
                hovered_block: ActiveBlock::ChatList,
                active_block: ActiveBlock::ChatList,
            });
            //app.cursor_horizontal_offset = 6;
            //app.cursor_vertical_offset = 50;
        }
        Key::Char(c) => {
            app.server_addr.push(c);
        }
        Key::Backspace => {
            app.server_addr.pop();
        }
        _ => {}
    }
    app.input_cursor_pos = app.server_addr.len() as u16;
}
