use std::{collections::HashMap, io::Write};

use dist_system::{main_loop, utils::merge_messages, Init, Message, Node};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Send {
        key: String,
        msg: usize,
    },
    SendOk {
        offset: usize,
    },
    Poll {
        offsets: HashMap<String, usize>,
    },
    PollOk {
        msgs: HashMap<String, Vec<[usize; 2]>>,
    },
    CommitOffsets {
        offsets: HashMap<String, usize>,
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, usize>,
    },
    Propogate {
        messages: Vec<PropogateInfo>,
    },
    PropogateOk,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct PropogateInfo {
    delta: usize,
    uuid: String,
}

struct KafkaLog {
    id: usize,
    node_name: String,
    near_nodes: Vec<String>,
    counter: usize,
    messages: Vec<PropogateInfo>,
}

impl KafkaLog {
    fn propogate(&mut self, out: &mut impl Write) -> anyhow::Result<()> {
        for node in self.near_nodes.clone() {
            Message::new(
                self.node_name.clone(),
                node.clone(),
                Payload::Propogate {
                    messages: self.messages.clone(),
                },
                &mut self.id,
            )
            .send(out)?;
        }
        Ok(())
    }
}

impl Node<Payload, ()> for KafkaLog {
    fn new(_state: (), init: Init) -> Self {
        KafkaLog {
            id: 2,
            counter: 0,
            node_name: init.node_id.clone(),
            near_nodes: {
                let mut m = init.node_ids;
                m.retain(|x| *x != init.node_id);
                m
            },
            messages: Vec::new(),
        }
    }

    fn handle(&mut self, message: Message<Payload>, out: &mut impl Write) -> anyhow::Result<()> {
        match &message.body.payload {
            Payload::Send { key, msg } => todo!(),
            Payload::SendOk { offset } => todo!(),
            Payload::Poll { offsets } => todo!(),
            Payload::PollOk { msgs } => todo!(),
            Payload::CommitOffsets { offsets } => todo!(),
            Payload::CommitOffsetsOk => todo!(),
            Payload::ListCommittedOffsets { keys } => todo!(),
            Payload::ListCommittedOffsetsOk { offsets } => todo!(),
            Payload::Propogate { messages } => todo!(),
            Payload::PropogateOk => todo!(),
        }
        Ok(())
    }

    fn timed_call(&mut self, out: &mut impl Write) -> anyhow::Result<()> {
        self.propogate(out)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<(), Payload, KafkaLog>(())?;
    Ok(())
}
