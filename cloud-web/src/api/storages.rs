use std::collections::HashMap;
use axum::extract::{Extension, Path, Multipart, BodyStream, Query};
use axum::{Json, Router, debug_handler};
use axum::routing::{get, post, put, delete};
use crate::api::{extractor::{AuthUser, AuthUploadInfo}, ApiContext, Result, error::CustomError};
use crate::api_common::storages::{StorageBody, Storage, Session, CreateSessionReq, UpdateFileReq,
                                  SessionInfo, BlockInfo, UploadFinishReq, ListStorageReq,
                                  UploadFileReq};
use crate::api::workspaces;
use cloud_core::db_schema::files::Files as DbFile;
use cloud_core::store_service::{cloud_file::CloudFile, cloud_block::CloudBlock};
use std::sync::Arc;
use axum::headers::{Header, HeaderValue};
use axum::http::HeaderMap;
use uuid::Uuid;
use bytes::{Bytes, BytesMut};
use redis::{self, AsyncCommands};
use futures::StreamExt;
use serde_json;
use crate::api;

const ROOT_DIR_ID: i64 = -1;

pub fn router() -> Router {
    Router::new()
        .route("/api/:ws_id/storages", post(create_storage).get(list_storage))
        .route("/api/:ws_id/storages/:id", get(get_storage)
            .delete(delete_storage).put(update_file_info))
        .route("/api/upload_sessions", post(create_session))
        .route("/api/upload_sessions/chunks", post(upload_chunk))
        .route("/api/upload_sessions/:session_id", post(finish_upload))
}

// TODO: convert it to extractor
async fn check_file_owner(user_id: Uuid, id: i64, ws_id: Uuid, ctx: &Extension<ApiContext>) -> Result<DbFile> {
    let db_file = DbFile::check_owner(user_id, id, ws_id, &ctx.db).await?;

    // if db_file is None, return Error::Forbidden
    if db_file.is_none() {
        return Err(CustomError::Forbidden.into());
    }

    Ok(db_file.unwrap())
}

async fn check_permission(user_id: Uuid, parent_dir_id: i64, ws_id: Uuid, ctx: &Extension<ApiContext>) -> Result<DbFile> {
    if parent_dir_id == -1 {
        let ws = workspaces::check_ws_owner(user_id, ws_id, &ctx).await?;
        Ok(DbFile::root_dir(ws_id, user_id, ws.name))
    } else {
        Ok(check_file_owner(user_id, parent_dir_id, ws_id, &ctx).await?)
    }
}

async fn create_session(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    data: Json<CreateSessionReq>,
) -> Result<Json<Session>> {
    let session_id = Uuid::now_v7();
    // redis set session_id [user_id, filename, parent_dir_id]
    let redis_client = Arc::clone(&ctx.redis_client);
    let mut conn = redis_client.get_async_connection().await?;

    let session_info = SessionInfo {
        user_id: auth_user.user_id,
        ws_id: data.ws_id,
        filename: data.filename.clone(),
        parent_dir_id: data.parent_dir_id,
    };
    check_permission(auth_user.user_id, data.parent_dir_id, data.ws_id, &ctx).await?;

    conn.set(session_id.to_string(), serde_json::to_string(&session_info)?).await?;

    Ok(Json(Session{
        session_id
    }))
}

