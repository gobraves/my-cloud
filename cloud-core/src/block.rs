pub mod fs_handler;
pub mod s3_handler;
use anyhow::Result;
use bytes::Bytes;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct BlockHandlerWrapper<BH: BlockHandler>(pub Arc<BlockHandlerInner<BH>>);

pub struct BlockHandlerInner<BH: BlockHandler>{
    pub block_handler: BH,
}

impl <BH: BlockHandler> BlockHandlerWrapper<BH> {
    pub fn new(block_handler: BH) -> Self {
        Self(Arc::new(BlockHandlerInner { block_handler }))
    }
}

#[async_trait]
pub trait BlockHandler: Send + Sync {
    fn write_blocks(&self, blocks: Vec<Block>) -> Result<()>;
    fn get_blocks(&self, blocks_name: Vec<&str>) -> Result<Vec<Block>>;
}

pub struct Block {
    pub name: String,
    pub data: Bytes,
}

impl Block {
    pub fn new(block_name: String, data: Bytes) -> Self {
        Self { name: block_name, data} 
    }

    pub fn path(&self) -> PathBuf {
        let first_parent_dir = self.name.chars().nth(0).unwrap().to_owned().to_string();
        let second_parent_dir = self.name.chars().nth(1).unwrap().to_owned().to_string();
        Path::new(&first_parent_dir).join(&second_parent_dir).join(&self.name).to_path_buf()
    }
}

fn block_path_by_filename(block_name: String) -> Result<PathBuf> {
    let first_parent_dir = block_name.chars().nth(0).unwrap().to_owned().to_string();
    let second_parent_dir = block_name.chars().nth(1).unwrap().to_owned().to_string();
    let path = Path::new(&first_parent_dir).join(&second_parent_dir).join(&block_name);

    Ok(path)
}


#[cfg(test)]
mod tests;
