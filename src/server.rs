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


#[derive(Clone)]
pub enum Reason {
    NicknameAlreadyUsed,
    GeneralError,
}


#[derive(Clone)]
pub enum ServerMessage {
    ConnectOk,
    ConnectError(Reason),
    Disconnect(Reason),
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
                    Ok(value) => {
                        match value {
                            ClientMessage::TryConnect(info) => {
                                println!(
                                    "Client tries to connect with the nickname {}",
                                    info.nickname
                                );

                                if !clients.contains_key(&info.nickname) {
                                    let _ = info.tx.send(ServerMessage::ConnectOk);

                                    let _ = info.tx.send(ServerMessage::Text(
                                        format!("Greetings, {}\n", info.nickname),
                                    ));

                                    Server::multicast(
                                        &clients,
                                        ServerMessage::Text(format!(
                                            "Server: {} is connected to the conversation",
                                            info.nickname
                                        )),
                                    );

                                    println!(
                                        "Client with the nickname {} is connected",
                                        info.nickname
                                    );

                                    clients.insert(info.nickname.clone(), info);
                                } else {
                                    let _ = info.tx.send(ServerMessage::ConnectError(
                                        Reason::NicknameAlreadyUsed,
                                    ));
                                }
                            }
                            ClientMessage::Disconnect(nickname) => {
                                clients.remove(&nickname);
                                println!("{} is disconnected", nickname);

                                Server::multicast(
                                    &clients,
                                    ServerMessage::Text(format!("server: {} left\n", nickname)),
                                );
                            }
                            ClientMessage::Text(text) => {
                                Server::multicast(&clients, ServerMessage::Text(text));
                            }
                        }
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        Server::multicast(
                            &clients,
                            ServerMessage::Disconnect(Reason::GeneralError),
                        );
                        break;
                    }
                };
            }
        })
    }


    fn multicast(clients: &HashMap<String, ClientInfo>, msg: ServerMessage) {
        for (_, val) in clients.iter() {
            val.tx.send(msg.clone()).unwrap();
        }
    }
}
