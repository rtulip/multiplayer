use crate::game::GameID;
use crate::game::controller::GameController;
use crate::server_side::client::{ClientID, Client};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub mod client;
pub mod server;

pub type ClientHashmap = Arc<Mutex<HashMap<ClientID, Client>>>;
pub type GameHashmap = Arc<Mutex<HashMap<GameID, GameController>>>;
