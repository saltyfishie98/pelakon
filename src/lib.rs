use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

pub trait Message {}

pub trait Received<T: Message>
where
    Self: Actor,
{
    fn process(&mut self, msg: T);
}

pub trait Actor: Sized + Send + 'static {
    fn start(self) -> Handle<Self> {
        Context::new().run(self)
    }

    fn hello(&self) {}
}

trait EnvelopeCallProxy<A: Actor> {
    fn process(&mut self, act: &mut A);
}

pub struct Handle<A: Actor> {
    pub tx: mpsc::Sender<Envelope<A>>,
    pub thread_handle: JoinHandle<()>,
}
impl<A: Actor> Handle<A> {
    pub fn send<M>(&self, msg: M)
    where
        M: Message + Send + 'static,
        A: Received<M>,
    {
        self.tx.send(Envelope::new(msg)).unwrap();
    }
}

pub struct Context<A: Actor> {
    mailbox: (mpsc::Sender<Envelope<A>>, mpsc::Receiver<Envelope<A>>),
}
impl<A: Actor> Context<A> {
    fn new() -> Self {
        Self {
            mailbox: mpsc::channel(),
        }
    }

    fn run(self, mut actor: A) -> Handle<A> {
        let tx = self.mailbox.0.clone();

        let thread_handle = thread::spawn(move || loop {
            while let Ok(mut msg) = self.mailbox.1.recv() {
                msg.data.process(&mut actor);
            }
        });

        Handle { tx, thread_handle }
    }
}

pub struct Envelope<A: Actor> {
    data: Box<dyn EnvelopeCallProxy<A> + Send>,
}
impl<A: Actor> Envelope<A> {
    fn new<M>(msg: M) -> Self
    where
        A: Received<M>,
        M: Message + Send + 'static,
    {
        Envelope {
            data: Box::new(EnvelopeProxy { msg: Some(msg) }),
        }
    }
}

struct EnvelopeProxy<M>
where
    M: Message + Send,
{
    msg: Option<M>,
}
impl<A, M> EnvelopeCallProxy<A> for EnvelopeProxy<M>
where
    M: Message + Send + 'static,
    A: Actor + Received<M>,
{
    fn process(&mut self, act: &mut A) {
        if let Some(msg) = self.msg.take() {
            <A as Received<M>>::process(act, msg);
        }
    }
}
