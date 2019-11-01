use std::io::prelude::*;
use std::net::TcpStream;

use crate::comms::handler::Handler;
use crate::comms::message;
use crate::errors::InputHandleError;
use crate::threading::{dispatcher, threadpool};

pub struct Host {
    stream: TcpStream,
    pool: threadpool::ThreadPool,
    message_handler: HostHandler,
}

impl Host {
    pub fn new(ip: &str, size: usize) -> Host {
        let stream = TcpStream::connect(ip).expect("Unable to connect to server");
        let pool = threadpool::ThreadPool::new(size);
        let message_handler = HostHandler {
            dispatch: pool.dispatcher.clone(),
            socket: stream.try_clone().expect("Failed to clone stream"),
        };

        Host {
            stream,
            pool,
            message_handler,
        }
    }

    pub fn start(mut self) {
        let dispatch_clone = self.pool.dispatcher.clone();
        let stream_clone = self.stream.try_clone().expect("Unable to clone stream");

        // self.pool.dispatcher.execute_loop(move || {

        //     match read_input_line(){
        //         Ok(msg) => {
        //             let mut s_clone = stream_clone.try_clone().expect("Unable to clone stream");
        //             dispatch_clone.execute(move || {
        //                 let msg = message::TextMessage::new(msg);
        //                 message::send_json(msg, &mut s_clone);
        //             });
        //             Ok(())
        //         },
        //         Err(e) => Err(e),

        //     }
        // });

        loop {
            let mut buff = vec![0; message::MSG_SIZE];

            match self.stream.read(&mut buff) {
                Ok(0) => {
                    println!("Source Disconected!");
                    break;
                }
                Ok(_) => {
                    let mut h = self
                        .message_handler
                        .try_clone()
                        .expect("Failed to clone handler");
                    self.pool.dispatcher.execute(move || {
                        h.receive_json(&buff);
                    });
                }
                Err(_) => {
                    println!("Error: halting listener");
                    break;
                }
            }
        }
    }
}

fn read_input_line(prompt: &str) -> Result<String, InputHandleError> {
    let mut msg = String::new();
    println!("{}", prompt);
    match std::io::stdin().read_line(&mut msg) {
        Ok(_buff_size) => return Ok(msg),
        Err(_) => Err(InputHandleError),
    }
}

pub struct HostHandler {
    pub dispatch: dispatcher::Dispatcher,
    pub socket: TcpStream,
}

impl HostHandler {
    pub fn try_clone(&self) -> std::io::Result<Self> {
        let dispatch = self.dispatch.clone();
        match self.socket.try_clone() {
            Ok(socket) => Ok(HostHandler { dispatch, socket }),
            Err(e) => Err(e),
        }
    }
}

impl Handler for HostHandler {
    fn handle_text_msg(&mut self, msg: message::TextMessage) {
        println!("Received A Text Message: {}", msg.text);
    }

    fn handle_request_client_id(&mut self, msg: message::RequestClientID) {
        println!("Received a Request for Client ID");
        let mut socket_clone = self.socket.try_clone().expect("Failed to clone socket");
        self.dispatch.execute(move || {
            let id = read_input_line("Enter your ID:").expect("Error Reading Client ID from stdin");
            let response = message::RequestClientIDResponse { id };
            message::send_json(response, &mut socket_clone);
        })
    }

    fn handle_request_client_id_response(&mut self, msg: message::RequestClientIDResponse) {}
}