// check if session_id exists in redis and owned by this user
// first get session_id from redis and check owner
// put chunk to storage
#[debug_handler]
async fn upload_chunk(
    auth_upload_info: AuthUploadInfo,
    ctx: Extension<ApiContext>,
    mut stream: BodyStream
) -> Result<()> {
    let fs_handler = Arc::clone(&ctx.fs_handler);
    let block_name = Uuid::now_v7().to_string();

    let mut data = BytesMut::new();

    //TODO: optimize writing data to storage
    while let Some(bytes) = stream.next().await {
        let bytes = bytes.unwrap();
        data.extend_from_slice(&bytes);
    }

    let data = data.freeze();

    if data.len() != auth_upload_info.chunk_size as usize {
        return Err(CustomError::BadRequest.into());
    }

    let cloud_block = CloudBlock::new(block_name.as_str(), Bytes::from(data));
    if cloud_block.hash != auth_upload_info.hash {
        return Err(CustomError::BadRequest.into());
    }

    cloud_block.store_block(fs_handler).await?;

    let redis_client = Arc::clone(&ctx.redis_client);
    let mut conn = redis_client.get_async_connection().await?;

    let block_key = format!("{}_{}", auth_upload_info.session_id, auth_upload_info.chunk_num);
    let block_info = BlockInfo {
        block_name,
        block_index: auth_upload_info.chunk_num,
        block_hash: auth_upload_info.hash,
        block_size: auth_upload_info.chunk_size
    };
    conn.set(block_key, serde_json::to_string(&block_info)?).await?;

    Ok(())
}

// check if session_id exists in redis and owned by this user
// first get session_id from redis and check owner
// write record to db
async fn finish_upload(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    Path(session_id): Path<String>,
    upload_finish_req: Json<UploadFinishReq>
) -> Result<()> {
    let redis_client = Arc::clone(&ctx.redis_client);
    let mut conn = redis_client.get_async_connection().await?;
    let sesson_value: String = conn.get(session_id.as_str()).await?;
    if sesson_value.is_empty() {
        return Err(CustomError::BadRequest.into());
    }
    let session_info: SessionInfo = serde_json::from_str(sesson_value.as_str())?;

    if session_info.user_id != auth_user.user_id {
        return Err(CustomError::BadRequest.into());
    }

    let block_key_tpl = format!("{}_{}", session_id, "*");
    let block_keys: Vec<String> = conn.keys(block_key_tpl).await?;

    if block_keys.len() != upload_finish_req.total_chunk_num {
        //TODO: clean up redis data and block data
        return Err(CustomError::BadRequest.into());
    }

    let mut block_infos = Vec::new();
    for block_key in block_keys {
        let block_value: String = conn.get(block_key).await?;
        let block_info: BlockInfo = serde_json::from_str(block_value.as_str())?;
        block_infos.push(block_info);
    }
    
    // get block name and hash to blocks_name and blocks_hash
    let mut blocks_name = Vec::new();
    let mut blocks_hash = Vec::new();
    let mut file_size: usize = 0;

    block_infos.sort_by(|a, b| a.block_index.cmp(&b.block_index));
    for block_info in block_infos {
        blocks_name.push(block_info.block_name);
        blocks_hash.push(block_info.block_hash);
        file_size += block_info.block_size as usize;
    }

    let snowflake = Arc::clone(&ctx.snowflake);
    let id = snowflake.lock().unwrap().next_id();

    CloudBlock::store_file(auth_user.user_id,  session_info.ws_id, session_info.parent_dir_id,
                           id, blocks_name, blocks_hash, file_size as i64,
                           session_info.filename, &ctx.db).await?;
    Ok(())
}

