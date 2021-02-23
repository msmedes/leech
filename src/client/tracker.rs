// use rand::RngCore;
use super::torrent::TorrentFile;
use super::types::{InfoHash, PeerId};

use bytes::Bytes;
use serde_derive::Serialize;
use serde_urlencoded::ser::SeqSerializer;
use serde_urlencoded::Serializer;
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

impl From<&TorrentFile> for TrackerRequest {
    fn from(torrent: &TorrentFile) -> Self {
        Self {
            announce: torrent.announce.clone(),
            info_hash: torrent
                .info
                .info_hash
                .iter()
                .map(|v| format!("%{:02X}", v))
                .collect::<String>(),
            peer_id: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
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

impl TrackerRequest {
    pub fn url_encoded(&self) -> String {
        serde_urlencoded::to_string(self).expect("wow")
    }
}

impl fmt::Display for TrackerRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{announce}?compact={compact}&downloaded={downloaded}&info_hash={info_hash}&peer_id={peer_id}&port={port}&uploaded={uploaded}",
    announce = self.announce,
compact = self.compact, downloaded=self.downloaded, info_hash = self.info_hash, peer_id=self.peer_id, port=self.port, uploaded = self.uploaded)
    }
}

pub struct TrackerResponse {
    pub interval: u32,
    pub peers: String,
}
