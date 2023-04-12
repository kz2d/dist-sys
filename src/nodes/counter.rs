use std::{
    collections::{HashMap, HashSet},
    io::Write,
    ops::Add,
};

use dist_system::{main_loop, utils::merge_messages, Init, Message, Node};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Read,
    ReadOk { value: usize },
    Add { delta: usize },
    AddOk,
    Propogate { messages: Vec<PropogateInfo> },
    PropogateOk,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct PropogateInfo {
    delta: usize,
    uuid: String,
}

struct CounterNode {
    id: usize,
    node_name: String,
    near_nodes: Vec<String>,
    counter: usize,
    messages: Vec<PropogateInfo>,
}

impl CounterNode {
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

impl Node<Payload, ()> for CounterNode {
    fn new(state: (), init: Init) -> Self {
        CounterNode {
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
            Payload::Add { delta } => {
                self.messages.push(PropogateInfo {
                    delta: *delta,
                    uuid: Uuid::new_v4().to_string(),
                });
                self.counter += *delta;
                message.reply(Payload::AddOk, &mut self.id).send(out)?;
            }
            Payload::AddOk => {}
            Payload::Read => {
                message
                    .reply(
                        Payload::ReadOk {
                            value: self.counter,
                        },
                        &mut self.id,
                    )
                    .send(out)?;
            }
            Payload::ReadOk { value: _ } => {}
            Payload::Propogate { messages } => {
                self.messages = merge_messages(self.messages.clone(), (*messages).clone());
                self.counter = self.messages.iter().map(|x| x.delta).sum::<usize>();
                message
                    .reply(Payload::PropogateOk, &mut self.id)
                    .send(out)?;
            }
            Payload::PropogateOk => {}
        }
        Ok(())
    }

    fn timed_call(&mut self, out: &mut impl Write) -> anyhow::Result<()> {
        self.propogate(out)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<(), Payload, CounterNode>(())?;
    Ok(())
}
