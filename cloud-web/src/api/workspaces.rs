use axum::extract::{Extension, Path, Multipart, Form, BodyStream};
use axum::{Json, Router, debug_handler};
use axum::routing::{get, post, put, delete};
use crate::api::{extractor::{AuthUser, AuthUploadInfo}, ApiContext, Result, error::CustomError};
use crate::api_common::workspaces::{WsBody, WsReq};
use cloud_core::db_schema::workspaces::Workspaces as Ws;
use std::sync::Arc;
use uuid::Uuid;
use serde_json;


pub fn router() -> Router {
    Router::new()
        .route("/api/workspaces", post(create_ws).get(get_ws_list))
        .route("/api/workspaces/:ws_id",  put(update_ws).delete(delete_ws))
}

pub async fn check_ws_owner(
    user_id: Uuid,
    ws_id: Uuid,
    ctx: &ApiContext
) -> Result<Ws> {
    let ws = Ws::get(ws_id, user_id, &ctx.db).await?;
    Ok(ws)
}

#[debug_handler]
async fn create_ws(
    auth_user: AuthUser,
    ctx: Extension<ApiContext>,
    ws_req: Json<WsBody<WsReq>>,
) -> Result<Json<WsBody<Ws>>> {
    let id = Uuid::now_v7();

    let ws = Ws::new(id, ws_req.ws.name.clone(), auth_user.user_id, false);
    ws.insert(&ctx.db).await?;

    Ok(Json(WsBody { ws }))
}

async fn get_ws_list(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
) -> Result<Json<Vec<WsBody<Ws>>>> {
    let res = Ws::get_workspace_list(auth_user.user_id, &ctx.db).await?;

    let mut ws_list = Vec::new();
    for ws in res {
        ws_list.push(WsBody{ws:(ws)});
    }

    Ok(Json(ws_list))
}

async fn update_ws(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    Path(ws_id): Path<Uuid>,
    ws_req: Json<WsBody<WsReq>>
) -> Result<Json<WsBody<Ws>>> {
    let mut ws = check_ws_owner(auth_user.user_id, ws_id, &ctx).await?;
    ws.name = ws_req.ws.name.clone();
    let ws = ws.update(&ctx.db).await?;

    Ok(Json(WsBody {
        ws: Ws::new(ws_id, ws.name, auth_user.user_id, false)
    }))
}

async fn delete_ws(
    ctx: Extension<ApiContext>,
    auth_user: AuthUser,
    Path(ws_id): Path<Uuid>,
) -> Result<()> {
    let ws = check_ws_owner(auth_user.user_id, ws_id, &ctx).await?;
    ws.delete(&ctx.db).await?;

    Ok(())
}
