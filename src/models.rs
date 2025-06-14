use serde::Deserialize;
use sqlx::FromRow;

use crate::websocket_client::MessageToClient;

#[derive(Debug, Deserialize, FromRow)]
pub struct NotficationDataModel {
    pub connection_id: String,
    pub data: sqlx::types::Json<MessageToClient>,
}
