use axum::extract::{Extension, Query, Path, Multipart, Form};
use axum::{Json, Router};
use axum::routing::{get, post, put, delete};
use super::extractor::AuthUser;
use super::ApiContext;
use super::error::CustomError;
use crate::api::Result;
use crate::api_common::storage::{StorageBody, StorageReq, Storage, Session, CreateSession, UpdateFileReq};
use cloud_core::db_schema::files::Files as DbFile;
use cloud_core::store_service::cloud_file::CloudFile;
use std::sync::Arc;
use uuid::Uuid;
use bytes::{Bytes, BytesMut};


pub fn router() -> Router {
    Router::new()
        .route("/api/storage/:id", post(create_storage).put(update_storage).get(get_storage).delete(delete_storage))
        .route("/api/storage/:id/list", get(list_storage))
        .route("/api/upload_sessions/", post(create_session))
        .route("/api/upload_sessions/:session_id/chunks", post(upload_chunk))
        .route("/api/upload_sessions/:session_id", get(finish_upload))
}

// TODO: convert it to extractor
async fn check_owner(user_id: Uuid, id: i64, ctx: &Extension<ApiContext>) -> Result<DbFile> {
    let db_file = DbFile::check_owner(user_id, id, &ctx.db).await?;

    // if db_file is None, return Error::Forbidden
    if db_file.is_none() {
        return Err(CustomError::Forbidden.into());
    }

    Ok(db_file.unwrap())
}

async fn create_session(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    Json(data): Json<StorageBody<CreateSession>>,
) -> Result<Json<Session>> {
    let parent_dir_id = data.storage.parent_dir_id.parse::<i64>().unwrap();
    check_owner(auth_user.user_id, parent_dir_id, &ctx).await?;

    let session_id = uuid::Uuid::new_v4().to_string();

    // redis set session_id [user_id, filename, parent_dir_id]

    // save session_id, filename and parent_dir_id to redis
    // return session_id created by uuid

    Ok(Json(Session{
        session_id 
    }))
}

// check if session_id exists in redis and owned by this user
// first get session_id from redis and check owner
// put chunk to storage
async fn upload_chunk(ctx: Extension<ApiContext>, auth_user: AuthUser, Path(id): Path<i64>, mut multipart: Multipart) -> Result<()> {
    while let Some(field) = multipart.next_field().await? {
        let value = field.bytes().await?;
    }

    Ok(())
}

// check if session_id exists in redis and owned by this user
// first get session_id from redis and check owner
// write record to db
async fn finish_upload(ctx: Extension<ApiContext>, auth_user: AuthUser, Path(id): Path<i64>) -> Result<Json<Storage>> {
    unimplemented!()
}

async fn create_storage(
    ctx: Extension<ApiContext>, 
    auth_user: AuthUser, 
    Path(id): Path<String>, 
    //url_params: Query<StorageReq>,
    mut multipart: Multipart
) -> Result<Json<StorageBody<Storage>>> {
    let parent_dir_id = id.parse::<i64>().unwrap();
    check_owner(auth_user.user_id, parent_dir_id, &ctx).await?;

    let mut data = BytesMut::new();
    let mut filename = String::new();
    let mut is_dir = false;
    let mut size = 0 as usize;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap();
        match name {
            "filename" => {
                let tmp_filename = field.bytes().await?.to_vec();
                filename = String::from_utf8(tmp_filename).unwrap();
            },
            "is_dir" => {
                is_dir = match field.bytes().await?.to_vec().as_slice() {
                    b"true" => true,
                    _ => false
                };
            },
            "data" => {
                let value = field.bytes().await?;
                data.extend_from_slice(&value);
                size = data.len();
            }
            _ => {}
        }
    };

    if filename == "" || (filename == "" && !is_dir && data.len() == 0) || (filename != "" && is_dir && data.len() != 0) {
        return Err(CustomError::BadRequest.into());
    }

    let snowflake = Arc::clone(&ctx.snowflake);
    let id = snowflake.lock().unwrap().next_id();


    let mut cloud_file = CloudFile::new(filename.as_str(), Bytes::new(), is_dir);
    //cloud_file.create_new_dir(auth_user.user_id, parent_dir_id, id, &ctx.db).await?;
    match is_dir {
        true => {
            cloud_file.create_new_dir(auth_user.user_id, parent_dir_id, id, &ctx.db).await?;
        },
        false => {
            cloud_file.data = Bytes::from(data);
            let fs_handler = Arc::clone(&ctx.fs_handler);
            cloud_file.store_new_file(auth_user.user_id, parent_dir_id, id, fs_handler, &ctx.db).await?;
        }
    };
    Ok(Json(StorageBody {
        storage: Storage::new(id, filename, is_dir, parent_dir_id, size)
    }))
}

// check if file exists and owned by this user
async fn update_storage(auth_user: AuthUser,ctx: Extension<ApiContext>, Path(id): Path<i64>, update_file_req: Form<UpdateFileReq>) -> Result<Json<StorageBody<Storage>>> {
    let db_file = check_owner(auth_user.user_id, id, &ctx).await?;
    db_file.update_file_info(&update_file_req.filename, &ctx.db).await?;

    Ok(Json(StorageBody {
        storage: Storage::new(db_file.id, update_file_req.filename.clone(), db_file.is_dir, db_file.parent_dir_id, db_file.size as usize)
    }))
}

async fn list_storage(ctx: Extension<ApiContext>, auth_user: AuthUser, Path(id): Path<i64>) -> Result<Json<Vec<Storage>>> {
    let db_dir = check_owner(auth_user.user_id, id, &ctx).await?;

    if db_dir.is_dir == false {
        return Err(CustomError::NotFound);
    }

    let files = DbFile::get_by_parent_dir_id_and_uid(db_dir.id, auth_user.user_id, &ctx.db).await?;

    let mut storages = Vec::new();
    for file in files {
        storages.push(Storage::new(file.id, file.filename, file.is_dir, file.parent_dir_id, file.size as usize))
    }

    Ok(Json(storages))
}


async fn get_storage(ctx: Extension<ApiContext>, auth_user: AuthUser, Path(id): Path<i64>) -> Result<Json<StorageBody<Storage>>> {
    let db_file = check_owner(auth_user.user_id, id, &ctx).await?;

    Ok(Json(
        StorageBody {
            storage: Storage::new(db_file.id, db_file.filename, db_file.is_dir, db_file.parent_dir_id, db_file.size as usize)
        }
    ))
}

async fn delete_storage(ctx: Extension<ApiContext>, auth_user: AuthUser, Path(id): Path<i64>) -> Result<()> {
    let db_file = check_owner(auth_user.user_id, id, &ctx).await?;
    db_file.delete(&ctx.db).await?;
    Ok(())
}

