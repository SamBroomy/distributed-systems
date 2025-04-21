use distributed_systems::*;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNone {
    id: usize,
}

impl Node<(), Payload> for EchoNone {
    fn from_init(_state: (), _init: Init) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(EchoNone { id: 1 })
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Echo { echo } => {
                reply.body.payload = Payload::EchoOk { echo };
                reply.send(output).context("Failed to send reply")?;
            }
            Payload::EchoOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    main_loop::<(), EchoNone, Payload>(())
}
