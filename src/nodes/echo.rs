use std::io::Write;

use dist_system::{main_loop, Init, Message, Node};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    id: usize,
}

impl Node<Payload, ()> for EchoNode {
    fn new(state: (), init: Init) -> Self {
        EchoNode { id: 2 }
    }

    fn handle(&mut self, message: Message<Payload>, out: &mut impl Write) -> anyhow::Result<()> {
        match &message.body.payload {
            Payload::Echo { echo } => {
                message
                    .reply(
                        Payload::EchoOk {
                            echo: echo.to_string(),
                        },
                        &mut self.id,
                    )
                    .send(out)?;
            }
            Payload::EchoOk { echo: _ } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<(), Payload, EchoNode>(())?;
    Ok(())
}
