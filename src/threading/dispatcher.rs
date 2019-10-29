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
            F: FnMut() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job::Message::NewJob(job)).unwrap();
    }

    pub fn send(&self, msg: job::Message){
        self.sender.send(msg).unwrap();
    }

}