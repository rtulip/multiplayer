use std::io::prelude::*;
use std::net::TcpStream;

use crate::threading::threadpool;
use crate::MSG_SIZE;

pub struct Client {

    stream: TcpStream,
    pool: threadpool::ThreadPool,
}

impl Client {

    pub fn new(ip: &str, size: usize) -> Client {

        let stream = TcpStream::connect(ip).expect("Unable to connect to server");
        let pool = threadpool::ThreadPool::new(size);

        Client {
            stream,
            pool, 
        }

    }

    pub fn start(mut self) {

        let dispatch_clone = self.pool.dispatcher.clone();
        let stream_clone = self.stream.try_clone().expect("Unable to clone stream");
        
        self.pool.dispatcher.execute_loop(move || {
            
            let mut msg = String::new();
            println!("Type a message!");
            let _buff_size = std::io::stdin().read_line(&mut msg).unwrap();

            let stream_clone = stream_clone.try_clone().expect("Failed to clone stream");
        
            dispatch_clone.execute(move || {
                send_msg(&msg, &stream_clone.try_clone().expect("Failed to clone stream"));
            });
            
        });

        loop {
            
            let mut buff = vec![0; MSG_SIZE];

            match self.stream.read(&mut buff) {
                Ok(0) => {
                    println!("Source Disconected!");
                    break;
                },
                Ok(_) => {
                    self.pool.dispatcher.execute(move || {
                        receive_msg(&buff);
                    });
                },
                Err(_) => {
                    println!("Error: halting listener");
                    break;
                }
            }

        }

    }

}

fn receive_msg(buff: &Vec<u8>) {
    
    let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
    let string = String::from_utf8(msg).expect("Invlaid utf8 message");
    println!("Recieved: {}", string);
}

fn send_msg(string: &String, mut out_stream: &TcpStream) {
    let buff = string.clone().into_bytes();
    out_stream.write_all(&buff).expect("Problem sending message");
}
