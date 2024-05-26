#![allow(dead_code)]

use std::{error::Error, thread::sleep, time::Duration};

use pelakon::*;

#[derive(Message)]
enum ActorMessage {
    Data(String),
    Sleep(Duration),
    Exit,
}

struct MyActor {
    state: String,
}
impl Actor for MyActor {
    fn on_exit(&mut self) {
        println!("exited MyActor!");
    }
}
impl Receives<ActorMessage> for MyActor {
    fn process(&mut self, msg: ActorMessage, ctx: &mut Context<MyActor>) {
        match msg {
            ActorMessage::Data(data) => {
                self.state = data;
                println!("MyActor state = {}", self.state);
            }
            ActorMessage::Sleep(dur) => {
                sleep(dur);
            }
            ActorMessage::Exit => ctx.terminate(),
        };
    }
}

struct AnotherActor {
    state: String,
}
impl Actor for AnotherActor {
    fn on_exit(&mut self) {
        println!("exited AnotherActor!");
    }
}
impl Receives<ActorMessage> for AnotherActor {
    fn process(&mut self, msg: ActorMessage, ctx: &mut Context<AnotherActor>) {
        match msg {
            ActorMessage::Data(data) => {
                self.state = data;
                println!("AnotherActor state = {}", self.state);
            }
            ActorMessage::Sleep(dur) => {
                sleep(dur);
            }
            ActorMessage::Exit => ctx.terminate(),
        };
    }
}

fn main() {
    run_single().unwrap();
}

fn run_single() -> Result<(), Box<dyn Error>> {
    let actor1_contacts = MyActor {
        state: String::default(),
    }
    .start();

    {
        let actor1_ctrlc_tx = actor1_contacts.get_inbox_address();

        ctrlc::set_handler(move || {
            actor1_ctrlc_tx.send(ActorMessage::Exit).unwrap();
            println!("...cleanly exiting");
        })
        .expect("error setting ctrl-c handler!");
    }

    let actor1_tx = actor1_contacts.get_inbox_address();

    actor1_tx.send(ActorMessage::Sleep(Duration::from_secs(2)))?;
    actor1_tx.send(ActorMessage::Data(String::from("first")))?;
    actor1_tx.send(ActorMessage::Data(String::from("second")))?;
    actor1_tx.send(ActorMessage::Exit)?;
    actor1_tx.send(ActorMessage::Data(String::from("third")))?;

    actor1_contacts.join().unwrap();
    Ok(())
}

fn run_multiple() -> Result<(), Box<dyn Error>> {
    let actor1_contacts = MyActor {
        state: String::default(),
    }
    .start();

    let actor2_contacts = AnotherActor {
        state: String::default(),
    }
    .start();

    {
        let actor1_ctrlc_tx = actor1_contacts.get_inbox_address();
        let actor2_ctrlc_tx = actor2_contacts.get_inbox_address();

        ctrlc::set_handler(move || {
            actor1_ctrlc_tx.send(ActorMessage::Exit).unwrap();
            actor2_ctrlc_tx.send(ActorMessage::Exit).unwrap();
            println!("...cleanly exiting");
        })
        .expect("error setting ctrl-c handler!");
    }

    let actor1_tx = actor1_contacts.get_inbox_address();
    let actor2_tx = actor2_contacts.get_inbox_address();

    actor2_tx.send(ActorMessage::Data(String::from("start")))?;
    actor2_tx.send(ActorMessage::Sleep(Duration::from_secs(3)))?;
    actor2_tx.send(ActorMessage::Data(String::from("after 1st pause")))?;
    actor1_tx.send(ActorMessage::Data(String::from("start")))?;
    actor1_tx.send(ActorMessage::Sleep(Duration::from_secs(5)))?;
    actor2_tx.send(ActorMessage::Data(String::from("after 1st pause")))?;
    actor1_tx.send(ActorMessage::Data(String::from("after 2nd pause")))?;

    actor1_contacts.join().unwrap();
    actor2_contacts.join().unwrap();

    Ok(())
}
