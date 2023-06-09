use super::*;
use bytes::Bytes;
use fs_handler::FsHandler;
use uuid::Uuid;

#[test]
fn test_fs_handler() {
    let content = Bytes::from("Hello World");
    let uuid = Uuid::now_v7().to_string();

    println!("uuid: {}", uuid);
    let block = Block::new(uuid.clone(), content.clone());

    let target_dir = ".";
    let fs_handler = FsHandler::new(target_dir);
    let blocks = vec![block];
    fs_handler.write_blocks(blocks).unwrap();

    let blocks_name = vec![uuid.as_str()];
    let blocks = fs_handler.get_blocks(blocks_name).unwrap();
    assert_eq!(blocks[0].data, content);
}
