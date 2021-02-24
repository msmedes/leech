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

    fn extract_peers(&mut self, peer_blob: Bytes) {
        let peer_addrs: Vec<PeerAddr> =
            peer_blob.chunks(6).map(|p| p.try_into().unwrap()).collect();
        self.peers = peer_addrs.iter().map(|addr| Peer::from(*addr)).collect();
    }

    pub async fn download(&mut self) -> Result<()> {
        let req = TrackerRequest::new_from_torrent(&self.torrent_file, self.peer_id);
        let res = tracker_req(req).await?;
        self.poll_interval = res.interval;
        self.extract_peers(res.peers);
        println!("{:?}", self.peers);
        Ok(())
    }
}

fn generate_peer_id() -> PeerId {
    rand::thread_rng().gen::<PeerId>()
}

async fn tracker_req(req: TrackerRequest) -> Result<TrackerResponse> {
    let res = reqwest::get(&req.to_string())
        .await
        .expect("Request failed");
    let body = res.bytes().await.expect("could not parse response body");
    let tracker_res = de::from_bytes::<TrackerResponse>(&body)?;
    Ok(tracker_res)
}
