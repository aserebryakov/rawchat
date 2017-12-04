use std;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::net::TcpStream;
use std::io::{ErrorKind, Write};
use std::thread::Builder;
use std::time::Duration;
use std::clone::Clone;
use utils;
use server::ServerMessage;


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
    Connect(ClientInfo),
    Disconnect(String),
    Text(String),
}


pub struct Client {
    info: ClientInfo,
}


impl Client {
    pub fn new(
        mut stream: TcpStream,
        server_tx: Sender<ClientMessage>,
    ) -> Result<(), std::io::Error> {
        let (tx, rx): (Sender<ServerMessage>, Receiver<ServerMessage>) = channel();

        let builder = Builder::new();
        builder.spawn(move || {
            if let Err(e) = stream.write("Greetings!\nPlease enter your nickname: ".as_bytes()) {
                eprintln!("Failed to request the clients nickname : {:?}", e);
                ()
            }

            let nickname = utils::read_line(&stream).unwrap();

            let info = ClientInfo { nickname, tx };

            let _ = server_tx
                .send(ClientMessage::Connect(info.clone()))
                .unwrap();

            if let Err(e) = Client::main_loop(
                Client { info: info.clone() },
                stream,
                &server_tx,
                rx,
            )
            {
                eprintln!("Client exit with error {:?}", e);
                if let Err(e) = server_tx.send(ClientMessage::Disconnect(String::from(
                    format!("{} disconnected", info.nickname.clone()),
                )))
                {
                    eprintln!("Couldn't send the disconnect {:?}", e);
                }
            }
        })?;

        Ok(())
    }


    fn main_loop(
        self,
        mut stream: TcpStream,
        server_tx: &Sender<ClientMessage>,
        client_rx: Receiver<ServerMessage>,
    ) -> Result<(), std::sync::mpsc::SendError<ClientMessage>> {
        let _ = stream.set_read_timeout(Some(Duration::new(1, 0)));

        loop {
            match utils::read_line(&stream) {
                Ok(line) => {
                    server_tx.send(ClientMessage::Text(
                        self.info.nickname.clone() + " : " +
                            line.as_str() + "\n",
                    ))?;
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::TimedOut | ErrorKind::WouldBlock => (),
                        e => {
                            println!("{:?}", e);
                            server_tx.send(ClientMessage::Disconnect(
                                self.info.nickname.clone(),
                            ))?;

                            break;
                        }
                    }
                }
            };

            match client_rx.recv_timeout(Duration::new(1, 0)) {
                Ok(msg) => {
                    match msg {
                        ServerMessage::Text(line) => {
                            match stream.write(line.as_bytes()).unwrap() {
                                _ => (),
                            }
                        }
                        _ => eprintln!("Message is not supported"),
                    }
                }
                _ => (),
            };
        }

        Ok(())
    }
}
