extern crate rawchatserver;
use std::process;

fn main() {
    rawchatserver::run().unwrap_or_else(|err| {
        eprintln!("Error occured : {:?}", err);
        process::exit(1);
    });
}
