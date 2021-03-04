use super::types::PeerAddr;

use byteorder::{BigEndian, ReadBytesExt};
use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug, PartialEq, Eq)]
pub struct Peer {
    pub addr: Ipv4Addr,
    pub port: String,
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.addr, self.port)
    }
}

impl From<PeerAddr> for Peer {
    // PeerIds are given as chunks of 6 bytes in compact mode, like so:
    // [192, 0, 2, 123, 26, 225].  The first four bytes are the IP, and the
    // last two are the port address in BigEndian format. To get the port
    // you just squish the two together.
    // Eg: [26, 225] or [0x1A, 0xE1] -> 6881
    fn from(peer_addr: PeerAddr) -> Self {
        let addr = &peer_addr[0..4];
        let mut port = &peer_addr[4..6];

        // There's no way I could find to spread the slice to the function arguments,
        // but we're only using IPv4 so it should be fine.
        let addr = Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]);
        let port = port
            .read_u16::<BigEndian>()
            .expect("port parse failed")
            .to_string();

        Peer { addr, port }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_bytes() {
        let addr: PeerAddr = [192, 0, 2, 123, 26, 225];
        let peer = Peer::from(addr);
        let expected = Peer {
            addr: Ipv4Addr::new(192, 0, 2, 123),
            port: "6881".to_string(),
        };
        assert_eq!(peer, expected);
    }

    #[test]
    fn format() {
        let addr: PeerAddr = [192, 0, 2, 123, 26, 225];
        let peer = Peer::from(addr);
        assert_eq!(peer.to_string(), "192.0.2.123:6881");
    }
}
