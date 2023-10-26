// use env_logger;
use crate::config::Config;
use crate::nymstr::Nymstr;
use clap::Parser;
use futures::StreamExt;
use log::info;
use nostr_sdk::Keys as NostrKeys;
use nym_sdk::mixnet::MixnetMessageSender;
use std::error::Error;



mod config;
mod nymstr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    nym_bin_common::logging::setup_logging();

    let cfg = Config::parse();

    info!("Starting Nostr Nym Proxy");

    let nostr_keys = NostrKeys::generate();

    let mut nymstr = Nymstr::setup(cfg, nostr_keys).await.unwrap();

    let our_address = *nymstr.nym_client.nym_address();
    info!(target: "nym-client", "Our client nym address is: {our_address}");

    let _sender = nymstr.nym_client.split_sender();

    let receiving_task_handle = tokio::spawn(async move {
        while let Some(received) = nymstr.nym_client.next().await {
            let msg = String::from_utf8_lossy(&received.message);
            info!(target: "nym-client", "Got new message! Publishing to Nostr...");
            nymstr.nostr_client.publish_text_note(msg, &[]).await;
        }

        nymstr.nym_client.disconnect().await;
    });

    let sending_task_handle = tokio::spawn(async move {
        for _ in 1..10 {
            nymstr
                .nym_sender
                .send_plain_message(our_address, "hello!")
                .await
                .unwrap();
        }
    });

    sending_task_handle.await.unwrap();
    receiving_task_handle.await.unwrap();

    Ok(())
}
