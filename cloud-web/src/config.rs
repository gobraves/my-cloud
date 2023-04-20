use serde::{Serialize, Deserialize};

#[derive(clap::Parser, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[clap(long, env)]
    pub debug: bool,

    #[clap(long, env)]
    pub port: u16,

    #[clap(long, env)]
    pub host: String,

    #[clap(long, env)]
    pub db_connection_str: String,

    #[clap(long, env, default_value = "0")]
    pub datacenter_id: i64,

    #[clap(long, env, default_value = "0")]
    pub worker_id: i64,

    #[clap(long, env, default_value = "/tmp/")]
    pub data_dir: String,

    #[clap(long, env)]
    pub hmac_key: String,

    #[clap(long, env)]
    pub redis_connection_str: String
}
