use std::collections::BTreeMap;

use sha1::{Digest, Sha1};

pub struct Piece {
    pub hash: [u8; 20],
    pub len: u32,
    pub blocks: BTreeMap<u32, Vec<u8>>,
}

impl Piece {
    // This should only be executed on a thread pool and not the executor.
    pub fn matches_hash(&self) -> bool {
        let mut hasher = Sha1::new();
        for block in self.blocks.values() {
            hasher.update(block);
        }
        let hash = hasher.finalize();
        hash.as_slice() == self.hash
    }
}
