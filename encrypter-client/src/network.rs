use crate::App;
use crossbeam::channel::select;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use encrypter_core::{Protocol, Result};

pub fn connect_to_server(app: &mut App) -> Result<()> {
    let stream = TcpStream::connect(&app.server_addr)?;
    let (incoming_sender, incoming_receiver) = unbounded();
    let (outgoing_sender, outgoing_receiver) = unbounded();
    app.outgoing_traffic_sender = Some(outgoing_sender.clone());
    app.incoming_traffic_receiver = Some(incoming_receiver.clone());
    app.net_thread_scope.spawn(move |_| {
        if let Err(err) = communicate_with_server(stream, outgoing_receiver, incoming_sender) {
            eprintln!(
                "Something went wrong when communicating with server {:#?}",
                err
            );
        }
    });
    Ok(())
}

fn communicate_with_server(
    stream: TcpStream,
    outgoing_traffic_receiver: Receiver<Protocol>,
    incoming_traffic_sender: Sender<Protocol>,
) -> Result<()> {
    let (reader, mut writer) = (&stream, &stream);
    loop {
        let reader = BufReader::new(reader);
        for msg in reader.lines() {
            let message = bincode::deserialize::<Protocol>(&msg?.as_bytes())?;
            println!("Message from server {:#?}", message);
            incoming_traffic_sender.send(message)?;
        }
        while let Ok(msg) =
            outgoing_traffic_receiver.recv_timeout(std::time::Duration::from_millis(1000))
        {
            writer.write_all(&bincode::serialize(&msg)?)?;
        }
        // if something should be sent to the server
    }
    Ok(())
}
