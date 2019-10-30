use std::net::{TcpListener, TcpStream, SocketAddr};
use crate::threading::{threadpool, dispatcher};
use crate::errors;
use crate::MSG_SIZE;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

pub struct Server {
    clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    listener: TcpListener,
    pool: threadpool::ThreadPool,
}

impl Server {

    pub fn new(ip: &str, size: usize) -> Server {

        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        let pool = threadpool::ThreadPool::new(size);

        let clients = HashMap::new();
        let clients = Arc::new(Mutex::new(clients));

        Server{
            clients,
            listener,
            pool,
        }    

    }

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

fn client_listen(mut socket: TcpStream, addr: SocketAddr, map_mutex: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>, dispatch: &dispatcher::Dispatcher) -> errors::ConnectionStatus {
    
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
                echo_message(&mut socket, &msg);
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

fn echo_message(socket: &mut TcpStream, message: &String){

    let buff = message.clone().into_bytes();
    socket.write_all(&buff).expect("Failed to write to socket!");

}

fn add_client(addr: SocketAddr, socket: TcpStream, map_mutex: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>){

    let mut clients = map_mutex.lock().unwrap();
    
    if let Some(_) = clients.insert(addr, socket){
        println!("Client {} already in map", addr);
    } else {
        println!("Client {} successfully added to map", addr);
    }

}

fn remove_client(addr: &SocketAddr, map_mutex: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>) {

    let mut clients = map_mutex.lock().unwrap();
    if let Some(_) = clients.remove(addr){
        println!("Client {} successfully removed from map", addr);
    } else {
        println!("FAILED TO REMOVE {} FROM MAP!!!!", addr);
    }

}

fn publish_data(map_mutex: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>, dispatch: &dispatcher::Dispatcher) -> errors::ExpectedSuccess {

    let mut clients = map_mutex.lock().unwrap();
    for (addr, socket) in clients.iter_mut(){

        let mut socket_clone = socket.try_clone().expect("Failed to clone socket");
        dispatch.execute(move || {
            let msg = "Game data".to_owned();
            echo_message(&mut socket_clone, &msg);
        })

    }

    std::mem::drop(clients);
    thread::sleep(Duration::from_secs(2));

    Ok(())

}