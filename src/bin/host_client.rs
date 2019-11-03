extern crate multiplayer;
use multiplayer::host_side::host_server::HostServer;

fn main() {
    let host_server = HostServer::new("127.0.0.1:7878", 10);
    host_server.start();
}
