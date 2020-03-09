use server::{Reason, ServerMessage};
use std;
use std::clone::Clone;
use std::io::{ErrorKind, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::Builder;
use std::time::Duration;
use utils;

pub struct ClientInfo {
    pub nickname: String,
    pub tx: Sender<ServerMessage>,
}

impl Clone for ClientInfo {
    fn clone(&self) -> ClientInfo {
        ClientInfo {
            nickname: self.nickname.clone(),
            tx: self.tx.clone(),
        }
    }
}

pub enum ClientMessage {
    TryConnect(ClientInfo),
    Disconnect(String),
    Text(String),
}

pub struct Client {
    info: ClientInfo,
    stream: TcpStream,
    server_tx: Sender<ClientMessage>,
    client_rx: Receiver<ServerMessage>,
}

impl Client {
    pub fn new(stream: TcpStream, server_tx: Sender<ClientMessage>) -> Result<(), std::io::Error> {
        let (tx, rx): (Sender<ServerMessage>, Receiver<ServerMessage>) = channel();

        let builder = Builder::new();
        builder.spawn(
            move || Client::try_to_connect(stream, server_tx, tx, rx).and_then(Client::run_main_loop)
        )?;

        Ok(())
    }

    fn run_main_loop(mut self) -> Result<(), std::io::Error> {
        if let Err(e) = self.main_loop() {
            eprintln!("Client exit with error {:?}", e);
            if let Err(e) = self.server_tx.send(ClientMessage::Disconnect(String::from(format!(
                "{} disconnected",
                self.info.nickname.clone()
            )))) {
                eprintln!("Couldn't send the disconnect {:?}", e);
            }
        }

        Ok(())
    }

    fn main_loop(&mut self) -> Result<(), std::io::Error> {
        let _ = self.stream.set_read_timeout(Some(Duration::from_millis(200)));

        loop {
            self.read_line_from_stream(&self.stream, &self.server_tx)?;
            self.write_to_stream();
        }
    }

    fn try_to_connect(
        mut stream: TcpStream,
        server_tx: Sender<ClientMessage>,
        client_tx: Sender<ServerMessage>,
        client_rx: Receiver<ServerMessage>,
    ) -> Result<Client, std::io::Error> {
        loop {
            stream.write("Greetings!\nPlease enter your nickname: ".as_bytes())?;

            let nickname = utils::read_line(&stream).unwrap();
            let info = ClientInfo {
                nickname,
                tx: client_tx.clone(),
            };

            if let Err(e) = server_tx.send(ClientMessage::TryConnect(info.clone())) {
                eprintln!("Couldn't send the message {:?}", e);

                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Server connection error",
                ));
            }

            match client_rx.recv() {
                Ok(msg) => match msg {
                    ServerMessage::ConnectOk => return Ok(Client{info, stream, server_tx, client_rx}),
                    ServerMessage::ConnectError(e) => match e {
                        Reason::NicknameAlreadyUsed => {
                            stream.write("Nickname already used. Try again.\n".as_bytes())?;
                        }
                        Reason::GeneralError => {
                            stream.write("Server error, you will be disconnected.\n".as_bytes())?;
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "General Server Error",
                            ));
                        }
                    },
                    _ => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Unexpected message received",
                        ))
                    }
                },
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Couldn't receive message from server",
                    ))
                }
            }
        }
    }

    fn read_line_from_stream(
        &self,
        stream: &TcpStream,
        server_tx: &Sender<ClientMessage>,
    ) -> Result<(), std::io::Error> {
        match utils::read_line(&stream) {
            Ok(line) => {
                server_tx.send(ClientMessage::Text(self.info.nickname.clone() + " : " + line.as_str() + "\n")).unwrap();
                Ok(())
            },
            Err(e) => match e.kind() {
                ErrorKind::TimedOut | ErrorKind::WouldBlock => Ok(()),
                e => {
                    println!("{:?}", e);
                    server_tx.send(ClientMessage::Disconnect(self.info.nickname.clone())).unwrap();
                    Err(std::io::Error::new(e, "Error reading line"))
                }
            },
        }
    }

    fn write_to_stream(&mut self) {
        match self.client_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(msg) => match msg {
                ServerMessage::Text(line) => match self.stream.write(line.as_bytes()).unwrap() {
                    _ => (),
                },
                _ => eprintln!("Message is not supported"),
            },
            _ => (),
        };
    }
}
