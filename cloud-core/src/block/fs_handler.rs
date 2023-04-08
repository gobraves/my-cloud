use super::*;
use anyhow::Result;
use bytes::Bytes;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct FsHandler{
    target_dir: PathBuf,
}

impl FsHandler {
    pub fn new(target_dir: &str) -> Self {
        let target_dir = Path::new(target_dir).to_path_buf();
        FsHandler { target_dir }
    }
}

impl BlockHandler for FsHandler {
    fn write_blocks(&self, blocks: Vec<Block>) -> Result<()> {
        for block in blocks {
            let path = self.target_dir.join(block.path());
            if !path.exists() {
                std::fs::create_dir_all(path.parent().unwrap())?;
            }
            let mut file = File::create(path)?;
            file.write_all(&block.data)?;
        }
        Ok(())
    }

    fn get_blocks(&self, blocks_name: Vec<&str>) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        for block_name in blocks_name {
            let first_parent_dir = block_name.chars().nth(0).unwrap().to_string();
            let second_parent_dir = block_name.chars().nth(1).unwrap().to_string();
            let path = Path::new(&self.target_dir)
                .join(first_parent_dir)
                .join(second_parent_dir)
                .join(block_name.clone());
            let mut file = File::open(path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            let block = Block::new(block_name.to_string(), Bytes::from(data));
            blocks.push(block);
        }

        Ok(blocks)
    }
    
}

