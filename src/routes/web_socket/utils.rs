use super::schemas::ProcessType;
use crate::utils::save_notification_to_database;
use crate::websocket_client::Server;
use crate::websocket_client::{MessageToClient, SessionExists, WebSocketActionType, WebSocketData};
use actix::Addr;
use anyhow::anyhow;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn send_notification(
    pool: &PgPool,
    websocket_srv: &Addr<Server>,
    action_type: WebSocketActionType,
    process_type: ProcessType,
    user_id: Option<Uuid>,
    message: String,
    business_id: Option<Uuid>,
) -> Result<(), anyhow::Error> {
    let msg: MessageToClient = MessageToClient::new(
        action_type,
        serde_json::to_value(WebSocketData {
            message,
            business_id: business_id,
        })
        .unwrap(),
        user_id,
        None,
        None,
    );
    let id = msg.id.as_ref().unwrap();
    let connection_exist = websocket_srv
        .send(SessionExists { id: id.clone() })
        .await
        .unwrap_or(false);
    if process_type == ProcessType::Immediate || connection_exist {
        websocket_srv.do_send(msg);
    } else {
        let message_json = serde_json::to_value(&msg).unwrap();
        save_notification_to_database(pool, id, &message_json)
            .await
            .map_err(|_| anyhow!("Something went wrong while saving date to database"))?;
    }

    Ok(())
}
