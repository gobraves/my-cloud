use crate::block::fs_handler::FsHandler;
use crate::block::{Block, BlockHandler};
use crate::db_schema::files::Files as DbFile;
use anyhow::Result;
use bytes::Bytes;
use sqlx::PgPool;
use super::inner_utils;
use cloud_utils::digest;
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
            false => digest::sha256_digest(&data),
        };

        Self {
            name: name.to_string(),
            data,
            hash,
        }
    }

    pub async fn create_new_dir(
        &self,
        ws_id: Uuid,
        uid: Uuid,
        parent_dir_id: i64,
        id: i64,
        pool: &PgPool,
    ) -> Result<DbFile> {
        let db_file = DbFile::new(
            id,
            uid,
            ws_id,
            self.name.clone(),
            parent_dir_id,
            0,
            false,
        );

        let dir = db_file.insert_dir(pool).await?;
        Ok(dir)
    }

    // TODO: support generic block handler
    pub async fn store_new_file(
        &self,
        ws_id: Uuid,
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
            ws_id,
            self.name.clone(),
            parent_dir_id,
            file_size,
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
