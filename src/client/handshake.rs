use super::types::{InfoHash, PeerId};

struct Handshake {
    pub pstr: String,
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
}
