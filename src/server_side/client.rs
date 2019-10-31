use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use crate::game::GameID;
use crate::state::State;

pub type ClientID = u32;
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

        let id = self.id;
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
    fn get_state(&mut self) -> &mut ClientState {
        &mut self.state
    }
}