use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use crate::game::GameID;
use crate::state::State;
use crate::comms::handler::Handler;
use crate::comms::message;

pub type ClientID = String;
pub type ClientCollection = Arc<Mutex<HashSet<ClientID>>>;

#[derive(Clone, Copy)]
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

    pub fn try_clone(&self) -> std::io::Result<Client> {

        let id = self.id.clone();
        let state = self.state.clone();
        match (&self.socket, self.game_id) {
            (Some(socket), Some(game_id)) => {
                let socket = socket.try_clone()?;
                
                Ok(Client {
                    id,
                    socket: Some(socket),
                    game_id: Some(game_id),
                    state,

                })

            },
            (Some(socket), None) => {

                let socket = socket.try_clone()?;
                
                Ok(Client {
                    id,
                    socket: Some(socket),
                    game_id: None,
                    state,

                })

            },
            (None, Some(gid)) => {

                Ok(Client {
                    id,
                    socket: None,
                    game_id: Some(gid),
                    state,
                })

            },
            (None, None) => {

                Ok(Client {
                    id,
                    socket: None,
                    game_id: None,
                    state,

                })

            }
        }

    }

}

impl State for Client {
    type StateEnum = ClientState;
    fn change_state(&mut self, new_state: ClientState){
        self.state = new_state;
    }
}

pub struct ClientHandler {

}

impl Handler for ClientHandler {

    fn handle_text_msg(&mut self, msg: message::TextMessage){

    }

    fn handle_request_client_id(&mut self, msg: message::RequestClientID){

    }

    fn handle_request_client_id_response(&mut self, msg: message::RequestClientIDResponse){

    }

}