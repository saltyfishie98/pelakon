use crate::{Actor, Context};
use std::sync::mpsc;

pub(crate) struct Mailbox<A: Actor> {
    pub(crate) tx: mpsc::Sender<Mail<A>>,
    pub(crate) rx: mpsc::Receiver<Mail<A>>,
}
impl<A: Actor> From<(mpsc::Sender<Mail<A>>, mpsc::Receiver<Mail<A>>)> for Mailbox<A> {
    fn from(value: (mpsc::Sender<Mail<A>>, mpsc::Receiver<Mail<A>>)) -> Self {
        Self {
            tx: value.0,
            rx: value.1,
        }
    }
}

pub(crate) struct Mail<A: Actor> {
    pub(crate) content: Box<dyn MailContent<A> + Send>,
}

pub(crate) trait MailContent<A: Actor> {
    fn process(&mut self, act: &mut A, ctx: &mut Context<A>);
}
