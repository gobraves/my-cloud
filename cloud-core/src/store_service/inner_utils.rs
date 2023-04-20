use bytes::Bytes;
use crate::block::Block;
use cloud_utils::digest;

const BLOCK_MAX_SIZE: usize = 4 * 1024 * 1024;

// #[cfg(feature = "v7")]
pub fn cut(data: &Bytes) -> (Vec<Block>, Vec<String>) {
    // TODO: before cutting block, encrpyt it with AES 256 CBC mode and a random key and iv first
    let mut blocks = Vec::new();
    let mut blocks_hash = Vec::new();

    let mut index = 0;
    let data_len = data.len();
    while index < data_len {
        let end = BLOCK_MAX_SIZE + index;
        let data = match end < data_len {
            true => data.slice(index..end),
            false => data.slice(index..data_len),
        };
        let block_name = uuid::Uuid::now_v7().to_string();
        let block = Block::new(block_name, data);
        let result = digest::sha256_digest(&block.data);
        blocks_hash.push(result);
        blocks.push(block);
        index = end;
    }
    (blocks, blocks_hash)
}

//pub fn sha256_digest(input: &Bytes) -> String {
    //let mut hasher = Sha256::new();
    //hasher.update(input);
    //let result = hasher.finalize();
    //format!("{:x}", result)
//}

#[test]
fn test_cut_file() {
    use super::cloud_file::CloudFile;

    let ten_mb = Bytes::from(vec![0; 10_000_000]);
    let hash = digest::sha256_digest(&ten_mb);
    let block_max_size: usize = 4 * 1024 * 1024;
    let cloud_file = CloudFile::new("test", ten_mb.into(), false);
    let (blocks, blocks_hash) = cut(&cloud_file.data);
    let merged_file = CloudFile::merge(blocks, "test");

    assert_eq!(hash, merged_file.hash);
}
