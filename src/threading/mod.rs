pub mod worker;
pub mod threadpool;
pub mod job;
pub mod dispatcher;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub type ConnectionCollection = Arc<Mutex<HashSet<u32>>>;