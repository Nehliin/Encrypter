use futures::{channel::mpsc, select, FutureExt, SinkExt, StreamExt};

use async_std::{
    io::{BufReader, Write},
    net::TcpStream,
    prelude::*,
    task,
};

use encrypter_core::{Protocol, Receiver, Result, Sender};

pub async fn connect_to_server(
    server_addr: &str,
    incoming_traffic_sender: Sender<Protocol>,
    outgoing_traffic_receiver: Receiver<Protocol>,
) -> Result<()> {
    let stream = TcpStream::connect(server_addr).await?;

    task::spawn(async move {
        if let Err(err) =
            communicate_with_server(stream, outgoing_traffic_receiver, incoming_traffic_sender)
                .await
        {
            eprintln!(
                "Something went wrong when communicating with server {:#?}",
                err
            );
        }
    });
    Ok(())
}

async fn communicate_with_server(
    stream: TcpStream,
    outgoing_traffic_receiver: Receiver<Protocol>,
    mut incoming_traffic_sender: Sender<Protocol>,
) -> Result<()> {
    let (reader, mut writer) = (&stream, &stream);
    let reader = BufReader::new(reader);
    let mut msg_from_server = StreamExt::fuse(reader.lines());
    let mut outgoing_traffic_receiver = StreamExt::fuse(outgoing_traffic_receiver);
    loop {
        select! {
            // if something should be sent to the server
            msg = outgoing_traffic_receiver.next().fuse() => match msg {
                Some(msg) => {
                    writer.write_all(&bincode::serialize(&msg)?).await?;
                }
                None => break,
            },
            // if a message have been received from the server
            msg = msg_from_server.next().fuse() => match msg {
                Some(msg) => {
                    let message = bincode::deserialize::<Protocol>(&msg?.as_bytes())?;
                    println!("Message from server {:#?}", message);
                    incoming_traffic_sender.send(message).await?;
                },
                None => break,
            }
        }
    }
    Ok(())
}
