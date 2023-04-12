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
    Propogate {
        message: usize,
        id: usize,
    },
}

struct BroadcastNode {
    id: usize,
    count: usize,
    node_name: String,
    near_nodes: Vec<String>,
    messages: Vec<usize>,
}

impl Node<Payload, ()> for BroadcastNode {
    fn new(_state: (), init: Init) -> Self {
        BroadcastNode {
            id: 2,
            count: 0,
            node_name: init.node_id,
            near_nodes: init.node_ids,
            messages: Vec::new(),
        }
    }

    fn handle(&mut self, message: Message<Payload>, out: &mut impl Write) -> anyhow::Result<()> {
        match &message.body.payload {
            Payload::Broadcast { message: data } => {
                self.messages.push(*data);
                self.count = self.messages.len();
                for node in &self.near_nodes {
                    let mut m = message.reply(
                        Payload::Propogate {
                            message: *data,
                            id: self.count,
                        },
                        &mut self.id,
                    );
                    m.dest = node.clone();
                    m.send(out)?;
                }
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
            Payload::ReadOk { messages: _ } => {}
            Payload::Topology { topology: _ } => {
                message.reply(Payload::TopologyOk, &mut self.id).send(out)?;
            }
            Payload::TopologyOk => {}
            Payload::Propogate { message: data, id } => {
                eprintln!("something");
                if *id <= self.count {
                    return Ok(());
                }

                self.messages.push(*data);
                self.count = self.messages.len();
                for node in &self.near_nodes {
                    let mut m = message.reply(
                        Payload::Propogate {
                            message: *data,
                            id: self.count,
                        },
                        &mut self.id,
                    );
                    m.dest = node.clone();
                    m.send(out)?;
                }
                message
                    .reply(Payload::BroadcastOk, &mut self.id)
                    .send(out)?;
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<(), Payload, BroadcastNode>(())?;
    Ok(())
}
