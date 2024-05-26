use pelakon::*;

#[derive(Message)]
enum Msg {
    Data(String),
    Exit,
}

#[derive(Default)]
struct Parent {
    child: Option<ContactInfo<Child>>,
}
impl Actor for Parent {}
impl Receives<Msg> for Parent {
    fn process(&mut self, msg: Msg, ctx: &mut Context<Self>) {
        match msg {
            Msg::Data(d) => {
                if let Some(child) = &self.child {
                    child.get_inbox_address().send(Msg::Data(d));
                } else {
                    println!("no child");
                }
            }
            Msg::Exit => {
                if let Some(child) = &self.child {
                    child.get_inbox_address().send(Msg::Exit);
                    ctx.terminate();
                } else {
                    ctx.terminate();
                }
            }
        }
    }
}

struct Child;
impl Actor for Child {}
impl Receives<Msg> for Child {
    fn process(&mut self, msg: Msg, ctx: &mut Context<Self>) {
        match msg {
            Msg::Data(d) => println!("{d}"),
            Msg::Exit => ctx.terminate(),
        }
    }
}

fn main() {
    let parent = Parent {
        child: Some(Child.start()),
    }
    .start();

    let tx = parent.get_inbox_address();
    tx.send(Msg::Data("hello".into()));
    tx.send(Msg::Exit);

    parent.join().unwrap();
}
