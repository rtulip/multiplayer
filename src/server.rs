use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

use crate::threading::{threadpool, dispatcher};
use crate::game::controller;
use crate::{errors,message};

/// All client connections are held in a hashmap. The key to this Hashmap is the socket address, and the value is the TcpStream.Arc
/// Since multiple threads are going to be trying to add, remove, and maniuplate the values in hashmap, it must be protected behind
/// a mutex.
type ClientHashmap = Arc<Mutex<HashMap<SocketAddr, TcpStream>>>;
type GameList = Arc<Mutex<Vec<controller::GameController>>>;

/// Encapsulation of a server
pub struct Server {
    /// Servers have a ClientHashmap to asyncronously track client connections.
    clients: ClientHashmap,
    games: GameList,
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
    /// // server.start();
    /// 
    /// ```
    ///
    pub fn new(ip: &str, size: usize) -> Server {

        let listener = TcpListener::bind(ip).unwrap();
        let pool = threadpool::ThreadPool::new(size);

        let clients = HashMap::new();
        let clients = Arc::new(Mutex::new(clients));

        let games: Vec<controller::GameController> = Vec::new();
        let games = Arc::new(Mutex::new(games));

        Server{
            clients,
            games,
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

        let mut games = self.games.lock().unwrap();
        games.push(controller::GameController::new());
        std::mem::drop(games);

        // Publish data continually to each client. 
        let map_mutex = Arc::clone(&self.clients);
        let dispatch = self.pool.dispatcher.clone();
        self.pool.dispatcher.execute_loop(move || {

            publish_data(&map_mutex, &dispatch)

        });

        let games_clone = Arc::clone(&self.games);
        self.pool.dispatcher.execute_loop(move ||{
            dispatch_sys(&games_clone)
        });

        loop {

            if let Ok((stream, addr)) = self.listener.accept(){
                
                // Dispatch add_client().
                let stream_clone = stream.try_clone().expect("Unable to clone stream");
                let addr_clone = addr.clone();
                let map_clone = Arc::clone(&self.clients);
                let games_clone = Arc::clone(&self.games);
                self.pool.dispatcher.execute(move || {
                    add_client(addr_clone, stream_clone, map_clone, games_clone);
                });

                // Dispatch client_listen() on loop.
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

/// Listen to a client on a socket.
/// 
/// # Arguments
/// 
/// * 'mut socket' - The TcpStream of the client.
/// * 'addr' - The SocketAddr of the Client.
/// * 'map_mutex' - A reference to the ClientHashmap.
/// * 'dispatch' - A reference to a Dispatcher.
/// 
/// # Returns
/// 
/// * ConnectionStatus
fn client_listen(mut socket: TcpStream, addr: SocketAddr, map_mutex: &ClientHashmap, dispatch: &dispatcher::Dispatcher) -> errors::ConnectionStatus {
    
    let mut buff = vec![0; message::MSG_SIZE];

    // Read from socket.
    match socket.read(&mut buff){
        // Socket Disconnected.
        Ok(0) => {
            
            // Dispatch remove_client() to remove this client from the hashmap.
            let map_clone = Arc::clone(map_mutex);
            dispatch.execute(move || {
                remove_client(&addr, &map_clone);
            });
            Err(errors::ClientDisconnectError{
                addr,
            })

        },
        // Successfully read to the buffer.
        Ok(_) => {
            
            let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
            let msg = String::from_utf8(msg).expect("Invalid utf8 message");
            println!("MSG: {}", msg);

            // Dispatch send_message() to echo the message to the client.
            dispatch.execute(move || {
                message::send_text_message(&mut socket, msg);
            });

            // Say everything is Ok
            Ok(())

        },
        // Failed to read to buffer.
        Err(_) => {
            
            // Dispatch remove client to remove this client from the hashmap.
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

/// Adds a client to the HashMap.
/// 
/// # Arguments
///
/// * 'addr' - The SocketAddr which will serve as a key to the hashmap.
/// * 'socket' - The TcpStream of the client which will serve as the value to the hashmap.
/// * 'map_mutex' - A ClientHashMap where the client will be inserted.
fn add_client(addr: SocketAddr, socket: TcpStream, map_mutex: ClientHashmap, games: GameList){

    let mut clients = map_mutex.lock().unwrap();
    
    if let Some(_) = clients.insert(addr, socket.try_clone().expect("Unable to clone socket")){
        println!("Client {} already in map", addr);
    } else {
        println!("Client {} successfully added to map", addr);
    }

    std::mem::drop(clients);

    let mut games = games.lock().unwrap();
    let players = games[0].model.players.lock().unwrap();
    let len = players.len();
    std::mem::drop(players);
    games[0].model.add_player(len as u32, socket.try_clone().expect("Unable to clone socket"));


}

/// Removes a client from the HashMap.
/// 
/// # Arguments
///
/// * 'addr' - The key of the client.
/// * 'map_mutex' - A ClientHashMap from which the client will be removed.
fn remove_client(addr: &SocketAddr, map_mutex: &ClientHashmap) {

    let mut clients = map_mutex.lock().unwrap();
    if let Some(_) = clients.remove(addr){
        println!("Client {} successfully removed from map", addr);
    } else {
        println!("Faile to remove client  {} from map!", addr);
    }

}

/// Writes "Game Data" to each client periodically.
/// 
/// # Arguments
/// * 'map_mutex' - A reference to a ClientHashMap from which each client connection will be sent a message.
/// * 'diapatch' - A reference to a dispatcher which will execute the sending of messages.
/// 
/// # Returns
/// * ExpectedSuccess - This function shouldn't break out of a loop unless something very strange happens.
fn publish_data(map_mutex: &ClientHashmap, dispatch: &dispatcher::Dispatcher) -> errors::ExpectedSuccess {

    let mut clients = map_mutex.lock().unwrap();
    for (addr, socket) in clients.iter_mut(){

        let mut socket_clone = socket.try_clone().expect("Failed to clone socket");
        dispatch.execute(move || {
            message::send_text_message(&mut socket_clone, "Game Data");
        })

    }

    std::mem::drop(clients);
    thread::sleep(Duration::from_secs(2));

    Ok(())

}

fn dispatch_sys(games: &GameList) -> errors::ExpectedSuccess {

    let mut games = games.lock().unwrap();
    for game in games.iter_mut(){

        game.dispatch();

    }

    std::mem::drop(games);
    thread::sleep(Duration::from_secs(1));

    Ok(())

}