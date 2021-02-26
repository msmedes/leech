mod bitfield;
mod handshake;
mod peer;
mod torrent;
mod tracker;
mod types;

use peer::Peer;
use torrent::TorrentFile;
use tracker::{TrackerRequest, TrackerResponse};
use types::{InfoHash, PeerAddr, PeerId, Peers};

use std::convert::TryInto;

use anyhow::Result;
use bytes::Bytes;
use rand::Rng;
use reqwest;
use serde_bencode::de;

#[derive(Debug)]
pub struct LeechClient {
    peers: Peers,
    filename: String,
    pub torrent_file: TorrentFile,
    info_hash: InfoHash,
    peer_id: PeerId,
    poll_interval: u32,
}

impl LeechClient {
    pub fn new(filename: &str) -> Self {
        let torrent_file = TorrentFile::new(filename);
        LeechClient {
            filename: String::from(filename),
            info_hash: torrent_file.info.info_hash,
            torrent_file,
            peers: Vec::<Peer>::new(),
            peer_id: generate_peer_id(),
            poll_interval: 0,
        }
    }

    fn set_peers(&mut self, peer_blob: Bytes) {
        let peer_addrs: Vec<PeerAddr> =
            peer_blob.chunks(6).map(|p| p.try_into().unwrap()).collect();
        self.peers = peer_addrs.iter().map(|addr| Peer::from(*addr)).collect();
    }

    pub async fn download(&mut self) -> Result<()> {
        self.poll_tracker().await?;
        println!("{:?}", self);
        Ok(())
    }
    
    async fn poll_tracker(&mut self) -> Result<()> {
        let req = TrackerRequest::new_from_torrent(&self.torrent_file, self.peer_id);
        let res = reqwest::get(&req.to_string())
            .await
            .expect("Request failed");
        let body = res.bytes().await.expect("could not parse response body");
        let res = de::from_bytes::<TrackerResponse>(&body)?;
        self.poll_interval = res.interval;
        self.set_peers(res.peers);
        Ok(())
    }
}

fn generate_peer_id() -> PeerId {
    rand::thread_rng().gen::<PeerId>()
}
