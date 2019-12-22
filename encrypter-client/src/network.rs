use crate::App;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    thread,
};
#[derive(Debug)]
pub struct Communticator {
    incoming_receiver: Receiver<Protocol>,
    outgoing_sender: Sender<Protocol>,
}

impl Communticator {
    fn new(incoming_receiver: Receiver<Protocol>, outgoing_sender: Sender<Protocol>) -> Self {
        Communticator {
            incoming_receiver,
            outgoing_sender,
        }
    }

    pub fn send(&self, message: Protocol) {
        self.outgoing_sender.send(message).unwrap();
    }

    pub fn receive(&self) -> Vec<String> {
        let mut result = Vec::new();
        while let Ok(msg) = self
            .incoming_receiver
            .recv_timeout(std::time::Duration::from_millis(100))
        {
            result.push(format!("{}: {}", msg.from, msg.message).to_string());
        }
        result
    }
}

use encrypter_core::{Protocol, Result};

pub fn connect_to_server(app: &mut App) -> Result<()> {
    let stream = TcpStream::connect(&app.server_addr)?;
    let (incoming_sender, incoming_receiver) = unbounded();
    let (outgoing_sender, outgoing_receiver) = unbounded();
    app.communicator = Some(Communticator::new(incoming_receiver, outgoing_sender));
    //app.outgoing_traffic_sender = Some(outgoing_sender.clone());
    //app.incoming_traffic_receiver = Some(incoming_receiver.clone());
    app.net_thread_scope = Some(thread::spawn(move || {
        if let Err(err) = communicate_with_server(stream, outgoing_receiver, incoming_sender) {
            eprintln!(
                "Something went wrong when communicating with server {:#?}",
                err
            );
        }
        Ok(())
    }));
    Ok(())
}

fn communicate_with_server(
    stream: TcpStream,
    outgoing_traffic_receiver: Receiver<Protocol>,
    incoming_traffic_sender: Sender<Protocol>,
) -> Result<()> {
    let (reader, mut writer) = (&stream, &stream);
    loop {
        let mut reader = BufReader::new(reader);
        //for msg in reader.lines() {
        let mut message = String::new();
        reader.read_line(&mut message)?;
        let message = bincode::deserialize::<Protocol>(&message.as_bytes())?;
        println!("Message from server {:#?}", message);
        incoming_traffic_sender.send(message)?;
        //}
        while let Ok(msg) =
            outgoing_traffic_receiver.recv_timeout(std::time::Duration::from_millis(1000))
        {
            writer.write_all(&bincode::serialize(&msg)?)?;
        }
        // if something should be sent to the server
    }
    Ok(())
}
