extern crate serde;
extern crate serde_bencode;
use serde_derive::{Deserialize, Serialize};
extern crate serde_bytes;
use serde_bencode::de;
use sha1;

use bytes::Bytes;

// use reqwest;

use std::io::Read;
use std::{convert::TryInto, fs};

use super::types::{InfoHash, PieceHashes};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct BencodeInfo {
    pub(crate) name: String,
    #[serde(rename = "piece length")]
    pub(crate) piece_length: usize,
    pub(crate) pieces: Bytes,
    #[serde(default)]
    pub(crate) length: Option<usize>,
}

impl BencodeInfo {
    fn hash(&self) -> InfoHash {
        let serialized = serde_bencode::to_bytes(&self).unwrap();
        sha1::Sha1::from(serialized).digest().bytes()
    }

    fn split_piece_hashes(&self) -> PieceHashes {
        let hash_len = 20;
        if self.pieces.len() % hash_len != 0 {
            panic!("Received malformed pieces of length {}", self.pieces.len());
        }
        let piece_hashes: PieceHashes = self
            .pieces
            .chunks(hash_len)
            .map(|w| w.try_into().unwrap())
            .collect();
        piece_hashes
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct BencodeTorrent {
    pub(crate) info: BencodeInfo,
    #[serde(default)]
    pub(crate) announce: Option<String>,
}

#[derive(Debug)]
pub struct Info {
    name: String,
    pub piece_length: usize,
    pub pieces: Bytes,
    pub length: usize,
    pub info_hash: InfoHash,
}

#[derive(Debug)]
pub struct TorrentFile {
    pub info: Info,
    pub announce: String,
    pub pieces_hashes: PieceHashes,
}

impl From<BencodeTorrent> for TorrentFile {
    fn from(bencode: BencodeTorrent) -> Self {
        TorrentFile {
            announce: bencode.announce.unwrap(),
            pieces_hashes: bencode.info.split_piece_hashes(),
            info: Info {
                name: bencode.info.name.clone(),
                piece_length: bencode.info.piece_length,
                pieces: bencode.info.pieces.clone(), // considered a partial move without clone
                length: bencode.info.length.unwrap(),
                info_hash: bencode.info.hash(),
            },
        }
    }
}

impl TorrentFile {
    // Opens the torrent file, allocs a buffer for the metadata to go into,
    // then uses serde to decode to a BencodeTorrent.  This is then converted
    // to a TorrentFile. Intermediate representation is required because a sha1
    // hash of the info dictionary object must be made for the tracker request,
    // however serde can't do that as it deserializes so this is the workaround.
    pub fn new(filename: &str) -> Self {
        let mut file = fs::File::open(filename).expect("unable to read file");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        file.read(&mut buffer).expect("buffer overflow");
        let t = match de::from_bytes::<BencodeTorrent>(&buffer) {
            Ok(t) => t,
            Err(e) => panic!("Error: {:?}", e),
        };
        let torrent = TorrentFile::from(t);
        torrent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_torrentfile() {}
}
