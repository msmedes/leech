mod block;
mod handshake;
mod message;
mod peer;
mod peerclient;
mod torrent;
mod tracker;
mod types;

use block::BlockInfo;
use message::Message;
use peer::Peer;
use peerclient::PeerClient;
use torrent::TorrentFile;
use tracker::{TrackerRequest, TrackerResponse};
use types::{InfoHash, PeerAddr, PeerId, Peers};

use std::{convert::TryInto, sync::Arc};

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use rand::Rng;
use serde_bencode::de;
use sha1::{Digest, Sha1};
use tokio::sync::{
    broadcast,
    mpsc::{unbounded_channel, UnboundedSender},
    Mutex,
};

#[derive(Debug)]
pub struct LeechClient {
    peers: Peers,
    filename: String,
    pub torrent_file: TorrentFile,
    info_hash: InfoHash,
    peer_id: PeerId,
    poll_interval: u32,
}

#[derive(Debug)]
struct PieceInProgress {
    index: usize,
    buffer: BytesMut,
    downloaded: usize,
    requested: usize,
    backlog: usize,
}

#[derive(Debug, Copy, Clone)]
struct PieceWork {
    index: usize,
    hash: [u8; 20],
    length: usize,
}

impl PieceWork {
    fn check_integrity(&self, buffer: &BytesMut) -> bool {
        let hash = Sha1::from(buffer.as_ref()).digest().bytes();
        hash.as_slice() == self.hash
    }
}

#[derive(Debug)]
struct PieceResult {
    index: usize,
    buf: BytesMut,
}

const MAX_BACKLOG: usize = 10;
const MAX_REQUEST_SIZE: usize = 16384;

impl LeechClient {
    pub async fn new(filename: &str) -> Result<Self> {
        let torrent_file = TorrentFile::new(filename);
        let mut client = LeechClient {
            filename: String::from(filename),
            info_hash: torrent_file.info.info_hash,
            torrent_file,
            peers: Vec::<Peer>::new(),
            peer_id: generate_peer_id(),
            poll_interval: 0,
        };
        client.poll_tracker().await?;
        Ok(client)
    }

    fn set_peers(&mut self, peer_blob: Bytes) {
        let peer_addrs: Vec<PeerAddr> =
            peer_blob.chunks(6).map(|p| p.try_into().unwrap()).collect();
        self.peers = peer_addrs.iter().map(|addr| Peer::from(*addr)).collect();
    }

    async fn handle_message(
        piece_progress: &mut PieceInProgress,
        client: &mut PeerClient,
    ) -> Result<()> {
        let message = client.handle_message().await?; // blocks?
        match message {
            Message::Unchoke => client.choked = false,
            Message::Choke => client.choked = true,
            Message::Have { piece_index } => client.bitfield.set(piece_index, true),
            Message::Block {
                piece_index,
                offset,
                block_data,
            } => {
                println!("block data length: {}", block_data.len());
                piece_progress.buffer[offset as usize..(offset as usize + block_data.len())]
                    .copy_from_slice(block_data.as_ref());
                piece_progress.downloaded += block_data.len();
                piece_progress.backlog -= 1;
            }
            _ => {}
        }
        println!(
            "message handled? {} {}",
            piece_progress.downloaded, piece_progress.backlog
        );
        Ok(())
    }

    async fn attempt_piece_download(
        client: &mut PeerClient,
        piece_work: PieceWork,
    ) -> Result<BytesMut> {
        println!("PIECE_WORK LENGTH: {}", piece_work.length);
        let mut piece_progress = PieceInProgress {
            index: piece_work.index,
            buffer: BytesMut::from(&vec![0_u8; piece_work.length][..]),
            downloaded: 0,
            requested: 0,
            backlog: 0,
        };

        while piece_progress.downloaded < piece_work.length {
            if !client.choked {
                while piece_progress.backlog < MAX_BACKLOG
                    && piece_progress.requested < piece_work.length
                {
                    let mut block_size = MAX_REQUEST_SIZE;
                    if piece_work.length - piece_progress.requested < block_size {
                        block_size = piece_work.length - piece_progress.requested;
                    }

                    client
                        .send_message(Message::Request(BlockInfo {
                            piece_index: piece_work.index,
                            block_offset: piece_progress.requested as u32,
                            block_length: block_size as u32,
                        }))
                        .await?;

                    piece_progress.backlog += 1;
                    piece_progress.requested += block_size;
                    println!(
                        "piece_progress requested: {} backlog: {}",
                        piece_progress.requested, piece_progress.backlog
                    );
                }
            }
            println!("awaiting message from peer {}", client.peer.addr);
            LeechClient::handle_message(&mut piece_progress, client).await?;
            println!(
                "handled message from peer, {} {}",
                piece_progress.downloaded, piece_work.length
            );
        }
        println!("we have a piece progress buffer");
        Ok(piece_progress.buffer)
    }

