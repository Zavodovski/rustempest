use rustempest::*;
use std::io::{StdoutLock, Write};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

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
    node: String,
    id: usize,
}

impl Node<(), Payload> for UniqueNode {
    fn from_init(_state: (), _init: Init) -> anyhow::Result<Self> where Self: Sized {
        Ok(UniqueNode { 
            node: _init.node_id,
            id: 1 
            }
        )
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Generate { .. } => {
                let guid = format!("{}-{}", self.node, self.id);
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::GenerateOk { guid },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialized response to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            },
            Payload::GenerateOk { .. } => bail!("GenerateOk should not be received by the node"),
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, UniqueNode, _>(())
}
