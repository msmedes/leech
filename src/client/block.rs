use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};

pub struct BlockInfo {
    pub piece_index: usize,
    pub block_offset: u32,
    pub block_length: u32,
}

impl BlockInfo {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<()> {
        let piece_index = self
            .piece_index
            .try_into()
            .map_err(|_| anyhow!("piece_index out of range"))?;
        buf.put_u32(piece_index);
        buf.put_u32(self.block_offset);
        buf.put_u32(self.block_length);

        Ok(())
    }
}
