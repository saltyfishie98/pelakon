use std::{
    sync::mpsc,
    thread::{self, JoinHandle, Result},
};

pub trait Message {}

pub trait Received<T: Message>
where
    Self: Actor,
{
    fn process(&mut self, msg: T, ctx: &mut Context<Self>);
}

pub trait Actor: Sized + Send + 'static {
    fn start(self) -> Handle<Self> {
        Context::new().run(self)
    }

    fn on_entry(&mut self) {}
    fn on_exit(&mut self) {}
}

trait EnvelopeCallProxy<A: Actor> {
    fn process(&mut self, act: &mut A, ctx: &mut Context<A>);
}

pub struct Sender<A: Actor> {
    tx: mpsc::Sender<Envelope<A>>,
}
impl<A: Actor> Sender<A> {
    pub fn send<M>(&self, msg: M)
    where
        M: Message + Send + 'static,
        A: Received<M>,
    {
        self.tx.send(Envelope::new(msg)).unwrap();
    }
}
impl<A: Actor> Clone for Sender<A> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub struct Handle<A: Actor> {
    sender: Sender<A>,
    thread_handle: JoinHandle<()>,
}
impl<A: Actor> Handle<A> {
    pub fn send<M>(&self, msg: M)
    where
        M: Message + Send + 'static,
        A: Received<M>,
    {
        self.sender.send(msg);
    }

    pub fn clone_sender(&self) -> Sender<A> {
        self.sender.clone()
    }

    pub fn join(self) -> Result<()> {
        self.thread_handle.join()
    }
}

pub struct Context<A: Actor> {
    mailbox: (mpsc::Sender<Envelope<A>>, mpsc::Receiver<Envelope<A>>),
    exit: bool,
}
impl<A: Actor> Context<A> {
    fn new() -> Self {
        Self {
            mailbox: mpsc::channel(),
            exit: false,
        }
    }

    fn run(mut self, mut actor: A) -> Handle<A> {
        let tx = self.mailbox.0.clone();

        let thread_handle = thread::spawn(move || {
            actor.on_entry();
            while let Ok(mut msg) = self.mailbox.1.recv() {
                msg.data.process(&mut actor, &mut self);
                if self.exit {
                    break;
                }
            }
            actor.on_exit();
        });

        Handle {
            sender: Sender { tx },
            thread_handle,
        }
    }

    pub fn terminate(&mut self) {
        self.exit = true;
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
    fn process(&mut self, act: &mut A, ctx: &mut Context<A>) {
        if let Some(msg) = self.msg.take() {
            <A as Received<M>>::process(act, msg, ctx);
        }
    }
}
