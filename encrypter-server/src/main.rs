use futures::{channel::mpsc, FutureExt, SinkExt, StreamExt};
use std::{collections::hash_map::HashMap, sync::Arc};

use async_std::{
    io::BufReader,
    io::ReadExt,
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use encrypter_core::Protocol;
use encrypter_core::Result;
use encrypter_core::MESSAGE_PACKET_SIZE;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
type Sender<T> = mpsc::UnboundedSender<T>;

struct NetEvent {
    pub message: Protocol,
    pub stream: Arc<TcpStream>,
}

fn main() -> Result<()> {
    task::block_on(accept_connections("127.0.0.1:1337"))
}

async fn accept_connections(addr: &str) -> Result<()> {
    let tcp_listener = TcpListener::bind(addr).await?;
    let (sender, receiver) = mpsc::unbounded::<NetEvent>();
    task::spawn(message_broker(receiver));
    let mut incoming = tcp_listener.incoming();
    while let Some(connection) = incoming.next().await {
        let stream = connection?;
        println!("New connection from: {}", stream.peer_addr()?);
        spawn_listener_task(sender.clone(), stream);
    }
    Ok(())
}

fn spawn_listener_task(sender: Sender<NetEvent>, stream: TcpStream) {
    task::spawn(async move {
        if let Err(e) = listen_to_traffic(sender, stream).await {
            eprintln!("Error parsing incomming traffic: {:#?}", e);
        }
    });
}

// This is where all the magic happens, there is only one message broker task on each server
// that means it's possible (but not scalable) to keep all peer info in memory.
async fn message_broker(mut receiver: Receiver<NetEvent>) -> Result<()> {
    let mut peers: HashMap<String, Arc<TcpStream>> = HashMap::new();
    // continue to wait for new NetEvents
    // fuse makes sure that the future won't be polled again, this shouldn't happen (I think)
    // but it's good to make sure either way.
    while let Some(event) = receiver.next().fuse().await {
        peers.insert(event.message.from.clone(), event.stream.clone());
        if let Some(receiving_participant) = peers.get(&event.message.to) {
            let mut receiveing_stream = &*receiving_participant.clone();
            receiveing_stream
                .write_all(&bincode::serialize(&event.message)?)
                .await?;
        } else {
            println!("No peer with id {} connected", event.message.to);
        }
    }
    Ok(())
}

// This creates a shared pointer to each TcpConnection and sends the pointer together with
// each message to the message broker who handles the actual propagation of messages.
async fn listen_to_traffic(mut sender: Sender<NetEvent>, stream: TcpStream) -> Result<()> {
    // Wrap the connection stream in Arc to be able to send it between tasks
    // This listens on incoming traffic but it's also needed when sending out messages
    let stream = Arc::new(stream);
    let mut reader = BufReader::new(&*stream);
    let mut buffer = vec![0 as u8; MESSAGE_PACKET_SIZE];
    loop {
        if let Ok(n) = reader.read(&mut buffer).await {
            match bincode::deserialize::<Protocol>(&buffer[..n]) {
                Ok(message) => {
                    println!("Message received: {:#?}", message);
                    sender
                        .send(NetEvent {
                            message,
                            stream: stream.clone(),
                        })
                        .await
                        .unwrap_or_else(|err| {
                            println!("Couldn't send message over channel {}", err)
                        });
                }
                Err(err) => {
                    println!("Could not parse message from incomming traffic {}", err);
                }
            }
        }
    }
}
