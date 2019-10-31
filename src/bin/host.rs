extern crate multiplayer;
use multiplayer::host;

fn main() {
    
    let host = host::Host::new("127.0.0.1:7878", 10);
    host.start();
    
} 