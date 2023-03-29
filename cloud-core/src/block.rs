use anyhow::Result;
use bytes::Bytes;
use std::fmt::format;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use ring::digest::{Context, Digest, SHA256};

pub struct Block {
    data: Bytes,
    name: String,
}

impl Block {
    pub fn new(name: String, data: Bytes) -> Self {
        Self { data, name } }

    pub fn write_block(&self, target_dir: &str) -> Result<()> {
        let path = Path::new(target_dir).join(
            format(format_args!(
                "{}/{}/{}",
                self.name.chars().nth(0).unwrap(),
                self.name.chars().nth(1).unwrap(),
                self.name
            ))
            .as_str(),
        );
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap())?;
        }
        let mut file = File::create(path)?;
        file.write_all(&self.data)?;
        Ok(())
    }

    pub fn parse_block(target_dir: &str, filename: String) -> Result<Self> {
        let path = Path::new(target_dir).join(
            format(format_args!(
                "{}/{}/{}",
                filename.chars().nth(0).unwrap(),
                filename.chars().nth(1).unwrap(),
                filename
            ))
            .as_str(),
        );
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let block = Self::new(filename, Bytes::from(data));

        Ok(block)
    }
}


#[cfg(test)]
mod tests;
