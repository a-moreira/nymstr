use clap::Parser;
use url::Url;

#[derive(Parser, Debug)]
pub struct Config {
    #[arg(short, long, default_value_t = Url::parse("ws://127.0.0.1:9000").unwrap())]
    pub nym_client_ws: Url,

    #[arg(default_value_t = Url::parse("ws://0.0.0.0:8080").unwrap())]
    pub nostr_relay_ws: Url,

    #[arg(short, long, default_value_t = String::from("/tmp/nym-client/"))]
    pub nym_client_config_dir: String,
}
