use crate::threading::job;
use std::sync::{mpsc,Arc,Mutex};

#[derive(Clone)]
pub struct Dispatcher {

    pub sender: mpsc::Sender<job::Message>,
    pub send_term: mpsc::Sender<job::Message>,
    pub recv_term: Arc<Mutex<mpsc::Receiver<job::Message>>>,

}

impl Dispatcher{

    /// Sends a one-time job to a worker.
    /// 
    /// # Example
    /// 
    /// ```
    /// extern crate multiplayer;
    /// use multiplayer::threading::threadpool;
    /// 
    /// let pool = threadpool::ThreadPool::new(5);
    /// for i in 0..10 {
    ///     let num = i.clone();  
    ///     pool.dispatcher.execute(move || {
    ///        println!("Num: {}", i);
    ///     });
    /// }
    /// ```
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job::Message::NewJob(job)).unwrap();
    }

    /// Sends a job to a worker which will be repeated until an error is thrown.
    /// 
    /// # Example
    /// 
    /// ```
    /// extern crate multiplayer;
    /// use multiplayer::threading::threadpool;
    /// use multiplayer::errors;
    /// use std::time::Duration;
    /// use std::thread;
    /// 
    /// fn repeat(num: &i32) -> errors::ExpectedSuccess {
    ///     println!("Num: {}", num);
    ///     thread::sleep(Duration::from_secs(1));
    ///     Ok(())
    /// }
    /// 
    /// let pool = threadpool::ThreadPool::new(5);
    /// for i in 0..3 {
    ///     pool.dispatcher.execute_loop(move || {
    ///         let num = i.clone();
    ///         repeat(&num)
    ///     });
    /// }
    /// 
    /// ```
    pub fn execute_loop<F, T, E>(&self, mut f: F)
        where
            F: FnMut() -> Result<T, E> + Send + 'static,
            E: std::error::Error,
    {
        let rcv = Arc::clone(&self.recv_term);

        let job2 = Box::new(move|| loop {

            let result = rcv.lock().unwrap().try_recv();

            match result {
                Ok(_) => break,
                Err(_) => {
                    match f() {
                        Err(e) => {
                            println!("{}", e);
                            break;
                        },
                        _ => (),
                    }
                }
            }

        });


        self.sender.send(job::Message::NewJob(job2)).unwrap();
    }

    /// Sends a generic job to a worker.
    /// Used to terminate the workers.
    pub fn send(&self, msg: job::Message){
        self.sender.send(msg).unwrap();
    }

}