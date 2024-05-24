use std::{thread::sleep, time::Duration};

use pelakon::*;

enum MyMessage {
    Data(u32),
    Exit,
}
impl Message for MyMessage {}

struct MyMessage2 {
    data: u32,
}
impl Message for MyMessage2 {}

struct MyActor;
impl Actor for MyActor {
    fn on_entry(&mut self) {
        println!("hello!");
    }
    fn on_exit(&mut self) {
        println!("exited!");
    }
}
impl Received<MyMessage> for MyActor {
    fn process(&mut self, msg: MyMessage, ctx: &mut Context<MyActor>) {
        match msg {
            MyMessage::Data(data) => {
                if data == 0 {
                    sleep(Duration::from_secs(5));
                }
                println!("MyMassage.data: {}", data);
            }
            MyMessage::Exit => ctx.terminate(),
        };
    }
}
impl Received<MyMessage2> for MyActor {
    fn process(&mut self, msg: MyMessage2, _: &mut Context<MyActor>) {
        println!("MyMassage2.data: {}", msg.data);
    }
}

fn main() {
    let handle = MyActor.start();
    let sender = handle.clone_sender();
    ctrlc::set_handler(move || {
        sender.send(MyMessage::Exit);
    })
    .unwrap();

    handle.send(MyMessage::Data(0));
    handle.send(MyMessage2 { data: 42 });

    handle.join().unwrap();
}
