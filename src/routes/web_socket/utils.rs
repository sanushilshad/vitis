use super::schemas::ProcessType;
use crate::models::NotficationDataModel;
use crate::pulsar_client::{PulsarClient, PulsarTopic, WSMessageData};
use crate::schemas::BulkNotificationData;
use crate::websocket_client::Server;
use crate::websocket_client::{MessageToClient, SessionExists, WebSocketActionType, WebSocketData};
use actix::Addr;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use sqlx::types::Json;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;
fn get_notification_bulk_insert_data<'a>(
    connection_list_list: &'a Vec<String>,
    message: &'a Value,
    created_on: &'a DateTime<Utc>,
) -> BulkNotificationData<'a> {
    let mut id_list = vec![];
    let mut connection_id_list = vec![];
    let mut created_on_list = vec![];
    let mut messsage_list = vec![];
    // let created_on = Utc::now();

    for connection in connection_list_list {
        id_list.push(Uuid::new_v4());
        created_on_list.push(created_on);
        connection_id_list.push(connection.as_str());
        messsage_list.push(message);
    }
    BulkNotificationData {
        id: id_list,
        created_on: created_on_list,
        connection_id: connection_id_list,
        data: messsage_list,
    }
}
#[allow(clippy::too_many_arguments)]
pub async fn send_notification(
    pool: &PgPool,
    websocket_srv: &Addr<Server>,
    action_type: WebSocketActionType,
    process_type: ProcessType,
    user_id_list: Vec<Uuid>,
    message: String,
    business_id: Option<Uuid>,
    producer_client: &PulsarClient,
) -> Result<(), anyhow::Error> {
    let mut connected_user_list = vec![];
    let data = WebSocketData {
        message: message.to_owned(),
        business_id,
        action_type: action_type.to_owned(),
    };
    let data_json = serde_json::to_value(&data).unwrap();
    for user_id in user_id_list {
        // let id = msg.id.as_ref().unwrap();
        let id = MessageToClient::get_ws_key(Some(user_id), None, None);

        let connection_exist = websocket_srv
            .send(SessionExists { id: id.clone() })
            .await
            .unwrap_or(false);

        if connection_exist && process_type == ProcessType::Immediate {
            let msg = MessageToClient::new(
                serde_json::to_value(&data).unwrap(),
                Some(user_id),
                None,
                None,
            );
            websocket_srv.do_send(msg);
        } else {
            connected_user_list.push(id);
        }
    }
    if !connected_user_list.is_empty() {
        let current_time = Utc::now();
        let data =
            get_notification_bulk_insert_data(&connected_user_list, &data_json, &current_time);

        save_notification_to_database(pool, data)
            .await
            .map_err(|_| anyhow!("Something went wrong while saving date to database"))?;
        let mut producer = producer_client
            .get_producer(producer_client.get_product_topic(PulsarTopic::WebSocket))
            .await;
        producer
            .send_non_blocking(WSMessageData {
                partition_key_list: connected_user_list,
            })
            .await?;
    }

    Ok(())
}

#[tracing::instrument(name = "save leave type to database", skip(pool, data))]
pub async fn save_notification_to_database<'a>(
    pool: &PgPool,
    data: BulkNotificationData<'a>,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO pending_notification (id, data, connection_id,  created_on)
        SELECT * FROM UNNEST($1::uuid[], $2::jsonb[], $3::text[],  $4::TIMESTAMP[]) 
        "#,
        &data.id[..] as &[Uuid],
        &data.data[..] as &[&Value],
        &data.connection_id[..] as &[&str],
        &data.created_on[..] as &[&DateTime<Utc>],
    );
    query.execute(pool).await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving notification data request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "fetch_notifications_by_connection_id", skip(transaction))]
pub async fn fetch_notifications_by_connection_id(
    transaction: &mut Transaction<'_, Postgres>,
    connection_id: &str,
) -> Result<Vec<NotficationDataModel>, anyhow::Error> {
    let records = sqlx::query_as!(
        NotficationDataModel,
        r#"
        SELECT data as "data: Json<WebSocketData>", connection_id
        FROM pending_notification
        WHERE connection_id = $1 ORDER BY created_on
        FOR UPDATE
        "#,
        connection_id
    )
    .fetch_all(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching notifications")
    })?;

    Ok(records)
}

#[tracing::instrument(name = "delete_notifications_by_connection_id", skip(transaction))]
pub async fn delete_notifications_by_connection_id(
    transaction: &mut Transaction<'_, Postgres>,
    connection_id: &str,
) -> Result<u64, anyhow::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM pending_notification
        WHERE connection_id = $1
        "#,
        connection_id
    )
    .execute(&mut **transaction) // Explicitly dereference the transaction
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute delete query: {:?}", e);
        anyhow::Error::new(e).context("Failed to delete notifications for the given connection_id")
    })?;

    Ok(result.rows_affected()) // Return the number of rows deleted
}
