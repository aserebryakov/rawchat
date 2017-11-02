use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread::{Builder, JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::collections::HashMap;


struct ClientInfo {
    nickname : String,
    tx : Sender<String>,
}

enum Message{
    Connect(ClientInfo),
    Disconnect(String),
    Text(String),
}


fn read_line(mut stream: &TcpStream) -> Result<String, std::io::Error> {
    let mut line = String::new();
    let mut end = false;

    while !end {
        let mut b : [u8; 1] = [0];
        stream.read(&mut b)?;

        match b[0] as char {
            '\n' => end = true,
            '\0' => {
                    return Err(std::io::Error::new(ErrorKind::UnexpectedEof , "Disconnect"))
                },
            _ => line.push(b[0] as char),
        }
    }

    Ok(line)
}


fn run_server_main(rx : Receiver<Message>) -> Result<JoinHandle<()>, std::io::Error> {
    let builder = Builder::new();

    builder.spawn(move || {
        let mut clients = HashMap::new();
        let mut end = false;
        println!("Running server main...");

        while !end {
            match rx.recv() {
                Ok(value) => match value {
                    Message::Connect(info) => {
                        println!("{} is connected", info.nickname);
                        let _ = info.tx.send(format!("Greetings, {}\n", info.nickname));
                        multicast_text(&clients, format!("server: {} is joined to conversation\n", info.nickname));
                        clients.insert(info.nickname.clone(), info);
                    },
                    Message::Disconnect(nickname) => {
                        clients.remove(&nickname);
                        println!("{} is disconnected", nickname);
                        multicast_text(&clients, format!("server: {} left\n", nickname));
                    },
                    Message::Text(text) => {
                        multicast_text(&clients, text);
                    },
                }
                Err(e) => {
                   println!("{:?}", e);
                   multicast_text(&clients, String::from("Server fault. You are disconnected.\n"));
                   end = true;
                }
            };
        }
    })
}


fn multicast_text(clients : &HashMap<String, ClientInfo>, text: String) {
    for (_, val) in clients.iter() {
        val.tx.send(text.clone()).unwrap();
    }
}


fn run_connection_handler(server_tx : Sender<Message>) -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("0.0.0.0:40000")?;

    println!("Waiting for clients...");

    for stream in listener.incoming() {
        let builder = Builder::new();
        let server_tx = server_tx.clone();

        builder.spawn(move || {
            run_client(stream.unwrap(), server_tx);
        })?;
    }

    Ok(())
}


fn client_main(mut stream: TcpStream, server_tx: Sender<Message>, client_rx: Receiver<String>, nickname : String) {
    let _ = stream.set_read_timeout(Some(Duration::new(1, 0)));
    let mut end = false;

    while !end {
        match read_line(&stream) {
            Ok(line) => server_tx.send(Message::Text(nickname.clone() + " : " + line.as_str() + "\n")).unwrap(),
            Err(e) => match e.kind() {
               ErrorKind::TimedOut | ErrorKind::WouldBlock => (),
               e => {
                   println!("{:?}", e);
                   end = true;
                   server_tx.send(Message::Disconnect(nickname.clone())).unwrap();
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


fn run_client(mut stream: TcpStream, server_tx : Sender<Message>){
    let (tx, rx) : (Sender<String>, Receiver<String>)  = channel();

    let builder = Builder::new();
    builder.spawn(move || {
        let _ = stream.write("Greetings!\n\0".as_bytes()).unwrap();
        let _ = stream.write("Please enter your nickname: ".as_bytes()).unwrap();

        let nickname = read_line(&stream).unwrap();

        let info = ClientInfo{ nickname : nickname.clone(), tx };
        let _ = server_tx.send(Message::Connect(info)).unwrap();

        client_main(stream, server_tx, rx, nickname);
    }).unwrap();
}


fn main() {
    println!("Initializing...");

    let (tx, rx) : (Sender<Message>, Receiver<Message>)  = channel();

    run_server_main(rx).expect("Failed to run server main");
    run_connection_handler(tx).expect("Error on connection handling");
}
