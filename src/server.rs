use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

use crate::threading::{threadpool, dispatcher};
use crate::errors;
use crate::MSG_SIZE;

/// All client connections are held in a hashmap. The key to this Hashmap is the socket address, and the value is the TcpStream.Arc
/// Since multiple threads are going to be trying to add, remove, and maniuplate the values in hashmap, it must be protected behind
/// a mutex.
type ClientHashmap = Arc<Mutex<HashMap<SocketAddr, TcpStream>>>;

/// Encapsulation of a server
pub struct Server {
    /// Servers have a ClientHashmap to asyncronously track client connections.
    clients: ClientHashmap,
    /// Servers have a TcpListener to listen for new client connections.
    listener: TcpListener,
    /// Servers have a ThreadPool which dispatches jobs.
    pool: threadpool::ThreadPool,
}

impl Server {
    /// Returns a new server. 
    /// 
    /// # Arguments:
    ///
    /// * 'ip' - A string slice which the TcpListener will bind to.
    /// * 'size' - The size of the ThreadPool. i.e. how many worker threads will be active.
    /// 
    /// # Example: 
    /// ```
    /// extern crate multiplayer;
    /// use multiplayer::server;
    /// 
    /// let server = server::Server::new("127.0.0.1:7878", 100);
    /// server.start();
    /// ```
    ///
    pub fn new(ip: &str, size: usize) -> Server {

        let listener = TcpListener::bind(ip).unwrap();
        let pool = threadpool::ThreadPool::new(size);

        let clients = HashMap::new();
        let clients = Arc::new(Mutex::new(clients));

        Server{
            clients,
            listener,
            pool,
        }    

    }

    /// Starts the server and various jobs.
    /// 
    /// # Jobs
    /// 
    /// * 'Publish Data' - Periodically sends data to all connected clients.
    ///     * Loops until UnexpectedError.
    ///     * Stars more jobs:
    ///         * 'Send Message' - Sends a message to a connected client.
    /// * 'Add Client' - Adds a newly connected client to the ClientHashMap.
    /// * 'Client Listen' - Listens to incoming messages from a connected client.
    ///     * Loops until ClientDisconnectError.
    ///     * Starts more jobs:
    ///         * 'Remove Client' Removes a client from the ClientHashMap.
    ///         * 'Send Message' - Sends a message to a connected client.
    pub fn start(self) {

        let map_mutex = Arc::clone(&self.clients);
        let dispatch = self.pool.dispatcher.clone();
        self.pool.dispatcher.execute_loop(move || {

            publish_data(&map_mutex, &dispatch)

        });

        loop {

            if let Ok((stream, addr)) = self.listener.accept(){
                
                let stream_clone = stream.try_clone().expect("Unable to clone stream");
                let addr_clone = addr.clone();
                let map_clone = Arc::clone(&self.clients);
                self.pool.dispatcher.execute(move || {
                    add_client(addr_clone, stream_clone, map_clone);
                });

                let dispatch_clone = self.pool.dispatcher.clone();
                let map_clone = Arc::clone(&self.clients);
                self.pool.dispatcher.execute_loop(move || {
                    client_listen(
                        stream.try_clone().expect("Uable to clone stream"), 
                        addr,
                        &map_clone,
                        &dispatch_clone
                    )
                });
                
            }

        }

    }

}

fn client_listen(mut socket: TcpStream, addr: SocketAddr, map_mutex: &ClientHashmap, dispatch: &dispatcher::Dispatcher) -> errors::ConnectionStatus {
    
    let mut buff = vec![0; MSG_SIZE];

    match socket.read(&mut buff){
        Ok(0) => {
            
            let map_clone = Arc::clone(map_mutex);
            dispatch.execute(move || {
                remove_client(&addr, &map_clone);
            });
            Err(errors::ClientDisconnectError{
                addr,
            })

        },
        Ok(_) => {
            
            let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
            let msg = String::from_utf8(msg).expect("Invalid utf8 message");
            println!("MSG: {}", msg);

            dispatch.execute(move || {
                send_message(&mut socket, &msg);
            });

            Ok(())

        },
        Err(_) => {
            
            let map_clone = Arc::clone(map_mutex);
            dispatch.execute(move || {
                remove_client(&addr, &map_clone);
            });
            Err(errors::ClientDisconnectError{
                addr,
            })

        }
    }

}

fn send_message(socket: &mut TcpStream, message: &String){

    let buff = message.clone().into_bytes();
    socket.write_all(&buff).expect("Failed to write to socket!");

}

fn add_client(addr: SocketAddr, socket: TcpStream, map_mutex: ClientHashmap){

    let mut clients = map_mutex.lock().unwrap();
    
    if let Some(_) = clients.insert(addr, socket){
        println!("Client {} already in map", addr);
    } else {
        println!("Client {} successfully added to map", addr);
    }

}

fn remove_client(addr: &SocketAddr, map_mutex: &ClientHashmap) {

    let mut clients = map_mutex.lock().unwrap();
    if let Some(_) = clients.remove(addr){
        println!("Client {} successfully removed from map", addr);
    } else {
        println!("Faile to remove client  {} from map!", addr);
    }

}

fn publish_data(map_mutex: &ClientHashmap, dispatch: &dispatcher::Dispatcher) -> errors::ExpectedSuccess {

    let mut clients = map_mutex.lock().unwrap();
    for (addr, socket) in clients.iter_mut(){

        let mut socket_clone = socket.try_clone().expect("Failed to clone socket");
        dispatch.execute(move || {
            let msg = "Game data".to_owned();
            send_message(&mut socket_clone, &msg);
        })

    }

    std::mem::drop(clients);
    thread::sleep(Duration::from_secs(2));

    Ok(())

}