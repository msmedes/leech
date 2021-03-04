extern crate serde_derive;
use anyhow::Result;
#[macro_use]
extern crate num_derive;

mod client;
use client::LeechClient;

#[tokio::main]
async fn main() -> Result<()> {
    let filename = "debian-10.8.0-amd64-netinst.iso.torrent";
    let mut client = LeechClient::new(filename);
    client.download().await?;
    Ok(())
}
