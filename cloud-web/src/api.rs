mod error;
mod extractor;
mod storage;
mod users;

use crate::config::Config;
use axum::Router;
use cloud_core::block::fs_handler::FsHandler;
use cloud_core::utils::snowflake::SnowFlake;
use error::{CustomError, ResultExt};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::trace::TraceLayer;

pub type Result<T, E = CustomError> = std::result::Result<T, E>;

#[derive(Clone)]
struct ApiContext {
    config: Arc<Config>,
    db: PgPool,
    snowflake: Arc<Mutex<SnowFlake>>,
    //block_handler: BlockHandlerWrapper<T>,
    fs_handler: Arc<FsHandler>,
}

pub async fn serve(config: Config, db: PgPool) -> Result<()> {
    let snowflake = SnowFlake::new(config.worker_id, config.datacenter_id);
    //let block_handler = match config.block_handler_type.as_str() {
        //"fs" => BlockHandlerWrapper::new(fs_handler::FsHandler {target_dir: config.data_dir}),
        //"s3" => BlockHandlerWrapper::new(s3_handler::S3Handler::new(config.s3_bucket_name)),
        //_ => panic!("block handler not supported"),
    //};
    let block_handler = FsHandler::new(&config.data_dir);
    let url = format!("{}:{}", &config.host, config.port);
    let url = url.parse::<SocketAddr>().unwrap();

    let app = api_router().layer(
        ServiceBuilder::new()
            .layer(AddExtensionLayer::new(ApiContext {
                config: Arc::new(config),
                db,
                snowflake: Arc::new(Mutex::new(snowflake)),
                fs_handler: Arc::new(block_handler),
            }))
            // Enables logging. Use `RUST_LOG=tower_http=debug`
            .layer(TraceLayer::new_for_http()),
    );

    axum::Server::bind(&url)
        .serve(app.into_make_service())
        .await
        .expect("error running HTTP server");
    Ok(())
}

fn api_router() -> Router {
    // This is the order that the modules were authored in.
    users::router().merge(storage::router())
    //.merge(articles::router())
}
