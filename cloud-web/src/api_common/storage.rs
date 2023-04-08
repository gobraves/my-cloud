use serde::{Deserialize, Serialize};

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
pub struct CreateSession {
    pub filename: String,
    pub parent_dir_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String
}
