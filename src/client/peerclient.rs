use futures::{SinkExt, StreamExt};

use super::handshake::{Handshake, HandshakeCodec};
use super::message::Message;
use super::message::PeerCodec;
use super::peer::Peer;
use super::types::{Bitfield, InfoHash, PeerId};

use anyhow::{anyhow, Result};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct PeerClient {
    peer: Peer,
    pub bitfield: Bitfield,
    pub connection: TcpStream,
    info_hash: InfoHash,
    peer_id: PeerId,
    pub choked: bool,
}

impl PeerClient {
    pub async fn new(peer: Peer, info_hash: InfoHash, peer_id: PeerId) -> Result<Self> {
        let mut connection = TcpStream::connect(peer.socket_addr).await?;
        println!("socked created to peer {}", peer.socket_addr);

        let _ = initial_handshake(&mut connection, info_hash, peer_id).await?;

        let bitfield = receive_bitfield(&mut connection).await?;

        Ok(PeerClient {
            peer,
            connection,
            bitfield,
            info_hash,
            peer_id,
            choked: true,
        })
    }

    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        let mut stream = Framed::new(&mut self.connection, PeerCodec);
        let _ = stream.send(message).await?;
        Ok(())
    }

    pub async fn handle_message(&mut self) -> Result<Message> {
        let mut socket = Framed::new(&mut self.connection, PeerCodec);
        loop {
            let msg = socket.next().await;
            if let Some(Ok(msg)) = msg {
                return Ok(msg);
            }
        }
    }
}

async fn receive_bitfield(connection: &mut TcpStream) -> Result<Bitfield> {
    let mut socket = Framed::new(connection, PeerCodec);
    loop {
        let msg = socket.next().await;
        if let Some(Ok(Message::Bitfield(bitfield))) = msg {
            return Ok(bitfield);
        }
    }
}

async fn initial_handshake(
    connection: &mut TcpStream,
    info_hash: InfoHash,
    peer_id: PeerId,
) -> Result<()> {
    let mut socket = Framed::new(connection, HandshakeCodec);
    let handshake = Handshake::new(info_hash, peer_id);
    let _ = socket.send(handshake).await?;

    if let Some(peer_handshake) = socket.next().await {
        let peer_handshake = peer_handshake?;
        println!("handshake complete: {:?}", peer_handshake);
    }
    Ok(())
}
