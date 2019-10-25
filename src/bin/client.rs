extern crate multiplayer;
use multiplayer::msg::Message;
use multiplayer::client::{Client, start_input_handler};
use std::sync::mpsc;

fn main() {
    
    let (tx, rx) = mpsc::channel::<Message>();
    
    let client = Client::new("127.0.0.1:7878");
    start_input_handler(tx.clone());
    client.start(rx, tx.clone());
    
    loop {}
    
} 