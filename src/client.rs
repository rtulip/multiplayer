use crate::msg::{MSG_SIZE, Message};
use std::io::prelude::*;
use std::sync::mpsc;
use std::net::TcpStream;
use std::thread;

pub struct Client {

    stream: TcpStream,

}

impl Client {

    pub fn new(ip: &str) -> Client {

        let stream = TcpStream::connect(ip).expect("Unable to connect to server");

        Client {
            stream, 
        }

    }

    pub fn start(mut self, input: mpsc::Receiver<Message>, out: mpsc::Sender<Message>) {

        let mut d = Client {
            stream: self.stream.try_clone().expect("Failed to clone stream"),
        };

        thread::spawn(move || loop {
            
            let mut buff = vec![0; MSG_SIZE];

            match self.stream.read(&mut buff) {
                Ok(0) => {
                    println!("Source Disconected!");
                    break;
                },
                Ok(_) => {
                    self.forward(buff, &out);
                },
                Err(_) => {
                    println!("Error: halting listener");
                    break;
                }
            }

        });

        thread::spawn(move || loop {
            if let Ok(msg) = input.try_recv(){
                
                match msg {
                    
                    Message::Recv(string) => {
                        d.handle_recv(string);
                    },
                    Message::Send(string) => {
                        d.handle_send(string);
                    },
                    Message::Terminate => break,
                    _ => (),

                }
                
            } 
       });

    }

    fn forward(&self, buff: Vec<u8>, out: &mpsc::Sender<Message>) {
        
        let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
        println!("\nMSG as Bytes:   {:?}", msg.clone());
        let string = String::from_utf8(msg).expect("Invlaid utf8 message");
        let msg = Message::Recv(string);

        out.send(msg).expect("failed to send msg");

    }

    fn handle_recv(&self, string: String) {
        println!("Recieved: {}", string);
    }

    fn handle_send(&mut self, string: String) {
        let buff = string.clone().into_bytes();
        self.stream.write_all(&buff).expect("Problem sending message");
    }

}

pub fn start_input_handler(out: mpsc::Sender<Message>) {

    thread::spawn(move || loop {
        let mut msg = String::new();
        println!("Type a message!");
        let _buff_size = std::io::stdin().read_line(&mut msg).unwrap();

        out.send(Message::Send(msg)).unwrap();
    });

}