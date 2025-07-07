use serde::Deserialize;
use sqlx::FromRow;

use crate::websocket_client::WebSocketData;

#[derive(Debug, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct NotficationDataModel {
    pub connection_id: String,
    pub data: sqlx::types::Json<WebSocketData>,
}
