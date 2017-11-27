use std::sync::mpsc::{channel, Sender, Receiver};
use std::net::TcpStream;
use std::io::{ErrorKind, Write};
use std::thread::Builder;
use std::time::Duration;
use std::clone::Clone;


use utils;


pub struct ClientInfo {
    pub nickname : String,
    pub tx : Sender<String>,
}


impl Clone for ClientInfo {
    fn clone(&self) -> ClientInfo {
        ClientInfo {
            nickname : self.nickname.clone(),
            tx : self.tx.clone(),
        }
    }
}


pub enum Message {
    Connect(ClientInfo),
    Disconnect(String),
    Text(String),
}


pub struct Client {
    info : ClientInfo,
}


impl Client {
    pub fn new(mut stream: TcpStream, server_tx : Sender<Message>) {
        let (tx, rx) : (Sender<String>, Receiver<String>)  = channel();

        let builder = Builder::new();
        builder.spawn(move || {
            let _ = stream.write("Greetings!\n\0".as_bytes()).unwrap();
            let _ = stream.write("Please enter your nickname: ".as_bytes()).unwrap();

            let nickname = utils::read_line(&stream).unwrap();

            let info = ClientInfo{ nickname : nickname.clone(), tx };

            let _ = server_tx.send(Message::Connect(info.clone())).unwrap();

            Client::main_loop(Client{info : info}, stream, server_tx, rx);
        }).unwrap();
    }


    fn main_loop(self, mut stream: TcpStream, server_tx: Sender<Message>, client_rx: Receiver<String>) {
        let _ = stream.set_read_timeout(Some(Duration::new(1, 0)));

        loop {
            match utils::read_line(&stream) {
                Ok(line) => server_tx.send(Message::Text(self.info.nickname.clone() + " : " + line.as_str() + "\n")).unwrap(),
                Err(e) => match e.kind() {
                   ErrorKind::TimedOut | ErrorKind::WouldBlock => (),
                   e => {
                       println!("{:?}", e);
                       server_tx.send(Message::Disconnect(self.info.nickname.clone())).unwrap();
                       break;
                   },
                }
            };

            match client_rx.recv_timeout(Duration::new(1, 0)) {
                Ok(line) => match stream.write(line.as_bytes()).unwrap() {
                    _ => (),
                }
                _ => (),
            };
        }
    }
}
