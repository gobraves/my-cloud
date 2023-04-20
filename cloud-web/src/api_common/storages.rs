use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageBody<T> {
    pub storage: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageReq {
    pub is_dir: Option<&'static str>,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListStorageReq {
    pub parent_dir_id: Option<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UploadFileReq {
    pub filename: String,
    pub is_dir: bool,
    pub parent_dir_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFileReq {
    pub filename: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    pub id: String,
    pub is_dir: bool,
    pub filename: String,
    pub parent_dir_id: String,
    pub size: String,
}

impl Storage {
    pub fn new(id: i64, filename: String, is_dir: bool, parent_dir_id: i64, size: usize) -> Self {
        let id = id.to_string();
        let parent_dir_id = parent_dir_id.to_string();
        let size = size.to_string();
        Self {
            id,
            is_dir,
            filename,
            parent_dir_id,
            size,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionReq {
    pub filename: String,
    pub ws_id: Uuid,
    pub parent_dir_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: Uuid,
    pub ws_id: Uuid,
    pub filename: String,
    pub parent_dir_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInfo {
    pub block_name: String,
    pub block_index: usize,
    pub block_size: usize,
    pub block_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFinishReq {
    pub total_chunk_num: usize,
}


