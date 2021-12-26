use std::any::Any;

use super::bitfield::Bitfield;
use super::handshake::Handshake;
use super::message::{Message, MessageType};
use super::peer::Peer;
use super::types::{InfoHash, PeerId};

use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};
// use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

struct PeerClient {
    peer: Peer,
    connection: TcpStream,
    bitfield: Bitfield,
    infoHash: InfoHash,
    peerId: PeerId,
    choked: bool,
}

impl PeerClient {
    async fn new(peer: Peer, infoHash: InfoHash, peerId: PeerId) -> Result<Self> {
        let mut connection = TcpStream::connect(peer.socket_addr).await?;

        let handshake = initial_handshake(&mut connection, infoHash, peerId).await?;

        let bitfield = receive_bitfield(&mut connection).await;

        if bitfield.is_err() {
            connection.shutdown().await?;
            return bitfield?;
        }

        Ok(PeerClient {
            peer,
            connection,
            bitfield,
            infoHash,
            peerId,
            choked: true,
        })
    }
}

async fn initial_handshake(
    connection: &mut TcpStream,
    peer_id: PeerId,
    info_hash: InfoHash,
) -> Result<Handshake> {
    let handshake = Handshake::new(peer_id, info_hash);
    let request = connection.write_all(&handshake.serialize()).await?;

    let handshake_res = Handshake::read_from_stream(connection).await?;
    if handshake_res.info_hash != info_hash {
        return Err(anyhow!(
            "Expected info hash {:?} but got {:?}",
            info_hash,
            handshake_res.info_hash
        ));
    }

    Ok(handshake_res)
}

async fn receive_bitfield(connection: &mut TcpStream) -> Result<Bitfield> {
    let message = Message::read(connection).await?;

    if message.payload.is_none() {
        return Err(anyhow!("Received empty bitfield"));
    }

    if message.message_type != MessageType::Bitfield {
        return Err(anyhow!("Expected bitfield but got {:?}", message.type_id()));
    }

    let bitfield = message.payload.unwrap();
    Ok(Bitfield { bitfield })
}
