pub use macros::*;
use std::{
    sync::mpsc,
    thread::{self, JoinHandle, Result},
};

pub trait Message {}

pub trait Receives<T: Message>
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

pub struct Sender<A: Actor> {
    tx: mpsc::Sender<Envelope<A>>,
}
impl<A: Actor> Sender<A> {
    pub fn send<M>(&self, msg: M)
    where
        M: Message + Send + 'static,
        A: Receives<M>,
    {
        self.tx
            .send(Envelope(Box::new(MessageLetter { msg: Some(msg) })))
            .unwrap();
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
        A: Receives<M>,
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
    mailbox: Mailbox<A>,
    exit: bool,
}
impl<A: Actor> Context<A> {
    fn new() -> Self {
        Self {
            mailbox: mpsc::channel().into(),
            exit: false,
        }
    }

    fn run(mut self, mut actor: A) -> Handle<A> {
        let tx = self.mailbox.tx.clone();

        let thread_handle = thread::spawn(move || {
            actor.on_entry();
            while let Ok(mut letter) = self.mailbox.rx.recv() {
                letter.0.process(&mut actor, &mut self);
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

struct Mailbox<A: Actor> {
    tx: mpsc::Sender<Envelope<A>>,
    rx: mpsc::Receiver<Envelope<A>>,
}
impl<A: Actor> From<(mpsc::Sender<Envelope<A>>, mpsc::Receiver<Envelope<A>>)> for Mailbox<A> {
    fn from(value: (mpsc::Sender<Envelope<A>>, mpsc::Receiver<Envelope<A>>)) -> Self {
        Self {
            tx: value.0,
            rx: value.1,
        }
    }
}

pub struct Envelope<A: Actor>(Box<dyn EnvelopContent<A> + Send>);

trait EnvelopContent<A: Actor> {
    fn process(&mut self, act: &mut A, ctx: &mut Context<A>);
}

struct MessageLetter<M>
where
    M: Message + Send,
{
    msg: Option<M>,
}

impl<A, M> EnvelopContent<A> for MessageLetter<M>
where
    M: Message + Send + 'static,
    A: Actor + Receives<M>,
{
    fn process(&mut self, act: &mut A, ctx: &mut Context<A>) {
        if let Some(msg) = self.msg.take() {
            <A as Receives<M>>::process(act, msg, ctx);
        }
    }
}
