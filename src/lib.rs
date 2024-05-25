mod mailbox;

pub use macros::*;

use mailbox::*;
use std::{
    sync::mpsc,
    thread::{self, JoinHandle, Result},
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
            actor.on_entry();
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

    pub fn join(self) -> Result<()> {
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

/// The inbox of an actor to send messages to
pub struct Inbox<A: Actor> {
    tx: mpsc::Sender<Mail<A>>,
}
impl<A: Actor> Inbox<A> {
    pub fn send<M>(&self, msg: M)
    where
        M: Message + Send + 'static,
        A: Receives<M>,
    {
        self.tx
            .send(Mail {
                content: Box::new(MessageLetter { msg: Some(msg) }),
            })
            .unwrap();
    }
}
impl<A: Actor> Clone for Inbox<A> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}
