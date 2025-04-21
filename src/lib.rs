use std::io::{BufRead, StdoutLock, Write};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    pub fn into_reply(self, id: Option<&mut usize>) -> Self {
        Self {
            src: self.dst,
            dst: self.src,
            body: Body {
                output_message_id: id.map(|id| {
                    let reply_id = *id;
                    *id += 1;
                    reply_id
                }),
                in_reply_to: self.body.output_message_id,
                payload: self.body.payload,
            },
        }
    }

    pub fn send(&self, output: &mut StdoutLock) -> Result<()>
    where
        Payload: Serialize,
    {
        serde_json::to_writer(&mut *output, self).context("Failed to serialize reply")?;
        output
            .write_all(b"\n")
            .context("Writing trailing newline")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub output_message_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<S, Payload> {
    fn from_init(state: S, init: Init) -> Result<Self>
    where
        Self: Sized;

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> Result<()>;
}

pub fn main_loop<S, N, P>(init_state: S) -> Result<()>
where
    P: DeserializeOwned,
    N: Node<S, P>,
{
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();

    let init_message: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("No init message received")
            .context("Failed to read init message")?,
    )
    .context("Failed to parse init message")?;

    let InitPayload::Init(init) = init_message.body.payload else {
        panic!("First message should be Init");
    };

    let mut node: N = N::from_init(init_state, init).context("Node initialization failed")?;

    let reply = Message {
        src: init_message.dst,
        dst: init_message.src,
        body: Body {
            output_message_id: Some(0),
            in_reply_to: init_message.body.output_message_id,
            payload: InitPayload::InitOk,
        },
    };

    reply
        .send(&mut stdout)
        .context("Failed to send init reply")?;
    drop(stdin);
    let stdin = std::io::stdin().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>();

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be parsed")?;
        node.step(input, &mut stdout)
            .context("Failed to process input")?;
    }

    Ok(())
}
