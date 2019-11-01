pub mod dispatcher;
pub mod job;
pub mod threadpool;
pub mod worker;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub type ConnectionCollection = Arc<Mutex<HashSet<u32>>>;
