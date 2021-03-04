use byteorder::{BigEndian, ByteOrder};
// use bytes::{BufMut, Bytes, BytesMut};
// use tokio::net::TcpStream;

extern crate num;

#[derive(FromPrimitive, Copy, Clone)]
enum MessageType {
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

struct Message {
    pub message_type: MessageType,
    pub payload: Vec<u8>,
}

impl Message {
    fn serialize(&self) -> Vec<u8> {
        let buffer = match self.message_type {
            MessageType::KeepAlive => vec![],
            _ => {
                let mut message_length = vec![0; 4];
                let mut buffer = vec![];
                BigEndian::write_u32(&mut message_length, (self.payload.len() + 1) as u32);
                buffer.append(&mut message_length);
                buffer.push(self.message_type as u8);
                buffer.append(&mut self.payload.clone());
                buffer
            }
        };
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_have() {
        let message_interested = Message {
            message_type: MessageType::Interested,
            payload: vec![1, 2, 3, 4],
        };
        let expected = [0, 0, 0, 5, 2, 1, 2, 3, 4];
        let serialized = message_interested.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn serialize_keep_alive() {
        let message_keep_alive = Message {
            message_type: MessageType::KeepAlive,
            payload: vec![],
        };
        let expected = [];
        let serialized = message_keep_alive.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn serialize_no_payload() {
        let message_no_payload = Message {
            message_type: MessageType::Choke,
            payload: vec![],
        };

        let expected = [0, 0, 0, 1, 0];
        let serialized = message_no_payload.serialize();
        assert_eq!(serialized, expected);
    }
}
