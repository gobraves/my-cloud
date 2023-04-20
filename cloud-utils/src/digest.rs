use bytes::Bytes;
use sha2::{Sha256, Digest};

pub fn sha256_digest(input: &Bytes) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("{:x}", result)
}
