use std::io::Cursor;

use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use super::types::{InfoHash, PeerId};

#[derive(Debug)]
pub struct Handshake {
    //The protocol string, which is the literal 'BitTorrent protocol'.
    pub pstr: [u8; 19],
    // 8 reserved bytes, all 0s.
    pub reserved: [u8; 8],
    // The torrent's SHA1 info hash, used to identify the torrent
    pub info_hash: InfoHash,
    // The peer's arbitrary SHA1 id, used to identify the torrent client
    pub peer_id: PeerId,
}

pub const PROTOCOL_STRING: &str = "BitTorrent protocol";

impl Handshake {
    pub fn new(peer_id: PeerId, info_hash: InfoHash) -> Self {
        let mut pstr = [0; 19];
        pstr.copy_from_slice(PROTOCOL_STRING.as_bytes());
        Handshake {
            pstr,
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }
}

pub struct HandshakeCodec;

impl Encoder<Handshake> for HandshakeCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, handshake: Handshake, buf: &mut BytesMut) -> Result<()> {
        let Handshake {
            pstr,
            reserved,
            info_hash,
            peer_id,
        } = handshake;

        buf.put_u8(pstr.len() as u8);
        buf.extend_from_slice(&pstr);
        buf.extend_from_slice(&reserved);
        buf.extend_from_slice(&info_hash);
        buf.extend_from_slice(&peer_id);

        Ok(())
    }
}

impl Decoder for HandshakeCodec {
    type Item = Handshake;
    type Error = anyhow::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Handshake>> {
        if buf.is_empty() {
            return Ok(None);
        }

        // `get_*` int extractors will consume the buffer, which we don't
        // want to do until we know that the full message, so we create a
        // temp buffer.
        let mut temp_buf = Cursor::new(&buf);
        let pstr_len = temp_buf.get_u8() as usize;
        if pstr_len != PROTOCOL_STRING.as_bytes().len() {
            return Err(anyhow!(
                "pstr_len is not equal to the length of the protocol string"
            ));
        }

        let payload_len = pstr_len + 8 + 20 + 20;
        if buf.remaining() > payload_len {
            buf.advance(1);
        } else {
            return Ok(None);
        }

        let mut pstr = [0; 19];
        buf.copy_to_slice(&mut pstr);
        let mut reserved = [0; 8];
        buf.copy_to_slice(&mut reserved);
        let mut info_hash = [0; 20];
        buf.copy_to_slice(&mut info_hash);
        let mut peer_id = [0; 20];
        buf.copy_to_slice(&mut peer_id);

        Ok(Some(Handshake {
            pstr,
            reserved,
            info_hash,
            peer_id,
        }))
    }
}
