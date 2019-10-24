use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

const MSG_SIZE: usize = 32;

fn main() {
    
    let stream = TcpStream::connect("127.0.0.1:7878").expect("Unable to connect to server");
    let mut rstream = stream.try_clone().expect("Unable to create read stream");
    let mut wstream = stream.try_clone().expect("Unable to create write stream");
    let (write_queue, writer) = mpsc::channel::<String>();
    let (send_queue, sender) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];

        match rstream.read(&mut buff){
            Ok(0) => {
                println!("Server left channel");
                write_queue.send("exit".to_owned()).expect("Unalbe to send exit message to reciever");
                break;
            },
            Ok(_) => {
                let msg = buff.clone().into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();

                println!("\nMSG as Bytes:   {:?}", msg.clone());
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                write_queue.send(msg).expect("failed to send msg to reciever");
            },
            Err(_) => {
                println!("server left the channel.");
                break;
            }
        }

        thread::sleep(Duration::from_millis(200));
    });

    thread::spawn(move || loop {
        let mut msg = String::new();
        println!("Type a message!");
        let _buff_size = std::io::stdin().read_line(&mut msg).unwrap();

        send_queue.send(msg).unwrap();

    });

    thread::spawn(move || loop {
        
        if let Ok(msg) = sender.try_recv(){
            println!("Sending Message: {}", msg);
            let buff = msg.clone().into_bytes();
            wstream.write_all(&buff).ok();
        }
    });

    loop {
        if let Ok(msg) = writer.try_recv() {
            match &msg[..]{
                "exit" => {
                    println!("Exiting");
                    break;
                },
                _ => println!("Msg: {}", msg),
            }
        } 
    }
    
} 