use crate::block::{BlockHandler, fs_handler::FsHandler};
use anyhow::Result;
use bytes::Bytes;
use crate::db_schema::files::Files as DbFile;
use sqlx::postgres::PgPool;
use std::sync::Arc;
use cloud_utils::digest;
use super::inner_utils;
use uuid::Uuid;


pub struct CloudBlock {
    pub name: String,
    pub data: Bytes,
    pub hash: String
}

impl CloudBlock {
    pub fn new(name: &str, data: Bytes) -> Self {
        let hash = digest::sha256_digest(&data);

        Self {
            name: name.to_string(),
            data,
            hash,
        }
    }

    pub async fn store_block(
        &self,
        block_handler: Arc<dyn BlockHandler>,
        //block_handler: Arc<FsHandler>,
    ) -> Result<(Vec<String>, Vec<String>)> {
        let (blocks, blocks_hash) = inner_utils::cut(&self.data);
        let blocks_name = blocks
            .iter()
            .map(|block| block.name.clone())
            .collect::<Vec<String>>();
        block_handler.write_blocks(blocks).unwrap();
        Ok((blocks_name, blocks_hash))
    }

    pub async fn store_file(
        uid: Uuid,
        ws_id: Uuid,
        parent_dir_id: i64,
        id: i64,
        blocks_name: Vec<String>,
        blocks_hash: Vec<String>,
        file_size: i64,
        filename: String,
        db: &PgPool,
    ) -> Result<()> {
        let db_file = DbFile::new(
            id,
            uid,
            ws_id,
            filename,
            parent_dir_id,
            file_size,
            false
        );
        db_file.insert_file(blocks_name, blocks_hash, db).await?;
        Ok(())
    }
}

