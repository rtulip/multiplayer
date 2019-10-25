use std::net::SocketAddr;
pub const MSG_SIZE: usize = 32;

#[derive(Debug)]
pub enum Message {
    Recv(String),
    Send(String),
    SendAll(String),
    SendPrivate(String, SocketAddr),
    Terminate,
}