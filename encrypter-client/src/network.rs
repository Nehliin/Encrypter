use crossbeam_channel::{unbounded, Receiver, Sender};
use encrypter_core::{Protocol, Result};
use std::io::{BufRead, BufReader, Write};
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
    pub fn new(server_addr: impl ToSocketAddrs) -> Result<Self> {
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
        connection.server_connection_loop()?;
        Ok(connection)
    }

    fn server_connection_loop(&self) -> Result<()> {
        let reader = self.stream.try_clone().unwrap();
        let sender = self.incoming_sender.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(reader);

            for line in reader.lines() {
                let message = bincode::deserialize::<Protocol>(&line.unwrap().as_bytes())
                    .expect("Failed to parse message"); // TODO: This should be handled better
                sender
                    .send(message) // TODO: this as well
                    .expect("Failed to sennd message from tcp listener thread");
            }
        });
        Ok(())
    }

    pub fn send(&self, message: Protocol) -> Result<()> {
        self.outgoing_sender.send(message)?;
        Ok(())
    }

    pub fn step(&mut self) -> Result<Option<String>> {
        if let Ok(outgoing) = self.outgoing_receiver.try_recv() {
            let message = &mut bincode::serialize(&outgoing)?;
            message.extend_from_slice("\n".as_bytes());
            self.stream.write_all(&message)?;
        }
        if let Ok(msg_from_server) = self.incoming_receiver.try_recv() {
            return Ok(Some(format!(
                "{}: {}",
                msg_from_server.from, msg_from_server.message
            )));
        }
        Ok(None)
    }
}