async fn create_storage(
    ctx: Extension<ApiContext>, 
    auth_user: AuthUser,
    Path(ws_id): Path<Uuid>,
    headers: HeaderMap,
    mut stream: BodyStream
) -> Result<Json<StorageBody<Storage>>> {
    let upload_file_req = headers
        .get("x-mycloud")
        .ok_or(CustomError::BadRequest)?
        .to_str()
        .map_err(|_| CustomError::BadRequest)?;
    // let upload_file_req = serde_json::from_str::<UploadFileReq>("{\"filename\":\"test_dir\",\"is_dir\":true,\"parent_dir_id\":-1}")?;
    let upload_file_req = serde_json::from_str::<UploadFileReq>(upload_file_req)?;

    let mut size = 0;
    let mut data = BytesMut::new();
    if !upload_file_req.is_dir {
        while let Some(bytes) = stream.next().await {
            let bytes = bytes.unwrap();
            data.extend_from_slice(&bytes);
        }
        size = data.len();
    }
    check_permission(auth_user.user_id, upload_file_req.parent_dir_id, ws_id, &ctx).await?;

    if upload_file_req.filename == "" ||
        (upload_file_req.filename == "" && !upload_file_req.is_dir && data.len() == 0) ||
        (upload_file_req.filename != "" && upload_file_req.is_dir && data.len() != 0)
    {
        return Err(CustomError::BadRequest.into());
    }

    let snowflake = Arc::clone(&ctx.snowflake);
    let id = snowflake.lock().unwrap().next_id();

    let mut cloud_file = CloudFile::new(
        upload_file_req.filename.as_str(),
        Bytes::from(data),
        upload_file_req.is_dir
    );

    match upload_file_req.is_dir {
        true => {
            cloud_file.create_new_dir(
                ws_id,
                auth_user.user_id,
                upload_file_req.parent_dir_id,
                id,
                &ctx.db
            ).await?;
        },
        false => {
            let fs_handler = Arc::clone(&ctx.fs_handler);
            cloud_file.store_new_file(
                ws_id,
                auth_user.user_id,
                upload_file_req.parent_dir_id,
                id,
                fs_handler,
                &ctx.db
            ).await?;
        }
    };
    Ok(Json(StorageBody {
        storage: Storage::new(
            id,
            upload_file_req.filename,
            upload_file_req.is_dir,
            upload_file_req.parent_dir_id,
            size
        )
    }))
}

// check if file exists and owned by this user
async fn update_file_info(
    auth_user: AuthUser,
    ctx: Extension<ApiContext>,
    Path((ws_id, id)): Path<(Uuid, i64)>,
    update_file_req: Json<UpdateFileReq>
) -> Result<Json<StorageBody<Storage>>> {
    let db_file = check_file_owner(auth_user.user_id, id, ws_id, &ctx).await?;
    db_file.update_file_info(&update_file_req.filename, &ctx.db).await?;

    Ok(Json(StorageBody {
        storage: Storage::new(db_file.id, update_file_req.filename.clone(), db_file.is_dir, db_file.parent_dir_id, db_file.size as usize)
    }))
}

async fn list_storage(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    Path(ws_id): Path<Uuid>,
    Query(list_storage_req): Query<ListStorageReq>
) -> Result<Json<Vec<StorageBody<Storage>>>> {
    let dir_id = list_storage_req.parent_dir_id.unwrap_or(ROOT_DIR_ID);

    let dir = check_permission(auth_user.user_id, dir_id, ws_id, &ctx).await?;

    if dir.is_dir == false {
        return Err(CustomError::NotFound);
    }

    let files = DbFile::get_by_parent_dir_id_and_uid(dir_id, auth_user.user_id, &ctx.db).await?;

    let mut storages = Vec::new();
    for file in files {
        storages.push(
            StorageBody {
                storage: Storage::new(
                    file.id,
                    file.filename,
                    file.is_dir,
                    file.parent_dir_id,
                    file.size as usize
                )
            }
        )
    }

    Ok(Json(storages))
}


// download file
async fn get_storage(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    Path((ws_id, id)): Path<(Uuid, i64)>,
) -> Result<Json<StorageBody<Storage>>> {
    let db_file = check_file_owner(auth_user.user_id, id, ws_id, &ctx).await?;

    Ok(Json(
        StorageBody {
            storage: Storage::new(
                db_file.id,
                db_file.filename,
                db_file.is_dir,
                db_file.parent_dir_id,
                db_file.size as usize
            )
        }
    ))
}

// TODO: if it's a dir, recursively delete all files.
// And if when you search all files, a new file is created, it maybe ignored. So we need to lock the dir.
// it need a trie to record all dirs and files.
async fn delete_storage(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    Path((ws_id, id)): Path<(Uuid, i64)>,
) -> Result<()> {
    let db_file = check_file_owner(auth_user.user_id, id, ws_id, &ctx).await?;
    db_file.delete(&ctx.db).await?;
    Ok(())
}


