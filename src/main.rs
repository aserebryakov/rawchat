use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;


struct Client {
    nickname : String,
    stream : TcpStream,
    thread_handle : thread::JoinHandle<()>,
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


fn handle_client(mut stream: TcpStream) -> Result<Sender<String>, std::io::Error> {
    let builder = thread::Builder::new();
    let (sender , receiver) : (Sender<String>, Receiver<String>)  = channel();

    builder.spawn(move || {
        let _ = stream.write("Greetings!\n\0".as_bytes()).unwrap();
        let _ = stream.write("Please enter your nickname: ".as_bytes()).unwrap();

        println!("Clients nickname is {}", read_line(&stream).unwrap());

        let _ = stream.write(&receiver.recv().unwrap().into_bytes());
    });

    Ok(sender)
}


fn main() {
    println!("Initializing...");

    let listener = TcpListener::bind("127.0.0.1:40000").unwrap();

    println!("Waiting for clients...");

    for stream in listener.incoming() {
        println!("Client is connected...");
        let sender = handle_client(stream.unwrap()).unwrap();
        sender.send(String::from("Greetings From Server!\n"));
    }
}
