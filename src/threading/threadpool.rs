use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::threading::dispatcher::Dispatcher;
use crate::threading::job;
use crate::threading::worker::Worker;

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

        let (send_term, recv_term) = mpsc::channel();
        let recv_term = Arc::new(Mutex::new(recv_term));

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        let dispatcher = Dispatcher {
            sender,
            send_term,
            recv_term,
        };

        ThreadPool {
            workers,
            dispatcher,
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.dispatcher.send(job::Message::Terminate);
            self.dispatcher
                .send_term
                .send(job::Message::Terminate)
                .unwrap();
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
