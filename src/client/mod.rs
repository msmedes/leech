mod block;
mod handshake;
mod message;
mod peer;
mod peerclient;
mod torrent;
mod tracker;
mod types;

use peer::Peer;
use peerclient::PeerClient;
use torrent::TorrentFile;
use tracker::{TrackerRequest, TrackerResponse};
use types::{InfoHash, PeerAddr, PeerId, Peers};

use std::convert::TryInto;

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use rand::Rng;
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

#[derive(Debug)]
struct PieceInProgress {
    index: usize,
    client: PeerClient,
    buffer: BytesMut,
    downloaded: usize,
    requested: usize,
    backlog: usize,
}

impl PieceInProgress {
    async fn from_peer(peer: Peer, info_hash: InfoHash, peer_id: PeerId) -> Result<Self> {
        let client = PeerClient::new(peer, info_hash, peer_id).await?;
        let buffer = BytesMut::new();
        let downloaded = 0;
        let requested = 0;
        let backlog = 0;
        Ok(PieceInProgress {
            index: 0,
            client,
            buffer,
            downloaded,
            requested,
            backlog,
        })
    }
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
        // println!("{:?}", self.peers);
    }

    pub async fn download(&mut self) -> Result<()> {
        self.poll_tracker().await?;
        let pieces_in_progress_tasks: Vec<_> = self
            .peers
            .iter()
            .map(|peer| {
                tokio::spawn(PieceInProgress::from_peer(
                    *peer,
                    self.info_hash,
                    self.peer_id,
                ))
            })
            .collect();
        let mut pieces_in_progress = Vec::new();
        for task in pieces_in_progress_tasks {
            let peer = task.await;
            if peer.is_ok() {
                let peer = peer.unwrap();
                println!("{:?}", peer);
                pieces_in_progress.push(peer);
            }
        }
        // dbg!(pieces_in_progress);
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
