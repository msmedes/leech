use super::types::PeerAddr;

use byteorder::{BigEndian, ReadBytesExt};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Peer {
    pub addr: IpAddr,
    pub port: u16,
    pub socket_addr: SocketAddr,
    pub piece_count: usize,
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
        let addr = IpAddr::V4(Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]));
        let port = port.read_u16::<BigEndian>().expect("port parse failed");
        let socket_addr = SocketAddr::new(addr, port);

        Peer {
            addr,
            port,
            socket_addr,
            piece_count: 0,
        }
    }
}
