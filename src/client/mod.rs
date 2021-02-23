mod torrent;
pub mod tracker;
mod types;

use torrent::TorrentFile;

#[derive(Debug)]
pub struct LeechClient {
    // peers: Vec<Peer>,
    filename: String,
    pub torrent_file: TorrentFile,
    // info_hash: [u8; 20],
    // peer_id: [u8; 20],
}

impl LeechClient {
    pub fn new(filename: &str) -> Self {
        let torrent_file = TorrentFile::new(filename);
        LeechClient {
            filename: String::from(filename),
            torrent_file,
        }
    }
}
