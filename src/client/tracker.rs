// use rand::RngCore;
use super::torrent::TorrentFile;
use super::types::PeerId;

use bytes::Bytes;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize)]
pub struct TrackerRequest {
    announce: String,
    info_hash: String,
    peer_id: String,
    port: i32,
    uploaded: u8,
    downloaded: u8,
    compact: u8,
    left: usize,
}

impl TrackerRequest {
    pub fn new_from_torrent(torrent: &TorrentFile, peer_id: PeerId) -> Self {
        Self {
            announce: torrent.announce.clone(),
            info_hash: torrent
                .info
                .info_hash
                .iter()
                .map(|v| format!("%{:02X}", v))
                .collect::<String>(),
            peer_id: peer_id
                .iter()
                .map(|v| format!("%{:02X}", v))
                .collect::<String>(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            compact: 1,
            left: torrent.info.length,
        }
    }
}

impl fmt::Display for TrackerRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{announce}?compact={compact}&downloaded={downloaded}&info_hash={info_hash}&peer_id={peer_id}&port={port}&uploaded={uploaded}",
    announce = self.announce,
compact = self.compact, downloaded=self.downloaded, info_hash = self.info_hash, peer_id=self.peer_id, port=self.port, uploaded = self.uploaded)
    }
}

#[derive(Debug, Deserialize)]
pub struct TrackerResponse {
    pub interval: u32,
    pub peers: Bytes,
}
