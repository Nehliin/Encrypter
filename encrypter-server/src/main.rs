#[macro_use]
extern crate log;
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
use futures::{channel::mpsc, FutureExt, SinkExt, StreamExt};
use simplelog::*;
use std::fs::File;
use std::sync::Arc;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
type Sender<T> = mpsc::UnboundedSender<T>;

mod peer;
use peer::Peer;
use peer::PeerSet;
#[derive(Debug)]
struct NetEvent {
    pub protocol_message: Protocol,
    pub stream: Arc<TcpStream>,
}

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed)
            .expect("Can't log to terminal"),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("server_logs.log").expect("Can't create log file"),
        ),
    ])
    .expect("Failed to initalize logger");
    task::block_on(accept_connections("127.0.0.1:1337"))
}

async fn accept_connections(addr: &str) -> Result<()> {
    let tcp_listener = TcpListener::bind(addr).await?;
    let (sender, receiver) = mpsc::unbounded::<NetEvent>();
    task::spawn(message_broker(receiver));
    let mut incoming = tcp_listener.incoming();
    while let Some(connection) = incoming.next().await {
        let stream = connection?;
        info!("New connection from: {}", stream.peer_addr()?);
        spawn_listener_task(sender.clone(), stream);
    }
    Ok(())
}

fn spawn_listener_task(sender: Sender<NetEvent>, stream: TcpStream) {
    task::spawn(async move {
        if let Err(e) = listen_to_traffic(sender, stream).await {
            error!("Error parsing incomming traffic: {:#?}", e);
        }
    });
}

// This is where all the magic happens, there is only one message broker task on each server
// that means it's possible (but not scalable) to keep all peer info in memory.
async fn message_broker(mut receiver: Receiver<NetEvent>) -> Result<()> {
    let mut peers = PeerSet::new();
    // continue to wait for new NetEvents
    // fuse makes sure that the future won't be polled again, this shouldn't happen (I think)
    // but it's good to make sure either way.
    while let Some(event) = receiver.next().fuse().await {
        match event.protocol_message {
            Protocol::NewConnection(id, public_key) => {
                let peer = Peer::new(id, event.stream.clone(), public_key);
                match peers.insert(peer) {
                    Ok(_) => {
                        let connected_peers = peers
                            .iter()
                            .map(|(id, peer)| (id.clone(), peer.public_key))
                            .collect::<Vec<(String, [u8; 32])>>();
                        send_to_all_peers(Protocol::PeerList(connected_peers), &peers).await?;
                    }
                    Err(err) => {
                        error!("Error: {}", err);
                    }
                }
            }
            Protocol::Disconnect(id) => {
                peers.remove_by_id(&id);
                send_to_all_peers(Protocol::Disconnect(id), &peers).await?;
            }
            // TODO: Split up internal and external Protocol?
            Protocol::InternalRemoveConnection => {
                if let Ok(socket_addr) = event.stream.peer_addr() {
                    if let Some(removed_peer) = peers.remove_by_ip(&socket_addr) {
                        send_to_all_peers(Protocol::Disconnect(removed_peer.peer_id), &peers)
                            .await?;
                    }
                } else {
                    error!("No socket addr was found in net event: {:?}", event);
                }
            }
            Protocol::Message(encrypted_message) => {
                let (_from, to) = encrypted_message.get_info();
                if let Some(receiving_participant) = peers.find_by_id(to) {
                    let mut receiveing_stream = &*receiving_participant.tcp_stream;
                    receiveing_stream
                        .write_all(&bincode::serialize(&Protocol::Message(encrypted_message))?)
                        .await?;
                } else {
                    warn!("Message couldn't be sent, no peer with id {} connected", to);
                }
            }
            _ => {}
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
        match reader.read(&mut buffer).await {
            Ok(0) | Err(_) => {
                // Probable disconnect from client
                sender
                    .send(NetEvent {
                        protocol_message: Protocol::InternalRemoveConnection,
                        stream: stream.clone(),
                    })
                    .await
                    .unwrap_or_else(|err| error!("Couldn't send message over channel {}", err));

                break Ok(());
            }
            Ok(n) => match bincode::deserialize::<Protocol>(&buffer[..n]) {
                Ok(protocol_message) => {
                    debug!("Protocol Message received: {:?}", protocol_message);
                    sender
                        .send(NetEvent {
                            protocol_message,
                            stream: stream.clone(),
                        })
                        .await
                        .unwrap_or_else(|err| error!("Couldn't send message over channel {}", err));
                }
                Err(err) => {
                    error!("Could not parse message from incomming traffic: {}", err);
                }
            },
        }
    }
}

async fn send_to_all_peers(message: Protocol, peers: &PeerSet) -> Result<()> {
    for peer in peers.values() {
        let mut stream = &*peer.tcp_stream;
        stream.write_all(&bincode::serialize(&message)?).await?;
    }
    Ok(())
}
