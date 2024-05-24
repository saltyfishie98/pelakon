use std::{thread::sleep, time::Duration};

use pelakon::*;

#[derive(Message)]
enum MyMessage {
    Data(u32),
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
                if data == 0 {
                    println!("MyMassage.data == {data} (sleeping for 5 secs)");
                    sleep(Duration::from_secs(5));
                } else {
                    self.state += data;
                    println!("state = {}", self.state);
                }
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
    let handle = MyActor { state: 0 }.start();

    let ctrlc_tx = handle.clone_sender();
    ctrlc::set_handler(move || {
        ctrlc_tx.send(MyMessage::Exit);
        println!(" cleanly exiting");
    })
    .expect("error setting ctrl-c handler!");

    handle.send(MyMessage::Data(0));
    handle.send(MyMessage::Data(11));
    handle.send(MyMessage2 { data: 42 });
    handle.send(MyMessage::Data(0));
    handle.send(MyMessage::Data(22));
    handle.send(MyMessage2 { data: 42 });

    handle.join().unwrap();
}
