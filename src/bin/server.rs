extern crate multiplayer;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::time::Duration;
use std::thread;
use std::sync::mpsc;

const MSG_SIZE: usize = 32;

fn main() {
    
    let server = TcpListener::bind("127.0.0.1:7878").expect("Listener failed to bind");
    server.set_nonblocking(true).expect("Unable to set server to non blocking");

    let mut clients: Vec<TcpStream> = vec![];
    let (sender, reciever) = mpsc::channel::<String>();

    let snd = sender.clone();
    thread::spawn(move || {

        let mut count = 0;
        loop {
            
            count += 1;
            snd.send(count.to_string()).expect("Failed to send count to reciever");
            thread::sleep(Duration::from_secs(1));

        }

    });
    
    loop {

        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {:?} connected to the channel", addr);

            let sender = sender.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

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
                        sender.send(msg).expect("failed to send msg to reciever");
                    },
                    Err(_) => {
                        println!("\nClient: {} left the channel.", addr);
                        break;
                    }
                }

                thread::sleep(Duration::from_millis(200));
            });
        }
        

        if let Ok(msg) = reciever.recv(){
            println!("Recieved!: {}", msg);
            clients = clients.into_iter().filter_map(|mut client| {
                let buff = msg.clone().into_bytes();
                buff.clone().resize(buff.len(), 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        thread::sleep(Duration::from_millis(200));
    }


}