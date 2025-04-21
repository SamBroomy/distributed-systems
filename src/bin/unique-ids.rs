use distributed_systems::*;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

struct UniqueNode {
    node_id: String,
    id: usize,
}

impl Node<(), Payload> for UniqueNode {
    fn from_init(_state: (), init: Init) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(UniqueNode {
            node_id: init.node_id,
            id: 1,
        })
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Generate => {
                // Each node has a unique id and so does each message. So we can
                // generate a unique id by combining the node id and the message id.
                let guid = format!("{}-{}", self.node_id, self.id);
                reply.body.payload = Payload::GenerateOk { guid };
                reply.send(output).context("Failed to send reply")?;
            }
            Payload::GenerateOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    main_loop::<_, UniqueNode, _>(())
}
