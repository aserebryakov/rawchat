use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};


fn handle_client(mut stream: TcpStream) {
    let _ = stream.write("Greetings!\n\0".as_bytes()).unwrap();
    let _ = stream.write("Please enter your nickname: ".as_bytes()).unwrap();

    let mut nickname: String = String::new();

    for b in stream.bytes() {
        let c = b.unwrap() as char;
        if c == '\n' {
            break;
        }

        nickname.push(c);
    }

    println!("Clients nickname is {}", nickname);
}


fn main() {
    println!("Initializing...");

    let listener = TcpListener::bind("127.0.0.1:40000").unwrap();

    println!("Waiting for clients...");

    for stream in listener.incoming() {
        println!("Client is connected...");
        handle_client(stream.unwrap());
        println!("Client is disconnected...");
    }
}
