use std::{convert::TryFrom, io::Cursor};

use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use super::block::BlockInfo;
use super::types::{Bitfield, PieceHash};

// #[derive(FromPrimitive, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageId {
    // Chokes the reciever
    Choke = 0,
    // Unchokes the receiver
    Unchoke = 1,
    // Whether or not we are interested in anything the peer has. Sent after
    // being unchoked, and requesting blocks.
    Interested = 2,
    // Sender is not interested.
    NotInterested = 3,
    // Have payload is zero indexed of a piece that has been downloaded and
    // verified by hash.
    Have = 4,
    // Sent immediately after handshaking,
    Bitfield = 5,
    Request = 6,
    Block = 7,
    Cancel = 8,
}

impl TryFrom<u8> for MessageId {
    type Error = anyhow::Error;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        use MessageId::*;
        match i {
            i if i == Choke as u8 => Ok(MessageId::Choke),
            i if i == Unchoke as u8 => Ok(MessageId::Unchoke),
            i if i == Interested as u8 => Ok(MessageId::Interested),
            i if i == NotInterested as u8 => Ok(MessageId::NotInterested),
            i if i == Have as u8 => Ok(MessageId::Have),
            i if i == Bitfield as u8 => Ok(MessageId::Bitfield),
            i if i == Request as u8 => Ok(MessageId::Request),
            i if i == Block as u8 => Ok(MessageId::Block),
            i if i == Cancel as u8 => Ok(MessageId::Cancel),
            _ => Err(anyhow!("Invalid message id")),
        }
    }
}

pub enum Message {
    KeepAlive,
    Bitfield(Bitfield),
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have {
        piece_index: usize,
    },
    Request(BlockInfo),
    Block {
        piece_index: usize,
        offset: u32,
        block_data: Vec<u8>,
    },
    Cancel(BlockInfo),
}

pub struct PeerCodec;

impl Encoder<Message> for PeerCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> anyhow::Result<()> {
        use Message::*;
        match msg {
            KeepAlive => {
                let msg_len = 0;
                buf.put_u32(msg_len);
                // no payload
            }
            Bitfield(bitfield) => {
                let msg_len = 1 + bitfield.len() / 8;
                buf.put_u32(msg_len as u32);
                buf.put_u8(MessageId::Bitfield as u8);
                buf.extend_from_slice(bitfield.as_raw_slice());
            }
            Choke => {
                let msg_len = 1;
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::Choke as u8);
            }
            Unchoke => {
                let msg_len = 1;
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::Unchoke as u8);
            }
            Interested => {
                let msg_len = 1;
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::Interested as u8);
            }
            NotInterested => {
                let msg_len = 1;
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::NotInterested as u8);
            }
            Have { piece_index } => {
                let msg_len = 5;
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::Have as u8);
                buf.put_u32(piece_index.try_into().unwrap());
            }
            Request(block_info) => {
                let msg_len = 13;
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::Request as u8);
                block_info.encode(buf)?;
            }
            Block {
                piece_index,
                offset,
                block_data,
            } => {
                let msg_len = 9 + block_data.len() as u32;
                // message length
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::Block as u8);
                // piece index
                buf.put_u32(piece_index.try_into().unwrap());
                // integer specifying the zero-based byte offset within the piece
                buf.put_u32(offset);
                buf.put(&block_data[..]);
            }
            Cancel(block_info) => {
                let msg_len = 13;
                buf.put_u32(msg_len);
                buf.put_u8(MessageId::Cancel as u8);
                block_info.encode(buf)?;
            }
        }
        Ok(())
    }
}

impl Decoder for PeerCodec {
    type Item = Message;
    type Error = anyhow::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> anyhow::Result<Option<Self::Item>> {
        if buf.remaining() < 4 {
            return Ok(None);
        }

        let mut temp_buf = Cursor::new(&buf);
        let msg_len = temp_buf.get_u32() as usize;

        temp_buf.set_position(0);

        // check the full payload is there, need to add the msg_len because
        // the buffers cursor was not advanced
        if buf.remaining() >= 4 + msg_len {
            // we have the full length of of the message in the buffer
            // so we can advance past the message length header
            buf.advance(4);

            // if the message length is 0 that's a KeepAlive message
            if msg_len == 0 {
                return Ok(Some(Message::KeepAlive));
            }
        } else {
            return Ok(None);
        }

        let msg_id = MessageId::try_from(buf.get_u8())?;
        let msg = match msg_id {
            MessageId::Choke => Message::Choke,
            MessageId::Unchoke => Message::Unchoke,
            MessageId::Interested => Message::Interested,
            MessageId::NotInterested => Message::NotInterested,
            MessageId::Have => {
                let piece_index = buf.get_u32().try_into()?;
                Message::Have { piece_index }
            }
            MessageId::Bitfield => {
                let mut bitfield = vec![0; msg_len - 1];
                buf.copy_to_slice(&mut bitfield);
                Message::Bitfield(Bitfield::from_vec(bitfield))
            }
            MessageId::Request => {
                let piece_index = buf.get_u32().try_into()?;
                let block_offset = buf.get_u32();
                let block_length = buf.get_u32();
                Message::Request(BlockInfo {
                    piece_index,
                    block_offset,
                    block_length,
                })
            }
            MessageId::Block => {
                let piece_index = buf.get_u32().try_into()?;
                let offset = buf.get_u32();
                // everything except the header
                let mut data = vec![0; msg_len - 9];
                buf.copy_to_slice(&mut data);
                Message::Block {
                    piece_index,
                    offset,
                    block_data: data.into(),
                }
            }
            MessageId::Cancel => {
                let piece_index = buf.get_u32().try_into()?;
                let block_offset = buf.get_u32();
                let block_length = buf.get_u32();
                Message::Cancel(BlockInfo {
                    piece_index,
                    block_offset,
                    block_length,
                })
            }
        };

        Ok(Some(msg))
    }
}
