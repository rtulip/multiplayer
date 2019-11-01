use std::collections::HashSet;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::comms::handler::Handler;
use crate::comms::message;
use crate::game::GameID;
use crate::state::State;

pub type ClientID = String;
pub type ClientCollection = Arc<Mutex<HashSet<ClientID>>>;

/// Describes the state of the Client
/// * Waiting - In lobby, not playing any game
/// * PendingGame - In lobby, waiting for other players
/// * InGame - Actively playing the game
#[derive(Clone, Copy)]
pub enum ClientState {
    Waiting,
    PendingGame,
    InGame,
}

/// Describes a server-side client
/// * id - Unique identifier
/// * message_handler - A ClientHandler to distribue and parse incoming and out going messages.
/// * game_id - The GameID of the game the client is currently playing. None if state is Waiting.
/// * state - The state of the client
pub struct Client {
    pub id: ClientID,
    pub message_handler: ClientHandler,
    pub game_id: Option<GameID>,
    pub state: ClientState,
}

impl Client {
    // Function to attempt to clone a Client.
    pub fn try_clone(&self) -> std::io::Result<Client> {
        let id = self.id.clone();
        let state = self.state.clone();
        let message_handler = self.message_handler.try_clone()?;
        let game_id = self.game_id.clone();

        Ok(Client {
            id,
            state,
            message_handler,
            game_id,
        })
    }
}

impl State for Client {
    type StateEnum = ClientState;
    fn change_state(&mut self, new_state: ClientState) {
        self.state = new_state;
    }
}

pub struct ClientHandler {
    pub socket: Option<TcpStream>,
}

impl ClientHandler {
    pub fn try_clone(&self) -> std::io::Result<Self> {
        match &self.socket {
            Some(socket) => Ok(ClientHandler {
                socket: Some(socket.try_clone()?),
            }),
            None => Ok(ClientHandler { socket: None }),
        }
    }
}
impl Handler for ClientHandler {
    fn handle_text_msg(&mut self, msg: message::TextMessage) {}

    fn handle_request_client_id(&mut self, msg: message::RequestClientID) {}

    fn handle_request_client_id_response(&mut self, msg: message::RequestClientIDResponse) {}
}
