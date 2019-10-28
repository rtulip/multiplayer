use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::threading::threadpool::Message;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) ->
        Worker {

        let thread = thread::spawn(move ||{
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);

                    job.call_box();
                },
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                },
            }
        
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
