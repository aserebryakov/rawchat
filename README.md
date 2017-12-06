# rawchatserver-rs 0.2.0

This is a raw socket chat server that doesn't require any specific client.
The program is written for Rust language learning purposes.

## Usage

Run the server and connect to it with e.g. `netcat` or `Putty` utility to
port `40000`.

## Features

* All people in the chat room have unique nicknames
* Client receives the list of nicknames of currently connected people
  on connect
