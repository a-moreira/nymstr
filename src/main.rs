use clap::Parser;
// use env_logger;
use log::info;
use nym_sdk::mixnet;
use nym_sdk::mixnet::MixnetMessageSender;
use std::error::Error;
use tokio;
use url::Url;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = Url::parse("ws://127.0.0.1").unwrap())]
    nym_client_ws: Url,

    #[arg(default_value_t = Url::parse("ws://127.0.0.1").unwrap())]
    nostr_relay_ws: Url,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logging();

    let _args = Args::parse();

    info!("Starting Nostr Nym Proxy");

    // Passing no config makes the client fire up an ephemeral session and figure shit out on its own
    let mut client = mixnet::MixnetClient::connect_new().await.unwrap();

    // Be able to get our client address
    let our_address = client.nym_address();
    println!("Our client nym address is: {our_address}");

    // Send a message throught the mixnet to ourselves
    client
        .send_plain_message(*our_address, "hello there")
        .await
        .unwrap();

    println!("Waiting for message (ctrl-c to exit)");
    client
        .on_messages(|msg| println!("Received: {}", String::from_utf8_lossy(&msg.message)))
        .await;

    Ok(())
}

fn init_logging() {
    nym_bin_common::logging::setup_logging();
    // env_logger::init();
}
