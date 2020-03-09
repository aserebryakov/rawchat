mod client;
mod server;
mod utils;
use client::ClientMessage;
use std::sync::mpsc::{channel, Receiver, Sender};

pub fn run() -> Result<(), std::io::Error> {
    println!("Initializing...");
    let (tx, rx): (Sender<ClientMessage>, Receiver<ClientMessage>) = channel();

    server::Server::new(rx).run()?;
    server::Listener::new(tx).and_then(server::Listener::listen)?;

    Ok(())
}
