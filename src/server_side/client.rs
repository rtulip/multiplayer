use std::net::TcpStream;

use crate::game::GameID;
use crate::state::State;

pub type ClientID = u32;

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

impl State for Client {
    type StateEnum = ClientState;
    fn get_state(&mut self) -> &mut ClientState {
        &mut self.state
    }
}