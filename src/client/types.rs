use super::peer::Peer;

pub type InfoHash = [u8; 20];
pub type HandshakeMsg = [u8; 68];
pub type PeerAddr = [u8; 6];
pub type PeerId = [u8; 20];
pub type PieceHash = [u8; 20];
pub type PieceHashes = Vec<PieceHash>;
pub type Peers = Vec<Peer>;
