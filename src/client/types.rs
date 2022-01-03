use bitvec::prelude::{BitVec, Msb0};
use bytes::Bytes;

use super::peer::Peer;

pub type Bitfield = BitVec<Msb0, u8>;
pub type InfoHash = [u8; 20];
pub type HandshakeMsg = Bytes;
pub type PeerAddr = [u8; 6];
pub type PeerId = [u8; 20];
pub type PieceHash = [u8; 20];
pub type PieceHashes = Vec<PieceHash>;
pub type PieceIndex = usize;
pub type Peers = Vec<Peer>;
