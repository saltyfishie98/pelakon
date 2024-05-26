mod mailbox;

pub use macros::*;

use mailbox::*;
use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

pub trait Actor: Sized + Send + 'static {
    fn start(self) -> ContactInfo<Self> {
        Context::new().run(self)
    }

    fn on_entry(&mut self) {}
    fn on_exit(&mut self) {}
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

    fn run(mut self, mut actor: A) -> ContactInfo<A> {
        let tx = self.mailbox.tx.clone();

        let thread_handle = thread::spawn(move || {
            while let Ok(mut letter) = self.mailbox.rx.recv() {
                letter.content.process(&mut actor, &mut self);
                if self.exit {
                    break;
                }
            }
            actor.on_exit();
        });

        ContactInfo {
            inbox: Inbox { tx },
            thread_handle,
        }
    }

    pub fn terminate(&mut self) {
        // call on_terminate
        self.exit = true;
    }
}

pub struct ContactInfo<A: Actor> {
    inbox: Inbox<A>,
    thread_handle: JoinHandle<()>,
}
impl<A: Actor> ContactInfo<A> {
    pub fn get_inbox_address(&self) -> Inbox<A> {
        self.inbox.clone()
    }

    pub fn join(self) -> thread::Result<()> {
        self.thread_handle.join()
    }
}

pub trait Message {}
pub trait Receives<T: Message>
where
    Self: Actor,
{
    fn process(&mut self, msg: T, ctx: &mut Context<Self>);
}

pub struct Inbox<A: Actor> {
    tx: mpsc::Sender<Mail<A>>,
}
impl<A: Actor> Inbox<A> {
    pub fn send<M>(&self, msg: M) -> Result<(), mpsc::SendError<()>>
    where
        M: Message + Send + 'static,
        A: Receives<M>,
    {
        self.tx
            .send(Mail {
                content: Box::new(ActorMessage { msg: Some(msg) }),
            })
            .map_err(|mpsc::SendError(_mail)| mpsc::SendError(()))
    }
}
impl<A: Actor> Clone for Inbox<A> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

struct ActorMessage<M>
where
    M: Message + Send,
{
    msg: Option<M>,
}
impl<A, M> MailContent<A> for ActorMessage<M>
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
