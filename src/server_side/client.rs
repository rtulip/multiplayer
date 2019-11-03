use std::collections::HashSet;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::comms::handler::{Handler, TryClone};
// use crate::comms::message;
use crate::game::GameID;
use crate::server_side::{ClientHashmap, GameHashmap};
use crate::state::State;

pub type ClientID = String;
pub type ClientCollection = Arc<Mutex<HashSet<ClientID>>>;

/// Describes the state of the Client
/// * Waiting - In lobby, not playing any game
/// * InQueue - In lobby, waiting for other players
/// * InGame - Actively playing the game
#[derive(Clone, Copy)]
pub enum ClientState {
    Waiting,
    InQueue,
    InGame,
}

/// Describes a server-side client
/// * id - Unique identifier
/// * message_handler - A ClientHandler to distribue and parse incoming and out going messages.
/// * game_id - The GameID of the game the client is currently playing. None if state is Waiting.
/// * state - The state of the client
pub struct Client {
    pub id: ClientID,
    pub socket: Option<TcpStream>,
    pub game_id: Option<GameID>,
    pub state: ClientState,
    pub clients: ClientHashmap,
    pub games: GameHashmap,
}

impl TryClone for Client {
    // Function to attempt to clone a Client.
    fn try_clone(&self) -> std::io::Result<Client> {
        let id = self.id.clone();
        let state = self.state.clone();
        let game_id = self.game_id.clone();
        let mut socket = None;
        match &self.socket {
            Some(sock) => {
                socket = Some(sock.try_clone()?);
            }
            None => (),
        };

        Ok(Client {
            id,
            state,
            socket,
            game_id,
            clients: Arc::clone(&self.clients),
            games: Arc::clone(&self.games),
        })
    }
}

impl State for Client {
    type StateEnum = ClientState;
    fn change_state(&mut self, new_state: ClientState) {
        self.state = new_state;
    }
}

impl Handler for Client {}
