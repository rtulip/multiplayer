use crate::msg::{MSG_SIZE, Message};
use std::io::prelude::*;
use std::sync::mpsc;
use std::net::{TcpStream, TcpListener, SocketAddr};
use std::thread;
use std::collections::HashMap;
use std::time::Duration;

pub struct Server {

    server: TcpListener,
    clients: HashMap<SocketAddr, TcpStream>,

}

impl Server {

    pub fn new(ip: &str) -> Server {

        let server = TcpListener::bind(ip).expect("Listener failed to bind");
        server.set_nonblocking(true).expect("Unable to set server to nonblocing");
        
        Server {
            server, 
            clients: HashMap::new(),
        }

    }

    pub fn start(mut self, input: mpsc::Receiver<Message>, out: mpsc::Sender<Message>) {
        
        let snd = out.clone();
        thread::spawn(move || {

            loop {
                
                snd.send(Message::SendAll("Custom Message".to_owned()))
                    .expect("Failed to send count to reciever");
                thread::sleep(Duration::from_secs(1));

            }

        });

        loop {

            if let Ok((socket, addr)) = self.server.accept() {
                self.attach_client(socket, addr, out.clone());
            }
            

            if let Ok(msg) = input.recv(){
               match msg {
                   Message::SendAll(text) => {
                       self.send_all(text);
                   },
                   Message::SendPrivate(text, address) => {
                       self.send_private(address, text);
                   },
                   _ => (),
               }
            }

        
        }


    }


    fn send_all(&mut self, msg: String) {

        self.clients.iter_mut().for_each(
            |(_addr, client)| {
                let buff = msg.clone().into_bytes();
                client.write_all(&buff).ok();
            }
        )
        
    }

    fn send_private(&mut self, addr: SocketAddr, msg: String){

        match self.clients.get_mut(&addr) {
            Some(client) => {
                let buff = msg.clone().into_bytes();
                client.write_all(&buff).ok();
            },
            None => {
                println!("Failed to send {} to {}. Client not found in HashMap", msg, addr);
            }
        }

    }

    fn attach_client(&mut self, mut socket: TcpStream, addr: SocketAddr, sender: mpsc::Sender<Message>){

        println!("Client {:?} connected to the channel", addr);

        self.clients.insert(addr, socket.try_clone().expect("Failed to clone socket"));
        thread::spawn(move || loop {
            let mut buff = vec![0; MSG_SIZE];

            match socket.read(&mut buff){
                Ok(0) => {
                    println!("Client left channel");
                    break;
                },
                Ok(_) => {
                    let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();

                    println!("\nMSG as Bytes:   {:?}", msg.clone());
                    let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                    println!("\n{}: {:?}", addr, msg);
                    sender.send(Message::SendPrivate(msg, addr)).expect("failed to send msg to reciever");
                },
                Err(_) => {
                    println!("\nClient: {} left the channel.", addr);
                    break;
                }
            }

            thread::sleep(Duration::from_millis(200));
        });

        

    }

}