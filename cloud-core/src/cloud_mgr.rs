use crate::utils::snowflake::SnowFlake;
use sqlx::postgres::PgPool;
use std::sync::{Arc, Mutex};

pub struct S3Config {
    access_key: String,
    secret_key: String,
    bucket: String,
    region: String,
}

pub struct LocalConfig {
    path: String,
}

pub struct CloudMgr {
    s3_config: Option<S3Config>,
    local_config: Option<LocalConfig>,
    block_max_size: usize,
    db_pool: PgPool,
    snowflake: Arc<Mutex<SnowFlake>>,
}

impl CloudMgr {
    pub async fn new(
        s3_config: Option<S3Config>,
        local_config: Option<LocalConfig>,
        block_max_size: usize,
        connection_str: &str,
        worker_id: i64,
        datacenter_id: i64,
    ) -> Self {
        assert!(
            s3_config.is_some() || local_config.is_some(),
            "s3_config and local_config can't be None at the same time"
        );
        assert!(
            s3_config.is_none() || local_config.is_none(),
            "s3_config and local_config can't be Some at the same time"
        );

        let db_pool = PgPool::connect(connection_str).await.unwrap();
        let snowflake = Arc::new(Mutex::new(SnowFlake::new(worker_id, datacenter_id)));
        CloudMgr {
            s3_config,
            local_config,
            block_max_size,
            db_pool,
            snowflake,
        }
    }

    /// 1. get file info from db
    /// 2. get blocks info from db
    /// 3. download blocks from s3 or read from local
    /// 4. validate blocks
    /// 5. merge blocks into file
    /// 6. return file
    pub fn get_file(&self) {
        unimplemented!()
    }

    /// 1. update file info
    pub fn change_file(&self) {
        unimplemented!()
    }

    /// 1. delete file
    /// Don't delete blocks and file info, only mark file as deleted
    pub fn delete_file(&self) {
        unimplemented!()
    }
}
