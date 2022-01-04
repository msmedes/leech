extern crate serde_derive;
use anyhow::Result;

mod client;
use client::LeechClient;

#[tokio::main]
async fn main() -> Result<()> {
    let filename = "debian-mac-11.2.0-amd64-netinst.iso.torrent";
    let client = LeechClient::new(filename).await?;
    println!("{:?}", client);
    client.download().await?;
    Ok(())
}
