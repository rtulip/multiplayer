extern crate multiplayer;
use multiplayer::server;
use multiplayer::threading::threadpool::{ThreadPool, new_job};
use multiplayer::threading::dispatcher::Dispatcher;
use multiplayer::msg;
use multiplayer::errors::ClientDisconnectError;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::prelude::*;

fn main() {
    
    let svr = server::Server::new("127.0.0.1:7878", 100);
    svr.start();

}



