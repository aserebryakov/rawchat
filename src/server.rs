use std;
use std::net::TcpListener;
use std::thread::{Builder, JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;


use client;


pub struct Server {
}


impl Server {
    pub fn new() {
        let (tx, rx) : (Sender<client::Message>, Receiver<client::Message>)  = channel();
        Server::run(rx).expect("Server initialization failed");
        Server::listen(tx).expect("Listener initialization failed");
    }


    pub fn run(rx : Receiver<client::Message>) -> Result<JoinHandle<()>, std::io::Error> {
        let builder = Builder::new();

        let mut clients = HashMap::new();

        builder.spawn(move || {
            println!("Running server main...");

            loop {
                match rx.recv() {
                    Ok(value) => match value {
                        client::Message::Connect(info) => {
                            println!("{} is connected", info.nickname);
                            let _ = info.tx.send(format!("Greetings, {}\n", info.nickname));
                            Server::multicast_text(&clients, format!("server: {} is joined to conversation\n", info.nickname));
                            clients.insert(info.nickname.clone(), info);
                        },
                        client::Message::Disconnect(nickname) => {
                            clients.remove(&nickname);
                            println!("{} is disconnected", nickname);
                            Server::multicast_text(&clients, format!("server: {} left\n", nickname));
                        },
                        client::Message::Text(text) => {
                            Server::multicast_text(&clients, text);
                        },
                    }
                    Err(e) => {
                       println!("{:?}", e);
                       Server::multicast_text(&clients, String::from("Server fault. You are disconnected.\n"));
                       break;
                    }
                };
            }
        })
    }


    pub fn listen(tx : Sender<client::Message>) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind("0.0.0.0:40000")?;

        println!("Waiting for clients...");

        for stream in listener.incoming() {
            let builder = Builder::new();
            let server_tx = tx.clone();

            builder.spawn(move || {
                client::Client::new(stream.unwrap(), server_tx);
            })?;
        }

        Ok(())
    }


    fn multicast_text(clients : &HashMap<String, client::ClientInfo>, text: String) {
        for (_, val) in clients.iter() {
            val.tx.send(text.clone()).unwrap();
        }
    }
}
