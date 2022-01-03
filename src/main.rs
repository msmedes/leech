extern crate serde_derive;
use anyhow::Result;

use std::sync::Arc;

mod client;
use client::LeechClient;

#[tokio::main]
async fn main() -> Result<()> {
    let filename = "debian-mac-11.2.0-amd64-netinst.iso.torrent";
    let mut client = Arc::new(LeechClient::new(filename));
    client.download().await?;
    Ok(())
}