    pub async fn download(self) -> Result<()> {
        // let clone = Arc::new(self);
        println!("downloading...");
        self.initialize_download().await?;
        Ok(())
    }

    pub async fn initialize_download(self) -> Result<()> {
        // let clone = self.clone();
        let (work_tx, mut work_rx) =
            broadcast::channel::<PieceWork>(self.torrent_file.piece_hashes.len());
        let (result_tx, mut result_rx) = unbounded_channel::<PieceResult>();

        let work_rx = Arc::new(Mutex::new(work_rx));

        let max_peers = std::cmp::min(self.peers.len(), 50);
        let mut peer_count = 0;
        for peer in self.peers {
            peer_count += 1;

            let worker_tx = work_tx.clone();
            let results_tx = result_tx.clone();
            let mut work_rx = work_rx.clone();
            if peer_count < max_peers {
                tokio::spawn(async move {
                    println!("spawning worker for peer {:?}", peer);
                    LeechClient::start_download_worker(
                        peer,
                        &mut work_rx,
                        worker_tx,
                        results_tx,
                        self.info_hash,
                        self.peer_id,
                    )
                    .await;
                });
            }
        }

        for (index, hash) in self.torrent_file.piece_hashes.iter().enumerate() {
            let work = PieceWork {
                index,
                hash: *hash,
                length: self.torrent_file.calculate_piece_size(index),
            };
            work_tx.send(work)?;
        }

        drop(work_tx);

        let mut buf = BytesMut::from(&vec![0_u8; self.torrent_file.info.length][..]);
        let mut done = 0;
        while let Some(result) = result_rx.recv().await {
            let (start, end) = self.torrent_file.calculate_bounds_for_piece(result.index);
            buf[start..end].copy_from_slice(&result.buf);
            done += 1;
            let percent = (done as f32 / self.torrent_file.piece_hashes.len() as f32) * 100.0;
            println!("{:.2}% completed", percent);
            if done == self.torrent_file.piece_hashes.len() {
                break;
            }
        }
        drop(result_rx);

        Ok(())
    }

    async fn start_download_worker(
        peer: Peer,
        work_rx: &mut Arc<Mutex<broadcast::Receiver<PieceWork>>>,
        work_tx: broadcast::Sender<PieceWork>,
        result_tx: UnboundedSender<PieceResult>,
        info_hash: InfoHash,
        peer_id: PeerId,
    ) -> Result<()> {
        let mut peer_client = PeerClient::new(peer, info_hash, peer_id).await?;

        let _ = peer_client.send_message(Message::Unchoke).await;
        let _ = peer_client.send_message(Message::Interested).await;

        loop {
            let piece_work = work_rx.lock().await.recv().await;
            if let Ok(piece_work) = piece_work {
                if peer_client.bitfield.get(piece_work.index) == None {
                    work_tx.send(piece_work)?;
                    continue;
                }

                let buf =
                    match LeechClient::attempt_piece_download(&mut peer_client, piece_work).await {
                        Ok(buf) => buf,
                        Err(e) => {
                            println!("attempt piece download failed: {:?}", e);
                            work_tx.send(piece_work)?;
                            continue;
                        }
                    };

                if !piece_work.check_integrity(&buf) {
                    println!("integrity check failed");
                    work_tx.send(piece_work)?;
                    continue;
                }

                println!("we got a buffer???????");

                peer_client
                    .send_message(Message::Have {
                        piece_index: piece_work.index,
                    })
                    .await?;

                result_tx.send(PieceResult {
                    index: piece_work.index,
                    buf,
                })?;
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn poll_tracker(&mut self) -> Result<()> {
        let req = TrackerRequest::new_from_torrent(&self.torrent_file, self.peer_id);
        let res = reqwest::get(&req.to_string())
            .await
            .expect("Request failed");
        let body = res.bytes().await.expect("could not parse response body");
        let res = de::from_bytes::<TrackerResponse>(&body)?;
        self.poll_interval = res.interval;
        self.set_peers(res.peers);
        Ok(())
    }
}

fn generate_peer_id() -> PeerId {
    rand::thread_rng().gen::<PeerId>()
}
