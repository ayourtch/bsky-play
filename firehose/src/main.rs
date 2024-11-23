use cbor::{Decoder, Encoder};
use rustc_serialize::json::{Json, ToJson};
use tungstenite::{connect, Message};
use url::Url;

fn main() {
    let uri = "wss://bsky.network/xrpc/com.atproto.sync.subscribeRepos";
    let (mut socket, response) = connect(Url::parse(uri).unwrap()).expect("Can't connect");
    loop {
        let msg = socket.read_message().expect("Error reading message");
        match &msg {
            Message::Binary(b) => {
                println!("Binary: {}", b.len());
                let mut d = Decoder::from_bytes(b.clone());
                for cbor in d.items() {
                    let cbor = cbor.unwrap();
                    let js = cbor.to_json();
                    println!("JS: {:?}", &js);
                }
            }
            x => {
                println!("Other: {:?}", &x);
            }
        }
    }
}
