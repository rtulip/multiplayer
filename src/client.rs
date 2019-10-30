use std::io::prelude::*;
use std::net::TcpStream;

use crate::threading::threadpool;
use crate::message;
use crate::errors::InputHandleError;

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
            
            match read_input_line(){
                Ok(msg) => {
                    let mut s_clone = stream_clone.try_clone().expect("Unable to clone stream");
                    dispatch_clone.execute(move || {
                        message::send_text_message(&mut s_clone, msg)
                    });
                    Ok(())
                },
                Err(e) => Err(e),

            }
        });

        loop {
            
            let mut buff = vec![0; message::MSG_SIZE];

            match self.stream.read(&mut buff) {
                Ok(0) => {
                    println!("Source Disconected!");
                    break;
                },
                Ok(_) => {
                    let dispatch_clone = self.pool.dispatcher.clone();
                    self.pool.dispatcher.execute(move || {
                        message::receive_json(&buff, dispatch_clone);
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



fn read_input_line() -> Result<String, InputHandleError>{
    let mut msg = String::new();
    println!("Type a message!");
    match std::io::stdin().read_line(&mut msg){
        Ok(_buff_size) => return Ok(msg),
        Err(_) => Err(InputHandleError), 
    }
}
