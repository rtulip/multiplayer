extern crate multiplayer;
use multiplayer::client;

fn main() {
    
    let clnt = client::Client::new("127.0.0.1:7878", 10);
    clnt.start();
    
} 