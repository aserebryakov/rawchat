use client::{Client, ClientInfo, ClientMessage};
use std;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{Builder, JoinHandle};

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
                    Ok(value) => match value {
                        ClientMessage::TryConnect(info) => {
                            println!(
                                "Client tries to connect with the nickname {}",
                                info.nickname
                            );

                            if !clients.contains_key(&info.nickname) {
                                let _ = info.tx.send(ServerMessage::ConnectOk);

                                Server::greet(&info, &clients);

                                Server::multicast(
                                    &clients,
                                    ServerMessage::Text(format!(
                                        "Server: {} is joined to the conversation",
                                        info.nickname
                                    )),
                                );

                                println!("Client with the nickname {} is connected", info.nickname);

                                clients.insert(info.nickname.clone(), info);
                            } else {
                                let _ = info
                                    .tx
                                    .send(ServerMessage::ConnectError(Reason::NicknameAlreadyUsed));
                            }
                        }
                        ClientMessage::Disconnect(nickname) => {
                            clients.remove(&nickname);
                            println!("{} is disconnected", nickname);

                            Server::multicast(
                                &clients,
                                ServerMessage::Text(format!(
                                    "Server: {} left the conversation\n",
                                    nickname
                                )),
                            );
                        }
                        ClientMessage::Text(text) => {
                            Server::multicast(&clients, ServerMessage::Text(text));
                        }
                    },
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

    fn greet(info: &ClientInfo, clients: &HashMap<String, ClientInfo>) {
        let _ = info.tx.send(ServerMessage::Text(format!(
            "Greetings, {}\nFollowing People are in chat room:\n",
            info.nickname
        )));

        for (key, _) in clients.iter() {
            let _ = info.tx.send(ServerMessage::Text(format!("- {}\n", key)));
        }
    }
}
