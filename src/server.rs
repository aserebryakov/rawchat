use std;
use std::net::TcpListener;
use std::thread::{Builder, JoinHandle};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;


use client;


pub struct Server {
    rx: Receiver<client::Message>,
}


pub struct Listener {
    listener: TcpListener,
    tx: Sender<client::Message>,
}


impl Listener {
    pub fn new(tx: Sender<client::Message>) -> Listener {
        let listener = TcpListener::bind("0.0.0.0:40000").expect("Failed to bind the socket");
        Listener { listener, tx }
    }


    pub fn listen(self) -> Result<(), std::io::Error> {

        println!("Waiting for clients...");

        for stream in self.listener.incoming() {
            let builder = Builder::new();
            let server_tx = self.tx.clone();

            builder.spawn(move || {
                client::Client::new(stream.unwrap(), server_tx);
            })?;
        }

        Ok(())
    }
}


impl Server {
    pub fn new(rx: Receiver<client::Message>) -> Server {
        Server { rx }
    }


    pub fn run(self) -> Result<JoinHandle<()>, std::io::Error> {
        let builder = Builder::new();

        builder.spawn(move || {
            println!("Running server main...");
            let mut clients = HashMap::new();

            loop {
                match self.rx.recv() {
                    Ok(value) => match value {
                        client::Message::Connect(info) => {
                            println!("{} is connected", info.nickname);
                            let _ = info.tx.send(format!("Greetings, {}\n", info.nickname));

                            Server::multicast_text(&clients,
                                format!("server: {} is joined to conversation\n", info.nickname));

                            clients.insert(info.nickname.clone(), info);
                        },
                        client::Message::Disconnect(nickname) => {
                            clients.remove(&nickname);
                            println!("{} is disconnected", nickname);

                            Server::multicast_text(&clients,
                                format!("server: {} left\n", nickname));
                        },
                        client::Message::Text(text) => {
                            Server::multicast_text(&clients, text);
                        },
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        Server::multicast_text(&clients,
                            String::from("Server fault. You are disconnected.\n"));
                        break;
                    },
                };
            }
        })
    }


    fn multicast_text(clients: &HashMap<String, client::ClientInfo>, text: String) {
        for (_, val) in clients.iter() {
            val.tx.send(text.clone()).unwrap();
        }
    }
}
