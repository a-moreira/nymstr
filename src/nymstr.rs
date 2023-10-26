use crate::config::Config;
use nostr_sdk::prelude::Client as NostrClient;
use nostr_sdk::prelude::Keys as NostrKeys;
use nym_sdk::mixnet;
use nym_sdk::mixnet::MixnetMessageSender;

use std::error::Error;
use std::path::PathBuf;

pub struct Nymstr {
    pub nostr_client: NostrClient,
    pub nym_client: mixnet::MixnetClient,
    pub nym_sender: mixnet::MixnetClientSender,
}

impl Nymstr {
    pub async fn setup(cfg: Config, nostr_keys: NostrKeys) -> Result<Self, Box<dyn Error>> {
        let nostr_client = NostrClient::new(&nostr_keys);
        let nym_config_dir = PathBuf::from(cfg.nym_client_config_dir);
        let storage_paths = mixnet::StoragePaths::new_from_dir(nym_config_dir).unwrap();
        let nym_client = mixnet::MixnetClientBuilder::new_with_default_storage(storage_paths)
            .await
            .unwrap()
            .build()
            .unwrap();

        let nym_client = nym_client.connect_to_mixnet().await.unwrap();

        nostr_client
            .add_relay(cfg.nostr_relay_ws.to_string(), None)
            .await?;
        // Connect to relays
        nostr_client.connect().await;

        let nym_sender = nym_client.split_sender();

        Ok(Nymstr {
            nostr_client,
            nym_client,
            nym_sender,
        })
    }
    pub async fn publish_to_mixnet(&mut self, msg: &str) -> Result<(), nym_sdk::Error> {
        self.nym_sender
            .send_plain_message(*self.nym_client.nym_address(), msg)
            .await?;

        Ok(())
    }

    pub async fn publish_to_nostr(&self, msg: &str) -> Result<(), Box<dyn Error>> {
        self.nostr_client.publish_text_note(msg, &[]).await?;

        Ok(())
    }
}
