use crate::{Actor, Context, Message, Receives};
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
    pub(crate) content: Box<dyn Letter<A> + Send>,
}

pub(crate) trait Letter<A: Actor> {
    fn process(&mut self, act: &mut A, ctx: &mut Context<A>);
}

pub(crate) struct MessageLetter<M>
where
    M: Message + Send,
{
    pub(crate) msg: Option<M>,
}
impl<A, M> Letter<A> for MessageLetter<M>
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
