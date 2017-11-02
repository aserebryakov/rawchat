use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::collections::HashMap;


struct ClientInfo {
    nickname : String,
    tx : Sender<String>,
}

enum Message{
    Connect(ClientInfo),
    Disconnect(ClientInfo),
    Text(String),
}


fn read_line(mut stream: &TcpStream) -> Result<String, std::io::Error> {
    let mut line = String::new();
    let mut end = false;

    while !end {
        let mut b : [u8; 1] = [0];
        stream.read(&mut b)?;

        match b[0] as char {
            '\n' | '\0' => end = true,
            _ => line.push(b[0] as char),
        }
    }

    Ok(line)
}


fn run_server_main(rx : Receiver<Message>) {
    let builder = thread::Builder::new();

    let _ = builder.spawn(move || {
        let mut clients = HashMap::new();
        println!("Running server main...");

        while true {
            match rx.recv().unwrap() {
                Message::Connect(info) => {
                    println!("{} is connected", info.nickname);
                    let _ = info.tx.send(format!("Greetings, {}\n", info.nickname));
                    multicast_text(&clients, format!("server: {} is joined to conversation\n", info.nickname));
                    clients.insert(info.nickname.clone(), info);
                    ()
                },
                Message::Disconnect(info) => {
                    clients.remove(&info.nickname);
                    ()
                },
                Message::Text(text) => {
                    multicast_text(&clients, text)
                },
            };
        }
    }).unwrap();
}


fn multicast_text(clients : &HashMap<String, ClientInfo>, text: String) {
    for (_, val) in clients.iter() {
        val.tx.send(text.clone()).unwrap();
    }
}


fn run_connection_handler(server_tx : Sender<Message>) {
    let listener = TcpListener::bind("127.0.0.1:40000").unwrap();

    println!("Waiting for clients...");

    for stream in listener.incoming() {
        let builder = thread::Builder::new();
        let server_tx = server_tx.clone();

        let _ = builder.spawn(move || {
            run_client(stream.unwrap(), server_tx);
        }).unwrap();
    }
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

    let builder = thread::Builder::new();
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

    run_server_main(rx);
    run_connection_handler(tx);
}
