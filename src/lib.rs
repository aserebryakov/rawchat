mod server;
mod client;
mod utils;
use std::sync::mpsc::{channel, Sender, Receiver};
use client::ClientMessage;


pub fn run() -> Result<(), std::io::Error> {
    println!("Initializing...");
    let (tx, rx): (Sender<ClientMessage>, Receiver<ClientMessage>) = channel();

    server::Server::new(rx).run()?;
    server::Listener::new(tx).and_then(server::Listener::listen)?;

    Ok(())
}
