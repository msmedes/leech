use std::any::Any;

use super::bitfield::Bitfield;
use super::handshake::Handshake;
use super::message::{Message, MessageType};
use super::peer::Peer;
use super::types::{InfoHash, PeerId};

use anyhow::{anyhow, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::{Decoder, Encoder, Framed};

#[derive(Debug)]
pub struct PeerClient {
    peer: Peer,
    connection: TcpStream,
    bitfield: Bitfield,
    info_hash: InfoHash,
    peer_id: PeerId,
    choked: bool,
}

impl PeerClient {
    pub async fn new(peer: Peer, info_hash: InfoHash, peer_id: PeerId) -> Result<Self> {
        let mut connection = TcpStream::connect(peer.socket_addr).await?;

        let _ = initial_handshake(&mut connection, info_hash, peer_id).await?;

        let bitfield = match receive_bitfield(&mut connection).await {
            Ok(bitfield) => bitfield,
            Err(_) => {
                connection.shutdown().await?;
                return Err(anyhow!("Could not receive bitfield"));
            }
        };

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
) -> Result<Handshake> {
    let handshake = Handshake::new(peer_id, info_hash);
    // dbg!(&handshake);

    let (mut rd, mut wr) = connection.split();
    tokio::spawn(async move {
        let _ = wr.write_all(&handshake.serialize()).await;
    });

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
