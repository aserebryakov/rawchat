mod server;
mod client;
mod utils;
use std::sync::mpsc::{channel, Sender, Receiver};


pub fn run() {
    println!("Initializing...");
    let (tx, rx): (Sender<client::Message>, Receiver<client::Message>) = channel();

    server::Server::new(rx).run().expect("Server running failed");
    server::Listener::new(tx).listen().expect("Listening failed");
}
