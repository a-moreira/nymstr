use clap::Parser;
// use env_logger;
use futures::StreamExt;
use log::info;
use nostr_sdk::prelude::Client as NostrClient;
use nostr_sdk::prelude::Keys as NostrKeys;
use nym_sdk::mixnet;
use nym_sdk::mixnet::MixnetMessageSender;
use std::error::Error;
use url::Url;

#[derive(Parser, Debug)]
struct Config {
    #[arg(short, long, default_value_t = Url::parse("ws://127.0.0.1:9000").unwrap())]
    nym_client_ws: Url,

    #[arg(default_value_t = Url::parse("ws://0.0.0.0:8080").unwrap())]
    nostr_relay_ws: Url,
}

struct Nymstr {
    nostr_client: NostrClient,
    nym_client: mixnet::MixnetClient,
}

impl Nymstr {
    async fn setup(cfg: Config, nostr_keys: NostrKeys) -> Result<Self, Box<dyn Error>> {
        let nostr_client = NostrClient::new(&nostr_keys);
        let nym_client = mixnet::MixnetClient::connect_new().await.unwrap();

        nostr_client
            .add_relay(cfg.nostr_relay_ws.to_string(), None)
            .await?;
        // Connect to relays
        nostr_client.connect().await;

        Ok(Nymstr {
            nostr_client,
            nym_client,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logging();

    let cfg = Config::parse();

    info!("Starting Nostr Nym Proxy");

    let nostr_keys = NostrKeys::generate();

    let mut nymstr = Nymstr::setup(cfg, nostr_keys).await.unwrap();

    // Be able to get our client address
    let our_address = nymstr.nym_client.nym_address().clone();
    println!("Our client nym address is: {our_address}");

    let sender = nymstr.nym_client.split_sender();

    // receiving task
    let receiving_task_handle = tokio::spawn(async move {
        while let Some(received) = nymstr.nym_client.next().await {
            let msg = String::from_utf8_lossy(&received.message);
            println!("{}", msg);
            nymstr.nostr_client.publish_text_note(msg, &[]).await;
        }

        nymstr.nym_client.disconnect().await;
    });

    // sending task
    let sending_task_handle = tokio::spawn(async move {
        for _ in 1..10 {
            sender
                .send_plain_message(our_address, "hello!")
                .await
                .unwrap();
        }
    });

    // wait for both tasks to be done
    println!("waiting for shutdown");
    sending_task_handle.await.unwrap();
    receiving_task_handle.await.unwrap();

    Ok(())
}

fn init_logging() {
    nym_bin_common::logging::setup_logging();
    // env_logger::init();
}
