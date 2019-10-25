extern crate multiplayer;
use multiplayer::server;
use std::sync::mpsc;

fn main() {
    
    let (tx, rx) = mpsc::channel();
    let server = server::Server::new("127.0.0.1:7878");
    server.start(rx, tx);

}