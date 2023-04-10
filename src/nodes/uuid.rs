use std::io::Write;

use dist_system::{main_loop, Init, Message, Node};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Generate,
    GenerateOk { id: String },
}

struct UUIDNode {
    id: usize,
    node_name: String,
}

impl Node<Payload, ()> for UUIDNode {
    fn new(state: (), init: Init) -> Self {
        UUIDNode {
            id: 2,
            node_name: init.node_id,
        }
    }

    fn handle(&mut self, message: Message<Payload>, out: &mut impl Write) -> anyhow::Result<()> {
        match &message.body.payload {
            Payload::Generate => {
                message
                    .reply(
                        Payload::GenerateOk {
                            id: format!("{}-{}", self.node_name, self.id),
                        },
                        &mut self.id,
                    )
                    .send(out)?;
            }
            Payload::GenerateOk { id: _ } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<(), Payload, UUIDNode>(())?;
    Ok(())
}
