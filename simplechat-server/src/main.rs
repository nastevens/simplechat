/// Simple chat server
use anyhow::Result;
use clap::Parser;
use futures::{SinkExt, TryStreamExt};
use simplechat_protocol::{
    ClientFrame, ClientFrameCodec, ReceivedMessage, ServerFrame, ServerFrameCodec,
};
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use tokio_util::codec::{FramedRead, FramedWrite};

// Types used by broadcast channel to distribute messages
type ClientId = usize;
type RelayedMessage = (ClientId, ReceivedMessage);

const DEFAULT_NAME: &str = "Anonymous";

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Bind to this addr
    #[arg(short, long, default_value = "localhost:3000")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(args.addr).await?;
    let (relay_tx, _relay_rx) = broadcast::channel::<RelayedMessage>(256);
    let client_id = AtomicUsize::from(0);

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::spawn(handle_client(
            client_id.fetch_add(1, Ordering::Relaxed),
            stream,
            addr,
            relay_tx.clone(),
            relay_tx.subscribe(),
        ));
    }
}

async fn handle_client(
    client_id: usize,
    stream: TcpStream,
    addr: SocketAddr,
    relay_tx: broadcast::Sender<RelayedMessage>,
    mut relay_rx: broadcast::Receiver<RelayedMessage>,
) {
    println!("connection from {:?} assigned #{}", addr, client_id);
    let (rx, tx) = tokio::io::split(stream);
    let mut reader = FramedRead::new(rx, ClientFrameCodec::default());
    let mut writer = FramedWrite::new(tx, ServerFrameCodec::default());
    let mut name = String::from(DEFAULT_NAME);
    loop {
        tokio::select! {
            // Receive messages from the client
            maybe_frame = reader.try_next() => {
                if let Ok(Some(frame)) = maybe_frame {
                    match frame {
                        ClientFrame::Send(msg) => {
                            name = msg.author.clone();
                            if let Err(e) = relay_tx.send((client_id, msg.into())) {
                                eprintln!("relay error: {:?}", e);
                            }
                        }
                        ClientFrame::Leave => {
                            println!("{} left", name);
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            // Forward messages to the client
            maybe_msg = relay_rx.recv() => {
                if let Ok((sender_id, msg)) = maybe_msg {
                    if sender_id != client_id {
                        writer.send(ServerFrame::receive(msg)).await.unwrap();
                    }
                }
            }
        }
    }
}
