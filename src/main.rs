use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;


fn read_line(mut stream: &TcpStream) -> Result<String, ()> {
    let mut line = String::new();
    let mut end = false;

    while !end {
        let mut b : [u8; 1] = [0];
        stream.read(&mut b).unwrap();

        match b[0] as char {
            '\n' | '\0' => end = true,
            _ => line.push(b[0] as char),
        }
    }

    Ok(line)
}


fn handle_client(mut stream: TcpStream) -> Result<thread::JoinHandle<()>, std::io::Error> {
    let builder = thread::Builder::new();

    builder.spawn(move || {
        let _ = stream.write("Greetings!\n\0".as_bytes()).unwrap();
        let _ = stream.write("Please enter your nickname: ".as_bytes()).unwrap();

        println!("Clients nickname is {}", read_line(&stream).unwrap());
    })
}


fn main() {
    println!("Initializing...");

    let listener = TcpListener::bind("127.0.0.1:40000").unwrap();

    println!("Waiting for clients...");

    for stream in listener.incoming() {
        println!("Client is connected...");
        let h = handle_client(stream.unwrap()).unwrap();
        h.join().unwrap();
        println!("Client is disconnected...");
    }
}
