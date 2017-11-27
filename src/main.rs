use std::net::TcpListener;
use std::thread::{Builder, JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;


mod utils;
mod client;


fn run_server_main(rx : Receiver<client::Message>) -> Result<JoinHandle<()>, std::io::Error> {
    let builder = Builder::new();

    builder.spawn(move || {
        let mut clients = HashMap::new();

        println!("Running server main...");

        loop {
            match rx.recv() {
                Ok(value) => match value {
                    client::Message::Connect(info) => {
                        println!("{} is connected", info.nickname);
                        let _ = info.tx.send(format!("Greetings, {}\n", info.nickname));
                        multicast_text(&clients, format!("server: {} is joined to conversation\n", info.nickname));
                        clients.insert(info.nickname.clone(), info);
                    },
                    client::Message::Disconnect(nickname) => {
                        clients.remove(&nickname);
                        println!("{} is disconnected", nickname);
                        multicast_text(&clients, format!("server: {} left\n", nickname));
                    },
                    client::Message::Text(text) => {
                        multicast_text(&clients, text);
                    },
                }
                Err(e) => {
                   println!("{:?}", e);
                   multicast_text(&clients, String::from("Server fault. You are disconnected.\n"));
                   break;
                }
            };
        }
    })
}


fn multicast_text(clients : &HashMap<String, client::ClientInfo>, text: String) {
    for (_, val) in clients.iter() {
        val.tx.send(text.clone()).unwrap();
    }
}


fn run_connection_handler(server_tx : Sender<client::Message>) -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("0.0.0.0:40000")?;

    println!("Waiting for clients...");

    for stream in listener.incoming() {
        let builder = Builder::new();
        let server_tx = server_tx.clone();

        builder.spawn(move || {
            client::Client::new(stream.unwrap(), server_tx);
        })?;
    }

    Ok(())
}


fn main() {
    println!("Initializing...");

    let (tx, rx) : (Sender<client::Message>, Receiver<client::Message>)  = channel();

    run_server_main(rx).expect("Failed to run server main");
    run_connection_handler(tx).expect("Error on connection handling");
}
