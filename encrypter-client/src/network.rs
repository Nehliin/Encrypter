use crossbeam_channel::{unbounded, Receiver, Sender};
use encrypter_core::{Protocol, Result, MESSAGE_PACKET_SIZE};
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::net::ToSocketAddrs;
#[derive(Debug)]
pub struct ServerConnection {
    outgoing_sender: Sender<Protocol>,
    outgoing_receiver: Receiver<Protocol>,
    incoming_receiver: Receiver<Protocol>,
    incoming_sender: Sender<Protocol>,
    stream: TcpStream,
    //  thread_handle: thread::JoinHandle fixa de här sen
}

impl ServerConnection {
    pub fn new(server_addr: impl ToSocketAddrs, id: String) -> Result<Self> {
        let stream = TcpStream::connect(&server_addr)?;
        let (incoming_sender, incoming_receiver) = unbounded();
        let (outgoing_sender, outgoing_receiver) = unbounded();
        let connection = ServerConnection {
            outgoing_sender,
            outgoing_receiver,
            incoming_receiver,
            incoming_sender,
            stream,
        };
        connection.send(Protocol::NewConnection(id))?;
        connection.server_connection_loop()?;
        Ok(connection)
    }

    fn server_connection_loop(&self) -> Result<()> {
        let reader = self.stream.try_clone().unwrap();
        let sender = self.incoming_sender.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut buffer = vec![0 as u8; MESSAGE_PACKET_SIZE];
            loop {
                if let Ok(n) = reader.read(&mut buffer) {
                    match bincode::deserialize::<Protocol>(&buffer[..n]) {
                        Ok(message) => {
                            sender
                                .send(message) // TODO: this as well
                                .expect("Failed to sennd message from tcp listener thread");
                        }
                        Err(err) => {
                            println!("Could not parse message from incomming traffic {}", err);
                        }
                    }
                }
            }
        });
        Ok(())
    }

    pub fn send(&self, message: Protocol) -> Result<()> {
        self.outgoing_sender.send(message)?;
        Ok(())
    }

    pub fn step(&mut self) -> Result<Option<Protocol>> {
        if let Ok(outgoing) = self.outgoing_receiver.try_recv() {
            let message = &mut bincode::serialize(&outgoing)?;
            debug_assert!(message.len() <= MESSAGE_PACKET_SIZE);
            self.stream.write_all(&message)?;
        }
        if let Ok(msg_from_server) = self.incoming_receiver.try_recv() {
            return Ok(Some(msg_from_server));
        }
        Ok(None)
    }
}
