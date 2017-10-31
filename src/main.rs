use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::time::Duration;


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


fn client_main(mut stream: TcpStream, server_tx: Sender<String>, client_rx: Receiver<String>, nickname : String) {
    let _ = stream.set_read_timeout(Some(Duration::new(1, 0)));
    let mut end = false;

    while !end {
        println!("client loop");
        match read_line(&stream) {
            Ok(line) => server_tx.send(nickname.clone() + line.as_str()).unwrap(),
            Err(e) => match e.kind() {
               ErrorKind::TimedOut => end = false, 
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


fn handle_client(mut stream: TcpStream, server_tx : Sender<String>) -> Result<Sender<String>, std::io::Error> {

    let (tx, rx) : (Sender<String>, Receiver<String>)  = channel();

    let builder = thread::Builder::new();
    builder.spawn(move || {
        let _ = stream.write("Greetings!\n\0".as_bytes()).unwrap();
        let _ = stream.write("Please enter your nickname: ".as_bytes()).unwrap();

        let nickname = read_line(&stream).unwrap();

        let _ = server_tx.send(nickname.clone());
        let _ = stream.write(&rx.recv().unwrap().into_bytes()).unwrap();

        client_main(stream, server_tx, rx, nickname);
    }).unwrap();

    Ok(tx)
}



fn main() {
    println!("Initializing...");

    let listener = TcpListener::bind("127.0.0.1:40000").unwrap();

    println!("Waiting for clients...");

    for stream in listener.incoming() {
        println!("Client is connected...");

        let builder = thread::Builder::new();
        let (tx, rx) : (Sender<String>, Receiver<String>)  = channel();

        builder.spawn(move || {
            let client_tx = handle_client(stream.unwrap(), tx.clone()).unwrap();
            let nickname = rx.recv().unwrap();
            let greeting = format!("Welcome, {}!\n", nickname);

            println!("Clients nickname is {}", nickname);
            client_tx.send(greeting).unwrap();
            client_tx.send(String::from("test")).unwrap();
        }).unwrap();
    }
}
