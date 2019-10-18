extern crate multiplayer;
use multiplayer::ThreadPool;

use std::io::prelude::*;
use std::io::{self};
use std::net::TcpStream;
use std::str::from_utf8;

fn main() -> std::io::Result<()> {
    
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    let pool = ThreadPool::new(4);

    pool.execute(|| {
            handle_connection(stream);
    });

   

    Ok(())
} // the stream is closed here

fn handle_connection(mut stream: TcpStream){
    let mut buffer = [0; 512];
    match stream.read(&mut buffer) {
        Ok(_) => {
            println!("{}",from_utf8(&buffer).unwrap());
            println!();
        },
        Err(e) => {
            println!("Failed to receive data: {}", e);
        }
    }

    stream.flush().unwrap();
    
}