use super::*;
use cloud_utils::digest;
use bytes::Bytes;

#[test]
fn test_cut_file() {
    let ten_mb = Bytes::from(vec![0; 10_000_000]);
    let hash = digest::sha256_digest(&ten_mb);
    let block_max_size: usize = 4 * 1024 * 1024;
    let cloud_file = CloudFile::new("test", ten_mb.into(), false);
    let (blocks, blocks_hash) = cloud_file.cut();
    let merged_file = CloudFile::merge(blocks, "test");

    assert_eq!(hash, merged_file.hash);
}
