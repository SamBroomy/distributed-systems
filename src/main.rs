use std::io::{StdoutLock, Write};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct EchoNone {
    id: usize,
}

impl EchoNone {
    pub fn step(&mut self, input: Message, output: &mut StdoutLock) -> Result<()> {
        match input.body.payload {
            Payload::Echo { echo } => {
                self.id = input.body.id.unwrap();
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply).context("Failed to serialize reply")?;
                output
                    .write_all(b"\n")
                    .context("Writing trailing newline")?;
                self.id += 1;
            }
            Payload::EchoOk { .. } => {}
            Payload::Init { .. } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply).context("Failed to serialize reply")?;
                output
                    .write_all(b"\n")
                    .context("Writing trailing newline")?;
                self.id += 1;
            }
            Payload::InitOk => {
                bail!("InitOk should not be received in EchoNone state");
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();

    let mut state = EchoNone { id: 0 };

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be parsed")?;
        state
            .step(input, &mut stdout)
            .context("Failed to process input")?;
    }

    Ok(())
}
