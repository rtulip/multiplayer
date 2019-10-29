use std::net::{TcpListener, TcpStream, SocketAddr};
use crate::threading::{threadpool, dispatcher};
use crate::errors::{ConnectionStatus, ClientDisconnectError};
use crate::MSG_SIZE;
use std::io::prelude::*;

pub struct Server {
    listener: TcpListener,
    pool: threadpool::ThreadPool,
}

impl Server {

    pub fn new(ip: &str, size: usize) -> Server {

        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        let pool = threadpool::ThreadPool::new(size);

        Server{
            listener,
            pool,
        }    

    }

    pub fn start(self) {

        loop {

            if let Ok((stream, addr)) = self.listener.accept(){
                
                let stream_clone = stream.try_clone().expect("Unable to clone stream");
                let dispatch_clone = self.pool.dispatcher.clone();
                self.pool.dispatcher.execute_loop(move || {
                    client_listen(stream_clone.try_clone().expect("unable to clone stream"), addr, dispatch_clone.clone())
                });
                
            }

        }

    }

}

fn client_listen(mut socket: TcpStream, addr: SocketAddr, dispatch: dispatcher::Dispatcher) -> ConnectionStatus {
    
    let mut buff = vec![0; MSG_SIZE];

    match socket.read(&mut buff){
        Ok(0) => {
            
            Err(ClientDisconnectError{
                addr,
            })

        },
        Ok(_) => {
            
            let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
            let msg = String::from_utf8(msg).expect("Invalid utf8 message");
            println!("MSG: {}", msg);

            dispatch.execute(move || {
                echo_message(&mut socket, &msg);
            });

            Ok(())

        },
        Err(_) => {
            
            Err(ClientDisconnectError{
                addr,
            })

        }
    }

}

fn echo_message(socket: &mut TcpStream, message: &String){

    let buff = message.clone().into_bytes();
    socket.write_all(&buff).expect("Failed to write to socket!");

}