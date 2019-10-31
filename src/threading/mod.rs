pub mod worker;
pub mod threadpool;
pub mod job;
pub mod dispatcher;

use std::net::TcpStream;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ConnectionCollection = Arc<Mutex<HashMap<u32,Option<TcpStream>>>>;