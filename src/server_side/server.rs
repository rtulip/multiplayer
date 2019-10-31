use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

use crate::threading::{threadpool, dispatcher};
use crate::game::controller;
use crate::{errors,message};
use crate::server_side::client;

/// All client connections are held in a hashmap. The key to this Hashmap is the socket address, and the value is the TcpStream.Arc
/// Since multiple threads are going to be trying to add, remove, and maniuplate the values in hashmap, it must be protected behind
/// a mutex.
type GameID = u32;
type ClientHashmap = Arc<Mutex<HashMap<client::ClientID, client::Client>>>;
type GameHashMap = Arc<Mutex<HashMap<GameID, controller::GameController>>>;

/// Encapsulation of a server
pub struct Server {
    /// Servers have a ClientHashmap to asyncronously track client connections.
    clients: ClientHashmap,
    games: GameHashMap,
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

        let games: HashMap<u32, controller::GameController> = HashMap::new();
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
        games.insert(0,controller::GameController::new());
        std::mem::drop(games);

        // Publish data continually to each client. 
        let games = Arc::clone(&self.games);
        let clients = Arc::clone(&self.clients);
        let dispatch = self.pool.dispatcher.clone();
        self.pool.dispatcher.execute_loop(move || {

            publish_data(&games, &clients, &dispatch)

        });

        let games_clone = Arc::clone(&self.games);
        self.pool.dispatcher.execute_loop(move ||{
            dispatch_sys(&games_clone)
        });

        loop {

            if let Ok((stream, _addr)) = self.listener.accept(){
                
                let clients = self.clients.lock().unwrap();
                let client_id: client::ClientID = clients.len() as client::ClientID;

                std::mem::drop(clients);            

                let new_client = client::Client {
                    id: client_id,
                    socket: Some(stream.try_clone().expect("Unabled to clone stream")),
                    game_id: Some(0 as GameID),
                    state: client::ClientState::InGame,
                };
                
                // Dispatch add_client().
                let map_clone = Arc::clone(&self.clients);
                let games_clone = Arc::clone(&self.games);
                self.pool.dispatcher.execute(move || {
                    add_client(new_client.try_clone().expect("Failed to clone Client"), map_clone, games_clone);
                });

                // Dispatch client_listen() on loop.
                let dispatch_clone = self.pool.dispatcher.clone();
                let map_clone = Arc::clone(&self.clients);
                let game_clone = Arc::clone(&self.games);
                self.pool.dispatcher.execute_loop(move || {
                    client_listen(
                        client_id,
                        stream.try_clone().expect("Uable to clone stream"), 
                        &map_clone,
                        &game_clone,
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
fn client_listen(
    client_id: client::ClientID, 
    mut socket: TcpStream, 
    map_mutex: &ClientHashmap, 
    game_mutex: &GameHashMap, 
    dispatch: &dispatcher::Dispatcher
    ) -> errors::ConnectionStatus {
    
    let mut buff = vec![0; message::MSG_SIZE];

    // Read from socket.
    match socket.read(&mut buff){
        // Socket Disconnected.
        Ok(0) => {
            
            // Dispatch remove_client() to remove this client from the hashmap.
            let map_clone = Arc::clone(map_mutex);
            let game_clone = Arc::clone(game_mutex);
            dispatch.execute(move || {
                remove_client(&client_id, &map_clone, &game_clone);
            });
            Err(errors::ClientDisconnectError{
                client_id,
            })

        },
        // Successfully read to the buffer.
        Ok(_) => {
            
            let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
            let msg = String::from_utf8(msg).expect("Invalid utf8 message");
            println!("MSG: {}", msg);

            let mut TEMP_CLIENT = client::Client{
                id: 0,
                socket: Some(socket.try_clone().expect("Failed to clone socket")),
                game_id: Some(0),
                state: client::ClientState::PendingID,
            };

            // Dispatch send_message() to echo the message to the client.
            dispatch.execute(move || {
                TEMP_CLIENT.send_text_message(msg)
            });

            // Say everything is Ok
            Ok(())

        },
        // Failed to read to buffer.
        Err(_) => {
            
            // Dispatch remove client to remove this client from the hashmap.
            let map_clone = Arc::clone(map_mutex);
            let game_clone = Arc::clone(game_mutex);
            dispatch.execute(move || {
                remove_client(&client_id, &map_clone, &game_clone);
            });
            Err(errors::ClientDisconnectError{
                client_id,
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
fn add_client(client: client::Client, map_mutex: ClientHashmap, games: GameHashMap){

    let mut clients = map_mutex.lock().unwrap();
    let id = client.id;
    if let Some(_) = clients.insert(client.id, client){
        println!("Client {} already in map", id);
    } else {
        println!("Client {} successfully added to map", id);
    }

    std::mem::drop(clients);

    let mut games = games.lock().unwrap();
    let game_id: u32 = 0;
    if let Some(game) = games.get_mut(&game_id){
        let players = game.model.players.lock().unwrap();
        let len = players.len();
        std::mem::drop(players);
        game.model.add_player(len as u32);
    }

}

/// Removes a client from the HashMap.
/// 
/// # Arguments
///
/// * 'addr' - The key of the client.
/// * 'clients' - A ClientHashMap from which the client will be removed.
fn remove_client(client_id: &client::ClientID, clients: &ClientHashmap, games: &GameHashMap) {

    let mut clients = clients.lock().unwrap();
    if let Some(clnt) = clients.remove(client_id){
        
        println!("Client {} successfully removed from ClientMap", client_id);
        match clnt.game_id {
            
            Some(id) => {    
                let mut games = games.lock().unwrap();
                if let Some(game) = games.get_mut(&id) {
                    
                    let mut players = game.model.players.lock().unwrap();
                    if players.remove(client_id){
                        println!("Client {} succefully removed from PlayerList", client_id);
                    }
                }
            },
            None => (),
        }
    } else {
        println!("Faile to remove client  {} from map!", client_id);
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
fn publish_data(games: &GameHashMap, clients: &ClientHashmap, dispatch: &dispatcher::Dispatcher) -> errors::ExpectedSuccess {

    let mut games = games.lock().unwrap();
    for (_game_id, game) in games.iter_mut(){

        let players = game.model.players.lock().unwrap();
        for player_id in players.iter() {
            let mut clients = clients.lock().unwrap();

            if let Some(client) = clients.get_mut(player_id){
                
                client.send_text_message("Game Data");

            }
            std::mem::drop(clients);

        }
        std::mem::drop(players);

    }

    std::mem::drop(games);

    thread::sleep(Duration::from_secs(1));

    Ok(())

}

fn dispatch_sys(games: &GameHashMap) -> errors::ExpectedSuccess {

    let mut games = games.lock().unwrap();
    for (_game_id, game) in games.iter_mut(){

        game.dispatch();

    }

    std::mem::drop(games);
    thread::sleep(Duration::from_secs(1));

    Ok(())

}