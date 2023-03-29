use super::*;
use bytes::Bytes;
use uuid::Uuid;

#[test]
fn test_block() {
    let content = Bytes::from("Hello World");
    let uuid = Uuid::new_v4().to_string().replace("-", "");

    println!("uuid: {}", uuid);
    let block = Block::new(uuid, content.clone());

    let target_dir = "/home/neo/Downloads/".to_string();
    block.write_block(target_dir.as_str()).unwrap();

    let data = Block::parse_block(target_dir.as_str(), uuid).unwrap();
    assert_eq!(data.data, content);
}


