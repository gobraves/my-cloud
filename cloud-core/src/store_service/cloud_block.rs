use crate::block::BlockHandler;
use anyhow::Result;
use bytes::Bytes;
use super::inner_utils;


pub struct CloudBlock {
    name: String,
    data: Bytes,
    hash: String
}

impl CloudBlock {
    pub fn new(name: &str, data: Bytes) -> Self {
        let hash = inner_utils::sha256_digest(&data);

        Self {
            name: name.to_string(),
            data,
            hash,
        }
    }

    pub async fn store_block(
        &self,
        block_handler: &dyn BlockHandler,
    ) -> Result<()> {
        let file_size = self.data.len() as i64;
        let (blocks, blocks_hash) = inner_utils::cut(&self.data);
        let blocks_name = blocks
            .iter()
            .map(|block| block.name.clone())
            .collect::<Vec<String>>();
        block_handler.write_blocks(blocks).unwrap();
        Ok(())
    }
}

