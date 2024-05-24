use pelakon::*;

struct MyMessage {
    data: u32,
}
impl Message for MyMessage {}

struct MyMessage2 {
    data: u32,
}
impl Message for MyMessage2 {}

struct MyActor;
impl Actor for MyActor {}
impl Received<MyMessage> for MyActor {
    fn process(&mut self, msg: MyMessage) {
        println!("{}", msg.data);
    }
}
impl Received<MyMessage2> for MyActor {
    fn process(&mut self, msg: MyMessage2) {
        println!("{}", msg.data);
    }
}

fn main() {
    let handle = MyActor.start();
    handle.send(MyMessage { data: 42 });
    handle.send(MyMessage2 { data: 42 });
    handle.thread_handle.join().unwrap();
}
