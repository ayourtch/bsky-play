use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tungstenite::{connect, Message};
use url::Url;
use serde_with::BytesOrString;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    op: u64,
    t: String,
}
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRecord {
    /// The stream sequence number of this message.
    pub seq: i64,
    /// DEPRECATED -- unused
    pub rebase: bool,
    /// Indicates that this commit contained too many ops, or data size was too large. Consumers will need to make a separate request to get missing data.
    pub tooBig: Option<bool>,
    /// The repo this event comes from.
    pub repo: String,
    /// Repo commit object CID.
    #[serde_as(as = "BytesOrString")]
    pub commit: Vec<u8>,
}
/*
    /// DEPRECATED -- unused. WARNING -- nullable and optional; stick with optional to ensure golang interoperability.
    pub prev: Option<String>,
    /// The rev of the emitted commit. Note that this information is also in the commit object included in blocks, unless this is a tooBig event.
    pub rev: String,
    /// The rev of the last emitted commit from this repo (if any).
    pub since: Option<String>,
    /// CAR file containing relevant blocks, as a diff since the previous repo state.
    pub blocks: Vec<u8>,
    pub ops: Vec<repoOp>,
    pub blobs: Vec<String>,
    /// Timestamp of when this message was originally broadcast.
    pub time: String,
}

*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct repoOp {
    pub action: String,
    pub path: String,
    /// For creates and updates, the new record CID. For deletions, null.
    pub cid: Option<String>,
}

fn main() {
    let uri = "wss://bsky.network/xrpc/com.atproto.sync.subscribeRepos";
    let (mut socket, response) = connect(Url::parse(uri).unwrap()).expect("Can't connect");
    loop {
        let msg = socket.read_message().expect("Error reading message");
        match &msg {
            Message::Binary(b) => {
                println!("Binary: {}", b.len());
                let mut data = b.clone();
                let mut deserializer = serde_cbor::Deserializer::from_mut_slice(&mut data);
                let hdr: Frame = serde::Deserialize::deserialize(&mut deserializer).unwrap();
                println!("hdr: {:?}", &hdr);
                if hdr.t == "#commit" {
                    let cr: CommitRecord =
                        serde::Deserialize::deserialize(&mut deserializer).unwrap();
                    println!("commit: {:?}", &cr);
                }
            }
            x => {
                println!("Other: {:?}", &x);
            }
        }
    }
}
