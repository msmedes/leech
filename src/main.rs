extern crate serde_derive;
use anyhow::Result;

extern crate num;
#[macro_use]
extern crate num_derive;
// extern crate num_traits;

mod client;
use client::LeechClient;

#[tokio::main]
async fn main() -> Result<()> {
    let filename = "debian-mac-11.2.0-amd64-netinst.iso.torrent";
    let mut client = LeechClient::new(filename);
    client.download().await?;
    Ok(())
}
