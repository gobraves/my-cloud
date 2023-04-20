use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use clap::Parser;
use std::time::Duration;
use log;
use redis;
use cloud_web::api;
use cloud_web::config::Config;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cloud_web=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::parse();
    log::info!("config: {:?}", config);
    
    let redis = redis::Client::open(config.redis_connection_str.as_str())?;

    let pool = PgPoolOptions::new()
        //.max_connections(config.connection_num)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.db_connection_str)
        .await
        .expect("can't connect to db");

    api::serve(config, pool, redis).await?;
    Ok(())
}
