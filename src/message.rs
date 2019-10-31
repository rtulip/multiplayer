use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::TcpStream;
use std::io::prelude::*;

use crate::threading::dispatcher;

pub const MSG_SIZE: usize = 4096;

#[derive(Deserialize, Serialize)]
pub struct TextMessage {
    msg_type: String,
    text: String,
}

impl TextMessage{

    pub fn new<S: Into<String>>(text: S) -> TextMessage{

        TextMessage{
            msg_type: "Text".to_owned(),
            text: text.into(),
        }

    }
    
    pub fn handle(&self) {
        println!("Received Text Message: {}", self.text);
    }

}

pub fn send_text_message<S: Into<String>>(socket: &mut TcpStream, message: S){

    let text_msg = TextMessage::new(message);
    send_json(text_msg, socket);
    
}

pub fn send_json<M: Serialize>(val: M, socket: &mut TcpStream) {

    let json_string = serde_json::to_string(&val).expect("Unable to convert message to json");
    let buff = json_string.into_bytes();
    socket.write_all(&buff).expect("Failed to write to socket!");

}

pub fn receive_json(buff: &Vec<u8>, dispatch: dispatcher::Dispatcher) {
    
    let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
    let string = String::from_utf8(msg).expect("Invlaid utf8 message");

    let v: Value = serde_json::from_str(string.as_str()).expect("Unable to convert to json");

    match &v["msg_type"] {
        Value::String(text) => {
            match text.as_ref() {
                "Text" => {
                    let text_msg: TextMessage = serde_json::from_value(v).expect("Invalid Text Message");
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