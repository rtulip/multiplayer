use crate::threading::job;
use std::sync::mpsc;

#[derive(Clone)]
pub struct Dispatcher{
    pub sender: mpsc::Sender<job::Message>,
}

impl Dispatcher{

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(job::Message::NewJob(job)).unwrap();
    }

    pub fn send(&self, msg: job::Message){
        self.sender.send(msg).unwrap();
    }

}