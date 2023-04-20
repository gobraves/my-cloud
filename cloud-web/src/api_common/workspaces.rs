#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WsBody<T> {
    pub ws: T,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WsReq {
    pub name: String,
}