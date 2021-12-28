use futures::{SinkExt, StreamExt};

use super::handshake::{Handshake, HandshakeCodec};
use super::message::Message;
use super::peer::Peer;
use super::types::{Bitfield, InfoHash, PeerId};

use anyhow::{anyhow, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct PeerClient {
    peer: Peer,
    bitfield: Bitfield,
    connection: TcpStream,
    info_hash: InfoHash,
    peer_id: PeerId,
    choked: bool,
}

impl PeerClient {
    pub async fn new(peer: Peer, info_hash: InfoHash, peer_id: PeerId) -> Result<Self> {
        let mut connection = TcpStream::connect(peer.socket_addr).await?;
        println!("socked created to peer {}", peer.socket_addr);

        let _ = initial_handshake(&mut connection, info_hash, peer_id).await?;

        let bitfield: Bitfield = Default::default();
        Ok(PeerClient {
            peer,
            connection,
            bitfield,
            info_hash,
            peer_id,
            choked: true,
        })
    }
}

async fn initial_handshake(
    connection: &mut TcpStream,
    peer_id: PeerId,
    info_hash: InfoHash,
) -> Result<()> {
    let mut socket = Framed::new(connection, HandshakeCodec);
    let handshake = Handshake::new(peer_id, info_hash);
    let _ = socket.send(handshake).await?;

    if let Some(peer_handshake) = socket.next().await {
        let peer_handshake = peer_handshake?;
        println!("{:?}", peer_handshake);
    }
    Ok(())
}
