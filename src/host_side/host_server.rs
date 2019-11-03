use std::io::prelude::*;

use crate::comms::handler::{Handler, TryClone};
use crate::comms::message;
use crate::host_side::host_client::HostClient;
use crate::threading::threadpool;

pub struct HostServer {
    client: HostClient,
    pool: threadpool::ThreadPool,
}

impl HostServer {
    pub fn new(ip: &str, size: usize) -> HostServer {
        let pool = threadpool::ThreadPool::new(size);
        let client = HostClient::new(ip, pool.dispatcher.clone());
        HostServer { client, pool }
    }

    pub fn start(mut self) {
        loop {
            let mut buff = vec![0; message::MSG_SIZE];
            let mut client_clone = self.client.try_clone().expect("Failed to clone HostClient");
            match self.client.socket.read(&mut buff) {
                Ok(0) => {
                    println!("Source Disconected!");
                    break;
                }
                Ok(_) => {
                    self.pool.dispatcher.execute(move || {
                        client_clone.receive_json(&buff);
                    });
                }
                Err(_) => {
                    println!("Error: halting listener");
                    break;
                }
            }
        }
    }
}
