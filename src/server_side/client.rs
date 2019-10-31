use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::io::prelude::*;

use serde::Serialize;

use crate::game::GameID;
use crate::state::State;
use crate::message;

pub type ClientID = u32;
pub type ClientCollection = Arc<Mutex<HashSet<ClientID>>>;

pub enum ClientState{
    PendingID,
    Waiting,
    PendingGame,
    InGame,
}

pub struct Client {
    pub id: ClientID,
    pub socket: Option<TcpStream>,
    pub game_id: Option<GameID>,
    pub state: ClientState
}

impl Client {

    pub fn send_text_message<S: Into<String>>(&mut self, message: S){

        let text_msg = message::TextMessage::new(message);
        self.send_json(text_msg);
        
    }

    pub fn send_json<M: Serialize>(&mut self, val: M) {

        if let Some(socket) = &self.socket {
            let mut socket_clone = socket.try_clone().expect("Failed to clone Socket");
            let json_string = serde_json::to_string(&val).expect("Unable to convert message to json");
            let buff = json_string.into_bytes();
            socket_clone.write_all(&buff).expect("Failed to write to socket!");
        }
    }

}

impl State for Client {
    type StateEnum = ClientState;
    fn get_state(&mut self) -> &mut ClientState {
        &mut self.state
    }
}