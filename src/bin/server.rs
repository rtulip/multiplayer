extern crate multiplayer;
use multiplayer::server;


fn main() {
    
    let svr = server::Server::new("127.0.0.1:7878", 100);
    svr.start();

}



