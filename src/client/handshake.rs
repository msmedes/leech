use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

// use tokio::prelude::Read;

use super::types::{HandshakeMsg, InfoHash, PeerId};

pub struct Handshake {
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
        // The handshake structure is as follows:
        // The length (as a byte) of the pstr.
        // The pstr, in this case literally "BitTorrent protocol".
        // 8 reserved bytes, in this case all 0s
        // The info_hash, a [u8;20].
        // The client peer_id, a [u8;20];
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.pstr.len() as u8);
        buffer.put_slice(self.pstr.as_bytes());
        buffer.put_slice(&[0; 8]);
        buffer.put_slice(&self.info_hash[..]);
        buffer.put_slice(&self.peer_id[..]);
        // buffer.try_into().unwrap()
        buffer.freeze()
    }

    pub async fn read_from_stream(connection: &mut TcpStream) -> Result<Handshake> {
        let mut buf = BytesMut::with_capacity(1);

        let _ = connection.read_buf(&mut buf).await?;

        let pstr_len = *buf.get(0).unwrap() as usize;
        if pstr_len == 0 {
            return Err(anyhow!("pstr_len is 0"));
        }

        let mut buf = BytesMut::with_capacity(48 + pstr_len);
        let _ = connection.read_buf(&mut buf).await?;

        let mut info_hash: InfoHash = Default::default();
        let mut peer_id: PeerId = Default::default();

        info_hash.copy_from_slice(&buf[pstr_len + 8..pstr_len + 8 + 20]);
        peer_id.copy_from_slice(&buf[pstr_len + 8 + 20..]);

        Ok(Handshake {
            pstr: String::from_utf8(buf.get(0..pstr_len).unwrap().to_vec()).unwrap(),
            info_hash,
            peer_id,
        })
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
        let expected: HandshakeMsg = Bytes::from_static(&[
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 0, 0, 0, 134, 212, 200, 0, 36, 164, 105, 190, 76, 80, 188, 90,
            16, 44, 247, 23, 128, 49, 0, 116, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20,
        ]);
        assert_eq!(serialized, expected);
    }
}
