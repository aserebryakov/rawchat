use std;
use std::net::TcpListener;
use std::thread::{Builder, JoinHandle};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use client::{Client, ClientMessage, ClientInfo};


pub struct Server {
    rx: Receiver<ClientMessage>,
}


pub struct Listener {
    listener: TcpListener,
    tx: Sender<ClientMessage>,
}


pub enum ConnectErrorCause {
    NicknameAlreadyUsed,
    GeneralError,
}


#[allow(dead_code)]
pub enum ServerMessage {
    ConnectOk,
    ConnectError(ConnectErrorCause),
    Disconnect,
    Text(String),
}


impl Listener {
    pub fn new(tx: Sender<ClientMessage>) -> Result<Listener, std::io::Error> {
        let listener = TcpListener::bind("0.0.0.0:40000")?;
        Ok(Listener { listener, tx })
    }


    pub fn listen(self) -> Result<(), std::io::Error> {

        println!("Waiting for connections...");

        for stream in self.listener.incoming() {
            let builder = Builder::new();
            let server_tx = self.tx.clone();

            builder.spawn(move || {
                match stream {
                    Ok(stream) => {
                        if let Err(e) = Client::new(stream, server_tx) {
                            eprintln!("Failed to create a client {:?}", e);
                        }
                    }
                    Err(e) => eprintln!("Couln't handle a connection {:?}", e),
                };
            })?;
        }

        Ok(())
    }
}


impl Server {
    pub fn new(rx: Receiver<ClientMessage>) -> Server {
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
                        ClientMessage::TryConnect(info) => {
                            println!("{} is connected", info.nickname);
                            let _ = info.tx.send(
                                ServerMessage::Text(format!("Greetings, {}\n", info.nickname)));

                            Server::multicast_text(&clients,
                                format!("server: {} is joined to conversation\n", info.nickname));

                            clients.insert(info.nickname.clone(), info);
                        },
                        ClientMessage::Disconnect(nickname) => {
                            clients.remove(&nickname);
                            println!("{} is disconnected", nickname);

                            Server::multicast_text(&clients,
                                format!("server: {} left\n", nickname));
                        },
                        ClientMessage::Text(text) => {
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


    fn multicast_text(clients: &HashMap<String, ClientInfo>, text: String) {
        for (_, val) in clients.iter() {
            val.tx.send(ServerMessage::Text(text.clone())).unwrap();
        }
    }
}
