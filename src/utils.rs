use std;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::TcpStream;


pub fn read_line(mut stream: &TcpStream) -> Result<String, std::io::Error> {
    let mut line = String::new();

    loop {
        let mut b : [u8; 1] = [0];
        stream.read(&mut b)?;

        match b[0] as char {
            '\n' => break,
                '\0' => {
                    return Err(std::io::Error::new(ErrorKind::UnexpectedEof , "Disconnect"))
                },
                _ => line.push(b[0] as char),
        }
    }

    Ok(line)
}
