#[derive(clap::Parser, Clone, Debug)]
pub struct Config {
    #[clap(long, env)]
    pub debug: bool,

    #[clap(long, env)]
    pub port: u16,

    #[clap(long, env)]
    pub host: String,

    #[clap(long, env)]
    pub db_connection_str: String,

    //#[clap(long, env)]
    //pub connection_num: u32,

    //#[clap(short, long, env)]
    //pub jwt_secret: String,

    #[clap(long, env, default_value = "0")]
    pub datacenter_id: i64,

    //#[clap(short, long, env)]
    //pub block_handler_type: String,

    #[clap(long, env, default_value = "0")]
    pub worker_id: i64,

    #[clap(long, env, default_value = "/tmp/")]
    pub data_dir: String,

    //#[clap(short, long, env)]
    //pub bucket: String,

    #[clap(long, env)]
    pub hmac_key: String

}
