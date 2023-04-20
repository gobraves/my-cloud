use std::{env, fs};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use axum::{http::StatusCode, Router};
use axum::body::Body;
use axum_test_helper::TestClient;
use bytes::Bytes;
use log::debug;
use reqwest::blocking::multipart::Form;
use sqlx::postgres::PgPoolOptions;
use cloud_core::db_schema::workspaces::Workspaces;
use cloud_core::utils::snowflake::SnowFlake;
use cloud_web::api::{api_router, ApiContext};
use cloud_web::api_common::users::{LoginUser, NewUser, UserBody, User};
use cloud_web::api_common::workspaces::{WsBody, WsReq};
use cloud_web::api_common::storages::{Storage, StorageBody, UpdateFileReq, UploadFileReq,
                                      CreateSessionReq, Session, UploadFinishReq};
use cloud_web::api::extractor::{AuthUser, AuthUploadInfo};
use cloud_web::config::Config;
use cloud_core::block::fs_handler::FsHandler;

fn load_config() -> Config {
    let file_path = env::var("CONFIG_TEST_PATH").unwrap_or("config-test.yaml".to_string());
    let contents = fs::read_to_string(file_path)
        .expect("Something went wrong reading the config");

    let config: Config = serde_yaml::from_str(&contents).unwrap();
    return config
}

async fn init_env() -> Router {
    let config = load_config();
    let redis = redis::Client::open(config.redis_connection_str.as_str()).unwrap();

    let pool = PgPoolOptions::new()
        //.max_connections(config.connection_num)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.db_connection_str)
        .await
        .expect("can't connect to db");

    let snowflake = SnowFlake::new(config.worker_id, config.datacenter_id);
    let block_handler = FsHandler::new(&config.data_dir);

    let api_ctx = ApiContext::new(config, pool, snowflake, block_handler, redis);
    api_router(api_ctx)
}

#[tokio::test]
async fn test_create_user() {
    let app = init_env().await;

    // user serialize to json as request body
    let user = NewUser {
        username: "name".to_string(),
        password: "password".to_string(),
        email: "email@test.com".to_string(),
    };

    let user = UserBody { user };

    let client = TestClient::new(app);
    let res = client
        .post("/api/users")
        .header("content-type", "application/json")
        .json(&user)
        .send()
        .await;
    let status_code = res.status();

    assert_eq!(status_code, StatusCode::OK);
}

async fn user_login() -> User {
    let app = init_env().await;
    let user = LoginUser {
        email: "email@test.com".to_string(),
        password: "password".to_string(),
    };

    let user = UserBody { user };

    let client = TestClient::new(app);
    let res = client
        .post("/api/users/login")
        .header("content-type", "application/json")
        .json(&user)
        .send()
        .await;
    let status_code = res.status();
    let user: UserBody<User> = res.json().await;
    assert_eq!(status_code, StatusCode::OK);
    user.user
}

#[tokio::test]
async fn test_create_ws() {
    let app = init_env().await;
    let client = TestClient::new(app);
    let user = user_login().await;

    let ws_req = WsBody {
        ws: WsReq {
            name: "ws1".to_string(),
        }
    };
    // let ws_req = serde_json::to_string(&ws_req).unwrap();

    let res = client
        .post("/api/workspaces")
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .header("content-type", "application/json")
        .json(&ws_req)
        .send()
        .await;
    let status_code = res.status();
    assert_eq!(status_code, StatusCode::OK);
}

async fn get_ws_list() -> Vec<WsBody<Workspaces>> {
    let user = user_login().await;

    let app = init_env().await;
    let client = TestClient::new(app);

    // prepare to get all workspace list
    let res = client
        .get("/api/workspaces")
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .send()
        .await;
    let status_code = res.status();
    assert_eq!(status_code, StatusCode::OK);

    let ws_list= res.json::<Vec<WsBody<Workspaces>>>().await;
    assert_ne!(ws_list.len(), 0);
    debug!("ws_list: {:?}", ws_list);
    ws_list
}

#[tokio::test]
async fn test_update_ws() {
    let app = init_env().await;
    let client = TestClient::new(app);
    let user = user_login().await;
    let ws_list = get_ws_list().await;

    // prepare to update workspace
    let ws_id = ws_list[0].ws.id;
    let ws_req = WsBody {
        ws: WsReq {
            name: "ws4".to_string(),
        }
    };
    // let ws_req = serde_json::to_string(&ws_req).unwrap();
    let url = "/api/workspaces/".to_string() + ws_id.to_string().as_str();
    let res = client
        .put(&url)
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .header("content-type", "application/json")
        .json(&ws_req)
        .send()
        .await;
    let status_code = res.status();
    assert_eq!(status_code, StatusCode::OK);
}

// TODO: test mkdir
#[tokio::test]
async fn test_mkdir_under_root() {
    let app = init_env().await;
    let client = TestClient::new(app);
    let user = user_login().await;

    let ws_list = get_ws_list().await;
    let ws_id = ws_list[0].ws.id;

    let url = "/api/".to_string() + ws_id.to_string().as_str() + "/storages";
    let upload_file_req = UploadFileReq {
        filename: "test_dir2".to_string(),
        is_dir: true,
        parent_dir_id: -1,
    };
    let upload_file_req_str = serde_json::to_string(&upload_file_req).unwrap();
    let res = client
        .post(&url)
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .header("x-mycloud", upload_file_req_str)
        .send()
        .await;
    let status_code = res.status();
    assert_eq!(status_code, StatusCode::OK);
}

