pub mod utils;

use core::fmt::Debug;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::ser::PrettyFormatter;
use std::{
    io::{self, stdin, stdout, BufReader, Write},
    sync::mpsc::channel,
    thread, time,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<P: Serialize + Debug> {
    pub src: String,
    pub dest: String,
    pub body: Body<P>,
}

impl<P: Serialize + Debug> Message<P> {
    pub fn new(src: String, dest: String, payload: P, id: &mut usize) -> Message<P> {
        *id += 1;
        Message {
            src,
            dest,
            body: Body {
                payload,
                msg_id: Some(*id),
                in_reply_to: None,
            },
        }
    }

    pub fn reply<L: Serialize + Debug>(&self, payload: L, id: &mut usize) -> Message<L> {
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
        eprintln!("out: {:?}", self);
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

pub trait Node<P: Serialize + Debug, InitState> {
    fn new(state: InitState, init: Init) -> Self;
    fn handle(&mut self, message: Message<P>, out: &mut impl Write) -> anyhow::Result<()>;
    fn timed_call(&mut self, out: &mut impl Write) -> anyhow::Result<()> {
        Ok(())
    }
}

//<'a, P: Deserialize<'a>, N: Node>
pub fn main_loop<
    'a,
    S,
    P: Deserialize<'a> + Debug + Clone + Serialize + Send + 'static,
    N: Node<P, S>,
>(
    state: S,
) -> anyhow::Result<()> {
    let stdin = stdin();
    let mut out = stdout().lock();

    // let init_msg = serde_json::from_str::<Message<InitPayload>>(r#"{"src": "1", "dest":"2", "body": {"type":     "init","msg_id":   1,"node_id":  "n3","node_ids": ["n1", "n2", "n3"]}}"#)?;
    let init_msg =
        serde_json::from_str::<Message<InitPayload>>(stdin.lines().next().unwrap()?.as_str())?;

    eprintln!("in: {:?}", init_msg);

    let InitPayload::Init(init) = init_msg.body.payload.clone() else {
			panic!("wrong init msg: {:?}",init_msg);
		};

    let mut node = N::new(state, init);

    init_msg
        .reply(InitPayload::InitOk {}, &mut 1)
        .send(&mut out)?;

    let (sn, rw) = channel();
    let thread_reader = thread::spawn(move || {
        let stdin = io::stdin().lock();
        for i in serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>() {
            let i = i.unwrap();
            eprintln!("in: {:?}", i);
            sn.send(i.clone()).unwrap();
        }
    });

    for i in 0.. {
        let val = rw.try_recv();
        if val.is_ok() {
            node.handle(val?, &mut out)?;
        }
        if i % 10 == 0 {
            node.timed_call(&mut out)?;
        }
        thread::sleep(time::Duration::from_millis(100));
        if thread_reader.is_finished() && rw.try_recv().is_err() {
            break;
        }
    }
    Ok(())
}
