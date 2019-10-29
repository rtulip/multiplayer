use crate::threading::job;
use std::sync::{mpsc,Arc,Mutex};

#[derive(Clone)]
pub struct Dispatcher {

    pub sender: mpsc::Sender<job::Message>,
    pub send_term: mpsc::Sender<job::Message>,
    pub recv_term: Arc<Mutex<mpsc::Receiver<job::Message>>>,

}

impl Dispatcher{

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job::Message::NewJob(job)).unwrap();
    }

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

    pub fn send(&self, msg: job::Message){
        self.sender.send(msg).unwrap();
    }

}