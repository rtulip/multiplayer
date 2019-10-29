use crate::threading::threadpool::Message;
use std::sync::mpsc;

#[derive(Clone)]
pub struct Dispatcher{
    pub sender: mpsc::Sender<Message>,
}

impl Dispatcher{

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }

    pub fn send(&self, msg: Message){
        self.sender.send(msg).unwrap();
    }

}