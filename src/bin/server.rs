extern crate multiplayer;
use multiplayer::threading::threadpool::{ThreadPool, Message, new_job};
use multiplayer::msg;
use std::sync::mpsc;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::prelude::*;

fn main() {
    
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(100);
    
    loop {

        if let Ok((stream, addr)) = listener.accept(){
            
            let stream_clone = stream.try_clone().expect("Unable to clone stream");
            let sender_clone = pool.sender.clone();
    
            pool.execute(move || loop {
                match client_listen(stream_clone.try_clone().expect("unable to clone stream"), addr, sender_clone.clone()) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("{}", e);
                        break;
                    },
                }
            });
        }

    }

}

type Result<T> = std::result::Result<T, ClientDisconnectError>;

#[derive(Debug, Clone)]
struct ClientDisconnectError{
    addr: SocketAddr,
}

impl std::fmt::Display for ClientDisconnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Client {} Disconnected", self.addr)
    }
}

impl std::error::Error for ClientDisconnectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    } 
}

fn client_listen(mut socket: TcpStream, addr: SocketAddr, out_stream: mpsc::Sender<Message>) -> Result<()> {
    
    let mut buff = vec![0; msg::MSG_SIZE];

    match socket.read(&mut buff){
        Ok(0) => {
            Err(ClientDisconnectError{
                addr,
            })
        },
        Ok(_) => {
            let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();

            println!("\nMSG as Bytes:   {:?}", msg.clone());
            let msg = String::from_utf8(msg).expect("Invalid utf8 message");
            println!("MSG: {}", msg);

            out_stream.send(
                new_job(move || {
                    echo_message(&mut socket, msg);
                })
            ).unwrap();

            Ok(())
        },
        Err(_) => {
            Err(ClientDisconnectError{
                addr,
            })
        }
    }

}

fn echo_message(socket: &mut TcpStream, message: String){

    let buff = message.clone().into_bytes();
    socket.write_all(&buff).expect("Failed to write to socket!");

}