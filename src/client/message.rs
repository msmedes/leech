use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

extern crate num;

#[derive(FromPrimitive, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    // Chokes the reciever
    Choke,
    // Unchokes the receiver
    Unchoke,
    // Whether or not we are interested in anything the peer has. Sent after
    // being unchoked, and requesting blocks.
    Interested,
    // Sender is not interested.
    Uninterested,
    // Have payload is zero indexed of a piece that has been downloaded and
    // verified by hash.
    Have,
    // Sent immediately after handshaking,
    Bitfield,
    Request,
    Piece,
    Cancel,
    // keep-alive message has zero bytes, no message ID or payload.
    KeepAlive,
}

pub struct Message {
    pub message_type: MessageType,
    pub payload: Option<Vec<u8>>,
}

impl Message {
    fn serialize(&self) -> Bytes {
        match self.message_type {
            MessageType::KeepAlive => Bytes::new(),
            _ => {
                let mut message_length = vec![0; 4];
                let mut buffer = BytesMut::new();
                let payload = self.payload.as_deref().unwrap_or(&[]);
                BigEndian::write_u32(&mut message_length, (payload.len() + 1) as u32);
                buffer.put_slice(&message_length);
                buffer.put_u8(self.message_type as u8);
                buffer.put_slice(payload);
                buffer.freeze()
            }
        }
    }

    pub async fn read(connection: &mut TcpStream) -> Result<Message> {
        let mut buf = BytesMut::with_capacity(4);
        let _ = connection.read_buf(&mut buf).await?;

        let message_length = BigEndian::read_u32(&buf[..]) as usize;

        if message_length == 0 {
            return Ok(Message {
                message_type: MessageType::KeepAlive,
                payload: None,
            });
        }

        let mut buf = BytesMut::with_capacity(message_length);
        let _ = connection.read_buf(&mut buf).await?;

        Ok(Message {
            message_type: num::FromPrimitive::from_u8(buf[0]).unwrap(),
            payload: Some(buf.get(1..).unwrap().to_vec()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_have() {
        let message_interested = Message {
            message_type: MessageType::Interested,
            payload: Some(vec![1, 2, 3, 4]),
        };
        let test_vec: &[u8] = &[0, 0, 0, 5, 2, 1, 2, 3, 4];
        let expected = Bytes::from(test_vec);
        let serialized = message_interested.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn serialize_keep_alive() {
        let message_keep_alive = Message {
            message_type: MessageType::KeepAlive,
            payload: None,
        };
        let expected = Bytes::new();
        let serialized = message_keep_alive.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn serialize_no_payload() {
        let message_no_payload = Message {
            message_type: MessageType::Choke,
            payload: None,
        };

        let expected = Bytes::from_static(&[0, 0, 0, 1, 0]);
        let serialized = message_no_payload.serialize();
        assert_eq!(serialized, expected);
    }
}
