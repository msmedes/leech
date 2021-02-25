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
    pub payload: [u8],
}
