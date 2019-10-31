use std::net::TcpStream;

use crate::game::GameID;

pub type ClientID = u32;

pub struct Client {
    pub id: ClientID,
    pub socket: TcpStream,
    pub game_id: Option<GameID>, 
}