use std::thread;
use std::sync::{mpsc, Arc, Mutex};

use crate::threading::job;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<job::Message>>>) ->
        Worker {

        let thread = thread::spawn(move || loop {
            println!("Worker {} waiting for job", id);
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                job::Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job.call_box();
                },
                job::Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                },
            }
        
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
