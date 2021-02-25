use std::convert::TryInto;

use super::types::{HandshakeMsg, InfoHash, PeerId};

struct Handshake {
    pub pstr: String,
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
}

impl Handshake {
    pub fn new(peer_id: PeerId, info_hash: InfoHash) -> Self {
        Handshake {
            pstr: "BitTorrent protocol".to_string(),
            info_hash,
            peer_id,
        }
    }

    pub fn serialize(&self) -> HandshakeMsg {
        let mut buffer: Vec<u8> = vec![];
        let pstr = "BitTorrent protocol";
        buffer.push(pstr.len() as u8);
        buffer.extend(pstr.as_bytes());
        buffer.extend(&[0; 8]);
        buffer.extend(&self.info_hash[..]);
        buffer.extend(&self.peer_id[..]);
        buffer.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let peer_id = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ];
        let info_hash = [
            134, 212, 200, 0, 36, 164, 105, 190, 76, 80, 188, 90, 16, 44, 247, 23, 128, 49, 0, 116,
        ];
        let shake = Handshake::new(peer_id, info_hash);
        let serialized = shake.serialize();
        let expected: HandshakeMsg = [
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 0, 0, 0, 134, 212, 200, 0, 36, 164, 105, 190, 76, 80, 188, 90,
            16, 44, 247, 23, 128, 49, 0, 116, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20,
        ];
        assert_eq!(serialized, expected);
    }
}
