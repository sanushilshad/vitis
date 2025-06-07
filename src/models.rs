use serde::Deserialize;
use sqlx::FromRow;

use crate::websocket::MessageToClient;

#[derive(Debug, Deserialize, FromRow)]
pub struct NotficationDataModel {
    pub connection_id: String,
    pub data: sqlx::types::Json<MessageToClient>,
}
