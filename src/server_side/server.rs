use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

use crate::threading::{threadpool, dispatcher};
use crate::game::controller;
use crate::errors;
use crate::comms::message;
use crate::comms::handler::{Handler, DefaultHandler};
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

        // Publish data continually to each client. 
        let games = Arc::clone(&self.games);
        let clients = Arc::clone(&self.clients);
        let dispatch = self.pool.dispatcher.clone();
        self.pool.dispatcher.execute_loop(move || {

            publish_data(&games, &clients, &dispatch)

        });

        // Run game systems
        let games_clone = Arc::clone(&self.games);
        self.pool.dispatcher.execute_loop(move ||{
            dispatch_sys(&games_clone)
        });

        loop {
            // Wait for connections
            if let Ok((stream, _addr)) = self.listener.accept(){
                
                let dispatch = self.pool.dispatcher.clone();
                let clients = Arc::clone(&self.clients);
                let games = Arc::clone(&self.games);
                // Get client info
                self.pool.dispatcher.execute(move || {
                    connect_client(
                        stream.try_clone().expect("Failed to clone stream"), 
                        &dispatch, 
                        &clients, 
                        &games
                    )
                })

            }

        }

    }

}

fn connect_client(mut socket: TcpStream, dispatch: &dispatcher::Dispatcher, clients: &ClientHashmap, games: &GameHashMap) {

    let handler = DefaultHandler{};
    
    // Send request for Client ID.
    let msg = message::RequestClientID;
    message::send_json(msg, &mut socket);

    // Wait for Reply
    let mut buff = vec![0; message::MSG_SIZE];
    match socket.read(&mut buff){
        // Socket disconnected
        Ok(0) => (),
        // Received Message
        Ok(_) => {
            
            // Check if message is a valid RequestClientIDResponse
            if handler.is_type(&buff.clone(), message::REQUEST_CLIENT_ID_RESPONSE_IDENTIFIER) {
                
                // Parse the response
                let v = handler.parse_json(&buff);
                let data = v.get("data").unwrap();
                let data_string = serde_json::to_string(data).expect("Failed to convert data");

                let resp: message::RequestClientIDResponse = serde_json::from_str(data_string.as_str()).expect("Improper Resonse Format");
                
                // Create the client object
                let new_client = client::Client {
                    id: resp.id,
                    message_handler: client::ClientHandler{
                        socket: Some(socket.try_clone().expect("Failed to clone socket"))
                    },
                    game_id: None,
                    state: client::ClientState::Waiting,
                };

                let client_clone = new_client.try_clone().expect("Failed to clone Client");
                let clients_clone = Arc::clone(clients);
                // Add clients to the ClientsHashmap as playing no game. 
                dispatch.execute(move || {
                    add_client(client_clone, clients_clone);
                });

                let clients_clone = Arc::clone(clients);
                let games_clone = Arc::clone(games);
                let dispatch_clone = dispatch.clone();
                // Listen to the client.
                dispatch.execute_loop(move || {
                    client_listen(
                        new_client.try_clone().expect("Failed to clone new Client"),
                        &clients_clone,
                        &games_clone,
                        &dispatch_clone
                    )
                });

            } else {
                println!("Failed Handshake with client. Dropping");
            }          

        },
        Err(e) => (),
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
    client: client::Client,
    map_mutex: &ClientHashmap, 
    game_mutex: &GameHashMap, 
    dispatch: &dispatcher::Dispatcher
    ) -> errors::ConnectionStatus {

    if let Some(mut socket) = client.message_handler.socket{
        let mut buff = vec![0; message::MSG_SIZE];

        match socket.read(&mut buff) {
            Ok(0) => {
                // Dispatch remove_client() to remove this client from the hashmap.
                let id = client.id.clone();
                let map_clone = Arc::clone(map_mutex);
                let game_clone = Arc::clone(game_mutex);
                dispatch.execute(move || {
                    remove_client(&id, &map_clone, &game_clone);
                });
                Err(errors::ClientDisconnectError{
                    client_id: client.id,
                })
            },
            Ok(_) => {
                let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                println!("MSG: {}", msg);

                // Dispatch send_message() to echo the message to the client.
                dispatch.execute(move || {
                    let msg = message::TextMessage::new(msg);
                    message::send_json(msg, &mut socket);
                });

                // Say everything is Ok
                Ok(())
            },
            // Failed to read to buffer.
            Err(_) => {
                
                // Dispatch remove client to remove this client from the hashmap.
                let id = client.id.clone();
                let map_clone = Arc::clone(map_mutex);
                let game_clone = Arc::clone(game_mutex);
                dispatch.execute(move || {
                    remove_client(&id, &map_clone, &game_clone);
                });
                Err(errors::ClientDisconnectError{
                    client_id: client.id.clone(),
                })

            }
        }
    } else {
        Err(errors::ClientDisconnectError{
            client_id: client.id
        })
    }

}



/// Adds a client to the HashMap.
/// 
/// # Arguments
///
/// * 'addr' - The SocketAddr which will serve as a key to the hashmap.
/// * 'socket' - The TcpStream of the client which will serve as the value to the hashmap.
/// * 'clients' - A ClientHashMap where the client will be inserted.
fn add_client(client: client::Client, clients: ClientHashmap){

    let mut clients = clients.lock().unwrap();
    let id = client.id.clone();
    if let Some(_) = clients.insert(id.clone(), client){
        println!("Client {} already in map", id);
    } else {
        println!("Client {} successfully added to map", id);
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
                
                let clone = client.try_clone().expect("Failed to clone Client");
                if let Some(mut socket) = clone.message_handler.socket{

                    let msg = message::TextMessage::new("Game Data");
                    message::send_json(msg, &mut socket);

                }
                
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