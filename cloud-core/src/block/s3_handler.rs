use super::*;

pub struct S3Handler {
    //client: S3Client,
    bucket: String,
}

impl S3Handler {
    pub fn new(target_dir: &str) -> Self {
        unimplemented!()
    }
}

impl BlockHandler for S3Handler {
    fn get_blocks(&self, blocks_name: Vec<&str>) -> Result<Vec<Block>> {
        unimplemented!()
    }

    fn write_blocks(&self, blocks: Vec<Block>) -> Result<()> {
        unimplemented!()
    }
}

