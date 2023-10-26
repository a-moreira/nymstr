// use env_logger;
use crate::config::Config;
use crate::nymstr::Nymstr;
use clap::Parser;
use futures::StreamExt;
use log::info;
use nostr_sdk::Keys as NostrKeys;
use nostr_sdk::{key::SecretKey, FromBech32};
use nym_sdk::mixnet::MixnetMessageSender;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod config;
mod nymstr;

// test PK taken from nostr-relay-rs examples
const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    nym_bin_common::logging::setup_logging();

    let cfg = Config::parse();

    info!("Starting Nostr Nym Proxy");

    let secret = SecretKey::from_bech32(BECH32_SK).unwrap();
    let nostr_keys = NostrKeys::new(secret);
    info!(target: "nostr-client", "Nostr Public Key: {}", nostr_keys.public_key());

    let mut nymstr = Nymstr::setup(cfg, nostr_keys).await.unwrap();

    let our_address = *nymstr.nym_client.nym_address();
    info!(target: "nym-client", "Our client nym address is: {our_address}");

    let _sender = nymstr.nym_client.split_sender();

    // TODO: improve SIGINT/termination signal catching
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))
        .expect("fail to setup ctrlc handler");

    let receiving_task_handle = tokio::spawn(async move {
        while let Some(received) = nymstr.nym_client.next().await {
            let msg = String::from_utf8_lossy(&received.message);
            info!(target: "nym-client", "Got new message! Publishing to Nostr...");
            nymstr.nostr_client.publish_text_note(msg, &[]).await;
            if running.load(Ordering::SeqCst) == false {
                break;
            };
        }

        nymstr.nym_client.disconnect().await;
    });

    let sending_task_handle = tokio::spawn(async move {
        for i in 1..100 {
            nymstr
                .nym_sender
                .send_plain_message(our_address, format!("hello #{i}"))
                .await
                .unwrap();
        }
    });

    sending_task_handle.await.unwrap();
    receiving_task_handle.await.unwrap();

    Ok(())
}
