use std::{thread::sleep, time::Duration};

use pelakon::*;

#[derive(Message)]
enum MyMessage {
    Data(u32),
    Sleep(Duration),
    Exit,
}

#[derive(Message)]
struct MyMessage2 {
    data: u32,
}

struct MyActor {
    state: u32,
}
impl Actor for MyActor {
    fn on_entry(&mut self) {
        println!("hello!");
    }
    fn on_exit(&mut self) {
        println!("exited!");
    }
}
impl Receives<MyMessage> for MyActor {
    fn process(&mut self, msg: MyMessage, ctx: &mut Context<MyActor>) {
        match msg {
            MyMessage::Data(data) => {
                self.state += data;
                println!("state = {}", self.state);
            }
            MyMessage::Sleep(dur) => {
                println!("sleeping for {} seconds", dur.as_secs_f32());
                sleep(dur);
            }
            MyMessage::Exit => ctx.terminate(),
        };
    }
}
impl Receives<MyMessage2> for MyActor {
    fn process(&mut self, msg: MyMessage2, _: &mut Context<MyActor>) {
        println!("MyMassage2.data: {}", msg.data);
    }
}

fn main() {
    let contacts = MyActor { state: 0 }.start();

    {
        let ctrlc_tx = contacts.get_inbox_address();

        ctrlc::set_handler(move || {
            ctrlc_tx.send(MyMessage::Exit);
            println!(" cleanly exiting");
        })
        .expect("error setting ctrl-c handler!");
    }

    let main_tx = contacts.get_inbox_address();
    main_tx.send(MyMessage::Data(11));
    main_tx.send(MyMessage::Sleep(Duration::from_secs(4)));
    main_tx.send(MyMessage2 { data: 42 });
    main_tx.send(MyMessage::Data(22));
    main_tx.send(MyMessage2 { data: 43 });
    main_tx.send(MyMessage::Sleep(Duration::from_secs(5)));

    contacts.join().unwrap();
}
