use std::{
    collections::hash_map::{Entry, HashMap},
    sync::Arc,
};

use futures::{channel::mpsc, select, FutureExt, SinkExt};

use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};
use encrypter_core::Protocol;
use futures::channel::mpsc::UnboundedReceiver;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

fn main() -> Result<()> {
    task::block_on(accept_connections("127.0.0.1:1337"))
}

async fn accept_connections(addr: &str) -> Result<()> {
    let tcp_listener = TcpListener::bind(addr).await?;
    let (sender, receiver) = mpsc::unbounded::<Protocol>();
    task::spawn(handle_incomming_messages(receiver));
    let mut incoming = tcp_listener.incoming();
    while let Some(connection) = incoming.next().await {
        let stream = connection?;
        println!("New connection from: {}", stream.peer_addr()?);
        spawn_parse_task(sender.clone(), stream);
    }
    Ok(())
}

async fn spawn_parse_task(sender: Sender<Protocol>, stream: TcpStream) {
    async move {
        if let Err(e) = parse_incoming_traffic(sender, stream).await {
            eprintln!("Error parsing incomming traffic: {:#?}", e);
        }
    }
}

async fn handle_incomming_messages(receiver: Receiver<Protocol>) {
    unimplemented!();
}

async fn parse_incoming_traffic(sender: Sender<Protocol>, stream: TcpStream) -> Result<()> {
    Ok(())
}
