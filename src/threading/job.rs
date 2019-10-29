pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnMut()> FnBox for F {
    fn call_box(mut self: Box<F>) {
        (*self)()
    }
}

pub type Job = Box<FnBox + Send + 'static>;

pub enum Message {
    NewJob(Job),
    Terminate,
}