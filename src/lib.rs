use core::fmt::Debug;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::ser::PrettyFormatter;
use std::io::{self, stdin, stdout, BufReader, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<P: Serialize> {
    pub src: String,
    pub dest: String,
    pub body: Body<P>,
}

impl<P: Serialize + Debug> Message<P> {
    pub fn reply<L: Serialize>(&self, payload: L, id: &mut usize) -> Message<L> {
        *id += 1;
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                payload,
                msg_id: Some(*id),
                in_reply_to: self.body.msg_id,
            },
        }
    }

    pub fn send(&self, out: &mut impl Write) -> anyhow::Result<()> {
        eprint!("{:?}", self);
        serde_json::to_writer(&mut *out, self)?;
        out.write(b"\n")?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<P> {
    #[serde(flatten)]
    pub payload: P,
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum InitPayload {
    Init(Init),
    InitOk {},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<P: Serialize, InitState> {
    fn new(state: InitState, init: Init) -> Self;
    fn handle(&mut self, message: Message<P>, out: &mut impl Write) -> anyhow::Result<()>;
}

//<'a, P: Deserialize<'a>, N: Node>
pub fn main_loop<'a, S, P: Deserialize<'a> + Serialize, N: Node<P, S>>(
    state: S,
) -> anyhow::Result<()> {
    let stdin = stdin();
    let mut stdout = stdout().lock();

    // let init_msg = serde_json::from_str::<Message<InitPayload>>(r#"{"src": "1", "dest":"2", "body": {"type":     "init","msg_id":   1,"node_id":  "n3","node_ids": ["n1", "n2", "n3"]}}"#)?;
    let init_msg =
        serde_json::from_str::<Message<InitPayload>>(stdin.lines().next().unwrap()?.as_str())?;

    let InitPayload::Init(init) = init_msg.body.payload.clone() else {
			panic!("wrong init msg: {:?}",init_msg);
		};

    let mut node = N::new(state, init);

    init_msg
        .reply(InitPayload::InitOk {}, &mut 1)
        .send(&mut stdout)?;

    let stdin = io::stdin().lock();
    for i in serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>() {
        node.handle(i?, &mut stdout)?;
    }
    Ok(())
}
