use crate::block::fs_handler::{FsHandler, self};
use crate::block::{Block, BlockHandler};
use crate::db_schema::files::Files as DbFile;
use anyhow::Result;
use bytes::Bytes;
use sqlx::PgPool;
use super::inner_utils;
use uuid::Uuid;
use std::sync::Arc;


pub struct CloudFile {
    pub name: String,
    pub data: Bytes,
    pub hash: String,
}

impl CloudFile {
    pub fn new(name: &str, data: Bytes, is_dir: bool) -> Self {
        let hash = match is_dir {
            true => "".to_string(),
            false => inner_utils::sha256_digest(&data),
        };

        Self {
            name: name.to_string(),
            data,
            hash,
        }
    }

    pub async fn create_new_dir(
        &self,
        uid: Uuid,
        parent_dir_id: i64,
        id: i64,
        pool: &PgPool,
    ) -> Result<DbFile> {
        let dir = DbFile::insert_dir(id, uid, self.name.as_str(), parent_dir_id, pool).await?;
        Ok(dir)
    }

    // TODO: support generic block handler
    pub async fn store_new_file(
        &self,
        uid: Uuid,
        parent_dir_id: i64,
        file_id: i64,
        fs_handler: Arc<FsHandler>,
        db: &PgPool,
    ) -> Result<DbFile> {
        let file_size = self.data.len() as i64;
        let (blocks, blocks_hash) = inner_utils::cut(&self.data);
        let blocks_name = blocks
            .iter()
            .map(|block| block.name.clone())
            .collect::<Vec<String>>();
        fs_handler.write_blocks(blocks).unwrap();

        let db_file = DbFile::new(
            file_id,
            uid,
            self.name.clone(),
            file_size,
            parent_dir_id,
            false,
        );

        db_file.insert_file(blocks_name, blocks_hash, db).await?;
        Ok(db_file)
    }

    /// merge blocks into file
    /// return a file
    pub fn merge(blocks: Vec<Block>, filename: &str) -> Self {
        let mut data = Vec::new();
        for block in blocks {
            data.extend(block.data);
        }
        Self::new(filename, Bytes::from(data), false)
    }
}
