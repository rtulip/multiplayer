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
    pub message_handler: ClientHandler,
    pub game_id: Option<GameID>,
    pub state: ClientState
}

impl Client {

    pub fn try_clone(&self) -> std::io::Result<Client> {

        let id = self.id.clone();
        let state = self.state.clone();
        let message_handler = self.message_handler.try_clone()?;
        let game_id = self.game_id.clone();

        Ok(Client{
            id,
            state,
            message_handler,
            game_id
        })
        
    }

}

impl State for Client {
    type StateEnum = ClientState;
    fn change_state(&mut self, new_state: ClientState){
        self.state = new_state;
    }
}

pub struct ClientHandler {
    pub socket: Option<TcpStream>,
}

impl ClientHandler{
    
    pub fn try_clone(&self) -> std::io::Result<Self>{

        match &self.socket{
            Some(socket) => {
                Ok(ClientHandler{
                    socket: Some(socket.try_clone()?),
                })
            }
            None => {
                Ok(ClientHandler{
                    socket: None,
                })
            }
        } 
    }

}
impl Handler for ClientHandler {

    fn handle_text_msg(&mut self, msg: message::TextMessage){

    }

    fn handle_request_client_id(&mut self, msg: message::RequestClientID){

    }

    fn handle_request_client_id_response(&mut self, msg: message::RequestClientIDResponse){

    }

}