async fn list_storages() -> Vec<StorageBody<Storage>> {
    let app = init_env().await;
    let client = TestClient::new(app);
    let user = user_login().await;

    let ws_list = get_ws_list().await;
    let ws_id = ws_list[0].ws.id;
    let url = "/api/".to_string() + ws_id.to_string().as_str() + "/storages?parent_dir_id=-1";
    let res = client
        .get(&url)
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .send()
        .await;
    let status_code = res.status();
    assert_eq!(status_code, StatusCode::OK);
    let storages = res.json::<Vec<StorageBody<Storage>>>().await;
    assert_ne!(storages.len(), 0);
    storages
}

// #[tokio::test]
// async fn test_delete_storage() {
//     let app = init_env().await;
//     let client = TestClient::new(app);
//     let user = user_login().await;
//
//     let ws_list = get_ws_list().await;
//     let ws_id = ws_list[0].ws.id;
//
//     let storages = list_storages().await;
//     let storage_id = &storages[0].storage.id;
//     let url = "/api/".to_string() + ws_id.to_string().as_str() + "/storages/" + storage_id.as_str();
//
//     let res = client
//         .delete(&url)
//         .header("Authorization", "Token ".to_string() + user.token.as_str())
//         .send()
//         .await;
//     let status_code = res.status();
//     assert_eq!(status_code, StatusCode::OK);
// }

#[tokio::test]
async fn test_upload_small_file() {
    let app = init_env().await;
    let client = TestClient::new(app);
    let user = user_login().await;

    let ws_list = get_ws_list().await;
    let ws_id = ws_list[0].ws.id;

    let url = "/api/".to_string() + ws_id.to_string().as_str() + "/storages";
    let upload_file_req = UploadFileReq {
        filename: "test_file1.txt".to_string(),
        is_dir: false,
        parent_dir_id: -1,
    };
    let upload_file_req_str = serde_json::to_string(&upload_file_req).unwrap();
    let res = client
        .post(&url)
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .header("x-mycloud", upload_file_req_str)
        .body(Body::from("test file content"))
        .send()
        .await;
    let status_code = res.status();
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn test_upload_big_file() {
    let app = init_env().await;
    let client = TestClient::new(app);
    let user = user_login().await;

    let ws_list = get_ws_list().await;
    let ws_id = ws_list[0].ws.id;

    // 1. build create session request and send
    // 1.1 prepare url
    // 1.2 prepare request body
    // 1.3 ensure http method is post
    let url = "/api/upload_sessions";
    let upload_session_req = CreateSessionReq {
        filename: "test_file2.txt".to_string(),
        parent_dir_id: -1,
        ws_id
    };
    // let upload_session_req = serde_json::to_string(&upload_session_req).unwrap();
    // println!("upload_session_req: {}", upload_session_req);
    let res = client
        .post(&url)
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .json(&upload_session_req)
        .send()
        .await;
    let status_code = res.status();
    assert_eq!(status_code, StatusCode::OK);
    let upload_session_res = res.json::<Session>().await;
    println!("test");

    // 2. prepare to upload file
    // 2.1 get upload session id
    // 2.2 prepare url
    // 2.3 prepare request body
    let url = "/api/upload_sessions/chunks";
    let data = Bytes::from("test file content");
    let hashsum = cloud_utils::digest::sha256_digest(&data);
    let session_value = AuthUploadInfo {
        session_id: upload_session_res.session_id,
        chunk_size: data.len(),
        chunk_num: 0,
        hash: hashsum
    };
    let session_value_str = serde_json::to_string(&session_value).unwrap();
    println!("session_value_str: {:?}", session_value_str);

    let res = client
        .post(&url)
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .header("x-cloud-session", &session_value_str)
        .body(Body::from("test file content"))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    // 3. send upload_finished request
    // 3.1 get upload session id
    // 3.2 prepare url
    // 3.3 prepare request body
    let url = "/api/upload_sessions/".to_string() + upload_session_res.session_id.to_string().as_str();
    let req = UploadFinishReq {
        total_chunk_num: 1
    };
    let res = client
        .post(&url)
        .header("Authorization", "Token ".to_string() + user.token.as_str())
        .header("x-cloud-session", &session_value_str)
        .json(&req)
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);
}

// #[tokio::test]
// async fn test_delete_ws() {
//     let app = init_env().await;
//     let client = TestClient::new(app);
//     let user = user_login().await;
//
//     let ws_list = get_ws_list().await;
//     let ws_id = ws_list[0].ws.id;
//
//     // prepare to delete workspace
//     let url = "/api/workspaces/".to_string() + ws_id.to_string().as_str();
//     let res = client
//         .delete(&url)
//         .header("Authorization", "Token ".to_string() + user.token.as_str())
//         .send()
//         .await;
//     let status_code = res.status();
//     assert_eq!(status_code, StatusCode::OK);
// }
