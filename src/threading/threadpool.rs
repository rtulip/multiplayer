use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::threading::worker::Worker;
use crate::threading::dispatcher::Dispatcher;
use crate::threading::job::Job;

pub enum Message {
    NewJob(Job),
    Terminate,
}

pub fn new_job<F>(f: F) -> Message
    where
        F: FnOnce() + Send + 'static
{
    let job = Box::new(f);

    Message::NewJob(job)
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    pub dispatcher: Dispatcher,
}


impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool {
            workers,
            dispatcher: Dispatcher{
                sender
            },
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.dispatcher.send(Message::Terminate);
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

