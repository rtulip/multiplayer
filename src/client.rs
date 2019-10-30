use std::io::prelude::*;
use std::net::TcpStream;

use serde_json::Value;
use serde_json;

use crate::threading::{threadpool, dispatcher};
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
                    let s_clone = stream_clone.try_clone().expect("Unable to clone stream");
                    dispatch_clone.execute(move || {
                        send_msg(&msg, &s_clone.try_clone().expect("Failed to clone stream"));
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
                        receive_msg(&buff, dispatch_clone);
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

fn receive_msg(buff: &Vec<u8>, dispatch: dispatcher::Dispatcher) {
    
    let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
    let string = String::from_utf8(msg).expect("Invlaid utf8 message");

    let v: Value = serde_json::from_str(string.as_str()).expect("Unable to convert to json");

    match &v["msg_type"] {
        Value::String(text) => {
            match text.as_ref() {
                "Text" => {
                    let text_msg: message::TextMessage = serde_json::from_value(v).expect("Invalid Text Message");
                    dispatch.execute(move ||{
                        text_msg.handle();
                    });
                },
                _ => println!("Unknown Message type!"),
            }
        },  
        _ => println!("Unrecognized Message!"),
    }
}

fn send_msg(string: &String, mut out_stream: &TcpStream) {
    let buff = string.clone().into_bytes();
    out_stream.write_all(&buff).expect("Problem sending message");
}

fn read_input_line() -> Result<String, InputHandleError>{
    let mut msg = String::new();
    println!("Type a message!");
    match std::io::stdin().read_line(&mut msg){
        Ok(_buff_size) => return Ok(msg),
        Err(_) => Err(InputHandleError), 
    }
}
