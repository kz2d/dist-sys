use std::{collections::HashMap, io::Write};

use dist_system::{main_loop, Init, Message, Node};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    id: usize,
    node_name: String,
    messages: Vec<usize>,
}

impl Node<Payload, ()> for BroadcastNode {
    fn new(state: (), init: Init) -> Self {
        BroadcastNode {
            id: 2,
            node_name: init.node_id,
            messages: Vec::new(),
        }
    }

    fn handle(&mut self, message: Message<Payload>, out: &mut impl Write) -> anyhow::Result<()> {
        match &message.body.payload {
            Payload::Broadcast { message: data } => {
                self.messages.push(*data);
                message
                    .reply(Payload::BroadcastOk, &mut self.id)
                    .send(out)?;
            }
            Payload::BroadcastOk => {}
            Payload::Read => {
                message
                    .reply(
                        Payload::ReadOk {
                            messages: self.messages.clone(),
                        },
                        &mut self.id,
                    )
                    .send(out)?;
            }
            Payload::ReadOk { messages } => {}
            Payload::Topology { topology } => {
                message.reply(Payload::TopologyOk, &mut self.id).send(out)?;
            }
            Payload::TopologyOk => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<(), Payload, BroadcastNode>(())?;
    Ok(())
}
