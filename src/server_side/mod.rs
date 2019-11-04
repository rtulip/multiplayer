use crate::game::controller::GameController;
use crate::game::GameID;
use crate::server_side::client::{Client, ClientID};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod client;
pub mod server;

pub type ClientHashmap = Arc<Mutex<HashMap<ClientID, Client>>>;
pub type GameHashmap = Arc<Mutex<HashMap<GameID, GameController>>>;
